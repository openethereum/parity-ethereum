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

//! State related errors

use derive_more::{Display, From};
use common_types::{
	engines::EngineError,
	block::{BlockError, ImportError},
	transaction::Error as TransactionError,
};
use rlp::DecoderError;

#[derive(Debug, Display, From)]
pub enum Error {
	/// todo: what errors do I need?
	Block(BlockError),
	Import(ImportError),
	Engine(EngineError),
	Decoder(DecoderError),
	Transaction(TransactionError),
	State(String), // todo: move account-state errors to `common_types`?
	Other(String)
}

impl std::error::Error for Error {}

impl From<&str> for Error {
	fn from(s: &str) -> Self {
		Error::Other(s.into())
	}
}

impl From<String> for Error {
	fn from(s: String) -> Self {
		Error::Other(s)
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Self {
		Error::from(*err)
	}
}
