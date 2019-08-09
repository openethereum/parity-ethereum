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

//! Helper for ancient block import.

use std::sync::Arc;

use engine::{Engine, EpochVerifier};

use blockchain::BlockChain;
use parking_lot::RwLock;
use rand::Rng;
use types::{
	header::Header,
	errors::EthcoreError,
};

// do "heavy" verification on ~1/50 blocks, randomly sampled.
const HEAVY_VERIFY_RATE: f32 = 0.02;

/// Ancient block verifier: import an ancient sequence of blocks in order from a starting
/// epoch.
pub struct AncientVerifier {
	cur_verifier: RwLock<Option<Box<dyn EpochVerifier>>>,
	engine: Arc<dyn Engine>,
}

impl AncientVerifier {
	/// Create a new ancient block verifier with the given engine.
	pub fn new(engine: Arc<dyn Engine>) -> Self {
		AncientVerifier {
			cur_verifier: RwLock::new(None),
			engine,
		}
	}

	/// Verify the next block header, randomly choosing whether to do heavy or light
	/// verification. If the block is the end of an epoch, updates the epoch verifier.
	pub fn verify<R: Rng>(
		&self,
		rng: &mut R,
		header: &Header,
		chain: &BlockChain,
	) -> Result<(), EthcoreError> {
		// perform verification
		let verified = if let Some(ref cur_verifier) = *self.cur_verifier.read() {
			match rng.gen::<f32>() <= HEAVY_VERIFY_RATE {
				true => cur_verifier.verify_heavy(header)?,
				false => cur_verifier.verify_light(header)?,
			}
			true
		} else {
			false
		};

		// when there is no verifier initialize it.
		// We use a bool flag to avoid double locking in the happy case
		if !verified {
			{
				let mut cur_verifier = self.cur_verifier.write();
				if cur_verifier.is_none() {
					*cur_verifier = Some(self.initial_verifier(header, chain)?);
				}
			}
			// Call again to verify.
			return self.verify(rng, header, chain);
		}

		// ancient import will only use transitions obtained from the snapshot.
		if let Some(transition) = chain.epoch_transition(header.number(), header.hash()) {
			let v = self.engine.epoch_verifier(&header, &transition.proof).known_confirmed()?;
			*self.cur_verifier.write() = Some(v);
		}

		Ok(())
	}

	fn initial_verifier(&self, header: &Header, chain: &BlockChain)
		-> Result<Box<dyn EpochVerifier>, EthcoreError>
	{
		trace!(target: "client", "Initializing ancient block restoration.");
		let current_epoch_data = chain.epoch_transitions()
			.take_while(|&(_, ref t)| t.block_number < header.number())
			.last()
			.map(|(_, t)| t.proof)
			.expect("At least one epoch entry (genesis) always stored; qed");

		self.engine.epoch_verifier(&header, &current_epoch_data).known_confirmed()
	}
}
