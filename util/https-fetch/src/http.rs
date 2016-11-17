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

//! HTTP format processor

use std::io::{self, Cursor, Write};
use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
	WaitingForStatus,
	WaitingForHeaders,
	WaitingForChunk,
	WritingBody,
	WritingChunk(usize),
	Finished,
}

pub struct HttpProcessor {
	state: State,
	buffer: Cursor<Vec<u8>>,
	status: Option<String>,
	headers: Vec<String>,
	body_writer: io::BufWriter<Box<io::Write>>,
	size_limit: Option<usize>,
}

const BREAK_LEN: usize = 2;

impl HttpProcessor {
	pub fn new(body_writer: Box<io::Write>, size_limit: Option<usize>) -> Self {
		HttpProcessor {
			state: State::WaitingForStatus,
			buffer: Cursor::new(Vec::new()),
			status: None,
			headers: Vec::new(),
			body_writer: io::BufWriter::new(body_writer),
			size_limit: size_limit,
		}
	}

	pub fn status(&self) -> Option<&String> {
		self.status.as_ref()
	}

	pub fn status_is_ok(&self) -> bool {
		self.status == Some("HTTP/1.1 200 OK".into())
	}

	#[cfg(test)]
	pub fn headers(&self) -> &[String] {
		&self.headers
	}

	fn find_break_index(&mut self) -> Option<usize> {
		let data = self.buffer.get_ref();
		let mut idx = 0;
		let mut got_r = false;
		// looks for \r\n in data
		for b in data {
			idx += 1;
			if got_r && b == &10u8 {
				return Some(idx);
			} else if !got_r && b == &13u8 {
				got_r = true;
			} else {
				got_r = false;
			}
		}
		None
	}

	// Consumes bytes from internal buffer
	fn buffer_consume(&mut self, bytes: usize) {
		let bytes = cmp::min(bytes, self.buffer.get_ref().len());
		// Drain data
		self.buffer.get_mut().drain(0..bytes);
		let len = self.buffer.position();
		self.buffer.set_position(len - bytes as u64);
	}

	fn buffer_to_string(&mut self, bytes: usize) -> String {
		let val = String::from_utf8_lossy(&self.buffer.get_ref()[0..bytes-BREAK_LEN]).into_owned();
		self.buffer_consume(bytes);
		val
	}

	fn is_chunked(&self) -> bool {
		self.headers
			.iter()
			.find(|item| item.to_lowercase().contains("transfer-encoding: chunked"))
			.is_some()
	}
	fn set_state(&mut self, state: State) {
		self.state = state;
		trace!("Changing state to {:?}", state);
	}

	fn process_buffer(&mut self) -> io::Result<()> {
		// consume data and perform state transitions
		loop {
			match self.state {
				State::WaitingForStatus => {
					if let Some(break_index) = self.find_break_index() {
						let status = self.buffer_to_string(break_index);
						debug!("Read status: {:?}", status);
						self.status = Some(status);
						self.set_state(State::WaitingForHeaders);
					} else {
						// wait for more data
						return Ok(());
					}
				},
				State::WaitingForHeaders => {
					match self.find_break_index() {
						// Last header - got empty line, body starts
						Some(BREAK_LEN) => {
							self.buffer_consume(BREAK_LEN);
							let is_chunked = self.is_chunked();
							self.set_state(match is_chunked {
								true => State::WaitingForChunk,
								false => State::WritingBody,
							});
						},
						Some(break_index) => {
							let header = self.buffer_to_string(break_index);
							debug!("Found header: {:?}", header);
							self.headers.push(header);
						},
						None => return Ok(()),
					}
				},
				State::WritingBody => {
					let len = self.buffer.get_ref().len();
					match self.size_limit {
						None => {},
						Some(limit) if limit > len => {},
						_ => {
							warn!("Finishing file fetching because limit was reached.");
							self.set_state(State::Finished);
							continue;
						}
					}
					try!(self.body_writer.write_all(self.buffer.get_ref()));
					self.buffer_consume(len);
					return self.body_writer.flush();
				},
				State::WaitingForChunk => {
					match self.find_break_index() {
						None => return Ok(()),
						// last chunk - size 0
						Some(BREAK_LEN) => {
							self.state = State::Finished;
						},
						Some(break_index) => {
							let chunk_size = self.buffer_to_string(break_index);
							self.set_state(if let Ok(size) = usize::from_str_radix(&chunk_size, 16) {
								State::WritingChunk(size)
							} else {
								warn!("Error parsing server chunked response. Invalid chunk size.");
								State::Finished
							});
						}
					}
				},
				State::WritingChunk(0) => {
					self.set_state(State::Finished);
				},
				// Buffers the data until we have a full chunk
				State::WritingChunk(left) if self.buffer.get_ref().len() >= left => {
					match self.size_limit {
						None => {},
						Some(limit) if limit > left => {
							self.size_limit = Some(limit - left);
						},
						_ => {
							warn!("Finishing file fetching because limit was reached.");
							self.set_state(State::Finished);
							continue;
						}
					}
					{
						let chunk = &self.buffer.get_ref()[0..left];
						trace!("Writing chunk: {:?}", String::from_utf8_lossy(chunk));
						try!(self.body_writer.write_all(chunk));
					}
					self.buffer_consume(left + BREAK_LEN);

					self.set_state(State::WaitingForChunk);
				},
				// Wait for more data
				State::WritingChunk(_) => return Ok(()),
				// Just consume buffer
				State::Finished => {
					let len = self.buffer.get_ref().len();
					self.buffer_consume(len);
					return self.body_writer.flush();
				},
			}
		}
	}

