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

use std::thread;
use std::time::Duration;
use std::io::{self, Read, Write};
use std::str::{self, Lines};
use std::net::{TcpStream, SocketAddr};

pub struct Response {
	pub status: String,
	pub headers: Vec<String>,
	pub headers_raw: String,
	pub body: String,
}

impl Response {
	pub fn assert_header(&self, header: &str, value: &str) {
		let header = format!("{}: {}", header, value);
		assert!(self.headers.iter().find(|h| *h == &header).is_some(), "Couldn't find header {} in {:?}", header, &self.headers)
	}

	pub fn assert_status(&self, status: &str) {
		assert_eq!(self.status, status.to_owned(), "Got unexpected code. Body: {:?}", self.body);
	}

	pub fn assert_security_headers_present(&self, port: Option<u16>) {
		assert_security_headers_present(&self.headers, port)
	}
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

fn connect(address: &SocketAddr) -> TcpStream {
	let mut retries = 0;
	let mut last_error = None;
	while retries < 10 {
		retries += 1;

		let res = TcpStream::connect(address);
		match res {
			Ok(stream) => {
				return stream;
			},
			Err(e) => {
				last_error = Some(e);
				thread::sleep(Duration::from_millis(retries * 10));
			}
		}
	}
	panic!("Unable to connect to the server. Last error: {:?}", last_error);
}

pub fn request(address: &SocketAddr, request: &str) -> Response {
	let mut req = connect(address);
	req.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
	req.write_all(request.as_bytes()).unwrap();

	let mut response = Vec::new();
	loop {
		let mut chunk = [0; 32 *1024];
		match req.read(&mut chunk) {
			Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => break,
			Err(err) => panic!("Unable to read response: {:?}", err),
			Ok(0) => break,
			Ok(read) => response.extend_from_slice(&chunk[..read]),
		}
	}

	let response = String::from_utf8_lossy(&response).into_owned();
	let mut lines = response.lines();
	let status = lines.next().expect("Expected a response").to_owned();
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
pub fn assert_security_headers_present(headers: &[String], port: Option<u16>) {
	if let None = port {
		assert!(
			headers.iter().find(|header| header.as_str() == "X-Frame-Options: SAMEORIGIN").is_some(),
			"X-Frame-Options: SAMEORIGIN missing: {:?}", headers
		);
	}
	assert!(
		headers.iter().find(|header| header.as_str() == "X-XSS-Protection: 1; mode=block").is_some(),
		"X-XSS-Protection missing: {:?}", headers
	);
	assert!(
		headers.iter().find(|header|  header.as_str() == "X-Content-Type-Options: nosniff").is_some(),
		"X-Content-Type-Options missing: {:?}", headers
	);
	assert!(
		headers.iter().find(|header| header.starts_with("Content-Security-Policy: ")).is_some(),
		"Content-Security-Policy missing: {:?}", headers
	)
}
