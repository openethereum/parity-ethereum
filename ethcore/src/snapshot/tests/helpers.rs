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

//! Snapshot test helpers. These are used to build blockchains and state tries
//! which can be queried before and after a full snapshot/restore cycle.

use basic_account::BasicAccount;
use account_db::AccountDBMut;
use rand::Rng;

use util::DBValue;
use util::hash::{FixedHash, H256};
use util::hashdb::HashDB;
use util::trie::{Alphabet, StandardMap, SecTrieDBMut, TrieMut, ValueMode};
use util::trie::{TrieDB, TrieDBMut, Trie};
use util::sha3::SHA3_NULL_RLP;

// the proportion of accounts we will alter each tick.
const ACCOUNT_CHURN: f32 = 0.01;

/// This structure will incrementally alter a state given an rng.
pub struct StateProducer {
	state_root: H256,
	storage_seed: H256,
}

impl StateProducer {
	/// Create a new `StateProducer`.
	pub fn new() -> Self {
		StateProducer {
			state_root: SHA3_NULL_RLP,
			storage_seed: H256::zero(),
		}
	}

	#[cfg_attr(feature="dev", allow(let_and_return))]
	/// Tick the state producer. This alters the state, writing new data into
	/// the database.
	pub fn tick<R: Rng>(&mut self, rng: &mut R, db: &mut HashDB) {
		// modify existing accounts.
		let mut accounts_to_modify: Vec<_> = {
			let trie = TrieDB::new(&*db, &self.state_root).unwrap();
			let temp = trie.iter().unwrap() // binding required due to complicated lifetime stuff
				.filter(|_| rng.gen::<f32>() < ACCOUNT_CHURN)
				.map(Result::unwrap)
				.map(|(k, v)| (H256::from_slice(&k), v.to_owned()))
				.collect();

			temp
		};

		// sweep once to alter storage tries.
		for &mut (ref mut address_hash, ref mut account_data) in &mut accounts_to_modify {
			let mut account: BasicAccount = ::rlp::decode(&*account_data);
			let acct_db = AccountDBMut::from_hash(db, *address_hash);
			fill_storage(acct_db, &mut account.storage_root, &mut self.storage_seed);
			*account_data = DBValue::from_vec(::rlp::encode(&account).to_vec());
		}

		// sweep again to alter account trie.
		let mut trie = TrieDBMut::from_existing(db, &mut self.state_root).unwrap();

		for (address_hash, account_data) in accounts_to_modify {
			trie.insert(&address_hash[..], &account_data).unwrap();
		}

		// add between 0 and 5 new accounts each tick.
		let new_accs = rng.gen::<u32>() % 5;

		for _ in 0..new_accs {
			let address_hash = H256(rng.gen());
			let balance: usize = rng.gen();
			let nonce: usize = rng.gen();
			let acc = ::state::Account::new_basic(balance.into(), nonce.into()).rlp();
			trie.insert(&address_hash[..], &acc).unwrap();
		}
	}

	/// Get the current state root.
	pub fn state_root(&self) -> H256 {
		self.state_root
	}
}

/// Fill the storage of an account.
pub fn fill_storage(mut db: AccountDBMut, root: &mut H256, seed: &mut H256) {
	let map = StandardMap {
		alphabet: Alphabet::All,
		min_key: 6,
		journal_key: 6,
		value_mode: ValueMode::Random,
		count: 100,
	};
	{
		let mut trie = if *root == SHA3_NULL_RLP {
			SecTrieDBMut::new(&mut db, root)
		} else {
			SecTrieDBMut::from_existing(&mut db, root).unwrap()
		};

		for (k, v) in map.make_with(seed) {
			trie.insert(&k, &v).unwrap();
		}
	}
}

/// Compare two state dbs.
pub fn compare_dbs(one: &HashDB, two: &HashDB) {
	let keys = one.keys();

	for key in keys.keys() {
		assert_eq!(one.get(&key).unwrap(), two.get(&key).unwrap());
	}
}
