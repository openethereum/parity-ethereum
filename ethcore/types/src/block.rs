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
//! To create a block locally, we start with an `OpenBlock`. This block is mutable
//! and can be appended to with transactions and uncles.
//!
//! When ready, `OpenBlock` can be closed and turned into a `ClosedBlock`. A `ClosedBlock` can
//! be re-opend again by a miner under certain circumstances. On block close, state commit is
//! performed.
//!
//! `LockedBlock` is a version of a `ClosedBlock` that cannot be reopened. It can be sealed
//! using an engine.
//!
//! `ExecutedBlock` is an underlying data structure used by all structs above to store block
//! related info.
use std::{
	fmt,
	error,
	time::SystemTime
};

use bytes::Bytes;
use derive_more::{Display, From};
use parity_util_mem::MallocSizeOf;

use header::Header;
use rlp::{Rlp, RlpStream, Decodable, DecoderError};
use transaction::{UnverifiedTransaction, SignedTransaction};
use unexpected::{Mismatch, OutOfBounds};
use BlockNumber;
use ethbloom::Bloom;
use ethereum_types::{H256, U256, Address};

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

/// Errors concerning block processing.
#[derive(Debug, Display, PartialEq, Clone, Eq)]
pub enum BlockError {
	/// Block has too many uncles.
	#[display(fmt = "Block has too many uncles. {}", _0)]
	TooManyUncles(OutOfBounds<usize>),
	/// Extra data is of an invalid length.
	#[display(fmt = "Extra block data too long. {}", _0)]
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	/// Seal is incorrect format.
	#[display(fmt = "Block seal in incorrect format: {}", _0)]
	InvalidSealArity(Mismatch<usize>),
	/// Block has too much gas used.
	#[display(fmt = "Block has too much gas used. {}", _0)]
	TooMuchGasUsed(OutOfBounds<U256>),
	/// Uncles hash in header is invalid.
	#[display(fmt = "Block has invalid uncles hash: {}", _0)]
	InvalidUnclesHash(Mismatch<H256>),
	/// An uncle is from a generation too old.
	#[display(fmt = "Uncle block is too old. {}", _0)]
	UncleTooOld(OutOfBounds<BlockNumber>),
	/// An uncle is from the same generation as the block.
	#[display(fmt = "Uncle from same generation as block. {}", _0)]
	UncleIsBrother(OutOfBounds<BlockNumber>),
	/// An uncle is already in the chain.
	#[display(fmt = "Uncle {} already in chain", _0)]
	UncleInChain(H256),
	/// An uncle is included twice.
	#[display(fmt = "Uncle {} already in the header", _0)]
	DuplicateUncle(H256),
	/// An uncle has a parent not in the chain.
	#[display(fmt = "Uncle {} has a parent not in the chain", _0)]
	UncleParentNotInChain(H256),
	/// State root header field is invalid.
	#[display(fmt = "Invalid state root in header: {}", _0)]
	InvalidStateRoot(Mismatch<H256>),
	/// Gas used header field is invalid.
	#[display(fmt = "Invalid gas used in header: {}", _0)]
	InvalidGasUsed(Mismatch<U256>),
	/// Transactions root header field is invalid.
	#[display(fmt = "Invalid transactions root in header: {}", _0)]
	InvalidTransactionsRoot(Mismatch<H256>),
	/// Difficulty is out of range; this can be used as an looser error prior to getting a definitive
	/// value for difficulty. This error needs only provide bounds of which it is out.
	#[display(fmt = "Difficulty out of bounds: {}", _0)]
	DifficultyOutOfBounds(OutOfBounds<U256>),
	/// Difficulty header field is invalid; this is a strong error used after getting a definitive
	/// value for difficulty (which is provided).
	#[display(fmt = "Invalid block difficulty: {}", _0)]
	InvalidDifficulty(Mismatch<U256>),
	/// Seal element of type H256 (max_hash for Ethash, but could be something else for
	/// other seal engines) is out of bounds.
	#[display(fmt = "Seal element out of bounds: {}", _0)]
	MismatchedH256SealElement(Mismatch<H256>),
	/// Proof-of-work aspect of seal, which we assume is a 256-bit value, is invalid.
	#[display(fmt = "Block has invalid PoW: {}", _0)]
	InvalidProofOfWork(OutOfBounds<U256>),
	/// Some low-level aspect of the seal is incorrect.
	#[display(fmt = "Block has invalid seal.")]
	InvalidSeal,
	/// Gas limit header field is invalid.
	#[display(fmt = "Invalid gas limit: {}", _0)]
	InvalidGasLimit(OutOfBounds<U256>),
	/// Receipts trie root header field is invalid.
	#[display(fmt = "Invalid receipts trie root in header: {}", _0)]
	InvalidReceiptsRoot(Mismatch<H256>),
	/// Timestamp header field is invalid.
	#[display(fmt = "Invalid timestamp in header: {}", _0)]
	InvalidTimestamp(OutOfBoundsTime),
	/// Timestamp header field is too far in future.
	#[display(fmt = "Future timestamp in header: {}", _0)]
	TemporarilyInvalid(OutOfBoundsTime),
	/// Log bloom header field is invalid.
	#[display(fmt = "Invalid log bloom in header: {}", _0)]
	InvalidLogBloom(Box<Mismatch<Bloom>>),
	/// Number field of header is invalid.
	#[display(fmt = "Invalid number in header: {}", _0)]
	InvalidNumber(Mismatch<BlockNumber>),
	/// Block number isn't sensible.
	#[display(fmt = "Implausible block number. {}", _0)]
	RidiculousNumber(OutOfBounds<BlockNumber>),
	/// Timestamp header overflowed
	#[display(fmt = "Timestamp overflow")]
	TimestampOverflow,
	/// Too many transactions from a particular address.
	#[display(fmt = "Too many transactions from: {}", _0)]
	TooManyTransactions(Address),
	/// Parent given is unknown.
	#[display(fmt = "Unknown parent: {}", _0)]
	UnknownParent(H256),
	/// Uncle parent given is unknown.
	#[display(fmt = "Unknown uncle parent: {}", _0)]
	UnknownUncleParent(H256),
	/// No transition to epoch number.
	#[display(fmt = "Unknown transition to epoch number: {}", _0)]
	UnknownEpochTransition(u64),
}

/// Newtype for Display impl to show seconds
#[derive(Debug, Clone, From, PartialEq, Eq)]
pub struct OutOfBoundsTime(OutOfBounds<SystemTime>);

impl fmt::Display for OutOfBoundsTime {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let seconds = self.0
			.map(|st| st.elapsed().unwrap_or_default().as_secs());
		f.write_fmt(format_args!("{}", seconds))
	}
}

impl error::Error for BlockError {
	fn description(&self) -> &str {
		"Block error"
	}
}
