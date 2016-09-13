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

//! Definition of valid items for the verification queue.

use engines::Engine;
use error::Error;
use header::Header;
use verification::{PreverifiedBlock, verify_block_basic, verify_block_unordered};

use util::{Bytes, HeapSizeOf, H256};

/// Something which can produce a hash and a parent hash.
pub trait HasHash {
	/// Get the hash of this item.
	fn hash(&self) -> H256;

	/// Get the hash of this item's parent.
	fn parent_hash(&self) -> H256;
}

/// Defines transitions between stages of verification.
///
/// It starts with a fallible transformation from an "input" into the unverified item.
/// This consists of quick, simply done checks as well as extracting particular data.
///
/// Then, there is a `verify` function which performs more expensive checks and
/// produces the verified output.
///
/// For correctness, the hashes produced by each stage of the pipeline should be
/// consistent.
pub trait Kind: 'static + Sized + Send + Sync {
	/// The first stage: completely unverified.
	type Input: Sized + Send + HasHash + HeapSizeOf;

	/// The second stage: partially verified.
	type Unverified: Sized + Send + HasHash + HeapSizeOf;

	/// The third stage: completely verified.
	type Verified: Sized + Send + HasHash + HeapSizeOf;

	/// Attempt to create the `Unverified` item from the input.
	fn create(input: Self::Input, engine: &Engine) -> Result<Self::Unverified, Error>;

	/// Attempt to verify the `Unverified` item using the given engine.
	fn verify(unverified: Self::Unverified, engine: &Engine) -> Result<Self::Verified, Error>;
}

/// A mode for verifying blocks.
pub struct Blocks;

impl Kind for Blocks {
	type Input = UnverifiedBlock;
	type Unverified = UnverifiedBlock;
	type Verified = PreverifiedBlock;

	fn create(input: Self::Input, engine: &Engine) -> Result<Self::Unverified, Error> {
		match verify_block_basic(&input.header, &input.bytes, engine) {
			Ok(()) => Ok(input),
			Err(e) => {
				warn!(target: "client", "Stage 1 block verification failed for {:?}", input.hash());
				Err(e)
			}
		}
	}

	fn verify(un: Self::Unverified, engine: &Engine) -> Result<Self::Verified, Error> {
		let hash = un.hash();
		match verify_block_unordered(un.header, un.bytes, engine) {
			Ok(verified) => Ok(verified),
			Err(e) => {
				warn!(target: "client", "Stage 2 block verification failed for {}\n Error: {:?}", hash, e);
				Err(e)
			}
		}
	}
}

/// An unverified block.
pub struct UnverifiedBlock {
	header: Header,
	bytes: Bytes,
}

impl UnverifiedBlock {
	/// Create an `UnverifiedBlock` from raw bytes.
	pub fn new(bytes: Bytes) -> Self {
		use views::BlockView;

		let header = BlockView::new(&bytes).header();
		UnverifiedBlock {
			header: header,
			bytes: bytes,
		}
	}
}

impl HeapSizeOf for UnverifiedBlock {
	fn heap_size_of_children(&self) -> usize {
		self.header.heap_size_of_children() + self.bytes.heap_size_of_children()
	}
}

impl HasHash for UnverifiedBlock {
	fn hash(&self) -> H256 {
		self.header.hash()
	}

	fn parent_hash(&self) -> H256 {
		self.header.parent_hash().clone()
	}
}

impl HasHash for PreverifiedBlock {
	fn hash(&self) -> H256 {
		self.header.hash()
	}

	fn parent_hash(&self) -> H256 {
		self.header.parent_hash().clone()
	}
}