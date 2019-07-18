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

//! Snapshot-related errors.

use std::error;
use std::fmt;

use ethereum_types::H256;
use ethtrie::TrieError;
use rlp::DecoderError;

use ids::BlockId;

/// Snapshot-related errors.
#[derive(Debug)]
pub enum SnapshotError {
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
	/// Aborted snapshot
	SnapshotAborted,
	/// Bad epoch transition.
	BadEpochProof(u64),
	/// Wrong chunk format.
	WrongChunkFormat(String),
	/// Unlinked ancient block chain; includes the parent hash where linkage failed
	UnlinkedAncientBlockChain(H256),
}

impl error::Error for SnapshotError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		use self::SnapshotError::*;
		match self {
			Trie(e) => Some(e),
			Decoder(e) => Some(e),
			Io(e) => Some(e),
			_ => None,
		}
	}
}

impl fmt::Display for SnapshotError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::SnapshotError::*;
		match *self {
			InvalidStartingBlock(ref id) => write!(f, "Invalid starting block: {:?}", id),
			BlockNotFound(ref hash) => write!(f, "Block not found in chain: {}", hash),
			IncompleteChain => write!(f, "Incomplete blockchain."),
			WrongStateRoot(ref expected, ref found) => write!(f, "Final block has wrong state root. Expected {:?}, got {:?}", expected, found),
			WrongBlockHash(ref num, ref expected, ref found) =>
				write!(f, "Block {} had wrong hash. expected {:?}, got {:?}", num, expected, found),
			TooManyBlocks(ref expected, ref found) => write!(f, "Snapshot contained too many blocks. Expected {}, got {}", expected, found),
			OldBlockPrunedDB => write!(f, "Attempted to create a snapshot at an old block while using \
				a pruned database. Please re-run with the --pruning archive flag."),
			MissingCode(ref missing) => write!(f, "Incomplete snapshot: {} contract codes not found.", missing.len()),
			UnrecognizedCodeState(state) => write!(f, "Unrecognized code encoding ({})", state),
			RestorationAborted => write!(f, "Snapshot restoration aborted."),
			Io(ref err) => err.fmt(f),
			Decoder(ref err) => err.fmt(f),
			Trie(ref err) => err.fmt(f),
			VersionNotSupported(ref ver) => write!(f, "Snapshot version {} is not supprted.", ver),
			ChunkTooSmall => write!(f, "Chunk size is too small."),
			ChunkTooLarge => write!(f, "Chunk size is too large."),
			SnapshotsUnsupported => write!(f, "Snapshots unsupported by consensus engine."),
			SnapshotAborted => write!(f, "Snapshot was aborted."),
			BadEpochProof(i) => write!(f, "Bad epoch proof for transition to epoch {}", i),
			WrongChunkFormat(ref msg) => write!(f, "Wrong chunk format: {}", msg),
			UnlinkedAncientBlockChain(parent_hash) => write!(f, "Unlinked ancient blocks chain at parent_hash={:#x}", parent_hash),
		}
	}
}

impl From<::std::io::Error> for SnapshotError {
	fn from(err: ::std::io::Error) -> Self {
		SnapshotError::Io(err)
	}
}

impl From<TrieError> for SnapshotError {
	fn from(err: TrieError) -> Self {
		SnapshotError::Trie(err)
	}
}

impl From<DecoderError> for SnapshotError {
	fn from(err: DecoderError) -> Self {
		SnapshotError::Decoder(err)
	}
}

impl<E> From<Box<E>> for SnapshotError where SnapshotError: From<E> {
	fn from(err: Box<E>) -> Self {
		SnapshotError::from(*err)
	}
}
