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

//! General error types for use in ethcore.

use std::{fmt, error};
use std::time::SystemTime;

use derive_more::Display;
use ethereum_types::{H256, U256, Address, Bloom};
use ethkey::Error as EthkeyError;
use ethtrie::TrieError;
use rlp;
use snappy::InvalidInput;
use snapshot::Error as SnapshotError;
use types::BlockNumber;
use types::transaction::Error as TransactionError;
use unexpected::{Mismatch, OutOfBounds};

use engines::EngineError;

pub use executed::{ExecutionError, CallError};

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

error_chain! {
	types {
		QueueError, QueueErrorKind, QueueErrorResultExt, QueueErrorResult;
	}

	errors {
		#[doc = "Queue is full"]
		Full(limit: usize) {
			description("Queue is full")
			display("The queue is full ({})", limit)
		}
	}

	foreign_links {
		Channel(::io::IoError) #[doc = "Io channel error"];
	}
}

error_chain! {
	types {
		ImportError, ImportErrorKind, ImportErrorResultExt, ImportErrorResult;
	}

	errors {
		#[doc = "Already in the block chain."]
		AlreadyInChain {
			description("Block already in chain")
			display("Block already in chain")
		}

		#[doc = "Already in the block queue"]
		AlreadyQueued {
			description("block already in the block queue")
			display("block already in the block queue")
		}

		#[doc = "Already marked as bad from a previous import (could mean parent is bad)."]
		KnownBad {
			description("block known to be bad")
			display("block known to be bad")
		}
	}
}

/// Api-level error for transaction import
#[derive(Debug, Clone)]
pub enum TransactionImportError {
	/// Transaction error
	Transaction(TransactionError),
	/// Other error
	Other(String),
}

impl From<Error> for TransactionImportError {
	fn from(e: Error) -> Self {
		match e {
			Error(ErrorKind::Transaction(transaction_error), _) => TransactionImportError::Transaction(transaction_error),
			_ => TransactionImportError::Other(format!("other block import error: {:?}", e)),
		}
	}
}

error_chain! {
	types {
		Error, ErrorKind, ErrorResultExt, EthcoreResult;
	}

	links {
		Import(ImportError, ImportErrorKind) #[doc = "Error concerning block import." ];
		Queue(QueueError, QueueErrorKind) #[doc = "Io channel queue error"];
	}

	foreign_links {
		Io(::io::IoError) #[doc = "Io create error"];
		StdIo(::std::io::Error) #[doc = "Error concerning the Rust standard library's IO subsystem."];
		Trie(TrieError) #[doc = "Error concerning TrieDBs."];
		Execution(ExecutionError) #[doc = "Error concerning EVM code execution."];
		Block(BlockError) #[doc = "Error concerning block processing."];
		Transaction(TransactionError) #[doc = "Error concerning transaction processing."];
		Snappy(InvalidInput) #[doc = "Snappy error."];
		Engine(EngineError) #[doc = "Consensus vote error."];
		Ethkey(EthkeyError) #[doc = "Ethkey error."];
		Decoder(rlp::DecoderError) #[doc = "RLP decoding errors"];
	}

	errors {
		#[doc = "Snapshot error."]
		Snapshot(err: SnapshotError) {
			description("Snapshot error.")
			display("Snapshot error {}", err)
		}

		#[doc = "PoW hash is invalid or out of date."]
		PowHashInvalid {
			description("PoW hash is invalid or out of date.")
			display("PoW hash is invalid or out of date.")
		}

		#[doc = "The value of the nonce or mishash is invalid."]
		PowInvalid {
			description("The value of the nonce or mishash is invalid.")
			display("The value of the nonce or mishash is invalid.")
		}

		#[doc = "Unknown engine given"]
		UnknownEngineName(name: String) {
			description("Unknown engine name")
			display("Unknown engine name ({})", name)
		}
	}
}

impl From<SnapshotError> for Error {
	fn from(err: SnapshotError) -> Error {
		match err {
			SnapshotError::Io(err) => ErrorKind::StdIo(err).into(),
			SnapshotError::Trie(err) => ErrorKind::Trie(err).into(),
			SnapshotError::Decoder(err) => err.into(),
			other => ErrorKind::Snapshot(other).into(),
		}
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Error {
		Error::from(*err)
	}
}
