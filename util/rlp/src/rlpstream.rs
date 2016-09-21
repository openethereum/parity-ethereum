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

use elastic_array::*;

use ::{Stream, Encoder, Encodable};
use bytes::{ToBytes, VecLike};
use rlptraits::{ByteEncodable, RlpEncodable};

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
	encoder: BasicEncoder,
	finished_list: bool,
}

impl Default  for RlpStream {
	fn default() -> Self {
		RlpStream::new()
	}
}

impl Stream for RlpStream {
	fn new() -> Self {
		RlpStream {
			unfinished_lists: ElasticArray16::new(),
			encoder: BasicEncoder::new(),
			finished_list: false,
		}
	}

	fn new_list(len: usize) -> Self {
		let mut stream = RlpStream::new();
		stream.begin_list(len);
		stream
	}

	fn append<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: RlpEncodable {
		self.finished_list = false;
		value.rlp_append(self);
		if !self.finished_list {
			self.note_appended(1);
		}
		self
	}

	fn begin_list(&mut self, len: usize) -> &mut RlpStream {
		self.finished_list = false;
		match len {
			0 => {
				// we may finish, if the appended list len is equal 0
				self.encoder.bytes.push(0xc0u8);
				self.note_appended(1);
				self.finished_list = true;
			},
			_ => {
				let position = self.encoder.bytes.len();
				self.unfinished_lists.push(ListInfo::new(position, len));
			},
		}

		// return chainable self
		self
	}

	fn append_empty_data(&mut self) -> &mut RlpStream {
		// self push raw item
		self.encoder.bytes.push(0x80);

		// try to finish and prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut RlpStream {
		// push raw items
		self.encoder.bytes.append_slice(bytes);

		// try to finish and prepend the length
		self.note_appended(item_count);

		// return chainable self
		self
	}

	fn clear(&mut self) {
		// clear bytes
		self.encoder.bytes.clear();

		// clear lists
		self.unfinished_lists.clear();
	}

	fn is_finished(&self) -> bool {
		self.unfinished_lists.len() == 0
	}

	fn as_raw(&self) -> &[u8] {
		&self.encoder.bytes
	}

	fn out(self) -> Vec<u8> {
		match self.is_finished() {
			true => self.encoder.out().to_vec(),
			false => panic!()
		}
	}
}

impl RlpStream {

	/// Appends primitive value to the end of stream
	fn append_value<E>(&mut self, object: &E) where E: ByteEncodable {
		// encode given value and add it at the end of the stream
		self.encoder.emit_value(object);
	}

	fn append_internal<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: Encodable {
		self.finished_list = false;
		value.rlp_append(self);
		if !self.finished_list {
			self.note_appended(1);
		}
		self
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
			let len = self.encoder.bytes.len() - x.position;
			self.encoder.insert_list_len_at_pos(len, x.position);
			self.note_appended(1);
		}
		self.finished_list = should_finish;
	}

	/// Drain the object and return the underlying ElasticArray.
	pub fn drain(self) -> ElasticArray1024<u8> {
		match self.is_finished() {
			true => self.encoder.bytes,
			false => panic!()
		}
	}
}

struct BasicEncoder {
	bytes: ElasticArray1024<u8>,
}

impl Default for BasicEncoder {
	fn default() -> Self {
		BasicEncoder::new()
	}
}

impl BasicEncoder {
	fn new() -> Self {
		BasicEncoder { bytes: ElasticArray1024::new() }
	}

	/// inserts list prefix at given position
	/// TODO: optimise it further?
	fn insert_list_len_at_pos(&mut self, len: usize, pos: usize) -> () {
		let mut res = ElasticArray16::new();
		match len {
			0...55 => res.push(0xc0u8 + len as u8),
			_ => {
				res.push(0xf7u8 + len.to_bytes_len() as u8);
				ToBytes::to_bytes(&len, &mut res);
			}
		};

		self.bytes.insert_slice(pos, &res);
	}

	/// get encoded value
	fn out(self) -> ElasticArray1024<u8> {
		self.bytes
	}
}

impl Encoder for BasicEncoder {
	fn emit_value<E: ByteEncodable>(&mut self, value: &E) {
		match value.bytes_len() {
			// just 0
			0 => self.bytes.push(0x80u8),
			// byte is its own encoding if < 0x80
			1 => {
				value.to_bytes(&mut self.bytes);
				let len = self.bytes.len();
				let last_byte = self.bytes[len - 1];
				if last_byte >= 0x80 {
					self.bytes.push(last_byte);
					self.bytes[len - 1] = 0x81;
				}
			}
			// (prefix + length), followed by the string
			len @ 2 ... 55 => {
				self.bytes.push(0x80u8 + len as u8);
				value.to_bytes(&mut self.bytes);
			}
			// (prefix + length of length), followed by the length, followd by the string
			len => {
				self.bytes.push(0xb7 + len.to_bytes_len() as u8);
				ToBytes::to_bytes(&len, &mut self.bytes);
				value.to_bytes(&mut self.bytes);
			}
		}
	}

	fn emit_raw(&mut self, bytes: &[u8]) -> () {
		self.bytes.append_slice(bytes);
	}
}

impl<T> ByteEncodable for T where T: ToBytes {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		ToBytes::to_bytes(self, out)
	}

	fn bytes_len(&self) -> usize {
		ToBytes::to_bytes_len(self)
	}
}

struct U8Slice<'a>(&'a [u8]);

impl<'a> ByteEncodable for U8Slice<'a> {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		out.vec_extend(self.0)
	}

	fn bytes_len(&self) -> usize {
		self.0.len()
	}
}

impl<'a> Encodable for &'a[u8] {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_value(&U8Slice(self))
	}
}

impl Encodable for Vec<u8> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_value(&U8Slice(self))
	}
}

impl<T> Encodable for T where T: ByteEncodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_value(self)
	}
}

struct EncodableU8 (u8);

impl ByteEncodable for EncodableU8 {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		if self.0 != 0 {
			out.vec_push(self.0)
		}
	}

	fn bytes_len(&self) -> usize {
		match self.0 { 0 => 0, _ => 1 }
	}
}

impl RlpEncodable for u8 {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_value(&EncodableU8(*self))
	}
}

impl<'a, T> Encodable for &'a[T] where T: Encodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(self.len());
		for el in self.iter() {
			s.append_internal(el);
		}
	}
}

impl<T> Encodable for Vec<T>  where T: Encodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.as_slice(), s);
	}
}

impl<T> Encodable for Option<T>  where T: Encodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			None => { s.begin_list(0); },
			Some(ref x) => {
				s.begin_list(1);
				s.append_internal(x);
			}
		}
	}
}

impl<T> RlpEncodable for T where T: Encodable {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(self, s)
	}
}

