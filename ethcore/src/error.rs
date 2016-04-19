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

//! General error types for use in ethcore.

use util::*;
use header::BlockNumber;
use basic_types::LogBloom;

/// Result of executing the transaction.
#[derive(PartialEq, Debug)]
pub enum ExecutionError {
	/// Returned when there gas paid for transaction execution is
	/// lower than base gas required.
	NotEnoughBaseGas {
		/// Absolute minimum gas required.
		required: U256,
		/// Gas provided.
		got: U256
	},
	/// Returned when block (gas_used + gas) > gas_limit.
	///
	/// If gas =< gas_limit, upstream may try to execute the transaction
	/// in next block.
	BlockGasLimitReached {
		/// Gas limit of block for transaction.
		gas_limit: U256,
		/// Gas used in block prior to transaction.
		gas_used: U256,
		/// Amount of gas in block.
		gas: U256
	},
	/// Returned when transaction nonce does not match state nonce.
	InvalidNonce {
		/// Nonce expected.
		expected: U256,
		/// Nonce found.
		got: U256
	},
	/// Returned when cost of transaction (value + gas_price * gas) exceeds
	/// current sender balance.
	NotEnoughCash {
		/// Minimum required balance.
		required: U512,
		/// Actual balance.
		got: U512
	},
	/// Returned when internal evm error occurs.
	Internal
}

#[derive(Debug, PartialEq)]
/// Errors concerning transaction processing.
pub enum TransactionError {
	/// Transaction is already imported to the queue
	AlreadyImported,
	/// Transaction is not valid anymore (state already has higher nonce)
	Old,
	/// Transaction has too low fee
	/// (there is already a transaction with the same sender-nonce but higher gas price)
	TooCheapToReplace,
	/// Transaction was not imported to the queue because limit has been reached.
	LimitReached,
	/// Transaction's gas price is below threshold.
	InsufficientGasPrice {
		/// Minimal expected gas price
		minimal: U256,
		/// Transaction gas price
		got: U256,
	},
	/// Sender doesn't have enough funds to pay for this transaction
	InsufficientBalance {
		/// Senders balance
		balance: U256,
		/// Transaction cost
		cost: U256,
	},
	/// Transactions gas is higher then current gas limit
	GasLimitExceeded {
		/// Current gas limit
		limit: U256,
		/// Declared transaction gas
		got: U256,
	},
	/// Transaction's gas limit (aka gas) is invalid.
	InvalidGasLimit(OutOfBounds<U256>),
}

#[derive(Debug, PartialEq, Eq)]
/// Errors concerning block processing.
pub enum BlockError {
	/// Block has too many uncles.
	TooManyUncles(OutOfBounds<usize>),
	/// Extra data is of an invalid length.
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	/// Seal is incorrect format.
	InvalidSealArity(Mismatch<usize>),
	/// Block has too much gas used.
	TooMuchGasUsed(OutOfBounds<U256>),
	/// Uncles hash in header is invalid.
	InvalidUnclesHash(Mismatch<H256>),
	/// An uncle is from a generation too old.
	UncleTooOld(OutOfBounds<BlockNumber>),
	/// An uncle is from the same generation as the block.
	UncleIsBrother(OutOfBounds<BlockNumber>),
	/// An uncle is already in the chain.
	UncleInChain(H256),
	/// An uncle has a parent not in the chain.
	UncleParentNotInChain(H256),
	/// State root header field is invalid.
	InvalidStateRoot(Mismatch<H256>),
	/// Gas used header field is invalid.
	InvalidGasUsed(Mismatch<U256>),
	/// Transactions root header field is invalid.
	InvalidTransactionsRoot(Mismatch<H256>),
	/// Difficulty is out of range; this can be used as an looser error prior to getting a definitive
	/// value for difficulty. This error needs only provide bounds of which it is out.
	DifficultyOutOfBounds(OutOfBounds<U256>),
	/// Difficulty header field is invalid; this is a strong error used after getting a definitive
	/// value for difficulty (which is provided).
	InvalidDifficulty(Mismatch<U256>),
	/// Seal element of type H256 (max_hash for Ethash, but could be something else for
	/// other seal engines) is out of bounds.
	MismatchedH256SealElement(Mismatch<H256>),
	/// Proof-of-work aspect of seal, which we assume is a 256-bit value, is invalid.
	InvalidProofOfWork(OutOfBounds<U256>),
	/// Gas limit header field is invalid.
	InvalidGasLimit(OutOfBounds<U256>),
	/// Receipts trie root header field is invalid.
	InvalidReceiptsRoot(Mismatch<H256>),
	/// Timestamp header field is invalid.
	InvalidTimestamp(OutOfBounds<u64>),
	/// Log bloom header field is invalid.
	InvalidLogBloom(Mismatch<LogBloom>),
	/// Parent hash field of header is invalid; this is an invalid error indicating a logic flaw in the codebase.
	/// TODO: remove and favour an assert!/panic!.
	InvalidParentHash(Mismatch<H256>),
	/// Number field of header is invalid.
	InvalidNumber(Mismatch<BlockNumber>),
	/// Block number isn't sensible.
	RidiculousNumber(OutOfBounds<BlockNumber>),
	/// Parent given is unknown.
	UnknownParent(H256),
	/// Uncle parent given is unknown.
	UnknownUncleParent(H256),
}

#[derive(Debug, PartialEq)]
/// Import to the block queue result
pub enum ImportError {
	/// Already in the block chain.
	AlreadyInChain,
	/// Already in the block queue.
	AlreadyQueued,
	/// Already marked as bad from a previous import (could mean parent is bad).
	KnownBad,
}

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum Error {
	/// Error concerning a utility.
	Util(UtilError),
	/// Error concerning block processing.
	Block(BlockError),
	/// Unknown engine given.
	UnknownEngineName(String),
	/// Error concerning EVM code execution.
	Execution(ExecutionError),
	/// Error concerning transaction processing.
	Transaction(TransactionError),
	/// Error concerning block import.
	Import(ImportError),
	/// PoW hash is invalid or out of date.
	PowHashInvalid,
	/// The value of the nonce or mishash is invalid.
	PowInvalid,
}

/// Result of import block operation.
pub type ImportResult = Result<H256, Error>;

impl From<TransactionError> for Error {
	fn from(err: TransactionError) -> Error {
		Error::Transaction(err)
	}
}

impl From<ImportError> for Error {
	fn from(err: ImportError) -> Error {
		Error::Import(err)
	}
}

impl From<BlockError> for Error {
	fn from(err: BlockError) -> Error {
		Error::Block(err)
	}
}

impl From<ExecutionError> for Error {
	fn from(err: ExecutionError) -> Error {
		Error::Execution(err)
	}
}

impl From<CryptoError> for Error {
	fn from(err: CryptoError) -> Error {
		Error::Util(UtilError::Crypto(err))
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Error {
		Error::Util(UtilError::Decoder(err))
	}
}

impl From<UtilError> for Error {
	fn from(err: UtilError) -> Error {
		Error::Util(err)
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Error {
		Error::Util(From::from(err))
	}
}

// TODO: uncomment below once https://github.com/rust-lang/rust/issues/27336 sorted.
/*#![feature(concat_idents)]
macro_rules! assimilate {
    ($name:ident) => (
		impl From<concat_idents!($name, Error)> for Error {
			fn from(err: concat_idents!($name, Error)) -> Error {
				Error:: $name (err)
			}
		}
    )
}
assimilate!(FromHex);
assimilate!(BaseData);*/
