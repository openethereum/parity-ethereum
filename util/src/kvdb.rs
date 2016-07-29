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

//! Key-Value store abstraction with `RocksDB` backend.

use std::default::Default;
use rocksdb::{DB, Writable, WriteBatch, WriteOptions, IteratorMode, DBVector, DBIterator,
	Options, DBCompactionStyle, BlockBasedOptions, Direction, Cache, Column};

const DB_BACKGROUND_FLUSHES: i32 = 2;
const DB_BACKGROUND_COMPACTIONS: i32 = 2;

/// Write transaction. Batches a sequence of put/delete operations for efficiency.
pub struct DBTransaction {
	batch: WriteBatch,
	cfs: Vec<Column>,
}

impl DBTransaction {
	/// Create new transaction.
	pub fn new(db: &Database) -> DBTransaction {
		DBTransaction {
			batch: WriteBatch::new(),
			cfs: db.cfs.clone(),
		}
	}

	/// Insert a key-value pair in the transaction. Any existing value value will be overwritten upon write.
	pub fn put(&self, col: Option<u32>, key: &[u8], value: &[u8]) -> Result<(), String> {
		col.map_or_else(|| self.batch.put(key, value), |c| self.batch.put_cf(self.cfs[c as usize], key, value))
	}

	/// Delete value by key.
	pub fn delete(&self, col: Option<u32>, key: &[u8]) -> Result<(), String> {
		col.map_or_else(|| self.batch.delete(key), |c| self.batch.delete_cf(self.cfs[c as usize], key))
	}
}

/// Compaction profile for the database settings
#[derive(Clone, Copy)]
pub struct CompactionProfile {
	/// L0-L1 target file size
	pub initial_file_size: u64,
	/// L2-LN target file size multiplier
	pub file_size_multiplier: i32,
	/// rate limiter for background flushes and compactions, bytes/sec, if any
	pub write_rate_limit: Option<u64>,
}

impl Default for CompactionProfile {
	/// Default profile suitable for most storage
	fn default() -> CompactionProfile {
		CompactionProfile {
			initial_file_size: 32 * 1024 * 1024,
			file_size_multiplier: 2,
			write_rate_limit: None,
		}
	}
}

impl CompactionProfile {
	/// Slow hdd compaction profile
	pub fn hdd() -> CompactionProfile {
		CompactionProfile {
			initial_file_size: 192 * 1024 * 1024,
			file_size_multiplier: 1,
			write_rate_limit: Some(8 * 1024 * 1024),
		}
	}
}

/// Database configuration
#[derive(Clone, Copy)]
pub struct DatabaseConfig {
	/// Max number of open files.
	pub max_open_files: i32,
	/// Cache-size
	pub cache_size: Option<usize>,
	/// Compaction profile
	pub compaction: CompactionProfile,
	/// Set number of columns
	pub columns: Option<u32>,
}

impl DatabaseConfig {
	/// Create new `DatabaseConfig` with default parameters and specified set of columns.
	pub fn with_columns(columns: Option<u32>) -> Self {
		let mut config = Self::default();
		config.columns = columns;
		config
	}
}

impl Default for DatabaseConfig {
	fn default() -> DatabaseConfig {
		DatabaseConfig {
			cache_size: None,
			max_open_files: 1024,
			compaction: CompactionProfile::default(),
			columns: None,
		}
	}
}

/// Database iterator
pub struct DatabaseIterator {
	iter: DBIterator,
}

impl<'a> Iterator for DatabaseIterator {
	type Item = (Box<[u8]>, Box<[u8]>);

    fn next(&mut self) -> Option<Self::Item> {
		self.iter.next()
	}
}

/// Key-Value database.
pub struct Database {
	db: DB,
	write_opts: WriteOptions,
	cfs: Vec<Column>,
}

impl Database {
	/// Open database with default settings.
	pub fn open_default(path: &str) -> Result<Database, String> {
		Database::open(&DatabaseConfig::default(), path)
	}

