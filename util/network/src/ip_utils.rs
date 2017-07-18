// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

// Based on original work by David Levy https://raw.githubusercontent.com/dlevy47/rust-interfaces

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::io;
use igd::{PortMappingProtocol, search_gateway_from_timeout};
use std::time::Duration;
use node_table::{NodeEndpoint};

/// Socket address extension for rustc beta. To be replaces with now unstable API
pub trait SocketAddrExt {
	/// Returns true for the special 'unspecified' address 0.0.0.0.
	fn is_unspecified_s(&self) -> bool;
	/// Returns true if the address appears to be globally routable.
	fn is_global_s(&self) -> bool;

	fn is_other_private(&self) -> bool;
	fn is_future_use(&self) -> bool;
	fn is_special_purpose(&self) -> bool;
	fn is_reserved(&self) -> bool;
}

impl SocketAddrExt for Ipv4Addr {
	fn is_unspecified_s(&self) -> bool {
		self.octets() == [0, 0, 0, 0]
	}

	fn is_global_s(&self) -> bool {
		!self.is_private() && !self.is_loopback() && !self.is_link_local() &&
			!self.is_broadcast() && !self.is_documentation()
	}

	// Used for communications between a service provider and its subscribers when using a carrier-grade NAT 
	// see: https://en.wikipedia.org/wiki/Reserved_IP_addresses
	fn is_other_private(&self) -> bool {
		self.octets()[0] == 100 &&
		self.octets()[1] >= 64 &&
		self.octets()[1] <= 127
	}

	// Used for the IANA IPv4 Special Purpose Address Registry
	// see: https://en.wikipedia.org/wiki/Reserved_IP_addresses
	fn is_special_purpose(&self) -> bool {
		self.octets()[0] == 192 &&
		self.octets()[1] == 0 &&
		self.octets()[2] == 0
	}

	// Reserved for future use
	// see: https://en.wikipedia.org/wiki/Reserved_IP_addresses
	fn is_future_use(&self) -> bool {
		self.octets()[0] >= 240 &&
		self.octets()[3] < 255
	}

	fn is_reserved(&self) -> bool {
		self.is_unspecified_s() ||
		self.is_private() ||
		self.is_loopback() ||
		self.is_link_local() ||
		self.is_broadcast() ||
		self.is_documentation() ||
		self.is_other_private() ||
		self.is_special_purpose() ||
		self.is_future_use()
	}
}

impl SocketAddrExt for Ipv6Addr {
	fn is_unspecified_s(&self) -> bool {
		self.segments() == [0, 0, 0, 0, 0, 0, 0, 0]
	}

	fn is_global_s(&self) -> bool {
		if self.is_multicast() {
			self.segments()[0] & 0x000f == 14
		} else {
			!self.is_loopback() && !((self.segments()[0] & 0xffc0) == 0xfe80) &&
            	!((self.segments()[0] & 0xffc0) == 0xfec0) && !((self.segments()[0] & 0xfe00) == 0xfc00)
		}
	}

	fn is_other_private(&self) -> bool { false }
	fn is_special_purpose(&self) -> bool { false }
	fn is_future_use(&self) -> bool { false }

	fn is_reserved(&self) -> bool {
		self.is_unspecified_s() ||
		self.is_loopback() ||
		self.is_global_s()
	}
}

impl SocketAddrExt for IpAddr {
	fn is_unspecified_s(&self) -> bool {
		match *self {
			IpAddr::V4(ref ip) => ip.is_unspecified_s(),
			IpAddr::V6(ref ip) => ip.is_unspecified_s(),
		}
	}

	fn is_global_s(&self) -> bool {
		match *self {
			IpAddr::V4(ref ip) => ip.is_global_s(),
			IpAddr::V6(ref ip) => ip.is_global_s(),
		}
	}

	fn is_other_private(&self) -> bool {
		match *self {
			IpAddr::V4(ref ip) => ip.is_other_private(),
			IpAddr::V6(ref ip) => ip.is_other_private(),
		}
	}
	fn is_future_use(&self) -> bool { 
		match *self {
			IpAddr::V4(ref ip) => ip.is_future_use(),
			IpAddr::V6(ref ip) => ip.is_future_use(),
		}
	}

	fn is_special_purpose(&self) -> bool {
		match *self {
			IpAddr::V4(ref ip) => ip.is_special_purpose(),
			IpAddr::V6(ref ip) => ip.is_special_purpose(),
		}
	}

	fn is_reserved(&self) -> bool {
		match *self {
			IpAddr::V4(ref ip) => ip.is_reserved(),
			IpAddr::V6(ref ip) => ip.is_reserved(),
		}
	}
}

