// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Blockchain DB extras.

use std::io::Write;
use std::convert::AsRef;

use common_types::BlockNumber;
use common_types::engines::epoch::Transition as EpochTransition;
use common_types::receipt::Receipt;
use ethereum_types::{H256, H264, U256};
use heapsize::HeapSizeOf;
use kvdb::PREFIX_LEN as DB_PREFIX_LEN;
use rlp;
use rlp_derive::{RlpEncodableWrapper, RlpDecodableWrapper, RlpEncodable, RlpDecodable};

use crate::db::Key;

/// Represents index of extra data in database
#[derive(Copy, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExtrasIndex {
	/// Block details index
	BlockDetails = 0,
	/// Block hash index
	BlockHash = 1,
	/// Transaction address index
	TransactionAddress = 2,
	/// Block receipts index
	BlockReceipts = 4,
	/// Epoch transition data index.
	EpochTransitions = 5,
	/// Pending epoch transition data index.
	PendingEpochTransition = 6,
}

fn with_index(hash: &H256, i: ExtrasIndex) -> H264 {
	let mut result = H264::default();
	result.as_bytes_mut()[0] = i as u8;
	result.as_bytes_mut()[1..].clone_from_slice(hash.as_bytes());
	result
}

/// Wrapper for block number used as a DB key.
pub struct BlockNumberKey([u8; 5]);

impl AsRef<[u8]> for BlockNumberKey {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
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

impl Key<common_types::engines::epoch::PendingTransition> for H256 {
	type Target = H264;

	fn key(&self) -> H264 {
		with_index(self, ExtrasIndex::PendingEpochTransition)
	}
}

/// length of epoch keys.
pub const EPOCH_KEY_LEN: usize = DB_PREFIX_LEN + 16;

/// epoch key prefix.
/// used to iterate over all epoch transitions in order from genesis.
pub const EPOCH_KEY_PREFIX: &'static [u8; DB_PREFIX_LEN] = &[
	ExtrasIndex::EpochTransitions as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Epoch transitions key
pub struct EpochTransitionsKey([u8; EPOCH_KEY_LEN]);

impl AsRef<[u8]> for EpochTransitionsKey {
	fn as_ref(&self) -> &[u8] { &self.0[..] }
}

impl Key<EpochTransitions> for u64 {
	type Target = EpochTransitionsKey;

	fn key(&self) -> Self::Target {
		let mut arr = [0u8; EPOCH_KEY_LEN];
		arr[..DB_PREFIX_LEN].copy_from_slice(&EPOCH_KEY_PREFIX[..]);

		write!(&mut arr[DB_PREFIX_LEN..], "{:016x}", self)
			.expect("format arg is valid; no more than 16 chars will be written; qed");

		EpochTransitionsKey(arr)
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
	/// Whether the block is considered finalized
	pub is_finalized: bool,
}

impl rlp::Encodable for BlockDetails {
	fn rlp_append(&self, stream: &mut rlp::RlpStream) {
		let use_short_version = !self.is_finalized;

		match use_short_version {
			true => { stream.begin_list(4); },
			false => { stream.begin_list(5); },
		}

		stream.append(&self.number);
		stream.append(&self.total_difficulty);
		stream.append(&self.parent);
		stream.append_list(&self.children);
		if !use_short_version {
			stream.append(&self.is_finalized);
		}
	}
}

impl rlp::Decodable for BlockDetails {
	fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
		let use_short_version = match rlp.item_count()? {
			4 => true,
			5 => false,
			_ => return Err(rlp::DecoderError::RlpIncorrectListLen),
		};

		Ok(BlockDetails {
			number: rlp.val_at(0)?,
			total_difficulty: rlp.val_at(1)?,
			parent: rlp.val_at(2)?,
			children: rlp.list_at(3)?,
			is_finalized: if use_short_version {
				false
			} else {
				rlp.val_at(4)?
			},
		})
	}
}

impl HeapSizeOf for BlockDetails {
	fn heap_size_of_children(&self) -> usize {
		self.children.heap_size_of_children()
	}
}

/// Represents address of certain transaction within block
#[derive(Debug, PartialEq, Clone, RlpEncodable, RlpDecodable)]
pub struct TransactionAddress {
	/// Block hash
	pub block_hash: H256,
	/// Transaction index within the block
	pub index: usize
}

impl HeapSizeOf for TransactionAddress {
	fn heap_size_of_children(&self) -> usize { 0 }
}

/// Contains all block receipts.
#[derive(Clone, RlpEncodableWrapper, RlpDecodableWrapper)]
pub struct BlockReceipts {
	/// Block receipts
	pub receipts: Vec<Receipt>,
}

impl BlockReceipts {
	/// Create new block receipts wrapper.
	pub fn new(receipts: Vec<Receipt>) -> Self {
		BlockReceipts {
			receipts: receipts
		}
	}
}

impl HeapSizeOf for BlockReceipts {
	fn heap_size_of_children(&self) -> usize {
		self.receipts.heap_size_of_children()
	}
}

/// Candidate transitions to an epoch with specific number.
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct EpochTransitions {
	/// Epoch number
	pub number: u64,
	/// List of candidate transitions
	pub candidates: Vec<EpochTransition>,
}

#[cfg(test)]
mod tests {
	use rlp::*;

	use super::BlockReceipts;

	#[test]
	fn encode_block_receipts() {
		let br = BlockReceipts::new(Vec::new());

		let mut s = RlpStream::new_list(2);
		s.append(&br);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&br);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}
}
