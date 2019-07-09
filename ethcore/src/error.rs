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

use std::error;

use derive_more::{Display, From};
use ethkey::Error as EthkeyError;
use ethtrie::TrieError;
use rlp;
use snappy::InvalidInput;
use snapshot::Error as SnapshotError;
use types::{
	engines::EngineError,
	transaction::Error as TransactionError,
	block::ImportError,
};

pub use executed::{ExecutionError, CallError};
pub use types::block::BlockError; // TODO prolly dont want to re-export

/// Ethcore Result
pub type EthcoreResult<T> = Result<T, Error>;

/// Ethcore Error
#[derive(Debug, Display, From)]
pub enum Error {
	/// Error concerning block import.
	#[display(fmt = "Import error: {}", _0)]
	Import(ImportError),
	/// Io channel queue error
	#[display(fmt = "Queue is full: {}", _0)]
	FullQueue(usize),
	/// Io create error
	#[display(fmt = "Io error: {}", _0)]
	Io(::io::IoError),
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
	/// State errors
	#[display(fmt = "State error ({})", _0)]
	State(account_state::Error),
}

impl error::Error for Error {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Error::Io(e) => Some(e),
			Error::StdIo(e) => Some(e),
			Error::Trie(e) => Some(e),
			Error::Execution(e) => Some(e),
			Error::Block(e) => Some(e),
			Error::Transaction(e) => Some(e),
			Error::Snappy(e) => Some(e),
			Error::Engine(e) => Some(e),
			Error::Ethkey(e) => Some(e),
			Error::Decoder(e) => Some(e),
			Error::Snapshot(e) => Some(e),
			Error::State(e) => Some(e),
			_ => None,
		}
	}
}

impl From<&str> for Error {
	fn from(s: &str) -> Self {
		Error::Msg(s.into())
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Error {
		Error::from(*err)
	}
}
