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

//! Helper for ancient block import.

use std::sync::Arc;

use engines::{Engine, EpochVerifier, EpochChange};
use error::Error;
use header::Header;

use rand::Rng;
use util::RwLock;

// do "heavy" verification on ~1/50 blocks, randomly sampled.
const HEAVY_VERIFY_RATE: f32 = 0.02;

/// Ancient block verifier: import an ancient sequence of blocks in order from a starting
/// epoch.
pub struct AncientVerifier {
	cur_verifier: RwLock<Box<EpochVerifier>>,
	engine: Arc<Engine>,
}

impl AncientVerifier {
	/// Create a new ancient block verifier with the given engine and initial verifier.
	pub fn new(engine: Arc<Engine>, start_verifier: Box<EpochVerifier>) -> Self {
		AncientVerifier {
			cur_verifier: RwLock::new(start_verifier),
			engine: engine,
		}
	}

	/// Verify the next block header, randomly choosing whether to do heavy or light
	/// verification. If the block is the end of an epoch, updates the epoch verifier.
	pub fn verify<R: Rng, F: Fn(u64) -> Result<Box<EpochVerifier>, Error>>(
		&self,
		rng: &mut R,
		header: &Header,
		block: &[u8],
		receipts: &[::receipt::Receipt],
		load_verifier: F,
	) -> Result<(), ::error::Error> {
		match rng.gen::<f32>() <= HEAVY_VERIFY_RATE {
			true => self.cur_verifier.read().verify_heavy(header)?,
			false => self.cur_verifier.read().verify_light(header)?,
		}

		if let EpochChange::Yes(num) = self.engine.is_epoch_end(header, Some(block), Some(receipts)) {
			*self.cur_verifier.write() = load_verifier(num)?;
		}

		Ok(())
	}
}