#[cfg(not(windows))]
mod getinterfaces {
	use std::{mem, io, ptr};
	use libc::{AF_INET, AF_INET6};
	use libc::{getifaddrs, freeifaddrs, ifaddrs, sockaddr, sockaddr_in, sockaddr_in6};
	use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

	fn convert_sockaddr (sa: *mut sockaddr) -> Option<IpAddr> {
		if sa == ptr::null_mut() { return None; }

		let (addr, _) = match unsafe { *sa }.sa_family as i32 {
			AF_INET => {
				let sa: *const sockaddr_in = unsafe { mem::transmute(sa) };
				let sa = & unsafe { *sa };
				let (addr, port) = (sa.sin_addr.s_addr, sa.sin_port);
				(IpAddr::V4(Ipv4Addr::new(
					(addr & 0x000000FF) as u8,
					((addr & 0x0000FF00) >>  8) as u8,
					((addr & 0x00FF0000) >> 16) as u8,
					((addr & 0xFF000000) >> 24) as u8)),
					port)
			},
			AF_INET6 => {
				let sa: *const sockaddr_in6 = unsafe { mem::transmute(sa) };
				let sa = & unsafe { *sa };
				let (addr, port) = (sa.sin6_addr.s6_addr, sa.sin6_port);
				let addr: [u16; 8] = unsafe { mem::transmute(addr) };
				(IpAddr::V6(Ipv6Addr::new(
					addr[0],
					addr[1],
					addr[2],
					addr[3],
					addr[4],
					addr[5],
					addr[6],
					addr[7])),
					port)
			},
			_ => return None,
		};
		Some(addr)
	}

	fn convert_ifaddrs (ifa: *mut ifaddrs) -> Option<IpAddr> {
		let ifa = unsafe { &mut *ifa };
		convert_sockaddr(ifa.ifa_addr)
	}

	pub fn get_all() -> io::Result<Vec<IpAddr>> {
		let mut ifap: *mut ifaddrs = unsafe { mem::zeroed() };
		if unsafe { getifaddrs(&mut ifap as *mut _) } != 0 {
			return Err(io::Error::last_os_error());
		}

		let mut ret = Vec::new();
		let mut cur: *mut ifaddrs = ifap;
		while cur != ptr::null_mut() {
			if let Some(ip_addr) = convert_ifaddrs(cur) {
				ret.push(ip_addr);
			}

			//TODO: do something else maybe?
			cur = unsafe { (*cur).ifa_next };
		}

		unsafe { freeifaddrs(ifap) };
		Ok(ret)
	}
}

#[cfg(not(windows))]
fn get_if_addrs() -> io::Result<Vec<IpAddr>> {
	getinterfaces::get_all()
}

#[cfg(windows)]
fn get_if_addrs() -> io::Result<Vec<IpAddr>> {
	Ok(Vec::new())
}