	/// Open database file. Creates if it does not exist.
	pub fn open(config: &DatabaseConfig, path: &str) -> Result<Database, String> {
		let mut opts = Options::new();
		if let Some(rate_limit) = config.compaction.write_rate_limit {
			try!(opts.set_parsed_options(&format!("rate_limiter_bytes_per_sec={}", rate_limit)));
		}
		opts.set_max_open_files(config.max_open_files);
		opts.create_if_missing(true);
		opts.set_use_fsync(false);

		// compaction settings
		opts.set_compaction_style(DBCompactionStyle::DBUniversalCompaction);
		opts.set_target_file_size_base(config.compaction.initial_file_size);
		opts.set_target_file_size_multiplier(config.compaction.file_size_multiplier);

		opts.set_max_background_flushes(DB_BACKGROUND_FLUSHES);
		opts.set_max_background_compactions(DB_BACKGROUND_COMPACTIONS);

		if let Some(cache_size) = config.cache_size {
			let mut block_opts = BlockBasedOptions::new();
			// all goes to read cache
			block_opts.set_cache(Cache::new(cache_size * 1024 * 1024));
			opts.set_block_based_table_factory(&block_opts);
		}

		let mut write_opts = WriteOptions::new();
		write_opts.disable_wal(true); // TODO: make sure this is safe

		let mut cfs: Vec<Column> = Vec::new();
		let db = match config.columns {
			Some(columns) => {
				let cfnames: Vec<_> = (0..columns).map(|c| format!("col{}", c)).collect();
				let cfnames: Vec<&str> = cfnames.iter().map(|n| n as &str).collect();
				match DB::open_cf(&opts, path, &cfnames) {
					Ok(db) => {
						cfs = cfnames.iter().map(|n| db.cf_handle(n).unwrap()).collect();
						assert!(cfs.len() == columns as usize);
						Ok(db)
					}
					Err(_) => {
						// retry and create CFs
						match DB::open_cf(&opts, path, &[]) {
							Ok(mut db) => {
								cfs = cfnames.iter().map(|n| db.create_cf(n, &opts).unwrap()).collect();
								Ok(db)
							},
							err @ Err(_) => err,
						}
					}
				}
			},
			None => DB::open(&opts, path)
		};
		let db = match db {
			Ok(db) => db,
			Err(ref s) if s.starts_with("Corruption:") => {
				info!("{}", s);
				info!("Attempting DB repair for {}", path);
				try!(DB::repair(&opts, path));
				try!(DB::open(&opts, path))
			},
			Err(s) => { return Err(s); }
		};
		Ok(Database { db: db, write_opts: write_opts, cfs: cfs })
	}

	/// Creates new transaction for this database.
	pub fn transaction(&self) -> DBTransaction {
		DBTransaction::new(self)
	}

	/// Commit transaction to database.
	pub fn write(&self, tr: DBTransaction) -> Result<(), String> {
		self.db.write_opt(tr.batch, &self.write_opts)
	}

	/// Get value by key.
	pub fn get(&self, col: Option<u32>, key: &[u8]) -> Result<Option<DBVector>, String> {
		col.map_or_else(|| self.db.get(key), |c| self.db.get_cf(self.cfs[c as usize], key))
	}

	/// Get value by partial key. Prefix size should match configured prefix size.
	pub fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		let mut iter = col.map_or_else(|| self.db.iterator(IteratorMode::From(prefix, Direction::Forward)),
			|c| self.db.iterator_cf(self.cfs[c as usize], IteratorMode::From(prefix, Direction::Forward)).unwrap());
		match iter.next() {
			// TODO: use prefix_same_as_start read option (not availabele in C API currently)
			Some((k, v)) => if k[0 .. prefix.len()] == prefix[..] { Some(v) } else { None },
			_ => None
		}
	}

	/// Check if there is anything in the database.
	pub fn is_empty(&self, col: Option<u32>) -> bool {
		self.iter(col).next().is_none()
	}

	/// Get database iterator.
	pub fn iter(&self, col: Option<u32>) -> DatabaseIterator {
		col.map_or_else(|| DatabaseIterator { iter: self.db.iterator(IteratorMode::Start) },
			|c| DatabaseIterator { iter: self.db.iterator_cf(self.cfs[c as usize], IteratorMode::Start).unwrap() })
	}
}

#[cfg(test)]
mod tests {
	use hash::*;
	use super::*;
	use devtools::*;
	use std::str::FromStr;
	use std::ops::Deref;

	fn test_db(config: &DatabaseConfig) {
		let path = RandomTempPath::create_dir();
		let db = Database::open(config, path.as_path().to_str().unwrap()).unwrap();
		let key1 = H256::from_str("02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key2 = H256::from_str("03c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key3 = H256::from_str("01c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();

		let batch = db.transaction();
		batch.put(None, &key1, b"cat").unwrap();
		batch.put(None, &key2, b"dog").unwrap();
		db.write(batch).unwrap();

		assert_eq!(db.get(None, &key1).unwrap().unwrap().deref(), b"cat");

		let contents: Vec<_> = db.iter(None).collect();
		assert_eq!(contents.len(), 2);
		assert_eq!(&*contents[0].0, key1.deref());
		assert_eq!(&*contents[0].1, b"cat");
		assert_eq!(&*contents[1].0, key2.deref());
		assert_eq!(&*contents[1].1, b"dog");

		let batch = db.transaction();
		batch.delete(None, &key1).unwrap();
		db.write(batch).unwrap();

		assert!(db.get(None, &key1).unwrap().is_none());

		let batch = db.transaction();
		batch.put(None, &key1, b"cat").unwrap();
		db.write(batch).unwrap();

		let transaction = db.transaction();
		transaction.put(None, &key3, b"elephant").unwrap();
		transaction.delete(None, &key1).unwrap();
		db.write(transaction).unwrap();
		assert!(db.get(None, &key1).unwrap().is_none());
		assert_eq!(db.get(None, &key3).unwrap().unwrap().deref(), b"elephant");

		assert_eq!(db.get_by_prefix(None, &key3).unwrap().deref(), b"elephant");
		assert_eq!(db.get_by_prefix(None, &key2).unwrap().deref(), b"dog");
	}

	#[test]
	fn kvdb() {
		let path = RandomTempPath::create_dir();
		let smoke = Database::open_default(path.as_path().to_str().unwrap()).unwrap();
		assert!(smoke.is_empty(None));
		test_db(&DatabaseConfig::default());
	}
}
