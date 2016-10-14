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

//! Bloom upgrade

use db::{COL_EXTRA, COL_HEADERS, COL_STATE, NUM_COLUMNS, COL_ACCOUNT_BLOOM};
use state_db::{ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET, StateDB, ACCOUNT_BLOOM_HASHCOUNT_KEY, ACCOUNT_BLOOM_SPACE_KEY};
use util::trie::TrieDB;
use views::HeaderView;
use bloom_journal::Bloom;
use util::migration::Error;
use util::{journaldb, Trie};
use util::{H256, FixedHash};
use util::{Database, DatabaseConfig, DBTransaction, CompactionProfile};
use std::path::Path;
use std::collections::HashMap;
use byteorder::{LittleEndian, ByteOrder};

fn check_bloom_exists(db: &Database) -> bool {
	let hash_count_entry = db.get(COL_ACCOUNT_BLOOM, ACCOUNT_BLOOM_HASHCOUNT_KEY)
		.expect("Low-level database error");

	hash_count_entry.is_some()
}

fn check_space_match(db: &Database) -> Result<(), usize> {
	let db_space = db.get(COL_ACCOUNT_BLOOM, ACCOUNT_BLOOM_SPACE_KEY)
		.expect("Low-level database error")
		.map(|val| LittleEndian::read_u64(&val[..]) as usize)
		// this was the initial size of the bloom which was not written in the database
		.unwrap_or(1048576);

	if db_space == ACCOUNT_BLOOM_SPACE { Ok(()) } else { Err(db_space) }
}

/// Account bloom upgrade routine. If bloom already present, does nothing.
/// If database empty (no best block), does nothing.
/// Can be called on upgraded database with no issues (will do nothing).
pub fn upgrade_account_bloom(db_path: &Path) -> Result<(), Error> {
	let mut progress = ::util::migration::Progress::default();

	let path = try!(db_path.to_str().ok_or(Error::MigrationImpossible));
	trace!(target: "migration", "Account bloom upgrade at {:?}", db_path);

	let source = try!(Database::open(&DatabaseConfig {
		max_open_files: 64,
		cache_sizes: HashMap::new(),
		compaction: CompactionProfile::default(),
		columns: NUM_COLUMNS,
		wal: true,
	}, path));

	let best_block_hash = match try!(source.get(COL_EXTRA, b"best")) {
		// no migration needed
		None => {
			trace!(target: "migration", "No best block hash, skipping");
			return Ok(());
		},
		Some(hash) => hash,
	};
	let best_block_header = match try!(source.get(COL_HEADERS, &best_block_hash)) {
		// no best block, nothing to do
		None => {
			trace!(target: "migration", "No best block header, skipping");
			return Ok(())
		},
		Some(x) => x,
	};
	let state_root = HeaderView::new(&best_block_header).state_root();

	let db = ::std::sync::Arc::new(source);
	let mut batch = DBTransaction::new(&db);

	if check_bloom_exists(&*db) {
		match check_space_match(&*db) {
			Ok(_) => {
				// bloom already exists and desired and stored spaces match, nothing to do
				trace!(target: "migration", "Bloom already present of the right space, skipping");
				return Ok(())
			},
			Err(wrong_size) => {
				// nullify existing bloom entries
				trace!(target: "migration", "Clearing old bloom of space {}", &wrong_size);
				let mut key = [0u8; 8];
				for idx in 0..(wrong_size as u64/8) {
					LittleEndian::write_u64(&mut key, idx);
					batch.put(COL_ACCOUNT_BLOOM, &key, &[0u8; 8]);

					if idx % 10000 == 1 { progress.tick(); };
				}

				LittleEndian::write_u64(&mut key, ACCOUNT_BLOOM_SPACE as u64);
				batch.put(COL_ACCOUNT_BLOOM, ACCOUNT_BLOOM_SPACE_KEY, &key);
			},
		}
	}

	println!("Adding/expanding accounts bloom (one-time upgrade)");
	let bloom_journal = {
		let mut bloom = Bloom::new(ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET);
		// no difference what algorithm is passed, since there will be no writes
		let state_db = journaldb::new(
			db.clone(),
			journaldb::Algorithm::OverlayRecent,
			COL_STATE);
		let account_trie = try!(TrieDB::new(state_db.as_hashdb(), &state_root).map_err(|e| Error::Custom(format!("Cannot open trie: {:?}", e))));
		for item in try!(account_trie.iter().map_err(|_| Error::MigrationImpossible)) {
			let (ref account_key, _) = try!(item.map_err(|_| Error::MigrationImpossible));
			let account_key_hash = H256::from_slice(&account_key);
			bloom.set(&*account_key_hash);
		}

		bloom.drain_journal()
	};

	trace!(target: "migration", "Generated {} bloom updates", bloom_journal.entries.len());

	try!(StateDB::commit_bloom(&mut batch, bloom_journal).map_err(|_| Error::Custom("Failed to commit bloom".to_owned())));
	try!(db.write(batch));

	trace!(target: "migration", "Finished bloom update");

	Ok(())
}
