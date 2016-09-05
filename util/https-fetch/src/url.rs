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

use std::net::SocketAddr;

#[derive(Debug)]
pub enum UrlError {
	InvalidAddress
}

/// Build a ClientConfig from our arguments
pub struct Url {
	address: SocketAddr,
	hostname: String,
	port: u16,
	path: String,
}

impl Url {
	pub fn new(hostname: &str, port: u16, path: &str) -> Result<Self, UrlError> {
		let addr = try!(Self::lookup_ipv4(hostname, port));
		Ok(Url {
			address: addr,
			hostname: hostname.into(),
			port: port,
			path: path.into(),
		})
	}

	fn lookup_ipv4(host: &str, port: u16) -> Result<SocketAddr, UrlError> {
		use std::net::ToSocketAddrs;

		let addrs = try!((host, port).to_socket_addrs().map_err(|_| UrlError::InvalidAddress));
		for addr in addrs {
			if let SocketAddr::V4(_) = addr {
				return Ok(addr.clone());
			}
		}
		Err(UrlError::InvalidAddress)
	}

	pub fn address(&self) -> &SocketAddr {
		&self.address
	}

	pub fn hostname(&self) -> &str {
		&self.hostname
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn path(&self) -> &str {
		&self.path
	}
}

#[cfg(test)]
#[test]
fn should_parse_url() {
	// given
	let url = Url::new("github.com", 443, "/").unwrap();

	assert_eq!(url.hostname(), "github.com");
	assert_eq!(url.port(), 443);
	assert_eq!(url.path(), "/");
}
