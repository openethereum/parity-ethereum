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

//! Snapshot-related errors.

use std::fmt;

use util::H256;
use util::trie::TrieError;
use util::rlp::DecoderError;

/// Snapshot-related errors.
#[derive(Debug)]
pub enum Error {
	/// Invalid starting block for snapshot.
	InvalidStartingBlock(H256),
	/// Block not found.
	BlockNotFound(H256),
	/// Trie error.
	Trie(TrieError),
	/// Decoder error.
	Decoder(DecoderError),
	/// Io error.
	Io(::std::io::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::InvalidStartingBlock(ref hash) => write!(f, "Invalid starting block hash: {}", hash),
			Error::BlockNotFound(ref hash) => write!(f, "Block not found in chain: {}", hash),
			Error::Io(ref err) => err.fmt(f),
			Error::Decoder(ref err) => err.fmt(f),
			Error::Trie(ref err) => err.fmt(f),
		}
	}
}

impl From<::std::io::Error> for Error {
	fn from(err: ::std::io::Error) -> Self {
		Error::Io(err)
	}
}

impl From<Box<TrieError>> for Error {
	fn from(err: Box<TrieError>) -> Self {
		Error::Trie(*err)
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err)
	}
}