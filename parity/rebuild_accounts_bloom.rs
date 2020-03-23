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


extern crate kvdb_rocksdb;
extern crate state_db;
extern crate patricia_trie_ethereum as ethtrie;
extern crate account_state;
extern crate ethcore_bloom_journal as accounts_bloom;
extern crate trie_db;

use std::{
	path::Path,
	sync::Arc,
};

use ethcore_db::{COL_EXTRA, COL_HEADERS, COL_STATE, COL_ACCOUNT_BLOOM};
use ethereum_types::{H256, U256};
use journaldb;
use kvdb::DBTransaction;
use self::{
	account_state::account::Account as StateAccount,
	accounts_bloom::Bloom, // todo[dvdplm] rename this crate
	ethtrie::TrieDB,
	kvdb_rocksdb::{Database, DatabaseConfig},
	state_db::{ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET, StateDB},
	trie_db::Trie,
};
use types::{
	errors::EthcoreError as Error,
	views::{HeaderView, ViewRlp},
};
use rlp::{RlpStream, Rlp};
use self::kvdb_rocksdb::CompactionProfile;

pub fn rebuild_accounts_bloom<P: AsRef<Path>>(
	path: P,
	compaction: CompactionProfile
) -> Result<(), Error> {
	let db_config = DatabaseConfig {
		compaction,
		columns: ethcore_db::NUM_COLUMNS,
		..Default::default()
	};
	let path_str = path.as_ref().to_string_lossy();
	let db = Arc::new(Database::open(&db_config, &path_str)?);

	generate_bloom(db)?;
	Ok(())
}

pub fn backup_bloom(bloom_backup_path: &Path, source: Arc<Database>) -> Result<(), Error> {
	let num_keys = source.num_keys(COL_ACCOUNT_BLOOM)? / 2;
	if num_keys == 0 {
		warn!("No bloom in the DB to back up");
		return Ok(())
	}

	let mut bloom_backup = std::fs::File::create(bloom_backup_path)
		.map_err(|_| format!("Cannot write to file given: {}", bloom_backup_path.display()))?;

	info!("Saving old bloom to '{}'", bloom_backup_path.display());
	let mut stream = RlpStream::new();
	stream.begin_unbounded_list();
	for (n, (k, v)) in source.iter(COL_ACCOUNT_BLOOM).enumerate() {
		stream
			.begin_list(2)
			.append(&k.to_vec())
			.append(&v.to_vec());
		if n > 0 && n % 50_000 == 0 {
			info!("  Bloom entries processed: {}", n);
		}
	}
	stream.finalize_unbounded_list();

	use std::io::Write;
	let written = bloom_backup.write(&stream.out())?;
	info!("Saved old bloom to '{}' ({} bytes, {} keys)", bloom_backup_path.display(), written, num_keys);
	Ok(())
}

fn restore_bloom(bloom_backup_path: &Path, db: Arc<Database>) -> Result<(), Error> {
	let mut bloom_backup = std::fs::File::open(bloom_backup_path)?;
	info!("Restoring bloom from '{}'", bloom_backup_path.display());
	let num_keys = db.num_keys(COL_ACCOUNT_BLOOM)? / 2;
	if num_keys != 0 {
		warn!("Will not overwrite existing bloom! ({} items found in the DB)", num_keys);
		return Err(format!("Blooms DB column is not empty").into())
	}
	let mut buf = Vec::with_capacity(10_000_000);
	use std::io::Read;
	let bytes_read = bloom_backup.read_to_end(&mut buf)?;
	let rlp = Rlp::new(&buf);
	info!("{} bloom key/values and {} bytes read from disk", rlp.item_count()?, bytes_read);

	let mut batch = DBTransaction::with_capacity(rlp.item_count()?);
	for (n, kv_rlp) in rlp.iter().enumerate() {
		let kv: Vec<Vec<u8>> = kv_rlp.as_list()?;
		assert_eq!(kv.len(), 2);
		batch.put(COL_ACCOUNT_BLOOM, &kv[0], &kv[1]);
		if n > 0 && n % 10_000 == 0 {
			info!("  Bloom entries prepared for restoration: {}", n);
		}
	}
	db.write(batch)?;
	db.flush()?;
	info!("Bloom restored ({} bytes)", bytes_read);
	Ok(())
}

