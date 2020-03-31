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

//! Resize the accounts bloom filter for modern times. ! The accounts bloom
//! filter provides a way to check if a given account (`Address`) exists or not
//! without touching the database. The filter cannot be resized with less than a
//! complete rebuild, i.e. iterate over all accounts in the state database and
//! mark each account in the bloom bitmap. At the time of writing the number of
//! ethereum accounts is ~85M and increasing. This module implements backing up,
//! clearing, rebuilding and restoring the accounts bloom filter.

extern crate account_state;
extern crate ethcore_bloom_journal as accounts_bloom;
extern crate kvdb_rocksdb;
extern crate patricia_trie_ethereum as ethtrie;
extern crate state_db;
extern crate trie_db;
extern crate crossbeam_utils;

use std::{
	path::Path,
	sync::{Arc, atomic::{AtomicU64, Ordering}},
};

use ethcore_db::{COL_EXTRA, COL_HEADERS, COL_STATE, COL_ACCOUNT_BLOOM};
use ethereum_types::{H256, U256};
use journaldb;
use kvdb::DBTransaction;
use parking_lot::Mutex;
use self::{
	account_state::account::Account as StateAccount,
	accounts_bloom::Bloom, // todo[dvdplm] rename this crate
	crossbeam_utils::thread,
	ethtrie::TrieDB,
	kvdb_rocksdb::{CompactionProfile, Database, DatabaseConfig},
	state_db::{StateDB, ACCOUNTS_BLOOM_ITEM_COUNT_KEY},
	trie_db::Trie,
};
use types::{
	BlockNumber,
	errors::EthcoreError as Error,
	views::{HeaderView, ViewRlp},
};
use rlp::{RlpStream, Rlp};

pub fn rebuild_accounts_bloom<P: AsRef<Path>>(
	db_path: P,
	compaction: CompactionProfile,
	backup_path: Option<String>,
	account_count: u64,
) -> Result<(), Error> {
	let db_config = DatabaseConfig {
		compaction,
		columns: ethcore_db::NUM_COLUMNS,
		..Default::default()
	};
	let db_path_str = db_path.as_ref().to_string_lossy();
	let db = Arc::new(Database::open(&db_config, &db_path_str)?);

	let (state_root, best_block) = load_state_root(db.clone())?;

	// todo[dvdplm] I can't make the `--backup-path` optional with the `usage!`
	// macro so having `Option<String>` here is pretty useless – it must be
	// specified. For the time being we'll always make a backup.
	if let Some(backup_path) = backup_path {
		let backup_path = dir::helpers::replace_home("", &backup_path);
		let backup_path = Path::new(&backup_path);
		backup_bloom(&backup_path, db.clone(), best_block)?;
	}

	rebuild_bloom(db, account_count, state_root, best_block)?;
	Ok(())
}

pub fn restore_accounts_bloom<P: AsRef<Path>>(
	db_path: P,
	compaction: CompactionProfile,
	backup_path: String,
) -> Result<(), Error> {
	let db_config = DatabaseConfig {
		compaction,
		columns: ethcore_db::NUM_COLUMNS,
		..Default::default()
	};
	let db_path_str = db_path.as_ref().to_string_lossy();
	let db = Arc::new(Database::open(&db_config, &db_path_str)?);

	let backup_path = dir::helpers::replace_home("", &backup_path);
	restore_bloom(&backup_path, db.clone())?;
	Ok(())
}

fn load_state_root(db: Arc<Database>) -> Result<(H256, BlockNumber), Error> {
	let best_block_hash = match db.get(COL_EXTRA, b"best")? {
		None => {
			warn!(target: "migration", "No best block hash, skipping");
			return Err(Error::Msg("No best block hash in the DB.".to_owned()));
		},
		Some(hash) => hash,
	};
	let best_block_header = match db.get(COL_HEADERS, &best_block_hash)? {
		// no best block, nothing to do
		None => {
			warn!(target: "migration", "No best block header, skipping");
			return Err(Error::Msg("No best block header in the DB.".to_owned()));
		},
		Some(x) => x,
	};
	let view = ViewRlp::new(&best_block_header, "", 1);
	let header = HeaderView::new(view);
	let best_block_nr = header.number();
	let state_root = header.state_root();
	Ok((state_root, best_block_nr))
}

