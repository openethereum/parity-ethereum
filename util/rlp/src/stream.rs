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

use std::borrow::Borrow;
use elastic_array::{ElasticArray16, ElasticArray1024};
use bytes::{ToBytes, VecLike};
use traits::RlpEncodable;
use Stream;

#[derive(Debug, Copy, Clone)]
struct ListInfo {
	position: usize,
	current: usize,
	max: usize,
}

impl ListInfo {
	fn new(position: usize, max: usize) -> ListInfo {
		ListInfo {
			position: position,
			current: 0,
			max: max,
		}
	}
}

/// Appendable rlp encoder.
pub struct RlpStream {
	unfinished_lists: ElasticArray16<ListInfo>,
	buffer: ElasticArray1024<u8>,
	finished_list: bool,
}

impl Default  for RlpStream {
	fn default() -> Self {
		RlpStream::new()
	}
}

impl RlpStream {
	pub fn new() -> Self {
		RlpStream {
			unfinished_lists: ElasticArray16::new(),
			buffer: ElasticArray1024::new(),
			finished_list: false,
		}
	}

	pub fn new_list(len: usize) -> Self {
		let mut stream = RlpStream::new();
		stream.begin_list(len);
		stream
	}

	pub fn append<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: RlpEncodable {
		self.finished_list = false;
		value.rlp_append(self);
		if !self.finished_list {
			self.note_appended(1);
		}
		self
	}

	pub fn append_list<'a, E, K>(&'a mut self, values: &[K]) -> &'a mut Self where E: RlpEncodable, K: Borrow<E> {
		self.begin_list(values.len());
		for value in values {
			self.append(value.borrow());
		}
		self
	}

	pub fn begin_list(&mut self, len: usize) -> &mut RlpStream {
		self.finished_list = false;
		match len {
			0 => {
				// we may finish, if the appended list len is equal 0
				self.buffer.push(0xc0u8);
				self.note_appended(1);
				self.finished_list = true;
			},
			_ => {
				let position = self.buffer.len();
				self.unfinished_lists.push(ListInfo::new(position, len));
			},
		}

		// return chainable self
		self
	}

	pub fn append_empty_data(&mut self) -> &mut RlpStream {
		// self push raw item
		self.buffer.push(0x80);

		// try to finish and prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	pub fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut RlpStream {
		// push raw items
		self.buffer.append_slice(bytes);

		// try to finish and prepend the length
		self.note_appended(item_count);

		// return chainable self
		self
	}

	pub fn clear(&mut self) {
		// clear bytes
		self.buffer.clear();

		// clear lists
		self.unfinished_lists.clear();
	}

	pub fn is_finished(&self) -> bool {
		self.unfinished_lists.len() == 0
	}

	pub fn as_raw(&self) -> &[u8] {
		//&self.encoder.bytes
		&self.buffer
	}

	pub fn out(self) -> Vec<u8> {
		match self.is_finished() {
			//true => self.encoder.out().to_vec(),
			true => self.buffer.to_vec(),
			false => panic!()
		}
	}

	/// Try to finish lists
	fn note_appended(&mut self, inserted_items: usize) -> () {
		if self.unfinished_lists.len() == 0 {
			return;
		}

		let back = self.unfinished_lists.len() - 1;
		let should_finish = match self.unfinished_lists.get_mut(back) {
			None => false,
			Some(ref mut x) => {
				x.current += inserted_items;
				if x.current > x.max {
					panic!("You cannot append more items then you expect!");
				}
				x.current == x.max
			}
		};

		if should_finish {
			let x = self.unfinished_lists.pop().unwrap();
			let len = self.buffer.len() - x.position;
			self.encoder().insert_list_payload(len, x.position);
			self.note_appended(1);
		}
		self.finished_list = should_finish;
	}

	pub fn encoder(&mut self) -> BasicEncoder {
		BasicEncoder::new(self)
	}

	/// Drain the object and return the underlying ElasticArray.
	pub fn drain(self) -> ElasticArray1024<u8> {
		match self.is_finished() {
			true => self.buffer,
			false => panic!()
		}
	}
}

pub struct BasicEncoder<'a> {
	buffer: &'a mut ElasticArray1024<u8>,
}

impl<'a> BasicEncoder<'a> {
	fn new(stream: &'a mut RlpStream) -> Self {
		BasicEncoder {
			buffer: &mut stream.buffer
		}
	}

	/// inserts list prefix at given position
	/// TODO: optimise it further?
	fn insert_list_payload(&mut self, len: usize, pos: usize) -> () {
		let mut res = ElasticArray16::new();
		match len {
			0...55 => res.push(0xc0u8 + len as u8),
			_ => {
				res.push(0xf7u8 + len.to_bytes_len() as u8);
				ToBytes::to_bytes(&len, &mut res);
			}
		};

		self.buffer.insert_slice(pos, &res);
	}

	pub fn encode_value(&mut self, value: &[u8]) {
		match value.len() {
			// just 0
			0 => self.buffer.push(0x80u8),
			// byte is its own encoding if < 0x80
			1 => {
				self.buffer.append_slice(value);
				let len = self.buffer.len();
				let last_byte = self.buffer[len - 1];
				if last_byte >= 0x80 {
					self.buffer.push(last_byte);
					self.buffer[len - 1] = 0x81;
				}
			}
			// (prefix + length), followed by the string
			len @ 2 ... 55 => {
				self.buffer.push(0x80u8 + len as u8);
				//value.to_bytes(self.buffer);
				self.buffer.append_slice(value);
			}
			// (prefix + length of length), followed by the length, followd by the string
			len => {
				self.buffer.push(0xb7 + len.to_bytes_len() as u8);
				ToBytes::to_bytes(&len, self.buffer);
				self.buffer.append_slice(value);
			}
		}
	}
}
