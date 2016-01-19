use util::*;
use header::BlockNumber;
use rocksdb::{DB, Writable};

/// Represents index of extra data in database
#[derive(Copy, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExtrasIndex {
	BlockDetails = 0,
	BlockHash = 1,
	TransactionAddress = 2,
	BlockLogBlooms = 3,
	BlocksBlooms = 4
} 

/// trait used to write Extras data to db
pub trait ExtrasWritable {
	fn put_extras<K, T>(&self, hash: &K, value: &T) where
		T: ExtrasIndexable + Encodable, 
		K: ExtrasSliceConvertable;
}

/// trait used to read Extras data from db
pub trait ExtrasReadable {
	fn get_extras<K, T>(&self, hash: &K) -> Option<T> where
		T: ExtrasIndexable + Decodable,
		K: ExtrasSliceConvertable;

	fn extras_exists<K, T>(&self, hash: &K) -> bool where
		T: ExtrasIndexable,
		K: ExtrasSliceConvertable;
}

impl<W> ExtrasWritable for W where W: Writable {
	fn put_extras<K, T>(&self, hash: &K, value: &T) where
		T: ExtrasIndexable + Encodable, 
		K: ExtrasSliceConvertable {
		
		self.put(&hash.to_extras_slice(T::extras_index()), &encode(value)).unwrap()
	}
}

impl ExtrasReadable for DB {
	fn get_extras<K, T>(&self, hash: &K) -> Option<T> where
		T: ExtrasIndexable + Decodable,
		K: ExtrasSliceConvertable {

		self.get(&hash.to_extras_slice(T::extras_index())).unwrap()
			.map(|v| decode(&v))
	}

	fn extras_exists<K, T>(&self, hash: &K) -> bool where
		T: ExtrasIndexable,
		K: ExtrasSliceConvertable {

		self.get(&hash.to_extras_slice(T::extras_index())).unwrap().is_some()
	}
}

/// Implementations should convert arbitrary type to database key slice
pub trait ExtrasSliceConvertable {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264;
	fn as_h256(&self) -> Option<&H256> { None }
}

impl ExtrasSliceConvertable for H256 {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		let mut slice = H264::from_slice(self);
		slice[32] = i as u8;
		slice
	}
	fn as_h256(&self) -> Option<&H256> { Some(self) }
}

impl ExtrasSliceConvertable for U256 {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		H256::from(self).to_extras_slice(i)
	}
}

// NICE: make less horrible.
impl ExtrasSliceConvertable for BlockNumber {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		U256::from(*self).to_extras_slice(i)
	}
}

/// Types implementing this trait can be indexed in extras database
pub trait ExtrasIndexable {
	fn extras_index() -> ExtrasIndex;
}

impl ExtrasIndexable for H256 {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlockHash
	}
}

/// Familial details concerning a block
#[derive(Debug, Clone)]
pub struct BlockDetails {
	pub number: BlockNumber,
	pub total_difficulty: U256,
	pub parent: H256,
	pub children: Vec<H256>
}

impl ExtrasIndexable for BlockDetails {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlockDetails
	}
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

/// Log blooms of certain block
#[derive(Clone)]
pub struct BlockLogBlooms {
	pub blooms: Vec<H2048>
}

impl ExtrasIndexable for BlockLogBlooms {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlockLogBlooms
	}
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

/// Neighboring log blooms on certain level
pub struct BlocksBlooms {
	pub blooms: [H2048; 16]
}

impl ExtrasIndexable for BlocksBlooms {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlocksBlooms
	}
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

/// Represents address of certain transaction within block
#[derive(Clone)]
pub struct TransactionAddress {
	pub block_hash: H256,
	pub index: u64
}

impl ExtrasIndexable for TransactionAddress {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::TransactionAddress
	}
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
