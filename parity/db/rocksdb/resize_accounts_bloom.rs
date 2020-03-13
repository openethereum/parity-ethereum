// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Resize the accounts bloom filter for modern times
//! todo[dvdplm] document the choice of parameters etc

use std::path::Path;
use types::errors::EthcoreError as Error;
use super::kvdb_rocksdb::{Database, DatabaseConfig};

use std::sync::Arc;
use ethcore_db::{COL_EXTRA, COL_HEADERS, COL_STATE, COL_ACCOUNT_BLOOM};
use super::state_db::{ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET, StateDB};
use super::ethtrie::TrieDB;
use super::accounts_bloom::Bloom; // todo[dvdplm] rename this crate
use ethereum_types::H256;
use journaldb;
use kvdb::{DBTransaction, KeyValueDB};
use trie_db::Trie;
use types::views::{HeaderView, ViewRlp};
use ethereum_types::U256;
use super::account_state::account::Account as StateAccount;
use std::str::FromStr;
use std::collections::HashSet;

pub fn resize_accounts_bloom<P: AsRef<Path>>(path: P, db_config: &DatabaseConfig) -> Result<(), Error> {
	let path_str = path.as_ref().to_string_lossy();
	let db = Arc::new(Database::open(&db_config, &path_str)?);

	generate_bloom(db)
}


