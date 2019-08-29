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

use std::{error, fmt};
use derive_more::{Display, From};
use ethereum_types::{U256, U512};
use ethtrie::TrieError;
use parity_snappy::InvalidInput;
use ethkey::Error as EthkeyError;

use errors::{BlockError, EngineError, ImportError, SnapshotError};
use transaction::Error as TransactionError;

/// Ethcore Result
pub type EthcoreResult<T> = Result<T, EthcoreError>;

/// Ethcore Error
#[derive(Debug, Display, From)]
pub enum EthcoreError {
	/// Error concerning block import.
	#[display(fmt = "Import error: {}", _0)]
	Import(ImportError),
	/// Io channel queue error
	#[display(fmt = "Queue is full: {}", _0)]
	FullQueue(usize),
	/// Io create error
	#[display(fmt = "Io error: {}", _0)]
	Io(ethcore_io::IoError),
	/// Error concerning the Rust standard library's IO subsystem.
	#[display(fmt = "Std Io error: {}", _0)]
	StdIo(::std::io::Error),
	/// Error concerning TrieDBs.
	#[display(fmt = "Trie error: {}", _0)]
	Trie(TrieError),
	/// Error concerning EVM code execution.
	#[display(fmt = "Execution error: {}", _0)]
	Execution(ExecutionError),
	/// Error concerning block processing.
	#[display(fmt = "Block error: {}", _0)]
	Block(BlockError),
	/// Error concerning transaction processing.
	#[display(fmt = "Transaction error: {}", _0)]
	Transaction(TransactionError),
	/// Snappy error
	#[display(fmt = "Snappy error: {}", _0)]
	Snappy(InvalidInput),
	/// Consensus vote error.
	#[display(fmt = "Engine error: {}", _0)]
	Engine(EngineError),
	/// Ethkey error."
	#[display(fmt = "Ethkey error: {}", _0)]
	Ethkey(EthkeyError),
	/// RLP decoding errors
	#[display(fmt = "Decoder error: {}", _0)]
	Decoder(rlp::DecoderError),
	/// Snapshot error.
	#[display(fmt = "Snapshot error {}", _0)]
	Snapshot(SnapshotError),
	/// PoW hash is invalid or out of date.
	#[display(fmt = "PoW hash is invalid or out of date.")]
	PowHashInvalid,
	/// The value of the nonce or mishash is invalid.
	#[display(fmt = "The value of the nonce or mishash is invalid.")]
	PowInvalid,
	/// A convenient variant for String.
	#[display(fmt = "{}", _0)]
	Msg(String),
}

impl error::Error for EthcoreError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		use self::EthcoreError::*;
		match self {
			Io(e) => Some(e),
			StdIo(e) => Some(e),
			Trie(e) => Some(e),
			Execution(e) => Some(e),
			Block(e) => Some(e),
			Transaction(e) => Some(e),
			Snappy(e) => Some(e),
			Engine(e) => Some(e),
			Ethkey(e) => Some(e),
			Decoder(e) => Some(e),
			Snapshot(e) => Some(e),
			_ => None,
		}
	}
}

impl From<&str> for EthcoreError {
	fn from(s: &str) -> Self {
		EthcoreError::Msg(s.into())
	}
}

impl<E> From<Box<E>> for EthcoreError where EthcoreError: From<E> {
	fn from(err: Box<E>) -> EthcoreError {
		EthcoreError::from(*err)
	}
}

/// Error type for executing a transaction.
#[derive(PartialEq, Debug, Clone)]
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
	/// When execution tries to modify the state in static context
	MutableCallInStaticContext,
	/// Returned when transacting from a non-existing account with dust protection enabled.
	SenderMustExist,
	/// Returned when internal evm error occurs.
	Internal(String),
	/// Returned when generic transaction occurs
	TransactionMalformed(String),
}

impl error::Error for ExecutionError {
	fn description(&self) -> &str {
		"Transaction execution error"
	}
}

impl From<Box<TrieError>> for ExecutionError {
	fn from(err: Box<TrieError>) -> Self {
		ExecutionError::Internal(format!("{:?}", err))
	}
}
impl From<TrieError> for ExecutionError {
	fn from(err: TrieError) -> Self {
		ExecutionError::Internal(format!("{:?}", err))
	}
}

impl fmt::Display for ExecutionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::ExecutionError::*;

		let msg = match *self {
			NotEnoughBaseGas { ref required, ref got } =>
				format!("Not enough base gas. {} is required, but only {} paid", required, got),
			BlockGasLimitReached { ref gas_limit, ref gas_used, ref gas } =>
				format!("Block gas limit reached. The limit is {}, {} has \
					already been used, and {} more is required", gas_limit, gas_used, gas),
			InvalidNonce { ref expected, ref got } =>
				format!("Invalid transaction nonce: expected {}, found {}", expected, got),
			NotEnoughCash { ref required, ref got } =>
				format!("Cost of transaction exceeds sender balance. {} is required \
					but the sender only has {}", required, got),
			MutableCallInStaticContext => "Mutable Call in static context".to_owned(),
			SenderMustExist => "Transacting from an empty account".to_owned(),
			Internal(ref msg) => msg.clone(),
			TransactionMalformed(ref err) => format!("Malformed transaction: {}", err),
		};

		f.write_fmt(format_args!("Transaction execution error ({}).", msg))
	}
}
