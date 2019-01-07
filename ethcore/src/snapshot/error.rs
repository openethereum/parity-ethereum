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

//! Snapshot-related errors.

use std::fmt;

use types::ids::BlockId;

use ethereum_types::H256;
use ethtrie::TrieError;
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
	/// Snapshot version is not supported.
	VersionNotSupported(u64),
	/// Max chunk size is to small to fit basic account data.
	ChunkTooSmall,
	/// Oversized chunk
	ChunkTooLarge,
	/// Snapshots not supported by the consensus engine.
	SnapshotsUnsupported,
	/// Bad epoch transition.
	BadEpochProof(u64),
	/// Wrong chunk format.
	WrongChunkFormat(String),
	/// Unlinked ancient block chain
	UnlinkedAncientBlockChain,
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
			Error::VersionNotSupported(ref ver) => write!(f, "Snapshot version {} is not supprted.", ver),
			Error::ChunkTooSmall => write!(f, "Chunk size is too small."),
			Error::ChunkTooLarge => write!(f, "Chunk size is too large."),
			Error::SnapshotsUnsupported => write!(f, "Snapshots unsupported by consensus engine."),
			Error::BadEpochProof(i) => write!(f, "Bad epoch proof for transition to epoch {}", i),
			Error::WrongChunkFormat(ref msg) => write!(f, "Wrong chunk format: {}", msg),
			Error::UnlinkedAncientBlockChain => write!(f, "Unlinked ancient blocks chain"),
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
