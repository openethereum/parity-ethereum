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

//! Epoch verifiers and transitions.

use util::H256;
use error::Error;
use header::Header;

/// A full epoch transition.
#[derive(Debug, Clone, RlpEncodable, RlpDecodable)]
pub struct Transition {
	/// Block hash at which the transition occurred.
	pub block_hash: H256,
	/// Block number at which the transition occurred.
	pub block_number: u64,
	/// "transition/epoch" proof from the engine combined with a finality proof.
	pub proof: Vec<u8>,
}

/// An epoch transition pending a finality proof.
/// Not all transitions need one.
#[derive(RlpEncodableWrapper, RlpDecodableWrapper)]
pub struct PendingTransition {
	/// "transition/epoch" proof from the engine.
	pub proof: Vec<u8>,
}

/// Verifier for all blocks within an epoch with self-contained state.
///
/// See docs on `Engine` relating to proving functions for more details.
pub trait EpochVerifier: Send + Sync {
	/// Lightly verify the next block header.
	/// This may not be a header belonging to a different epoch.
	fn verify_light(&self, header: &Header) -> Result<(), Error>;

	/// Perform potentially heavier checks on the next block header.
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_light(header)
	}

	/// Check a finality proof against this epoch verifier.
	/// Returns `Some(hashes)` if the proof proves finality of these hashes.
	/// Returns `None` if the proof doesn't prove anything.
	fn check_finality_proof(&self, _proof: &[u8]) -> Option<Vec<H256>> {
		None
	}
}

/// Special "no-op" verifier for stateless, epoch-less engines.
pub struct NoOp;

impl EpochVerifier for NoOp {
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
}
