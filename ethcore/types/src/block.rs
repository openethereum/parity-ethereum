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

//! Base data structure of this module is `Block`.
//!
//! Blocks can be produced by a local node or they may be received from the network.
//!
//! Other block types are found in `ethcore`

use bytes::Bytes;
use ethereum_types::{H256, U256};
use parity_util_mem::MallocSizeOf;

use BlockNumber;
use header::Header;
use rlp::{Rlp, RlpStream, Decodable, DecoderError};
use transaction::{UnverifiedTransaction, SignedTransaction};

/// A block, encoded as it is on the block chain.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Block {
	/// The header of this block.
	pub header: Header,
	/// The transactions in this block.
	pub transactions: Vec<UnverifiedTransaction>,
	/// The uncles of this block.
	pub uncles: Vec<Header>,
}

impl Block {
	/// Get the RLP-encoding of the block with the seal.
	pub fn rlp_bytes(&self) -> Bytes {
		let mut block_rlp = RlpStream::new_list(3);
		block_rlp.append(&self.header);
		block_rlp.append_list(&self.transactions);
		block_rlp.append_list(&self.uncles);
		block_rlp.out()
	}
}

impl Decodable for Block {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		if rlp.as_raw().len() != rlp.payload_info()?.total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		if rlp.item_count()? != 3 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(Block {
			header: rlp.val_at(0)?,
			transactions: rlp.list_at(1)?,
			uncles: rlp.list_at(2)?,
		})
	}
}

/// Preprocessed block data gathered in `verify_block_unordered` call
#[derive(MallocSizeOf)]
pub struct PreverifiedBlock {
	/// Populated block header
	pub header: Header,
	/// Populated block transactions
	pub transactions: Vec<SignedTransaction>,
	/// Populated block uncles
	pub uncles: Vec<Header>,
	/// Block bytes
	pub bytes: Bytes,
}

/// Brief info about inserted block.
#[derive(Clone)]
pub struct BlockInfo {
	/// Block hash.
	pub hash: H256,
	/// Block number.
	pub number: BlockNumber,
	/// Total block difficulty.
	pub total_difficulty: U256,
	/// Block location in blockchain.
	pub location: BlockLocation
}

/// Describes location of newly inserted block.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockLocation {
	/// It's part of the canon chain.
	CanonChain,
	/// It's not a part of the canon chain.
	Branch,
	/// It's part of the fork which should become canon chain,
	/// because its total difficulty is higher than current
	/// canon chain difficulty.
	BranchBecomingCanonChain(BranchBecomingCanonChainData),
}

/// Info about heaviest fork
#[derive(Debug, Clone, PartialEq)]
pub struct BranchBecomingCanonChainData {
	/// Hash of the newest common ancestor with old canon chain.
	pub ancestor: H256,
	/// Hashes of the blocks between ancestor and this block.
	pub enacted: Vec<H256>,
	/// Hashes of the blocks which were invalidated.
	pub retracted: Vec<H256>,
}
