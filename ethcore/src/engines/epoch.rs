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

// Epoch verifiers.

use error::Error;
use header::{BlockNumber, Header};

use rlp::{Encodable, Decodable, DecoderError, RlpStream, UntrustedRlp};
use util::H256;

/// A full epoch transition.
#[derive(Debug, Clone)]
pub struct Transition {
	/// Block hash at which the transition occurred.
	pub block_hash: H256,
	/// Block number at which the transition occurred.
	pub block_number: u64,
	/// "transition/epoch" proof from the engine.
	pub proof: Vec<u8>,
	/// Finality proof, if necessary.
	pub finality_proof: Option<Vec<u8>>,
}

impl Encodable for Transition {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4)
			.append(&self.block_hash)
			.append(&self.block_number)
			.append(&self.proof)
			.append(&self.finality_proof)
	}
}

impl Decodable for Transition {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		Ok(Transition {
			block_hash: rlp.val_at(0)?,
			block_number: rlp.val_at(1)?,
			proof: rlp.val_at(2)?,
			finality_proof: rlp.val_at(3)?,
		})
	}
}

/// An epoch transition pending a finality proof.
/// Not all transitions need one.
pub struct PendingTransition {
	/// "transition/epoch" proof from the engine.
	pub proof: Vec<u8>,
}

impl Encodable for PendingTransition {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.proof)
	}
}

impl Decodable for PendingTransition {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		Ok(PendingTransition {
			proof: rlp.as_val()?,
		})
	}
}

/// Verifier for all blocks within an epoch with self-contained state.
///
/// See docs on `Engine` relating to proving functions for more details.
pub trait EpochVerifier: Send + Sync {
	/// Get the epoch number.
	fn epoch_number(&self) -> u64;

	/// Lightly verify the next block header.
	/// This may not be a header belonging to a different epoch.
	fn verify_light(&self, header: &Header) -> Result<(), Error>;

	/// Perform potentially heavier checks on the next block header.
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_light(header)
	}

	/// Check a finality proof against this epoch verifier.
	fn check_finality_proof(&self, _proof: &[u8]) -> Result<(), Error> {
		Ok(())
	}
}

/// Special "no-op" verifier for stateless, epoch-less engines.
pub struct NoOp;

impl EpochVerifier for NoOp {
	fn epoch_number(&self) -> u64 { 0 }
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
}

/// Stores pending transitions.
pub trait PendingTransitionStore {
	/// Insert a pending transition by block hash.
	fn insert(&mut self, H256, PendingTransition);

	/// Remove a pending transition by block hash. Should only be called when
	/// that block reaches finality. Once a transition has been removed
	/// it cannot be removed again.
	fn remove(&mut self, H256) -> Option<PendingTransition>;
}
