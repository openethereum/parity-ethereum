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

pub mod account;
pub mod backend;
pub mod substate;
// TODO: WIP
pub mod state;

pub use {
	account::Account,
	backend::Backend,
	substate::Substate,
	state::{State, CleanupMode},
};

// TODO: move to an errors.rs
use derive_more::{Display, From};
// TODO: sort out error handling
#[derive(Debug, Display, From)]
pub enum Error {
	/// Trie error.
	Trie(ethtrie::TrieError),
	/// Decoder error.
	Decoder(rlp::DecoderError),
	// Io error.
//	Io(IoError), // TODO: maybe not needed?
}

// TODO: needed?
impl std::error::Error for Error {}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Self {
		Error::from(*err)
	}
}
