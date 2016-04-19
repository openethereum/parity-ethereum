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
use db::Key;

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

fn with_index(hash: &H256, i: ExtrasIndex) -> H264 {
	let mut slice = H264::from_slice(hash);
	slice[32] = i as u8;
	slice
}

pub trait ExtrasIndexable {
	fn index() -> ExtrasIndex;
}

impl ExtrasIndexable for H256 {
	fn index() -> ExtrasIndex {
		ExtrasIndex::BlockHash
	}
}

impl ExtrasIndexable for BlockDetails {
	fn index() -> ExtrasIndex {
		ExtrasIndex::BlockDetails
	}
}

impl ExtrasIndexable for TransactionAddress {
	fn index() -> ExtrasIndex {
		ExtrasIndex::TransactionAddress
	}
}

impl ExtrasIndexable for BlockLogBlooms {
	fn index() -> ExtrasIndex {
		ExtrasIndex::BlockLogBlooms
	}
}

impl ExtrasIndexable for BlocksBlooms {
	fn index() -> ExtrasIndex {
		ExtrasIndex::BlocksBlooms
	}
}

impl ExtrasIndexable for BlockReceipts {
	fn index() -> ExtrasIndex {
		ExtrasIndex::BlockReceipts
	}
}

impl Key<H256> for BlockNumber {
	fn key(&self) -> H264 {
		with_index(&H256::from(*self), ExtrasIndex::BlockHash)
	}
}

impl Key<BlockDetails> for H256 {
	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::BlockDetails)
	}
}

impl Key<TransactionAddress> for H256 {
	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::TransactionAddress)
	}
}

impl Key<BlockLogBlooms> for H256 {
	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::BlockLogBlooms)
	}
}

impl Key<BlocksBlooms> for H256 {
	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::BlocksBlooms)
	}
}

impl Key<BlockReceipts> for H256 {
	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::BlockReceipts)
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
	pub children: Vec<H256>
}

impl HeapSizeOf for BlockDetails {
	fn heap_size_of_children(&self) -> usize {
		self.children.heap_size_of_children()
	}
}

impl Decodable for BlockDetails {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
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
		BlocksBlooms { blooms: unsafe { ::std::mem::zeroed() }}
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
	pub index: usize
}

impl HeapSizeOf for TransactionAddress {
	fn heap_size_of_children(&self) -> usize { 0 }
}

impl Decodable for TransactionAddress {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
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
		BlockReceipts {
			receipts: receipts
		}
	}
}

impl Decodable for BlockReceipts {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Ok(BlockReceipts {
			receipts: try!(Decodable::decode(decoder))
		})
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
