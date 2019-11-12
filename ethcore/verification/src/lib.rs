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

//! Block verification utilities.

// The MallocSizeOf derive looks for this in the root
use parity_util_mem as malloc_size_of;

#[cfg(feature = "bench" )]
pub mod verification;
#[cfg(not(feature = "bench" ))]
mod verification;
pub mod queue;
#[cfg(any(test, feature = "bench" ))]
pub mod test_helpers;

pub use self::verification::{FullFamilyParams, verify_block_family, verify_block_final};
pub use self::queue::{BlockQueue, Config as QueueConfig};

/// Verifier type.
#[derive(Debug, PartialEq, Clone)]
pub enum VerifierType {
	/// Verifies block normally.
	Canon,
	/// Verifies block normally, but skips seal verification.
	CanonNoSeal,
}

impl VerifierType {
	/// Check if seal verification is enabled for this verifier type.
	pub fn verifying_seal(&self) -> bool {
		match *self {
			VerifierType::Canon => true,
			VerifierType::CanonNoSeal => false,
		}
	}
}
