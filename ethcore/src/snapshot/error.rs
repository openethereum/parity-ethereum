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

//! Snapshot-related errors.

use std::fmt;

use ids::BlockId;

use util::H256;
use util::trie::TrieError;
use rlp::DecoderError;

/// Snapshot-related errors.
#[derive(Debug)]
pub enum Error {
	/// Invalid starting block for snapshot.
	InvalidStartingBlock(BlockId),
	/// Block not found.
	BlockNotFound(H256),
	/// Incomplete chain.
	IncompleteChain,
	/// Best block has wrong state root.
	WrongStateRoot(H256, H256),
	/// Wrong block hash.
	WrongBlockHash(u64, H256, H256),
	/// Too many blocks contained within the snapshot.
	TooManyBlocks(u64, u64),
	/// Old starting block in a pruned database.
	OldBlockPrunedDB,
	/// Missing code.
	MissingCode(Vec<H256>),
	/// Unrecognized code encoding.
	UnrecognizedCodeState(u8),
	/// Restoration aborted.
	RestorationAborted,
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
			Error::InvalidStartingBlock(ref id) => write!(f, "Invalid starting block: {:?}", id),
			Error::BlockNotFound(ref hash) => write!(f, "Block not found in chain: {}", hash),
			Error::IncompleteChain => write!(f, "Incomplete blockchain."),
			Error::WrongStateRoot(ref expected, ref found) => write!(f, "Final block has wrong state root. Expected {:?}, got {:?}", expected, found),
			Error::WrongBlockHash(ref num, ref expected, ref found) =>
				write!(f, "Block {} had wrong hash. expected {:?}, got {:?}", num, expected, found),
			Error::TooManyBlocks(ref expected, ref found) => write!(f, "Snapshot contained too many blocks. Expected {}, got {}", expected, found),
			Error::OldBlockPrunedDB => write!(f, "Attempted to create a snapshot at an old block while using \
				a pruned database. Please re-run with the --pruning archive flag."),
			Error::MissingCode(ref missing) => write!(f, "Incomplete snapshot: {} contract codes not found.", missing.len()),
			Error::UnrecognizedCodeState(state) => write!(f, "Unrecognized code encoding ({})", state),
			Error::RestorationAborted => write!(f, "Snapshot restoration aborted."),
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

impl From<TrieError> for Error {
	fn from(err: TrieError) -> Self {
		Error::Trie(err)
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err)
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Self {
		Error::from(*err)
	}
}
