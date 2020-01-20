// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Epoch verifiers and transitions.

use ethereum_types::H256;
use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};

/// A full epoch transition.
#[derive(Debug, Clone)]
pub struct Transition {
	/// Block hash at which the transition occurred.
	pub block_hash: H256,
	/// Block number at which the transition occurred.
	pub block_number: u64,
	/// "transition/epoch" proof from the engine combined with a finality proof.
	pub proof: Vec<u8>,
}

impl Encodable for Transition {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3)
			.append(&self.block_hash)
			.append(&self.block_number)
			.append(&self.proof);
	}
}

impl Decodable for Transition {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		Ok(Transition {
			block_hash: rlp.val_at(0)?,
			block_number: rlp.val_at(1)?,
			proof: rlp.val_at(2)?,
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
		s.append(&self.proof);
	}
}

impl Decodable for PendingTransition {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		Ok(PendingTransition {
			proof: rlp.as_val()?,
		})
	}
}
