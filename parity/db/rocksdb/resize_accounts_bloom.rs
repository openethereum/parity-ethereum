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
use kvdb::DBTransaction;
use trie_db::Trie;
use types::views::{HeaderView, ViewRlp};

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

	let bloom_journal = {
		let mut bloom = Bloom::new(ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET);
		// no difference what algorithm is passed, since there will be no writes
		let state_db = journaldb::new(
			source.clone(),
			journaldb::Algorithm::OverlayRecent,
			COL_STATE);

		let db = state_db.as_hash_db();
		let account_trie = TrieDB::new(&db, &state_root)?;

		// for item in account_trie.iter().map_err(|_| Error::Msg("oh noes".to_string()))? {
		// 	let (ref account_key, _) = item.map_err(|_| Error::Msg("oh noes 2".to_string()))?;
		// 	let account_key_hash = H256::from_slice(&account_key);
		// 	bloom.set(&account_key_hash);
		// 	// if n > 0 && n % 10_000 == 0 {
		// 	// 	info!("Accounts processed: {}. Bloom saturation: {}", n, bloom.saturation());
		// 	// }
		// }

		for (n, (account_key, _)) in account_trie.iter()?.filter_map(Result::ok).enumerate() {
			if n > 0 && n % 10_000 == 0 {
				info!("Accounts processed: {}. Bloom saturation: {}", n, bloom.saturation());
			}
			let account_key_hash = H256::from_slice(&account_key);
			bloom.set(account_key_hash);
		}

		bloom.drain_journal()
	};

	info!(target: "migration", "Generated {} bloom updates", bloom_journal.entries.len());

	let mut batch = DBTransaction::new();
	StateDB::commit_bloom(&mut batch, bloom_journal)?;
	source.write(batch)?;
	source.flush()?;
	let num_keys_after_insert = source.num_keys(COL_ACCOUNT_BLOOM)?;
	debug!(target: "migration", "Keys after insert: {}", num_keys_after_insert);
	// debug!(target: "migration", "NOT WRITING THE NEW BLOOM");
	info!(target: "migration", "Finished bloom update");


	Ok(())
}
