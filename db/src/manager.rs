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

struct DatabaseManager<K: Eq + Hash> {
	databases: HashMap<K, Arc<DatabaseService>>,
	thread_pool: Pool,
	max_open: usize,
}

impl<K: Eq + Hash> DatabaseManager<K> {
	fn new(max_open: usize) {
		DatabasePool {
			databases: RwLock::new(HashMap::with_capacity(max_open)),
			max_open: max_open,
			thread_pool: Pool::new(max_open),
		}
	}

	fn open(&mut self, key: K, path: &str) -> &Arc<Mutex<DatabaseService>> {
		let new_db = DatabaseService::new();
		new_db.open(path);
		self.databases.insert(key, new_db);
	}

	fn flush(&self) {
		thread_pool.scoped(|scope| {
			for database in self.databases {
				let db_shared = database.clone();
				scope.execute(move || {
					database.flush_all();
				})
			}
		})
	}
}

#[test]
fn can_hold_arbitrary_tagged_dbs {
	enum Databases(Blocks, Extras, JournalDB);

	let db_pool = DatabasePool::new(3);
	let db_blocks = db_pool::open<Blocks>("/tmp/b1");
}
