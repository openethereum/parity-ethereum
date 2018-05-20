// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use ethereum_types::Address;
use rlp::DecoderError;
use trie::TrieError;
use ethcore::account_provider::SignError;
use ethcore::error::{Error as EthcoreError, ExecutionError};
use transaction::Error as TransactionError;
use ethkey::Error as KeyError;

error_chain! {
	foreign_links {
		Io(::std::io::Error) #[doc = "Error concerning the Rust standard library's IO subsystem."];
		Decoder(DecoderError) #[doc = "RLP decoding error."];
		Trie(TrieError) #[doc = "Error concerning TrieDBs."];
	}

	errors {
		#[doc = "Encryption error."]
		Encrypt(err: String) {
			description("Encryption error"),
			display("Encryption error. ({})", err),
		}

		#[doc = "Decryption error."]
		Decrypt(err: String) {
			description("Decryption error"),
			display("Decryption error. ({})", err),
		}

		#[doc = "Address not authorized."]
		NotAuthorised(address: Address) {
			description("Address not authorized"),
			display("Private transaction execution is not authorised for {}", address),
		}

		#[doc = "Transaction creates more than one contract."]
		TooManyContracts {
			description("Transaction creates more than one contract."),
			display("Private transaction created too many contracts"),
		}

		#[doc = "Contract call error."]
		Call(err: String) {
			description("Contract call error."),
			display("Contract call error. ({})", err),
		}

		#[doc = "State is not available."]
		StatePruned {
			description("State is not available."),
			display("State is not available"),
		}

		#[doc = "State is incorrect."]
		StateIncorrect {
			description("State is incorrect."),
			display("State is incorrect"),
		}

		#[doc = "Wrong private transaction type."]
		BadTransactonType {
			description("Wrong private transaction type."),
			display("Wrong private transaction type"),
		}

		#[doc = "Contract does not exist or was not created."]
		ContractDoesNotExist {
			description("Contract does not exist or was not created."),
			display("Contract does not exist or was not created"),
		}

		#[doc = "Reference to the client is corrupted."]
		ClientIsMalformed {
			description("Reference to the client is corrupted."),
			display("Reference to the client is corrupted"),
		}

		#[doc = "Queue of private transactions for verification is full."]
		QueueIsFull {
			description("Queue of private transactions for verification is full."),
			display("Queue of private transactions for verification is full"),
		}

		#[doc = "The transaction already exists in queue of private transactions."]
		PrivateTransactionAlreadyImported {
			description("The transaction already exists in queue of private transactions."),
			display("The transaction already exists in queue of private transactions."),
		}

		#[doc = "The information about private transaction is not found in the store."]
		PrivateTransactionNotFound {
			description("The information about private transaction is not found in the store."),
			display("The information about private transaction is not found in the store."),
		}

		#[doc = "Account for signing public transactions not set."]
		SignerAccountNotSet {
			description("Account for signing public transactions not set."),
			display("Account for signing public transactions not set."),
		}

		#[doc = "Account for validating private transactions not set."]
		ValidatorAccountNotSet {
			description("Account for validating private transactions not set."),
			display("Account for validating private transactions not set."),
		}

		#[doc = "Account for signing requests to key server not set."]
		KeyServerAccountNotSet {
			description("Account for signing requests to key server not set."),
			display("Account for signing requests to key server not set."),
		}

		#[doc = "Encryption key is not found on key server."]
		EncryptionKeyNotFound(address: Address) {
			description("Encryption key is not found on key server"),
			display("Encryption key is not found on key server for {}", address),
		}

		#[doc = "Key server URL is not set."]
		KeyServerNotSet {
			description("Key server URL is not set."),
			display("Key server URL is not set."),
		}

		#[doc = "VM execution error."]
		Execution(err: ExecutionError) {
			description("VM execution error."),
			display("VM execution error {}", err),
		}

		#[doc = "General signing error."]
		Key(err: KeyError) {
			description("General signing error."),
			display("General signing error {}", err),
		}

		#[doc = "Account provider signing error."]
		Sign(err: SignError) {
			description("Account provider signing error."),
			display("Account provider signing error {}", err),
		}

		#[doc = "Error of transactions processing."]
		Transaction(err: TransactionError) {
			description("Error of transactions processing."),
			display("Error of transactions processing {}", err),
		}

		#[doc = "General ethcore error."]
		Ethcore(err: EthcoreError) {
			description("General ethcore error."),
			display("General ethcore error {}", err),
		}
	}
}

impl From<SignError> for Error {
	fn from(err: SignError) -> Self {
		ErrorKind::Sign(err).into()
	}
}

impl From<KeyError> for Error {
	fn from(err: KeyError) -> Self {
		ErrorKind::Key(err).into()
	}
}

impl From<ExecutionError> for Error {
	fn from(err: ExecutionError) -> Self {
		ErrorKind::Execution(err).into()
	}
}

impl From<TransactionError> for Error {
	fn from(err: TransactionError) -> Self {
		ErrorKind::Transaction(err).into()
	}
}

impl From<EthcoreError> for Error {
	fn from(err: EthcoreError) -> Self {
		ErrorKind::Ethcore(err).into()
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Error {
		Error::from(*err)
	}
}
