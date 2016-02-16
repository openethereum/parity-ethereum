// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::io;
use igd::{PortMappingProtocol, search_gateway_from_timeout};
use std::time::Duration;
use network::node_table::{NodeEndpoint};

pub enum IpAddr{
	V4(Ipv4Addr),
	V6(Ipv6Addr),
}

#[cfg(not(windows))]
mod getinterfaces {
	use std::{mem, io, ptr};
	use libc::{AF_INET, AF_INET6};
	use libc::{getifaddrs, freeifaddrs, ifaddrs, sockaddr, sockaddr_in, sockaddr_in6};
	use std::net::{Ipv4Addr, Ipv6Addr};
	use super::IpAddr;

	fn convert_sockaddr (sa: *mut sockaddr) -> Option<IpAddr> {
		if sa == ptr::null_mut() { return None; }

		let (addr, _) = match unsafe { *sa }.sa_family as i32 {
			AF_INET => {
				let sa: *const sockaddr_in = unsafe { mem::transmute(sa) };
				let sa = & unsafe { *sa };
				let (addr, port) = (sa.sin_addr.s_addr, sa.sin_port);
				(
					IpAddr::V4(Ipv4Addr::new(
							(addr & 0x000000FF) as u8,
							((addr & 0x0000FF00) >>  8) as u8,
							((addr & 0x00FF0000) >> 16) as u8,
							((addr & 0xFF000000) >> 24) as u8,
							)),
							port
				)
			},
			AF_INET6 => {
				let sa: *const sockaddr_in6 = unsafe { mem::transmute(sa) };
				let sa = & unsafe { *sa };
				let (addr, port) = (sa.sin6_addr.s6_addr, sa.sin6_port);
				let addr: [u16; 8] = unsafe { mem::transmute(addr) };
				(
					IpAddr::V6(Ipv6Addr::new(
							addr[0],
							addr[1],
							addr[2],
							addr[3],
							addr[4],
							addr[5],
							addr[6],
							addr[7],
							)),
							port
				)
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
					IpAddr::V4(a) if !a.is_unspecified() && !a.is_loopback() && !a.is_link_local() => {
						return SocketAddr::V4(SocketAddrV4::new(a, port));
					},
					_ => {},
				}
			}
			for addr in list {
				match addr {
					IpAddr::V6(a) if !a.is_unspecified() && !a.is_loopback() => {
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