/// Select the best available public address
pub fn select_public_address(port: u16) -> SocketAddr {
	match get_if_addrs() {
		Ok(list) => {
			//prefer IPV4 bindings
			for addr in &list { //TODO: use better criteria than just the first in the list
				match *addr {
					IpAddr::V4(a) if !a.is_unspecified_s() && !a.is_loopback() && !a.is_link_local() => {
						return SocketAddr::V4(SocketAddrV4::new(a, port));
					},
					_ => {},
				}
			}
			for addr in list {
				match addr {
					IpAddr::V6(a) if !a.is_unspecified_s() && !a.is_loopback() => {
						return SocketAddr::V6(SocketAddrV6::new(a, port, 0, 0));
					},
					_ => {},
				}
			}
		},
		Err(e) => debug!("Error listing public interfaces: {:?}", e)
	}
	SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

pub fn map_external_address(local: &NodeEndpoint) -> Option<NodeEndpoint> {
	if let SocketAddr::V4(ref local_addr) = local.address {
		match search_gateway_from_timeout(local_addr.ip().clone(), Duration::new(5, 0)) {
			Err(ref err) => debug!("Gateway search error: {}", err),
			Ok(gateway) => {
				match gateway.get_external_ip() {
					Err(ref err) => {
						debug!("IP request error: {}", err);
					},
					Ok(external_addr) => {
						match gateway.add_any_port(PortMappingProtocol::TCP, SocketAddrV4::new(local_addr.ip().clone(), local_addr.port()), 0, "Parity Node/TCP") {
							Err(ref err) => {
								debug!("Port mapping error: {}", err);
							},
							Ok(tcp_port) => {
								match gateway.add_any_port(PortMappingProtocol::UDP, SocketAddrV4::new(local_addr.ip().clone(), local.udp_port), 0, "Parity Node/UDP") {
									Err(ref err) => {
										debug!("Port mapping error: {}", err);
									},
									Ok(udp_port) => {
										return Some(NodeEndpoint { address: SocketAddr::V4(SocketAddrV4::new(external_addr, tcp_port)), udp_port: udp_port });
									},
								}
							},
						}
					},
				}
			},
		}
	}
	None
}

#[test]
fn can_select_public_address() {
	let pub_address = select_public_address(40477);
	assert!(pub_address.port() == 40477);
}

#[ignore]
#[test]
fn can_map_external_address_or_fail() {
	let pub_address = select_public_address(40478);
	let _ = map_external_address(&NodeEndpoint { address: pub_address, udp_port: 40478 });
}

#[test]
fn ipv4_properties() {

	#![cfg_attr(feature="dev", allow(too_many_arguments))]
	fn check(octets: &[u8; 4], unspec: bool, loopback: bool,
			 private: bool, link_local: bool, global: bool,
			 multicast: bool, broadcast: bool, documentation: bool) {
		let ip = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
		assert_eq!(octets, &ip.octets());

		assert_eq!(ip.is_unspecified_s(), unspec);
		assert_eq!(ip.is_loopback(), loopback);
		assert_eq!(ip.is_private(), private);
		assert_eq!(ip.is_link_local(), link_local);
		assert_eq!(ip.is_global_s(), global);
		assert_eq!(ip.is_multicast(), multicast);
		assert_eq!(ip.is_broadcast(), broadcast);
		assert_eq!(ip.is_documentation(), documentation);
	}

	//    address                unspec loopbk privt  linloc global multicast brdcast doc
	check(&[0, 0, 0, 0],         true,  false, false, false, true,  false,    false,  false);
	check(&[0, 0, 0, 1],         false, false, false, false, true,  false,    false,  false);
	check(&[1, 0, 0, 0],         false, false, false, false, true,  false,    false,  false);
	check(&[10, 9, 8, 7],        false, false, true,  false, false, false,    false,  false);
	check(&[127, 1, 2, 3],       false, true,  false, false, false, false,    false,  false);
	check(&[172, 31, 254, 253],  false, false, true,  false, false, false,    false,  false);
	check(&[169, 254, 253, 242], false, false, false, true,  false, false,    false,  false);
	check(&[192, 0, 2, 183],     false, false, false, false, false, false,    false,  true);
	check(&[192, 1, 2, 183],     false, false, false, false, true,  false,    false,  false);
	check(&[192, 168, 254, 253], false, false, true,  false, false, false,    false,  false);
	check(&[198, 51, 100, 0],    false, false, false, false, false, false,    false,  true);
	check(&[203, 0, 113, 0],     false, false, false, false, false, false,    false,  true);
	check(&[203, 2, 113, 0],     false, false, false, false, true,  false,    false,  false);
	check(&[224, 0, 0, 0],       false, false, false, false, true,  true,     false,  false);
	check(&[239, 255, 255, 255], false, false, false, false, true,  true,     false,  false);
	check(&[255, 255, 255, 255], false, false, false, false, false, false,    true,   false);
}

#[test]
fn test_ipv4_other_private() {
	assert!(Ipv4Addr::new(100, 64, 0, 0).is_other_private());
	assert!(Ipv4Addr::new(100, 127, 255, 255).is_other_private());
	assert!(!Ipv4Addr::new(100, 63, 255, 255).is_other_private());
	assert!(!Ipv4Addr::new(100, 128, 0, 0).is_other_private());
}

#[test]
fn test_ipv4_special_purpose() {
	assert!(Ipv4Addr::new(192, 0, 0, 0).is_special_purpose());
	assert!(Ipv4Addr::new(192, 0, 0, 255).is_special_purpose());
	assert!(!Ipv4Addr::new(191, 255, 255, 255).is_special_purpose());
	assert!(!Ipv4Addr::new(192, 0, 1, 255).is_special_purpose());
}

#[test]
fn test_ipv4_future_use() {
	assert!(Ipv4Addr::new(240, 0, 0, 0).is_future_use());
	assert!(Ipv4Addr::new(255, 255, 255, 254).is_future_use());
	assert!(!Ipv4Addr::new(239, 255, 255, 255).is_future_use());
	assert!(!Ipv4Addr::new(255, 255, 255, 255).is_future_use());
}

#[test]
fn ipv6_properties() {
	fn check(str_addr: &str, unspec: bool, loopback: bool, global: bool) {
		let ip: Ipv6Addr = str_addr.parse().unwrap();
		assert_eq!(str_addr, ip.to_string());

		assert_eq!(ip.is_unspecified_s(), unspec);
		assert_eq!(ip.is_loopback(), loopback);
		assert_eq!(ip.is_global_s(), global);
	}

	//    unspec loopbk global
	check("::", true,  false, true);
	check("::1", false, true, false);
}
