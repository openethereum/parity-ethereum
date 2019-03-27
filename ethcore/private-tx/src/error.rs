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

use std::error;
use derive_more::Display;
use ethereum_types::Address;
use rlp::DecoderError;
use ethtrie::TrieError;
use ethcore::error::{Error as EthcoreError, ExecutionError};
use types::transaction::Error as TransactionError;
use ethkey::Error as KeyError;
use ethkey::crypto::Error as CryptoError;
use txpool::{Error as TxPoolError};

#[derive(Debug, Display)]
pub enum Error {
	/// Error concerning the Rust standard library's IO subsystem.
	#[display(fmt = "Io Error: {}", _0)]
	Io(::std::io::Error),
	/// RLP decoding error.
	#[display(fmt = "Decoder Error: {}", _0)]
	Decoder(DecoderError),
	/// Error concerning TrieDBs.
	#[display(fmt = "Trie Error: {}", _0)]
	Trie(TrieError),
	/// Transaction pool error.
	#[display(fmt = "Transaction Pool Error: {}", _0)]
	TxPool(TxPoolError),
	/// Crypto error.
	#[display(fmt = "Crypto Error {}", _0)]
	Crypto(CryptoError),
	/// Encryption error.
	#[display(fmt = "Encryption error. ({})", _0)]
	Encrypt(String),
	/// Decryption error.
	#[display(fmt = "Decryption error. ({})", _0)]
	Decrypt(String),
	/// Address not authorized.
	#[display(fmt = "Private transaction execution is not authorised for {}", _0)]
	NotAuthorised(Address),
	/// Transaction creates more than one contract.
	#[display(fmt = "Private transaction created too many contracts")]
	TooManyContracts,
	/// Contract call error.
	#[display(fmt = "Contract call error. ({})", _0)]
	Call(String),
	/// State is not available.
	#[display(fmt = "State is not available")]
	StatePruned,
	/// State is incorrect.
	#[display(fmt = "State is incorrect")]
	StateIncorrect,
	/// Wrong private transaction type.
	#[display(fmt = "Wrong private transaction type")]
	BadTransactionType,
	/// Contract does not exist or was not created.
	#[display(fmt = "Contract does not exist or was not created")]
	ContractDoesNotExist,
	/// Reference to the client is corrupted.
	#[display(fmt = "Reference to the client is corrupted")]
	ClientIsMalformed,
	/// Queue of private transactions for verification is full.
	#[display(fmt = "Queue of private transactions for verification is full")]
	QueueIsFull,
	/// The transaction already exists in queue of private transactions.
	#[display(fmt = "The transaction already exists in queue of private transactions.")]
	PrivateTransactionAlreadyImported,
	/// The information about private transaction is not found in the store.
	#[display(fmt = "The information about private transaction is not found in the store.")]
	PrivateTransactionNotFound,
	/// Account for signing public transactions not set.
	#[display(fmt = "Account for signing public transactions not set.")]
	SignerAccountNotSet,
	/// Account for validating private transactions not set.
	#[display(fmt = "Account for validating private transactions not set.")]
	ValidatorAccountNotSet,
	/// Account for signing requests to key server not set.
	#[display(fmt = "Account for signing requests to key server not set.")]
	KeyServerAccountNotSet,
	/// Encryption key is not found on key server.
	#[display(fmt = "Encryption key is not found on key server for {}", _0)]
	EncryptionKeyNotFound(Address),
	/// Key server URL is not set.
	#[display(fmt = "Key server URL is not set.")]
	KeyServerNotSet,
	/// VM execution error.
	#[display(fmt = "VM execution error {}", _0)]
	Execution(ExecutionError),
	/// General signing error.
	#[display(fmt = "General signing error {}", _0)]
	Key(KeyError),
	/// Error of transactions processing.
	#[display(fmt = "Error of transactions processing {}", _0)]
	Transaction(TransactionError),
	/// General ethcore error.
	#[display(fmt = "General ethcore error {}", _0)]
	Ethcore(EthcoreError),
	/// A convenient variant for String.
	#[display(fmt = "{}", _0)]
	Msg(String),
}

impl error::Error for Error {
	fn source(&self) -> Option<&(error::Error + 'static)> {
		match self {
			Error::Io(e) => Some(e),
			Error::Decoder(e) => Some(e),
			Error::Trie(e) => Some(e),
			Error::TxPool(e) => Some(e),
			Error::Crypto(e) => Some(e),
			Error::Execution(e) => Some(e),
			Error::Key(e) => Some(e),
			Error::Transaction(e) => Some(e),
			Error::Ethcore(e) => Some(e),
			_ => None,
		}
	}
}

impl From<String> for Error {
	fn from(s: String) -> Self {
		Error::Msg(s)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::Io(err).into()
	}
}

impl From<KeyError> for Error {
	fn from(err: KeyError) -> Self {
		Error::Key(err).into()
	}
}

impl From<CryptoError> for Error {
	fn from(err: CryptoError) -> Self {
		Error::Crypto(err).into()
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err).into()
	}
}

impl From<ExecutionError> for Error {
	fn from(err: ExecutionError) -> Self {
		Error::Execution(err).into()
	}
}

impl From<TransactionError> for Error {
	fn from(err: TransactionError) -> Self {
		Error::Transaction(err).into()
	}
}

impl From<TrieError> for Error {
	fn from(err: TrieError) -> Self {
		Error::Trie(err).into()
	}
}

impl From<TxPoolError> for Error {
	fn from(err: TxPoolError) -> Self {
		Error::TxPool(err).into()
	}
}

impl From<EthcoreError> for Error {
	fn from(err: EthcoreError) -> Self {
		Error::Ethcore(err).into()
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Error {
		Error::from(*err)
	}
}
