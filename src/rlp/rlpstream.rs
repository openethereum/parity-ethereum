use elastic_array::*;
use bytes::ToBytes;
use rlp::{Stream, Encoder, Encodable};

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
}

impl Stream for RlpStream {
	fn new() -> Self {
		RlpStream {
			unfinished_lists: ElasticArray16::new(),
			encoder: BasicEncoder::new(),
		}
	}

	fn new_list(len: usize) -> Self {
		let mut stream = RlpStream::new();
		stream.append_list(len);
		stream
	}

	fn append<'a, E>(&'a mut self, object: &E) -> &'a mut RlpStream where E: Encodable {
		// encode given value and add it at the end of the stream
		object.encode(&mut self.encoder);

		// if list is finished, prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	fn append_list<'a>(&'a mut self, len: usize) -> &'a mut RlpStream {
		match len {
			0 => {
				// we may finish, if the appended list len is equal 0
				self.encoder.bytes.push(0xc0u8);
				self.note_appended(1);
			},
			_ => {
				let position = self.encoder.bytes.len();
				self.unfinished_lists.push(ListInfo::new(position, len));
			},
		}

		// return chainable self
		self
	}

	fn append_empty_data<'a>(&'a mut self) -> &'a mut RlpStream {
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

	fn raw(&self) -> &[u8] {
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
	}
}

struct BasicEncoder {
	bytes: ElasticArray1024<u8>,
}

impl BasicEncoder {
	fn new() -> BasicEncoder {
		BasicEncoder { bytes: ElasticArray1024::new() }
	}

	/// inserts list prefix at given position
	/// TODO: optimise it further?
	fn insert_list_len_at_pos(&mut self, len: usize, pos: usize) -> () {
		let mut res = vec![];
		match len {
			0...55 => res.push(0xc0u8 + len as u8),
			_ => {
				res.push(0xf7u8 + len.to_bytes_len() as u8);
				res.extend(len.to_bytes());
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
	fn emit_value(&mut self, bytes: &[u8]) -> () {
		match bytes.len() {
			// just 0
			0 => self.bytes.push(0x80u8),
			// byte is its own encoding
			1 if bytes[0] < 0x80 => self.bytes.append_slice(bytes),
			// (prefix + length), followed by the string
			len @ 1 ... 55 => {
				self.bytes.push(0x80u8 + len as u8);
				self.bytes.append_slice(bytes);
			}
			// (prefix + length of length), followed by the length, followd by the string
			len => {
				self.bytes.push(0xb7 + len.to_bytes_len() as u8);
				self.bytes.append_slice(&len.to_bytes());
				self.bytes.append_slice(bytes);
			}
		}
	}

	fn emit_raw(&mut self, bytes: &[u8]) -> () {
		self.bytes.append_slice(bytes);
	}

	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> () {
		// get len before inserting a list
		let before_len = self.bytes.len();

		// insert all list elements
		f(self);

		// get len after inserting a list
		let after_len = self.bytes.len();

		// diff is list len
		let list_len = after_len - before_len;
		self.insert_list_len_at_pos(list_len, before_len);
	}
}

impl<T> Encodable for T where T: ToBytes {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_value(&self.to_bytes())
	}
}

impl<'a, T> Encodable for &'a [T] where T: Encodable + 'a {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(|e| {
			// insert all list elements
			for el in self.iter() {
				el.encode(e);
			}
		})
	}
}

impl<T> Encodable for Vec<T> where T: Encodable {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		let r: &[T] = self.as_ref();
		r.encode(encoder)
	}
}

/// lets treat bytes differently than other lists
/// they are a single value
impl<'a> Encodable for &'a [u8] {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_value(self)
	}
}

/// lets treat bytes differently than other lists
/// they are a single value
impl Encodable for Vec<u8> {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_value(self)
	}
}

impl<T> Encodable for Option<T> where T: Encodable {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		match *self {
			Some(ref x) => x.encode(encoder),
			None => encoder.emit_value(&[])
		}
	}
}
