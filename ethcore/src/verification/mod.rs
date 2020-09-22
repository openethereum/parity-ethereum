// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Block verification utilities.

mod canon_verifier;
mod noop_verifier;
pub mod queue;
mod verification;
mod verifier;

pub use self::{
    canon_verifier::CanonVerifier,
    noop_verifier::NoopVerifier,
    queue::{BlockQueue, Config as QueueConfig, QueueInfo, VerificationQueue},
    verification::*,
    verifier::Verifier,
};

use call_contract::CallContract;
use client::BlockInfo;

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
