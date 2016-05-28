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

use bloomchain;
use util::*;
use header::BlockNumber;
use receipt::Receipt;
use db::Key;
use blooms::{GroupPosition, BloomGroup};

/// Represents index of extra data in database
#[derive(Copy, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExtrasIndex {
	/// Block details index
	BlockDetails = 0,
	/// Block hash index
	BlockHash = 1,
	/// Transaction address index
	TransactionAddress = 2,
	/// Block blooms index
	BlocksBlooms = 3,
	/// Block receipts index
	BlockReceipts = 4,
}

fn with_index(hash: &H256, i: ExtrasIndex) -> H264 {
	let mut result = H264::default();
	result[0] = i as u8;
	result.deref_mut()[1..].clone_from_slice(hash);
	result
}

pub struct BlockNumberKey([u8; 5]);

impl Deref for BlockNumberKey {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Key<H256> for BlockNumber {
	type Target = BlockNumberKey;

	fn key(&self) -> Self::Target {
		let mut result = [0u8; 5];
		result[0] = ExtrasIndex::BlockHash as u8;
		result[1] = (self >> 24) as u8;
		result[2] = (self >> 16) as u8;
		result[3] = (self >> 8) as u8;
		result[4] = *self as u8;
		BlockNumberKey(result)
	}
}

impl Key<BlockDetails> for H256 {
	type Target = H264;

	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::BlockDetails)
	}
}

pub struct LogGroupKey([u8; 6]);

impl Deref for LogGroupKey {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct LogGroupPosition(GroupPosition);

impl From<bloomchain::group::GroupPosition> for LogGroupPosition {
	fn from(position: bloomchain::group::GroupPosition) -> Self {
		LogGroupPosition(From::from(position))
	}
}

impl HeapSizeOf for LogGroupPosition {
	fn heap_size_of_children(&self) -> usize {
		self.0.heap_size_of_children()
	}
}

impl Key<BloomGroup> for LogGroupPosition {
	type Target = LogGroupKey;

	fn key(&self) -> Self::Target {
		let mut result = [0u8; 6];
		result[0] = ExtrasIndex::BlocksBlooms as u8;
		result[1] = self.0.level;
		result[2] = (self.0.index >> 24) as u8;
		result[3] = (self.0.index >> 16) as u8;
		result[4] = (self.0.index >> 8) as u8;
		result[5] = self.0.index as u8;
		LogGroupKey(result)
	}
}

impl Key<TransactionAddress> for H256 {
	type Target = H264;

	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::TransactionAddress)
	}
}

impl Key<BlockReceipts> for H256 {
	type Target = H264;

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
