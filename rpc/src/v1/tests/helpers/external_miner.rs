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
use v1::helpers::ExternalMinerService;

/// Test ExternalMinerService;
pub struct TestExternalMiner {
	/// External miners hashrates.
	pub hashrates: Arc<RwLock<HashMap<H256, U256>>>
}

impl TestExternalMiner {
	/// Creates new external miner.
	pub fn new(hashrates: Arc<RwLock<HashMap<H256, U256>>>) -> Self {
		TestExternalMiner {
			hashrates: hashrates,
		}
	}
}

impl ExternalMinerService for TestExternalMiner {
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
