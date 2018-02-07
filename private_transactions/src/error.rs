// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::fmt;
use ethereum_types::Address;
use trie::TrieError;
use ethcore::account_provider::SignError;
use ethcore::error::ExecutionError;
use transaction::Error as TransactionError;
use ethkey::Error as KeyError;

#[derive(Debug)]
/// Errors concerning private transaction processing.
pub enum PrivateTransactionError {
	/// Encryption error.
	Encrypt(String),
	/// Decryption error.
	Decrypt(String),
	/// Contract does not exist or is unavailable.
	NotAuthorised(Address),
	/// Transaction creates more than one contract.
	TooManyContracts,
	/// Contract call error.
	Call(String),
	/// State is not available.
	StatePruned,
	/// State is incorrect.
	StateIncorrect,
	/// Wrong private transaction type.
	BadTransactonType,
	/// Contract does not exist or was not created.
	ContractDoesNotExist,
	/// Reference to the client is corrupted.
	ClientIsMalformed,
	/// Reference to account provider is corrupted.
	AccountProviderIsMalformed,
	/// Queue of private transactions is full.
	QueueIsFull,
	/// The transaction already exists in queue of private transactions.
	PrivateTransactionAlreadyImported,
	/// The information about private transaction is not found in the store
	PrivateTransactionNotFound,
	/// Account for signing public transactions not set
	SignerAccountNotSet,
	/// Account for signing requests to key server not set
	KeyServerAccountNotSet,
	/// Encryption key is not found on key server
	EncryptionKeyNotFound(Address),
	/// Key server URL is not set
	KeyServerNotSet,
	/// RLP decoder error.
	Decode(::rlp::DecoderError),
	/// Error concerning TrieDBs
	Trie(TrieError),
	/// Account provider signing errors
	Sign(SignError),
	/// General signing errors
	Key(KeyError),
	/// VM execution errors
	Execution(ExecutionError),
	/// standard io errors
	StdIo(::std::io::Error),
	/// Errors of transactions processing
	Transaction(TransactionError),
}

impl fmt::Display for PrivateTransactionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::PrivateTransactionError::*;
		match *self {
			Encrypt(ref msg) => f.write_fmt(format_args!("Encryption error. ({})", msg)),
			Decrypt(ref msg) => f.write_fmt(format_args!("Decryption error. ({})", msg)),
			NotAuthorised(address) => f.write_fmt(format_args!("Private trsnaction execution is not authorised for {}.", address)),
			TooManyContracts => f.write_str("Private transaction created too many contracts."),
			Call(ref msg) => f.write_fmt(format_args!("Contract call error. ({})", msg)),
			StatePruned => f.write_str("State is not available."),
			StateIncorrect => f.write_str("State is incorrect."),
			BadTransactonType => f.write_str("Bad transaction type."),
			ContractDoesNotExist => f.write_str("Private contract does not exist."),
			ClientIsMalformed => f.write_str("Client is not registered."),
			AccountProviderIsMalformed => f.write_str("Account provider is not registered."),
			QueueIsFull => f.write_str("Private transactions queue is full."),
			PrivateTransactionAlreadyImported => f.write_str("Private transactions already imported."),
			PrivateTransactionNotFound => f.write_str("Private transactions is not found in the store."),
			SignerAccountNotSet => f.write_str("Account for signing public transactions not set."),
			KeyServerAccountNotSet => f.write_str("Account for signing requets to key server not set."),
			EncryptionKeyNotFound(address) => f.write_fmt(format_args!("Encryption key is not found on key server for {}.", address)),
			KeyServerNotSet => f.write_str("URL for key server is not set."),
			Decode(ref err) => err.fmt(f),
			Trie(ref err) => err.fmt(f),
			Sign(ref err) => err.fmt(f),
			Key(ref err) => err.fmt(f),
			Execution(ref err) => err.fmt(f),
			StdIo(ref err) => err.fmt(f),
			Transaction(ref err) => err.fmt(f),
		}
	}
}

impl From<::rlp::DecoderError> for PrivateTransactionError {
	fn from(err: ::rlp::DecoderError) -> PrivateTransactionError {
		PrivateTransactionError::Decode(err)
	}
}

impl From<TrieError> for PrivateTransactionError {
	fn from(err: TrieError) -> PrivateTransactionError {
		PrivateTransactionError::Trie(err)
	}
}

impl From<SignError> for PrivateTransactionError {
	fn from(err: SignError) -> PrivateTransactionError {
		PrivateTransactionError::Sign(err)
	}
}

impl From<KeyError> for PrivateTransactionError {
	fn from(err: KeyError) -> PrivateTransactionError {
		PrivateTransactionError::Key(err)
	}
}

impl From<ExecutionError> for PrivateTransactionError {
	fn from(err: ExecutionError) -> PrivateTransactionError {
		PrivateTransactionError::Execution(err)
	}
}

impl From<::std::io::Error> for PrivateTransactionError {
	fn from(err: ::std::io::Error) -> PrivateTransactionError {
		PrivateTransactionError::StdIo(err)
	}
}

impl From<TransactionError> for PrivateTransactionError {
	fn from(err: TransactionError) -> PrivateTransactionError {
		PrivateTransactionError::Transaction(err)
	}
}

impl<E> From<Box<E>> for PrivateTransactionError where PrivateTransactionError: From<E> {
	fn from(err: Box<E>) -> PrivateTransactionError {
		PrivateTransactionError::from(*err)
	}
}

