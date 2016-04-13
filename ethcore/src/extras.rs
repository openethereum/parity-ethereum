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

//! Blockchain DB extras.

use util::*;
use header::BlockNumber;
use receipt::Receipt;

/// Represents index of extra data in database
#[derive(Copy, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExtrasIndex {
	/// Block details index
	BlockDetails = 0,
	/// Block hash index
	BlockHash = 1,
	/// Transaction address index
	TransactionAddress = 2,
	/// Block log blooms index
	BlockLogBlooms = 3,
	/// Block blooms index
	BlocksBlooms = 4,
	/// Block receipts index
	BlockReceipts = 5,
}

/// trait used to write Extras data to db
pub trait ExtrasWritable {
	/// Write extra data to db
	fn put_extras<K, T>(&self, hash: &K, value: &T)
		where T: ExtrasIndexable + Encodable,
		      K: ExtrasSliceConvertable;
}

/// trait used to read Extras data from db
pub trait ExtrasReadable {
	/// Read extra data from db
	fn get_extras<K, T>(&self, hash: &K) -> Option<T>
		where T: ExtrasIndexable + Decodable,
		      K: ExtrasSliceConvertable;

	/// Check if extra data exists in the db
	fn extras_exists<K, T>(&self, hash: &K) -> bool
		where T: ExtrasIndexable,
		      K: ExtrasSliceConvertable;
}

impl ExtrasWritable for DBTransaction {
	fn put_extras<K, T>(&self, hash: &K, value: &T)
		where T: ExtrasIndexable + Encodable,
		      K: ExtrasSliceConvertable,
	{

		self.put(&hash.to_extras_slice(T::extras_index()), &encode(value)).unwrap()
	}
}

impl ExtrasReadable for Database {
	fn get_extras<K, T>(&self, hash: &K) -> Option<T>
		where T: ExtrasIndexable + Decodable,
		      K: ExtrasSliceConvertable,
	{

		self.get(&hash.to_extras_slice(T::extras_index()))
		    .unwrap()
		    .map(|v| decode(&v))
	}

	fn extras_exists<K, T>(&self, hash: &K) -> bool
		where T: ExtrasIndexable,
		      K: ExtrasSliceConvertable,
	{

		self.get(&hash.to_extras_slice(T::extras_index())).unwrap().is_some()
	}
}

/// Implementations should convert arbitrary type to database key slice
pub trait ExtrasSliceConvertable {
	/// Convert self, with `i` (the index), to a 264-bit extras DB key.
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264;
	/// Interpret self as a 256-bit hash, if natively `H256`.
	fn as_h256(&self) -> Option<&H256> {
		None
	}
}

impl ExtrasSliceConvertable for H256 {
	fn to_extras_slice(&self, i: ExtrasIndex) -> H264 {
		let mut slice = H264::from_slice(self);
		slice[32] = i as u8;
		slice
	}
	fn as_h256(&self) -> Option<&H256> {
		Some(self)
	}
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
	/// Returns this data index
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
	/// Block number
	pub number: BlockNumber,
	/// Total difficulty of the block and all its parents
	pub total_difficulty: U256,
	/// Parent block hash
	pub parent: H256,
	/// List of children block hashes
	pub children: Vec<H256>,
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
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>
		where D: Decoder,
	{
		let d = decoder.as_rlp();
		let details = BlockDetails {
			number: try!(d.val_at(0)),
			total_difficulty: try!(d.val_at(1)),
			parent: try!(d.val_at(2)),
			children: try!(d.val_at(3)),
		};
		Ok(details)
	}
}

impl Encodable for BlockDetails {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.number);
		s.append(&self.total_difficulty);
		s.append(&self.parent);
		s.append(&self.children);
	}
}

/// Log blooms of certain block
#[derive(Clone)]
pub struct BlockLogBlooms {
	/// List of log blooms for the block
	pub blooms: Vec<H2048>,
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
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>
		where D: Decoder,
	{
		let block_blooms = BlockLogBlooms { blooms: try!(Decodable::decode(decoder)) };

		Ok(block_blooms)
	}
}

impl Encodable for BlockLogBlooms {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.blooms);
	}
}

/// Neighboring log blooms on certain level
pub struct BlocksBlooms {
	/// List of block blooms.
	pub blooms: [H2048; 16],
}

impl Default for BlocksBlooms {
	fn default() -> Self {
		BlocksBlooms::new()
	}
}

impl BlocksBlooms {
	pub fn new() -> Self {
		BlocksBlooms { blooms: unsafe { ::std::mem::zeroed() } }
	}
}

impl ExtrasIndexable for BlocksBlooms {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlocksBlooms
	}
}

impl HeapSizeOf for BlocksBlooms {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}

impl Clone for BlocksBlooms {
	fn clone(&self) -> Self {
		let mut blooms: [H2048; 16] = unsafe { ::std::mem::uninitialized() };

		for i in 0..self.blooms.len() {
			blooms[i] = self.blooms[i].clone();
		}

		BlocksBlooms { blooms: blooms }
	}
}

impl Decodable for BlocksBlooms {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>
		where D: Decoder,
	{
		let blocks_blooms = BlocksBlooms { blooms: try!(Decodable::decode(decoder)) };

		Ok(blocks_blooms)
	}
}

impl Encodable for BlocksBlooms {
	fn rlp_append(&self, s: &mut RlpStream) {
		let blooms_ref: &[H2048] = &self.blooms;
		s.append(&blooms_ref);
	}
}

/// Represents address of certain transaction within block
#[derive(Clone)]
pub struct TransactionAddress {
	/// Block hash
	pub block_hash: H256,
	/// Transaction index within the block
	pub index: usize,
}

impl ExtrasIndexable for TransactionAddress {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::TransactionAddress
	}
}

impl HeapSizeOf for TransactionAddress {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}

impl Decodable for TransactionAddress {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>
		where D: Decoder,
	{
		let d = decoder.as_rlp();
		let tx_address = TransactionAddress {
			block_hash: try!(d.val_at(0)),
			index: try!(d.val_at(1)),
		};

		Ok(tx_address)
	}
}

impl Encodable for TransactionAddress {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.block_hash);
		s.append(&self.index);
	}
}

/// Contains all block receipts.
#[derive(Clone)]
pub struct BlockReceipts {
	pub receipts: Vec<Receipt>,
}

impl BlockReceipts {
	pub fn new(receipts: Vec<Receipt>) -> Self {
		BlockReceipts { receipts: receipts }
	}
}

impl Decodable for BlockReceipts {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>
		where D: Decoder,
	{
		Ok(BlockReceipts { receipts: try!(Decodable::decode(decoder)) })
	}
}

impl Encodable for BlockReceipts {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.receipts);
	}
}

impl HeapSizeOf for BlockReceipts {
	fn heap_size_of_children(&self) -> usize {
		self.receipts.heap_size_of_children()
	}
}

impl ExtrasIndexable for BlockReceipts {
	fn extras_index() -> ExtrasIndex {
		ExtrasIndex::BlockReceipts
	}
}
