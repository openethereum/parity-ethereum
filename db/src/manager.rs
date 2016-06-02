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

//! Ethcore database pool

use std::collections::HashMap;
use scoped_threadpool::Pool;
use std::sync::{Arc, RwLock};
use std::hash::Hash;
use database::Database;
use traits::DatabaseService;
use std::thread::{park_timeout, spawn};
use std::time::Duration;
use types::{Error, DatabaseConfig};

pub struct DatabaseManager<K: Eq + Hash> {
	databases: RwLock<HashMap<K, Arc<Database>>>,
	thread_pool: RwLock<Pool>,
}

// thread_pool is not exposed and cannot be used across threads, safe otherwise
unsafe impl<K: Eq + Hash> Send for DatabaseManager<K> {}
unsafe impl<K: Eq + Hash> Sync for DatabaseManager<K> {}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum QueuedDatabase { JournalDB }

impl<K: Eq + Hash> DatabaseManager<K> {
	pub fn new(max_open: usize) -> DatabaseManager<K> {
		DatabaseManager {
			databases: RwLock::new(HashMap::with_capacity(max_open)),
			thread_pool: RwLock::new(Pool::new(max_open as u32)),
		}
	}

	pub fn open(&self, key: K, path: &str, opts: DatabaseConfig) -> Result<Arc<Database>, Error> {
		let new_db = Arc::new(Database::new());
		try!(new_db.open(opts, path.to_owned()));
		self.databases.write().unwrap().insert(key, new_db.clone());
		Ok(new_db)
	}

	// flush is the only method that uses internal thread pool
	// and is private so that self.thread_pool is not shared across threads
	// and used only in `run_manager`
	fn flush(&self) {
		let mut thread_pool = self.thread_pool.write().unwrap();
		let dbs = self.databases.read().unwrap().values().cloned().collect::<Vec<Arc<Database>>>();
		thread_pool.scoped(|scope| {
			for database in dbs.iter() {
				park_timeout(Duration::from_millis(10));
				scope.execute(move || {
					database.flush_all().unwrap();
				})
			}
		})
	}
}

impl<K: Hash + Eq> Drop for DatabaseManager<K> {
	fn drop(&mut self) {
		self.flush();
	}
}

pub fn run_manager() -> Arc<DatabaseManager<QueuedDatabase>> {
	let manager = Arc::new(DatabaseManager::new(1));
	let shared_manager = manager.clone();

	spawn(move || {
		loop {
			park_timeout(Duration::from_millis(10));
			shared_manager.flush();
		}
	});

	manager
}

mod tests {
	#![allow(unused_imports)]
	use traits::DatabaseService;
	use super::{DatabaseManager, QueuedDatabase};
	use devtools::*;
	use super::run_manager;
	use types::DatabaseConfig;

	#[test]
	fn can_hold_arbitrary_tagged_dbs() {
		#[derive(Hash, PartialEq, Eq)]
		enum Databases { Other, JournalDB };
		{
			let path = RandomTempPath::new();
			let man = DatabaseManager::new(1);
			let other_db = man.open(Databases::Other, path.as_str(), DatabaseConfig::default()).unwrap();
			assert!(other_db.put("111".as_bytes(), "x".as_bytes()).is_ok());
			man.flush();
		}

		{
			let path = RandomTempPath::new();
			let man = DatabaseManager::new(1);
			let jdb = man.open(Databases::JournalDB, path.as_str(), DatabaseConfig::default()).unwrap();
			assert!(jdb.get("111".as_bytes()).unwrap().is_none())
		}
	}

	#[test]
	fn can_run_manager() {
		let man = run_manager();
		{
			let path = RandomTempPath::new();
			let jdb = man.open(QueuedDatabase::JournalDB, path.as_str(), DatabaseConfig::default()).unwrap();
			assert!(jdb.put("111".as_bytes(), "x".as_bytes()).is_ok());
		}
	}
}
