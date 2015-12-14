use std::collections::HashMap;
use std::sync::RwLock;
use std::ops::Deref;
use std::hash::Hash;
use util::uint::*;
use util::hash::*;
use util::rlp::*;

/// workaround for lack of integer templates in Rust
#[derive(Copy, Clone)]
pub enum ExtrasIndex {
	BlockDetails = 0,
	BlockHash = 1,
}

/// rw locked extra data with slice suffix
// consifer if arc needed here, since blockchain itself will be wrapped
pub struct Extras<K, T>(RwLock<HashMap<K, T>>, ExtrasIndex) where K: Eq + Hash;

impl<K, T> Extras<K, T> where K: Eq + Hash {
	pub fn new(i: ExtrasIndex) -> Extras<K, T> {
		Extras(RwLock::new(HashMap::new()), i)
	}

	pub fn index(&self) -> ExtrasIndex { self.1 }
}

impl<K, T> Deref for Extras<K, T> where K : Eq + Hash {
	type Target = RwLock<HashMap<K, T>>;

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