fn backup_bloom<P: AsRef<Path>>(
	bloom_backup_path: &P,
	source: Arc<Database>,
	best_block: BlockNumber,
) -> Result<(), Error> {
	let num_keys = source.num_keys(COL_ACCOUNT_BLOOM)? / 2;
	if num_keys == 0 {
		warn!("No bloom in the DB to back up");
		return Ok(())
	}

	let mut bloom_backup = std::fs::File::create(bloom_backup_path)
		.map_err(|_| format!("Cannot write to file at path: {}", bloom_backup_path.as_ref().display()))?;

	info!("Saving old bloom as of block #{} to '{}'", best_block, bloom_backup_path.as_ref().display());
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
	info!("Saved old bloom as of block #{} to '{}' ({} bytes, {} keys)", best_block, bloom_backup_path.as_ref().display(), written, num_keys);
	Ok(())
}

fn restore_bloom<P: AsRef<Path>>(
	bloom_backup_path: &P,
	db: Arc<Database>
) -> Result<(), Error> {
	let mut bloom_backup = std::fs::File::open(bloom_backup_path)?;
	info!("Restoring bloom from '{}'", bloom_backup_path.as_ref().display());
	let mut buf = Vec::with_capacity(10_000_000);
	use std::io::Read;
	// todo[dvdplm]: this is a little terrible – what's the better way?
	let bytes_read = bloom_backup.read_to_end(&mut buf)?;
	let rlp = Rlp::new(&buf);
	let item_count = rlp.item_count()?;
	info!("{} bloom key/values and {} bytes read from disk", item_count, bytes_read);

	let mut batch = DBTransaction::with_capacity(item_count);
	for (n, kv_rlp) in rlp.iter().enumerate() {
		let kv: Vec<Vec<u8>> = kv_rlp.as_list()?;
		assert_eq!(kv.len(), 2);
		batch.put(COL_ACCOUNT_BLOOM, &kv[0], &kv[1]);
		if n > 0 && n % 10_000 == 0 {
			info!("  Bloom entries prepared for restoration: {}", n);
		}
	}
	clear_bloom(db.clone())?;
	db.write(batch)?;
	db.flush()?;
	info!("Bloom restored (wrote {} entries, {} bytes)", item_count, bytes_read);
	Ok(())
}

