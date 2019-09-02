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

use call_contract::CallContract;
use client_traits::BlockInfo;
// The MallocSizeOf derive looks for this in the root
use parity_util_mem as malloc_size_of;

mod verification;
mod verifier;
pub mod queue;
mod canon_verifier;
mod noop_verifier;

pub use self::verification::FullFamilyParams;
pub use self::verifier::Verifier;
pub use self::queue::{BlockQueue, Config as QueueConfig};

use self::canon_verifier::CanonVerifier;
use self::noop_verifier::NoopVerifier;

/// Verifier type.
#[derive(Debug, PartialEq, Clone)]
pub enum VerifierType {
	/// Verifies block normally.
	Canon,
	/// Verifies block normally, but skips seal verification.
	CanonNoSeal,
	/// Does not verify block at all.
	/// Used in tests.
	Noop,
}

/// Create a new verifier based on type.
pub fn new<C: BlockInfo + CallContract>(v: VerifierType) -> Box<dyn Verifier<C>> {
	match v {
		VerifierType::Canon | VerifierType::CanonNoSeal => Box::new(CanonVerifier),
		VerifierType::Noop => Box::new(NoopVerifier),
	}
}

impl VerifierType {
	/// Check if seal verification is enabled for this verifier type.
	pub fn verifying_seal(&self) -> bool {
		match *self {
			VerifierType::Canon => true,
			VerifierType::Noop | VerifierType::CanonNoSeal => false,
		}
	}
}
