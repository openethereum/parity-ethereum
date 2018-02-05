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

//! External Miner hashrate tracker.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, Duration};
use ethereum_types::{H256, U256};
use parking_lot::Mutex;

/// External miner interface.
pub trait ExternalMinerService: Send + Sync {
	/// Submit hashrate for given miner.
	fn submit_hashrate(&self, hashrate: U256, id: H256);

	/// Total hashrate.
	fn hashrate(&self) -> U256;
}

/// External Miner.
pub struct ExternalMiner {
	hashrates: Arc<Mutex<HashMap<H256, (Instant, U256)>>>,
}

impl Default for ExternalMiner {
	fn default() -> Self {
		ExternalMiner {
			hashrates: Arc::new(Mutex::new(HashMap::new())),
		}
	}
}

impl ExternalMiner {
	/// Creates new external miner with prefilled hashrates.
	pub fn new(hashrates: Arc<Mutex<HashMap<H256, (Instant, U256)>>>) -> Self {
		ExternalMiner {
			hashrates: hashrates,
		}
	}
}

const ENTRY_TIMEOUT: u64 = 2;

impl ExternalMinerService for ExternalMiner {
	fn submit_hashrate(&self, hashrate: U256, id: H256) {
		self.hashrates.lock().insert(id, (Instant::now() + Duration::from_secs(ENTRY_TIMEOUT), hashrate));
	}

	fn hashrate(&self) -> U256 {
		let mut hashrates = self.hashrates.lock();
		let h = hashrates.drain().filter(|&(_, (t, _))| t > Instant::now()).collect();
		*hashrates = h;
		hashrates.iter().fold(U256::from(0), |sum, (_, &(_, v))| sum + v)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::thread::sleep;
	use std::time::Duration;
	use ethereum_types::{H256, U256};

	fn miner() -> ExternalMiner {
		ExternalMiner::default()
	}

	#[test]
	fn it_should_forget_old_hashrates() {
		// given
		let m = miner();
		assert_eq!(m.hashrate(), U256::from(0));
		m.submit_hashrate(U256::from(10), H256::from(1));
		assert_eq!(m.hashrate(), U256::from(10));

		// when
		sleep(Duration::from_secs(3));

		// then
		assert_eq!(m.hashrate(), U256::from(0));
	}

	#[test]
	fn should_sum_up_hashrate() {
		// given
		let m = miner();
		assert_eq!(m.hashrate(), U256::from(0));
		m.submit_hashrate(U256::from(10), H256::from(1));
		assert_eq!(m.hashrate(), U256::from(10));

		// when
		m.submit_hashrate(U256::from(15), H256::from(1));
		m.submit_hashrate(U256::from(20), H256::from(2));


		// then
		assert_eq!(m.hashrate(), U256::from(35));
	}
}
