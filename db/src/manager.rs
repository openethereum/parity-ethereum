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
use std::thread::park_timeout;
use std::time::Duration;
use types::Error;

pub struct DatabaseManager<K: Eq + Hash> {
	databases: RwLock<HashMap<K, Arc<Database>>>,
	thread_pool: RwLock<Pool>,
}

impl<K: Eq + Hash> DatabaseManager<K> {
	pub fn new(max_open: usize) -> DatabaseManager<K> {
		DatabaseManager {
			databases: RwLock::new(HashMap::with_capacity(max_open)),
			thread_pool: RwLock::new(Pool::new(max_open as u32)),
		}
	}

	pub fn open(&self, key: K, path: &str) -> Result<Arc<Database>, Error> {
		let new_db = Arc::new(Database::new());
		try!(new_db.open_default(path.to_owned()));
		self.databases.write().unwrap().insert(key, new_db.clone());
		Ok(new_db)
	}

	pub fn flush(&self) {
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

mod tests {
	use traits::DatabaseService;
	use super::DatabaseManager;
	use devtools::*;

	#[test]
	fn can_hold_arbitrary_tagged_dbs() {
		#[derive(Hash, PartialEq, Eq)]
		enum Databases { Other, JournalDB };
		{
			let path = RandomTempPath::new();
			let man = DatabaseManager::new(3);
			let other_db = man.open(Databases::Other, path.as_str()).unwrap();
			assert!(other_db.put("111".as_bytes(), "x".as_bytes()).is_ok());
			man.flush();
		}

		{
			let path = RandomTempPath::new();
			let man = DatabaseManager::new(3);
			let jdb = man.open(Databases::JournalDB, path.as_str()).unwrap();
			assert!(jdb.get("111".as_bytes()).unwrap().is_none())
		}
	}
}
