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

use std::time::Duration;
use std::io::{Read, Write};
use std::str::{self, Lines};
use std::net::{TcpStream, SocketAddr};

pub struct Response {
	pub status: String,
	pub headers: Vec<String>,
	pub headers_raw: String,
	pub body: String,
}

pub fn read_block(lines: &mut Lines, all: bool) -> String {
	let mut block = String::new();
	loop {
		let line = lines.next();
		match line {
			None => break,
			Some("") if !all => break,
			Some(v) => {
				block.push_str(v);
				block.push_str("\n");
			},
		}
	}
	block
}

pub fn request(address: &SocketAddr, request: &str) -> Response {
	let mut req = TcpStream::connect(address).unwrap();
	req.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	req.write_all(request.as_bytes()).unwrap();

	let mut response = String::new();
	let _ = req.read_to_string(&mut response);

	let mut lines = response.lines();
	let status = lines.next().unwrap().to_owned();
	let headers_raw = read_block(&mut lines, false);
	let headers = headers_raw.split('\n').map(|v| v.to_owned()).collect();
	let body = read_block(&mut lines, true);

	Response {
		status: status,
		headers: headers,
		headers_raw: headers_raw,
		body: body,
	}
}

/// Check if all required security headers are present
pub fn assert_security_headers_present(headers: &[String]) {
	assert!(
		headers.iter().find(|header| header.as_str() == "X-Frame-Options: SAMEORIGIN").is_some(),
		"X-Frame-Options missing: {:?}", headers
	);
	assert!(
		headers.iter().find(|header| header.as_str() == "X-XSS-Protection: 1; mode=block").is_some(),
		"X-XSS-Protection missing: {:?}", headers
	);
	assert!(
		headers.iter().find(|header|  header.as_str() == "X-Content-Type-Options: nosniff").is_some(),
		"X-Content-Type-Options missing: {:?}", headers
	);
}