fn clear_bloom(db: Arc<Database>) -> Result<(), Error> {
	let num_keys = db.num_keys(COL_ACCOUNT_BLOOM)? / 2;
	info!("Clearing out old accounts bloom ({} keys)", num_keys);
	let mut batch = DBTransaction::with_capacity(num_keys as usize);
	for (n, (k,_)) in db.iter(COL_ACCOUNT_BLOOM).enumerate() {
		batch.delete(COL_ACCOUNT_BLOOM, &k);
		if n > 0 && n % 50_000 == 0 {
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
fn rebuild_bloom(
	source: Arc<Database>,
	account_count: u64,
	state_root: H256,
	best_block: BlockNumber,
) -> Result<(), Error> {
	let num_keys = source.num_keys(COL_STATE)? / 2;
	info!(target: "migration", "Accounts bloom rebuild started for chain at #{}. There are {} accounts in the DB (estimate).", best_block, num_keys);
	if account_count <= num_keys {
		warn!("Rebuilding the bloom with space for {} accounts when the DB contains {} keys is not a good idea: the bloom filter will be saturated right away.",
			account_count, num_keys
		);
	}
	clear_bloom(source.clone())?;


	// Progress counters
	let empty_accounts = Arc::new(AtomicU64::new(0));
	let non_empty_accounts = Arc::new(AtomicU64::new(0));
	let total_accounts = Arc::new(AtomicU64::new(0));

	let state_db = journaldb::new(
		source.clone(),
		// It does not matter which `journaldb::Algorithm` is used since
		// there will be no writes to the state column.
		journaldb::Algorithm::OverlayRecent,
		COL_STATE);

	let db = state_db.as_hash_db();
	let start = std::time::Instant::now();

	// 1 thread:    49627s –> ~14h
	// 4 threads:   10825s –> ~3h
	// 6 threads:    9399s –> ~2.6h
	// 12 threads:   9401s -> ~2.6h (slightly bigger chain though)
	// 16 threads:   8805s –> ~2.45h
	let threads = num_cpus::get();
	// Chunk up the state in this many parts; each thread will be assigned one part at a time.
	const STATE_SUBPARTS: usize = 16;
	let bloom_result = thread::scope(|scope| -> Result<Arc<Mutex<Bloom>>, Error> {
		let bloom = Bloom::new_for_fp_rate(account_count, 0.01);
		let bloom = Arc::new(Mutex::new(bloom));
		for thr_idx in 0..threads {
			let tb = scope.builder().name(format!("accounts worker #{}", thr_idx).to_string());
			let my_bloom = bloom.clone();
			let my_total_accounts = total_accounts.clone();
			let my_non_empty_accounts = non_empty_accounts.clone();
			let my_empty_accounts = empty_accounts.clone();
			tb.spawn(move |_| -> Result<(), Error> {
				let mut part_start = std::time::Instant::now();
				// Don't insert empty accounts into the bloom
				let empty_account_rlp = StateAccount::new_basic(U256::zero(), U256::zero()).rlp();
				for part in (thr_idx..STATE_SUBPARTS).step_by(threads) {
					info!("Processing part {} of the accounts in thread {}", part, thr_idx);
					let account_trie = TrieDB::new(&db, &state_root)?;
					let mut account_iter = account_trie.iter()?;

					// Seek to the start of this data segment
					let mut seek_from = vec![0; 32];
					seek_from[0] = (part * STATE_SUBPARTS) as u8;
					account_iter.seek(&seek_from)?;
					// Set the upper-bound for this section of the data (but let the last part finish the whole range).
					let seek_to =
						if part < STATE_SUBPARTS - 1 {
							Some(((part + 1) * STATE_SUBPARTS) as u8)
						} else {
							None
						};
					let mut batch_start = std::time::Instant::now();
					for (n, (account_key, account_data)) in account_iter.filter_map(Result::ok).enumerate() {
						if seek_to.map_or(false, |seek_to| account_key[0] >= seek_to) {
							my_total_accounts.fetch_add(n as u64, Ordering::Relaxed);
							let sat = my_bloom.lock().saturation();
							info!("  {} accounts processed in {:?} – end of part {} by thread {}. Bloom saturation: {}", n, part_start.elapsed(), part, thr_idx, sat);
							part_start = std::time::Instant::now();

							break;
						}
						if n > 0 && n % 50_000 == 0 {
							info!("  Accounts processed: {} in {:?} by thread {}", n, batch_start.elapsed(), thr_idx);
							batch_start = std::time::Instant::now();
						}
						if account_data != empty_account_rlp {
							my_bloom.lock().set(&account_key);
							my_non_empty_accounts.fetch_add(1, Ordering::Relaxed);
						} else {
							my_empty_accounts.fetch_add(1, Ordering::Relaxed);
						}
					}
				}
				Ok(())
			})?;
		}
		Ok(bloom)
	});

	let bloom = match bloom_result {
		Ok(bloom_arc) => bloom_arc?,
		Err(e) => {
			warn!("One of the bloom-building threads panicked: {:?}", e);
			return Err("One of the bloom-building threads panicked".into())
		}
	};
	let mut bloom = bloom.lock();
	info!("Finished iterating over {} accounts as of block #{} in: {:?}. Bloom saturation: {}",
	      total_accounts.load(Ordering::Relaxed), best_block, start.elapsed(), bloom.saturation());
	let bloom_journal = bloom.drain_journal();
	info!(target: "migration", "Generated {} bloom entries; the DB has {} empty accounts and {} non-empty accounts",
	      bloom_journal.entries.len(), empty_accounts.load(Ordering::Relaxed), non_empty_accounts.load(Ordering::Relaxed));
	info!(target: "migration", "New bloom has {} k_bits (aka 'hash functions') and a bitmap size of {} bits", bloom_journal.hash_functions, bloom.number_of_bits());
	let mut batch = DBTransaction::new();
	StateDB::commit_bloom(&mut batch, bloom_journal)?;
	// Write the size of the bloom we just built to the db so we can load&rebuild the bloom at startup
	batch.put(COL_ACCOUNT_BLOOM, ACCOUNTS_BLOOM_ITEM_COUNT_KEY, &account_count.to_le_bytes());
	source.write(batch)?;
	source.flush()?;
	info!(target: "migration", "Finished bloom update for chain at #{}", best_block);

	Ok(())
}
