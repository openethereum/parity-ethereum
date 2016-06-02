// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use util::numbers::U256;
use util::hash::H256;

/// External miner interface.
pub trait ExternalMinerService: Send + Sync {
	/// Submit hashrate for given miner.
	fn submit_hashrate(&self, hashrate: U256, id: H256);

	/// Total hashrate.
	fn hashrate(&self) -> U256;

	/// Returns true if external miner is mining.
	fn is_mining(&self) -> bool;
}

/// External Miner.
pub struct ExternalMiner {
	hashrates: Arc<RwLock<HashMap<H256, U256>>>,
}

impl Default for ExternalMiner {
	fn default() -> Self {
		ExternalMiner {
			hashrates: Arc::new(RwLock::new(HashMap::new())),
		}
	}
}

impl ExternalMiner {
	/// Creates new external miner with prefilled hashrates.
	pub fn new(hashrates: Arc<RwLock<HashMap<H256, U256>>>) -> Self {
		ExternalMiner {
			hashrates: hashrates
		}
	}
}

impl ExternalMinerService for ExternalMiner {
	fn submit_hashrate(&self, hashrate: U256, id: H256) {
		self.hashrates.write().unwrap().insert(id, hashrate);
	}

	fn hashrate(&self) -> U256 {
		self.hashrates.read().unwrap().iter().fold(U256::from(0), |sum, (_, v)| sum + *v)
	}

	fn is_mining(&self) -> bool {
		!self.hashrates.read().unwrap().is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use util::{H256, U256};

	fn miner() -> ExternalMiner {
		ExternalMiner::default()
	}

	#[test]
	fn should_return_that_is_mining_if_there_is_at_least_one_entry() {
		// given
		let m = miner();
		assert_eq!(m.is_mining(), false);

		// when
		m.submit_hashrate(U256::from(10), H256::from(1));

		// then
		assert_eq!(m.is_mining(), true);
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
