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

//! Snapshot test helpers. These are used to build blockchains and state tries
//! which can be queried before and after a full snapshot/restore cycle.

use std::collections::HashMap;

use account_db::{AccountDB, AccountDBMut};
use rand::Rng;

use util::hash::{Address, H256};
use util::hashdb::HashDB;
use util::trie::{TrieDBMut, TrieDB};
use util::rlp::SHA3_NULL_RLP;

// the proportion of accounts we will alter each tick.
const ACCOUNT_CHURN: f32 = 0.01;

/// This structure will incrementally alter a state given an rng and can produce a list of "facts"
/// for checking state validity.
pub struct StateProducer {
	state_root: H256
}

impl StateProducer {
	/// Create a new `StateProducer`.
	pub fn new() -> Self {
		StateProducer {
			state_root: SHA3_NULL_RLP,
		}
	}

	/// Tick the state producer. This alters the state, writing new data into
	/// the database and returning the new state root.
	pub fn tick(&mut self, rng: &mut Rng, db: &mut HashDB) -> H256 {
		unimplemented!()
	}
}