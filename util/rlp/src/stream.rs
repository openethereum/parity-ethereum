// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Borrow;
use byteorder::{ByteOrder, BigEndian};
use elastic_array::{ElasticArray16, ElasticArray1024};
use traits::Encodable;

#[derive(Debug, Copy, Clone)]
struct ListInfo {
	position: usize,
	current: usize,
	max: Option<usize>,
}

impl ListInfo {
	fn new(position: usize, max: Option<usize>) -> ListInfo {
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

impl Default for RlpStream {
	fn default() -> Self {
		RlpStream::new()
	}
}

impl RlpStream {
	/// Initializes instance of empty `Stream`.
	pub fn new() -> Self {
		RlpStream {
			unfinished_lists: ElasticArray16::new(),
			buffer: ElasticArray1024::new(),
			finished_list: false,
		}
	}

	/// Initializes the `Stream` as a list.
	pub fn new_list(len: usize) -> Self {
		let mut stream = RlpStream::new();
		stream.begin_list(len);
		stream
	}

	/// Apends null to the end of stream, chainable.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append_empty_data().append_empty_data();
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc2, 0x80, 0x80]);
	/// }
	/// ```
	pub fn append_empty_data(&mut self) -> &mut Self {
		// self push raw item
		self.buffer.push(0x80);

		// try to finish and prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	/// Drain the object and return the underlying ElasticArray. Panics if it is not finished.
	pub fn drain(self) -> ElasticArray1024<u8> {
		match self.is_finished() {
			true => self.buffer,
			false => panic!()
		}
	}

	/// Appends raw (pre-serialised) RLP data. Use with caution. Chainable.
	pub fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self {
		// push raw items
		self.buffer.append_slice(bytes);

		// try to finish and prepend the length
		self.note_appended(item_count);

		// return chainable self
		self
	}

	/// Appends value to the end of stream, chainable.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append(&"cat").append(&"dog");
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	pub fn append<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: Encodable {
		self.finished_list = false;
		value.rlp_append(self);
		if !self.finished_list {
			self.note_appended(1);
		}
		self
	}

	/// Appends list of values to the end of stream, chainable.
	pub fn append_list<'a, E, K>(&'a mut self, values: &[K]) -> &'a mut Self where E: Encodable, K: Borrow<E> {
		self.begin_list(values.len());
		for value in values {
			self.append(value.borrow());
		}
		self
	}

	/// Appends value to the end of stream, but do not count it as an appended item.
	/// It's useful for wrapper types
	pub fn append_internal<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: Encodable {
		value.rlp_append(self);
		self
	}

	/// Declare appending the list of given size, chainable.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.begin_list(2).append(&"cat").append(&"dog");
	/// 	stream.append(&"");
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xca, 0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g', 0x80]);
	/// }
	/// ```
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
				// payload is longer than 1 byte only for lists > 55 bytes
				// by pushing always this 1 byte we may avoid unnecessary shift of data
				self.buffer.push(0);

				let position = self.buffer.len();
				self.unfinished_lists.push(ListInfo::new(position, Some(len)));
			},
		}

		// return chainable self
		self
	}

	/// Declare appending the list of unknown size, chainable.
	pub fn begin_unbounded_list(&mut self) -> &mut RlpStream {
		self.finished_list = false;
		// payload is longer than 1 byte only for lists > 55 bytes
		// by pushing always this 1 byte we may avoid unnecessary shift of data
		self.buffer.push(0);
		let position = self.buffer.len();
		self.unfinished_lists.push(ListInfo::new(position, None));
		// return chainable self
		self
	}

	/// Appends raw (pre-serialised) RLP data. Checks for size oveflow.
	pub fn append_raw_checked<'a>(&'a mut self, bytes: &[u8], item_count: usize, max_size: usize) -> bool {
		if self.estimate_size(bytes.len()) > max_size {
			return false;
		}
		self.append_raw(bytes, item_count);
		true
	}

	/// Calculate total RLP size for appended payload.
	pub fn estimate_size<'a>(&'a self, add: usize) -> usize {
		let total_size = self.buffer.len() + add;
		let mut base_size = total_size;
		for list in &self.unfinished_lists[..] {
			let len = total_size - list.position;
			if len > 55 {
				let leading_empty_bytes = (len as u64).leading_zeros() as usize / 8;
				let size_bytes = 8 - leading_empty_bytes;
				base_size += size_bytes;
			}
		}
		base_size
	}

	/// Returns current RLP size in bytes for the data pushed into the list.
	pub fn len<'a>(&'a self) -> usize {
		self.estimate_size(0)
	}

	/// Clear the output stream so far.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(3);
	/// 	stream.append(&"cat");
	/// 	stream.clear();
	/// 	stream.append(&"dog");
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0x83, b'd', b'o', b'g']);
	/// }
	pub fn clear(&mut self) {
		// clear bytes
		self.buffer.clear();

		// clear lists
		self.unfinished_lists.clear();
	}

	/// Returns true if stream doesnt expect any more items.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append(&"cat");
	/// 	assert_eq!(stream.is_finished(), false);
	/// 	stream.append(&"dog");
	/// 	assert_eq!(stream.is_finished(), true);
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	/// }
	pub fn is_finished(&self) -> bool {
		self.unfinished_lists.len() == 0
	}

	/// Get raw encoded bytes
	pub fn as_raw(&self) -> &[u8] {
		//&self.encoder.bytes
		&self.buffer
	}

	/// Streams out encoded bytes.
	///
	/// panic! if stream is not finished.
	pub fn out(self) -> Vec<u8> {
		match self.is_finished() {
			//true => self.encoder.out().into_vec(),
			true => self.buffer.into_vec(),
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
				match x.max {
					Some(ref max) if x.current > *max => panic!("You cannot append more items then you expect!"),
					Some(ref max) => x.current == *max,
					_ => false,
				}
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

	/// Finalize current ubnbound list. Panics if no unbounded list has been opened.
	pub fn complete_unbounded_list(&mut self) {
		let list = self.unfinished_lists.pop().expect("No open list.");
		if list.max.is_some() {
			panic!("List type mismatch.");
		}
		let len = self.buffer.len() - list.position;
		self.encoder().insert_list_payload(len, list.position);
		self.note_appended(1);
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

	fn insert_size(&mut self, size: usize, position: usize) -> u8 {
		let size = size as u32;
		let leading_empty_bytes = size.leading_zeros() as usize / 8;
		let size_bytes = 4 - leading_empty_bytes as u8;
		let mut buffer = [0u8; 4];
		BigEndian::write_u32(&mut buffer, size);
		self.buffer.insert_slice(position, &buffer[leading_empty_bytes..]);
		size_bytes as u8
	}

	/// Inserts list prefix at given position
	fn insert_list_payload(&mut self, len: usize, pos: usize) {
		// 1 byte was already reserved for payload earlier
		match len {
			0...55 => {
				self.buffer[pos - 1] = 0xc0u8 + len as u8;
			},
			_ => {
				let inserted_bytes = self.insert_size(len, pos);
				self.buffer[pos - 1] = 0xf7u8 + inserted_bytes;
			}
		};
	}

	/// Pushes encoded value to the end of buffer
	pub fn encode_value(&mut self, value: &[u8]) {
		match value.len() {
			// just 0
			0 => self.buffer.push(0x80u8),
			// byte is its own encoding if < 0x80
			1 if value[0] < 0x80 => self.buffer.push(value[0]),
			// (prefix + length), followed by the string
			len @ 1 ... 55 => {
				self.buffer.push(0x80u8 + len as u8);
				self.buffer.append_slice(value);
			}
			// (prefix + length of length), followed by the length, followd by the string
			len => {
				self.buffer.push(0);
				let position = self.buffer.len();
				let inserted_bytes = self.insert_size(len, position);
				self.buffer[position - 1] = 0xb7 + inserted_bytes;
				self.buffer.append_slice(value);
			}
		}
	}
}