fn clear_bloom(db: Arc<Database>) -> Result<(), Error> {
	let num_keys = db.num_keys(COL_ACCOUNT_BLOOM)? / 2;
	info!("Clearing out old accounts bloom ({} keys)", num_keys);
	let mut batch = DBTransaction::with_capacity(num_keys as usize);
	for (n, (k,_)) in db.iter(COL_ACCOUNT_BLOOM).enumerate() {
		batch.delete(COL_ACCOUNT_BLOOM, &k);
		if n > 0 && n % 10_000 == 0 {
			info!("  Bloom entries queued for deletion: {}", n);
		}
	}
	let deletions = batch.ops.len();
	db.write(batch)?;
	db.flush().map_err(|e| Error::StdIo(e))?;
	info!("Deleted {} old bloom items from the DB", deletions);
	Ok(())
}

/// Rebuild the account bloom.
fn generate_bloom(source: Arc<Database>) -> Result<(), Error> {
	info!(target: "migration", "Account bloom rebuild started");
	let best_block_hash = match source.get(COL_EXTRA, b"best")? {
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

	// todo[dvdplm]: need a param `--to=…` for the user to save the old bloom somewhere.
	use std::time::{SystemTime, UNIX_EPOCH};
	let bloom_backup_path_str = format!("./bloom-backup-{:?}.bin", SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock error").as_secs());
	let bloom_backup_path = std::path::Path::new(&bloom_backup_path_str);
	backup_bloom(bloom_backup_path, source.clone())?;
	clear_bloom(source.clone())?;

	// todo[dvdplm]: need a restore command for this
	// let test_path = std::path::Path::new("./bloom-backup-1584359135.bin");
	// restore_bloom(test_path, source.clone())?;
	// info!("STOP");
	// return Ok(());

	let mut empty_accounts = 0u64;
	let mut non_empty_accounts = 0u64;

	let mut bloom = {
		let mut bloom = Bloom::new(ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET);
		let state_db = journaldb::new(
			source.clone(),
			// It does not matter which `journaldb::Algorithm` is used since
			// there will be no writes to the state column.
			journaldb::Algorithm::OverlayRecent,
			COL_STATE);

		let db = state_db.as_hash_db();
		let view = ViewRlp::new(&best_block_header, "", 1);
		let state_root = HeaderView::new(view).state_root();
		let account_trie = TrieDB::new(&db, &state_root)?;
		// Don't insert empty accounts into the bloom
		let empty_account_rlp = StateAccount::new_basic(U256::zero(), U256::zero()).rlp();
		let start = std::time::Instant::now();
		let mut batch_start = std::time::Instant::now();
		for (n, (account_key, account_data)) in account_trie.iter()?.filter_map(Result::ok).enumerate() {
			if n > 0 && n % 50_000 == 0 {
				info!("  Accounts processed: {} in {:?}. Bloom saturation: {}", n, batch_start.elapsed(), bloom.saturation());
				batch_start = std::time::Instant::now();
			}
			if account_data != empty_account_rlp {
				non_empty_accounts += 1;
				let account_key_hash = H256::from_slice(&account_key);
				bloom.set(account_key_hash);
			} else {
				empty_accounts += 1;
			}
		}
		info!("Finished iterating over the accounts in: {:?}. Bloom saturation: {}", start.elapsed(), bloom.saturation());
		bloom
	};

	let bloom_journal = bloom.drain_journal();
	info!(target: "migration", "Generated {} bloom entries; the DB has {} empty accounts and {} non-empty accounts", bloom_journal.entries.len(), empty_accounts, non_empty_accounts);
	info!(target: "migration", "New bloom has {} k_bits (aka 'hash functions')", bloom_journal.hash_functions);
	let mut batch = DBTransaction::new();
	StateDB::commit_bloom(&mut batch, bloom_journal)?;
	source.write(batch)?;
	source.flush()?;
	info!(target: "migration", "Finished bloom update");
	Ok(())
}
