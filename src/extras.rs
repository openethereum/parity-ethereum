use std::collections::HashMap;
use std::cell::RefCell;
use std::ops::Deref;
use std::hash::Hash;
use heapsize::HeapSizeOf;
use util::uint::*;
use util::hash::*;
use util::rlp::*;

/// workaround for lack of integer templates in Rust
#[derive(Copy, Clone)]
pub enum ExtrasIndex {
	BlockDetails = 0,
	BlockHash = 1,
	TransactionAddress = 2,
	BlockLogBlooms = 3,
	BlocksBlooms = 4
} 

/// rw locked extra data with slice suffix
// consifer if arc needed here, since blockchain itself will be wrapped
pub struct Extras<K, T>(RefCell<HashMap<K, T>>, ExtrasIndex) where K: Eq + Hash;

impl<K, T> Extras<K, T> where K: Eq + Hash {
	pub fn new(i: ExtrasIndex) -> Extras<K, T> {
		Extras(RefCell::new(HashMap::new()), i)
	}

	pub fn index(&self) -> ExtrasIndex { self.1 }
}

impl<K, T> Deref for Extras<K, T> where K : Eq + Hash {
	type Target = RefCell<HashMap<K, T>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub trait ExtrasSliceConvertable {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264;
}

impl ExtrasSliceConvertable for H256 {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		let mut slice = H264::from_slice(self);
		slice[32] = i as u8;
		slice
	}
}

impl ExtrasSliceConvertable for U256 {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		H256::from(self).to_extras_slice(i)
	}
}


#[derive(Clone)]
pub struct BlockDetails {
	pub number: U256,
	pub total_difficulty: U256,
	pub parent: H256,
	pub children: Vec<H256>
}

impl HeapSizeOf for BlockDetails {
	fn heap_size_of_children(&self) -> usize {
		self.children.heap_size_of_children()
	}
}

impl Decodable for BlockDetails {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = try!(decoder.as_list());
		let details = BlockDetails {
			number: try!(Decodable::decode(&d[0])),
			total_difficulty: try!(Decodable::decode(&d[1])),
			parent: try!(Decodable::decode(&d[2])),
			children: try!(Decodable::decode(&d[3]))
		};
		Ok(details)
	}
}

impl Encodable for BlockDetails {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.number.encode(e);
			self.total_difficulty.encode(e);
			self.parent.encode(e);
			self.children.encode(e);
		})
	}
}

#[derive(Clone)]
pub struct BlockLogBlooms {
	pub blooms: Vec<H2048>
}

impl HeapSizeOf for BlockLogBlooms {
	fn heap_size_of_children(&self) -> usize {
		self.blooms.heap_size_of_children()
	}
}

impl Decodable for BlockLogBlooms {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let block_blooms = BlockLogBlooms {
			blooms: try!(Decodable::decode(decoder))
		};

		Ok(block_blooms)
	}
}

impl Encodable for BlockLogBlooms {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		self.blooms.encode(encoder);
	}
}

pub struct BlocksBlooms {
	pub blooms: [H2048; 16]
}

impl HeapSizeOf for BlocksBlooms {
	fn heap_size_of_children(&self) -> usize { 0 }
}

impl Clone for BlocksBlooms {
	fn clone(&self) -> Self {
		let mut blooms: [H2048; 16] = unsafe { ::std::mem::uninitialized() };

		for i in 0..self.blooms.len() {
			blooms[i] = self.blooms[i].clone();
		}

		BlocksBlooms {
			blooms: blooms
		}
	}
}

impl Decodable for BlocksBlooms {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let blocks_blooms = BlocksBlooms {
			blooms: try!(Decodable::decode(decoder))
		};

		Ok(blocks_blooms)
	}
}

impl Encodable for BlocksBlooms {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		let blooms_ref: &[H2048] = &self.blooms;
		blooms_ref.encode(encoder);
	}
}

#[derive(Clone)]
pub struct TransactionAddress {
	pub block_hash: H256,
	pub index: u64
}

impl HeapSizeOf for TransactionAddress {
	fn heap_size_of_children(&self) -> usize { 0 }
}

impl Decodable for TransactionAddress {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = try!(decoder.as_list());
		let tx_address = TransactionAddress {
			block_hash: try!(Decodable::decode(&d[0])),
			index: try!(Decodable::decode(&d[1]))
		};

		Ok(tx_address)
	}
}

impl Encodable for TransactionAddress {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.block_hash.encode(e);
			self.index.encode(e);
		})
	}
}