/// Account bloom upgrade routine. If bloom already present, does nothing.
/// If database empty (no best block), does nothing.
/// Can be called on upgraded database with no issues (will do nothing).
// pub fn generate_bloom(source: Arc<Database>, dest: &mut Database) -> Result<(), Error> {
pub fn generate_bloom(source: Arc<Database>) -> Result<(), Error> {
	info!(target: "migration", "Account bloom upgrade started");
	let best_block_hash = match source.get(COL_EXTRA, b"best")? {
		// no migration needed
		None => {
			warn!(target: "migration", "No best block hash, skipping");
			return Ok(());
		},
		Some(hash) => hash,
	};
	let best_block_header = match source.get(COL_HEADERS, &best_block_hash)? {
		// no best block, nothing to do
		None => {
			warn!(target: "migration", "No best block header, skipping");
			return Ok(())
		},
		Some(x) => x,
	};
	let view = ViewRlp::new(&best_block_header, "", 1);
	let state_root = HeaderView::new(view).state_root();
	trace!(target: "dp", "state root migration: {:?}", state_root);

	let num_keys_before_del = source.num_keys(COL_ACCOUNT_BLOOM)?;
	info!("Clearing out old accounts bloom ({} keys)", num_keys_before_del);
	let mut batch = DBTransaction::new();
	for (n, (k,_)) in source.iter(COL_ACCOUNT_BLOOM).enumerate() {
		if n > 0 && n % 10_000 == 0 {
			info!("Bloom entries processed: {}", n);
		}
		batch.delete(COL_ACCOUNT_BLOOM, &k);
	}
	debug!(target: "migration", "bloom items to delete {}", batch.ops.len());
	source.write(batch)?;
	source.flush()?;
	let num_keys_after_del = source.num_keys(COL_ACCOUNT_BLOOM)?;
	info!("Cleared out old accounts bloom ({} keys)", num_keys_after_del);

	info!("Creating the accounts bloom (one-time upgrade)");

	let mut empties = 0u64;
	let mut non_empties = 0u64;

	let mut hs: HashSet<H256> = HashSet::new();
	// All hashes that are looked up before verification failure
	hs.insert(H256::from_str("1c5e6ae7c6cb17fc942a0c5c4061a4168fb31c0380dae26acea079b8508a4884").unwrap());
	hs.insert(H256::from_str("5380c7b7ae81a58eb98d9c78de4a1fd7fd9535fc953ed2be602daaa41767312a").unwrap());
	hs.insert(H256::from_str("59e7449aaced683b3ca8826910182e66444f16da575d9751b28a59f44e70d0b1").unwrap());
	hs.insert(H256::from_str("a7c33937fe86045ad5fc94249d77bd8d94d5ba11a9977602d8771e806e621621").unwrap());
	hs.insert(H256::from_str("bc370692f7423557740dbac30cb0aecf3af03d5900eba4f47a759381d4a18578").unwrap());
	hs.insert(H256::from_str("d0220c1805eeae7d9e997053ec79f29a83128d3394ecdf1916caa0ffea57b3dd").unwrap());
	hs.insert(H256::from_str("dc0b9c5df8db94d2d30c482ff54d87d8734b22ab667dadae2b3c3dfcf0174167").unwrap());

	// let bloom_journal = {
	let mut bloom = {
		let mut bloom = Bloom::new(ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET);
		// no difference what algorithm is passed, since there will be no writes
		let state_db = journaldb::new(
			source.clone(),
			journaldb::Algorithm::OverlayRecent,
			COL_STATE);

		let db = state_db.as_hash_db();

		let account_trie = TrieDB::new(&db, &state_root)?;

		let empty_account_rlp = StateAccount::new_basic(U256::zero(), U256::zero()).rlp();

		for (n, (account_key, account_data)) in account_trie.iter()?.filter_map(Result::ok).enumerate() {
			if n > 0 && n % 10_000 == 0 {
				info!("Accounts processed: {}. Bloom saturation: {} – length of an account key from the db: {}", n, bloom.saturation(), account_key.len());
			}
			// let basic_account: BasicAccount = rlp::decode(&*account_data).expect("rlp from disk is ok");
			let account_key_hash = H256::from_slice(&account_key);

			if account_data == empty_account_rlp {
				// debug!(target: "migration", "Empty account at hash: {:?}, data={:?}", account_key_hash, account_data);
				if hs.contains(&account_key_hash) {
					debug!(target: "migration", "DB contains {:?} (empty account though)", account_key_hash);
				}
				empties += 1;
			} else {
				if hs.contains(&account_key_hash) {
					debug!(target: "migration", "DB contains {:?} (not empty)", account_key_hash);
				}
				non_empties += 1;
				// bloom.set(account_key_hash);
				bloom.set(account_key);
			}
		}

		// bloom.drain_journal()
		bloom
	};

	for h in hs.iter() {
		if bloom.check(h) {
			debug!(target: "migration", "Bloom says {:?} is in the DB", h);
		} else {
			debug!(target: "migration", "Bloom says {:?} is NOT the DB", h);
		}
	}
	let bloom_journal = bloom.drain_journal();
	info!(target: "migration", "Generated {} bloom updates, empty accounts={}, non-empty accounts={}", bloom_journal.entries.len(), empties, non_empties);
	info!(target: "migration", "k_bits (aka 'hash functions') in the resized BloomJournal: {}", bloom_journal.hash_functions);
	let mut batch = DBTransaction::new();
	StateDB::commit_bloom(&mut batch, bloom_journal)?;
	source.write(batch)?;
	source.flush()?;
	let num_keys_after_insert = source.num_keys(COL_ACCOUNT_BLOOM)?;
	debug!(target: "migration", "Keys in account bloom after insert: {}", num_keys_after_insert);

	if let Ok(inner) = Arc::try_unwrap(source) {
		let bloom2 = StateDB::load_bloom(&inner as &dyn KeyValueDB);
		debug!(target: "migration", "Loaded new bloom from DB. Saturation={}, number_of_bits={}, number_of_hash_functions={}",
			bloom2.saturation(),
			bloom2.number_of_bits(),
			bloom2.number_of_hash_functions(),
		);
		for h in hs.iter() {
			if bloom2.check(h) {
				debug!(target: "migration", "Reloaded bloom says {:?} is in the DB", h);
			} else {
				debug!(target: "migration", "Reloaded bloom says {:?} is NOT the DB", h);
			}
		}
	}
	// debug!(target: "migration", "NOT WRITING THE NEW BLOOM");
	info!(target: "migration", "Finished bloom update");


	Ok(())
}