	#[cfg(test)]
	pub fn state(&self) -> State {
		self.state
	}
}

impl io::Write for HttpProcessor {
	fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
		let result = self.buffer.write(bytes);
		try!(self.process_buffer());
		result
	}

	fn flush(&mut self) -> io::Result<()> {
		self.buffer.flush().and_then(|_| {
			self.body_writer.flush()
		})
	}
}

#[cfg(test)]
mod tests {
	use std::rc::Rc;
	use std::cell::RefCell;
	use std::io::{self, Write, Cursor};
	use super::*;

	struct Writer {
		data: Rc<RefCell<Cursor<Vec<u8>>>>,
	}

	impl Writer {
		fn new() -> (Self, Rc<RefCell<Cursor<Vec<u8>>>>) {
			let data = Rc::new(RefCell::new(Cursor::new(Vec::new())));
			(Writer { data: data.clone() }, data)
		}
	}

	impl Write for Writer {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.data.borrow_mut().write(buf) }
		fn flush(&mut self) -> io::Result<()> { self.data.borrow_mut().flush() }
	}

	#[test]
	fn should_be_able_to_process_status_line() {
		// given
		let mut http = HttpProcessor::new(Box::new(Cursor::new(Vec::new())), None);

		// when
		let out =
			"\
				HTTP/1.1 200 OK\r\n\
				Server: Pari
			";
		http.write_all(out.as_bytes()).unwrap();
		http.flush().unwrap();

		// then
		assert_eq!(http.status().unwrap(), "HTTP/1.1 200 OK");
		assert_eq!(http.state(), State::WaitingForHeaders);
	}

	#[test]
	fn should_be_able_to_process_headers() {
		// given
		let mut http = HttpProcessor::new(Box::new(Cursor::new(Vec::new())), None);

		// when
		let out =
			"\
				HTTP/1.1 200 OK\r\n\
				Server: Parity/1.1.1\r\n\
				Connection: close\r\n\
				Content-Length: 2\r\n\
				Content-Type: application/json\r\n\
				\r\n\
			";
		http.write_all(out.as_bytes()).unwrap();
		http.flush().unwrap();

		// then
		assert_eq!(http.status().unwrap(), "HTTP/1.1 200 OK");
		assert_eq!(http.headers().len(), 4);
		assert_eq!(http.state(), State::WritingBody);
	}

	#[test]
	fn should_be_able_to_consume_body() {
		// given
		let (writer, data) = Writer::new();
		let mut http = HttpProcessor::new(Box::new(writer), None);

		// when
		let out =
			"\
				HTTP/1.1 200 OK\r\n\
				Server: Parity/1.1.1\r\n\
				Connection: close\r\n\
				Content-Length: 2\r\n\
				Content-Type: application/json\r\n\
				\r\n\
				Some data\
			";
		http.write_all(out.as_bytes()).unwrap();
		http.flush().unwrap();

		// then
		assert_eq!(http.status().unwrap(), "HTTP/1.1 200 OK");
		assert_eq!(http.headers().len(), 4);
		assert_eq!(http.state(), State::WritingBody);
		assert_eq!(data.borrow().get_ref()[..], b"Some data"[..]);
	}

	#[test]
	fn should_correctly_handle_chunked_content() {
		// given
		let (writer, data) = Writer::new();
		let mut http = HttpProcessor::new(Box::new(writer), None);

		// when
		let out =
			"\
				HTTP/1.1 200 OK\r\n\
				Host: 127.0.0.1:8080\r\n\
				Transfer-Encoding: chunked\r\n\
				Connection: close\r\n\
				\r\n\
				4\r\n\
				Pari\r\n\
				3\r\n\
				ty \r\n\
				D\r\n\
				in\r\n\
				\r\n\
				chunks.\r\n\
				0\r\n\
				\r\n\
			";
		http.write_all(out.as_bytes()).unwrap();
		http.flush().unwrap();

		// then
		assert_eq!(http.status().unwrap(), "HTTP/1.1 200 OK");
		assert_eq!(http.headers().len(), 3);
		assert_eq!(data.borrow().get_ref()[..], b"Parity in\r\n\r\nchunks."[..]);
		assert_eq!(http.state(), State::Finished);
	}

	#[test]
	fn should_stop_fetching_when_limit_is_reached() {
		// given
		let (writer, data) = Writer::new();
		let mut http = HttpProcessor::new(Box::new(writer), Some(5));

		// when
		let out =
			"\
				HTTP/1.1 200 OK\r\n\
				Host: 127.0.0.1:8080\r\n\
				Transfer-Encoding: chunked\r\n\
				Connection: close\r\n\
				\r\n\
				4\r\n\
				Pari\r\n\
				3\r\n\
				ty \r\n\
				D\r\n\
				in\r\n\
				\r\n\
				chunks.\r\n\
				0\r\n\
				\r\n\
			";
		http.write_all(out.as_bytes()).unwrap();
		http.flush().unwrap();

		// then
		assert_eq!(http.status().unwrap(), "HTTP/1.1 200 OK");
		assert_eq!(http.headers().len(), 3);
		assert_eq!(data.borrow().get_ref()[..], b"Pari"[..]);
		assert_eq!(http.state(), State::Finished);
	}

}
