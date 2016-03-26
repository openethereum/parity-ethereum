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

use std::io::*;
use std::cmp;

pub struct TestSocket {
	pub read_buffer: Vec<u8>,
	pub write_buffer: Vec<u8>,
	pub cursor: usize,
	pub buf_size: usize,
}

impl Default for TestSocket {
	fn default() -> Self {
		TestSocket::new()
	}
}

impl TestSocket {
	pub fn new() -> Self {
		TestSocket {
			read_buffer: vec![],
			write_buffer: vec![],
			cursor: 0,
			buf_size: 0,
		}
	}

	pub fn new_buf(buf_size: usize) -> TestSocket {
		TestSocket {
			read_buffer: vec![],
			write_buffer: vec![],
			cursor: 0,
			buf_size: buf_size,
		}
	}

	pub fn new_ready(data: Vec<u8>) -> TestSocket {
		TestSocket {
			read_buffer: data,
			write_buffer: vec![],
			cursor: 0,
			buf_size: 0,
		}
	}
}

impl Read for TestSocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let end_position = cmp::min(self.read_buffer.len(), self.cursor+buf.len());
		let len = cmp::max(end_position - self.cursor, 0);
		match len {
			0 => Ok(0),
			_ => {
				for i in self.cursor..end_position {
					buf[i-self.cursor] = self.read_buffer[i];
				}
				self.cursor = self.cursor + buf.len();
				Ok(len)
			}
		}
	}
}

impl Write for TestSocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		if self.buf_size == 0 || buf.len() < self.buf_size {
			self.write_buffer.extend(buf.iter().cloned());
			Ok(buf.len())
		}
		else {
			self.write_buffer.extend(buf.iter().take(self.buf_size).cloned());
			Ok(self.buf_size)
		}
	}

	fn flush(&mut self) -> Result<()> {
		unimplemented!();
	}
}
