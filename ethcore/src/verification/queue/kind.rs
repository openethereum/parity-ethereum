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

//! Definition of valid items for the verification queue.

use engine::Engine;

use parity_util_mem::MallocSizeOf;
use ethereum_types::{H256, U256};

use types::errors::EthcoreError as Error;

pub use self::blocks::Blocks;
pub use self::headers::Headers;

/// Something which can produce a hash and a parent hash.
pub trait BlockLike {
	/// Get the hash of this item.
	fn hash(&self) -> H256;

	/// Get the hash of this item's parent.
	fn parent_hash(&self) -> H256;

	/// Get the difficulty of this item.
	fn difficulty(&self) -> U256;
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
	type Input: Sized + Send + BlockLike + MallocSizeOf;

	/// The second stage: partially verified.
	type Unverified: Sized + Send + BlockLike + MallocSizeOf;

	/// The third stage: completely verified.
	type Verified: Sized + Send + BlockLike + MallocSizeOf;

	/// Attempt to create the `Unverified` item from the input.
	fn create(input: Self::Input, engine: &dyn Engine, check_seal: bool) -> Result<Self::Unverified, (Self::Input, Error)>;

	/// Attempt to verify the `Unverified` item using the given engine.
	fn verify(unverified: Self::Unverified, engine: &dyn Engine, check_seal: bool) -> Result<Self::Verified, Error>;
}

/// The blocks verification module.
pub mod blocks {
	use super::{Kind, BlockLike};

	use engine::Engine;
	use types::{
		block::PreverifiedBlock,
		errors::{EthcoreError as Error, BlockError},
		verification::Unverified,
	};
	use verification::{verify_block_basic, verify_block_unordered};

	use ethereum_types::{H256, U256};

	/// A mode for verifying blocks.
	pub struct Blocks;

	impl Kind for Blocks {
		type Input = Unverified;
		type Unverified = Unverified;
		type Verified = PreverifiedBlock;

		fn create(input: Self::Input, engine: &dyn Engine, check_seal: bool) -> Result<Self::Unverified, (Self::Input, Error)> {
			match verify_block_basic(&input, engine, check_seal) {
				Ok(()) => Ok(input),
				Err(Error::Block(BlockError::TemporarilyInvalid(oob))) => {
					debug!(target: "client", "Block received too early {}: {:?}", input.hash(), oob);
					Err((input, BlockError::TemporarilyInvalid(oob).into()))
				},
				Err(e) => {
					warn!(target: "client", "Stage 1 block verification failed for {}: {:?}", input.hash(), e);
					Err((input, e))
				}
			}
		}

		fn verify(un: Self::Unverified, engine: &dyn Engine, check_seal: bool) -> Result<Self::Verified, Error> {
			let hash = un.hash();
			match verify_block_unordered(un, engine, check_seal) {
				Ok(verified) => Ok(verified),
				Err(e) => {
					warn!(target: "client", "Stage 2 block verification failed for {}: {:?}", hash, e);
					Err(e)
				}
			}
		}
	}

	impl BlockLike for Unverified {
		fn hash(&self) -> H256 {
			self.header.hash()
		}

		fn parent_hash(&self) -> H256 {
			self.header.parent_hash().clone()
		}

		fn difficulty(&self) -> U256 {
			self.header.difficulty().clone()
		}
	}

	impl BlockLike for PreverifiedBlock {
		fn hash(&self) -> H256 {
			self.header.hash()
		}

		fn parent_hash(&self) -> H256 {
			self.header.parent_hash().clone()
		}

		fn difficulty(&self) -> U256 {
			self.header.difficulty().clone()
		}
	}
}

/// Verification for headers.
pub mod headers {
	use super::{Kind, BlockLike};

	use engine::Engine;
	use types::{
		header::Header,
		errors::EthcoreError as Error,
	};
	use verification::verify_header_params;

	use ethereum_types::{H256, U256};

	impl BlockLike for Header {
		fn hash(&self) -> H256 { self.hash() }
		fn parent_hash(&self) -> H256 { self.parent_hash().clone() }
		fn difficulty(&self) -> U256 { self.difficulty().clone() }
	}

	/// A mode for verifying headers.
	pub struct Headers;

	impl Kind for Headers {
		type Input = Header;
		type Unverified = Header;
		type Verified = Header;

		fn create(input: Self::Input, engine: &dyn Engine, check_seal: bool) -> Result<Self::Unverified, (Self::Input, Error)> {
			match verify_header_params(&input, engine, true, check_seal) {
				Ok(_) => Ok(input),
				Err(err) => Err((input, err))
			}
		}

		fn verify(unverified: Self::Unverified, engine: &dyn Engine, check_seal: bool) -> Result<Self::Verified, Error> {
			match check_seal {
				true => engine.verify_block_unordered(&unverified,).map(|_| unverified),
				false => Ok(unverified),
			}
		}
	}
}
