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

//! Block verification utilities.

pub mod verification;
pub mod verifier;
pub mod queue;
mod canon_verifier;
mod noop_verifier;

pub use self::verification::*;
pub use self::verifier::Verifier;
pub use self::canon_verifier::CanonVerifier;
pub use self::noop_verifier::NoopVerifier;
pub use self::queue::{BlockQueue, Config as QueueConfig, VerificationQueue, QueueInfo};

use client::{BlockInfo, CallContract};

/// Verifier type.
#[derive(Debug, PartialEq, Clone)]
pub enum VerifierType {
	/// Verifies block normally.
	Canon,
	/// Verifies block normallly, but skips seal verification.
	CanonNoSeal,
	/// Does not verify block at all.
	/// Used in tests.
	Noop,
}

impl Default for VerifierType {
	fn default() -> Self {
		VerifierType::Canon
	}
}

/// Create a new verifier based on type.
pub fn new<C: BlockInfo + CallContract>(v: VerifierType) -> Box<Verifier<C>> {
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
