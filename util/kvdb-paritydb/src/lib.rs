// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

#[macro_use]
extern crate log;

extern crate kvdb;
extern crate paritydb;
extern crate elastic_array;

use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};
use std::{fs, io};
use kvdb::{KeyValueDB, DBTransaction, DBValue, DBOp, Result};
use paritydb::{Options, Database as DB, Value, DatabaseIterator as DBIterator};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
	/// Set number of columns
	pub columns: Option<u32>,
}

pub struct Database {
	dbs: Vec<RwLock<Option<DB>>>,
	config: DatabaseConfig,
	path: String,
}

impl Database {
	fn open_column(mut path: PathBuf, subpath: &str) -> Result<DB> {
		path.push(subpath);

		let db_file = {
			let mut path = path.clone();
			path.push("data.db");
			path
		};

		let meta_file = {
			let mut path = path.clone();
			path.push("meta.db");
			path
		};

		if db_file.exists() && meta_file.exists() {
			Ok(DB::open(path, Options::default()).map_err(|e| format!("ParityDB error: {}", e))?)
		} else {
			Ok(DB::create(path, Options::default()).map_err(|e| format!("ParityDB error: {}", e))?)
		}
	}

	pub fn open(config: &DatabaseConfig, path: &str) -> Result<Database> {
		let mut dbs = Vec::new();

		dbs.push(RwLock::new(Some(Self::open_column(PathBuf::from(path), "default")?)));
		if let Some(columns) = config.columns {
			for i in 0..columns {
				dbs.push(RwLock::new(Some(Self::open_column(PathBuf::from(path), &i.to_string())?)));
			}
		}

		Ok(Database {
			dbs,
			config: config.clone(),
			path: path.to_string(),
		})
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> Result<Option<DBValue>> {
		let db_guard = match col {
			None => self.dbs[0].read(),
			Some(col) => {
				let col = col as usize;
				if col > self.dbs.len() {
					return Err(format!("ParityDB error: column does not exist.").into());
				}
				self.dbs[col].read()
			},
		}.map_err(|e| format!("ParityDB error: {}", e))?;
		let db = db_guard.as_ref().expect("DB should always exist");

		// Postfix the key to 32 bytes.
		let mut postfixed_key = [0u8; 32];
		if key.len() > 32 {
			return Err("ParityDB error: longest key length is 32 bytes.".into());
		}
		postfixed_key[0..key.len()].copy_from_slice(key);

		let raw_value = db.get(postfixed_key).map_err(|e| format!("ParityDB error: {}", e))?;
		let value = raw_value.map(|raw_value| {
			match raw_value {
				Value::Raw(data) => DBValue::from_slice(data),
				Value::Record(record) => {
					let mut array = Vec::new();
					array.resize(record.value_len(), 0u8);
					record.read_value(&mut array);
					DBValue::from_vec(array)
				}
			}
		});

		Ok(value)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		match self.iter_from_prefix(col, prefix).next() {
			Some((k, v)) => if k.starts_with(prefix) { Some(v) } else { None },
			_ => None
		}
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		let DBTransaction { ops } = transaction;
		let mut transactions = Vec::new();
		for _ in 0..self.dbs.len() {
			transactions.push(None);
		}

		for op in ops {
			let col = op.col().unwrap_or(0) as usize;
			if col > self.dbs.len() {
				warn!("ParityDB error: column does not exist.");
				continue;
			}
			if transactions[col].is_none() {
				transactions[col] = Some(self.dbs[col].read().expect("DB read cannot fail; qed").as_ref().expect("DB should always exist").create_transaction());
			}
			let transaction = transactions[col].as_mut().expect("None case checked above; qed");

			let postfixed_key = {
				let key = op.key();
				let mut postfixed_key = [0u8; 32];
				if key.len() > 32 {
					warn!("ParityDB error: longest key length is 32 bytes.");
					continue;
				}
				postfixed_key[0..key.len()].copy_from_slice(key);
				postfixed_key
			};

			match op {
				DBOp::Insert { value, .. } => {
					transaction.insert(postfixed_key, value).expect("Key length is checked above; qed");
				},
				DBOp::Delete { .. } => {
					transaction.delete(postfixed_key).expect("Key length is checked above; qed");
				},
			}
		}

		for (i, transaction) in transactions.into_iter().enumerate() {
			if let Some(transaction) = transaction {
				self.dbs[i].write().expect("DB write cannot fail; qed").as_mut().expect("DB should always exist").commit(&transaction).expect("Transaction commitment error indicate problem with underlying database folder.");
			}
		}
	}

	fn flush(&self) -> Result<()> {
		for db in self.dbs.iter() {
			db.write().map_err(|e| format!("ParityDB error: {}", e))?.as_mut()
				.expect("DB should always exist")
				.flush_journal(None).map_err(|e| format!("ParityDB error: {}", e))?;
		}

		Ok(())
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let col = col.unwrap_or(0) as usize;
		if col > self.dbs.len() {
			panic!("ParityDB error: column does not exist");
		}

		let guard = self.dbs[col].read().expect("DB read cannot fail; qed");
		// We need some unsafe black magic here because we need an "owning reference", but Rust's type system cannot
		// figure this out. The reference is safe as long as we make sure the following properties are true:
		//
		// 1. `iter` is in a "stable address". This is guaranteed by Box.
		// 2. `iter` is dropped before `guard` is dropped. So that all references within 'a are still valid.
		let iter = unsafe {
			Box::from_raw(Box::into_raw(Box::new(guard.as_ref().expect("DB should always exist").iter().expect("Iterator building failure indicate problem with underlying database folder."))) as *mut () as *mut DBIterator<'a>)
		};

		Box::new(DatabaseIterator { _guard: guard, iter })
	}

	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
							-> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		Box::new(self.iter(col).filter(move |v| v.0.starts_with(prefix)))
	}

	fn restore(&self, new_db: &str) -> Result<()> {
		use std::mem;

		for i in 0..self.dbs.len() {
			let mut db = self.dbs[i].write().map_err(|e| format!("DB write error: {}", e))?;
			*db = None;
		}

		let mut backup_db = PathBuf::from(&self.path);
		backup_db.pop();
		backup_db.push("backup_db");

		let existed = match fs::rename(&self.path, &backup_db) {
			Ok(_) => true,
			Err(e) => if let io::ErrorKind::NotFound = e.kind() {
				false
			} else {
				return Err(e.into());
			}
		};

		match fs::rename(&new_db, &self.path) {
			Ok(_) => {
				// clean up the backup.
				if existed {
					fs::remove_dir_all(&backup_db)?;
				}
			}
			Err(e) => {
				// restore the backup.
				if existed {
					fs::rename(&backup_db, &self.path)?;
				}
				return Err(e.into())
			}
		}

		let new_db = Self::open(&self.config, &self.path)?;
		for i in 0..self.dbs.len() {
			let mut db = self.dbs[i].write().map_err(|e| format!("DB acquire error: {}", e))?;
			*db = mem::replace(&mut *new_db.dbs[i].write().map_err(|e| format!("DB acquire error: {}", e))?, None);
		}

		Ok(())
	}
}

struct DatabaseIterator<'a> {
	_guard: RwLockReadGuard<'a, Option<DB>>,
	iter: Box<DBIterator<'a>>,
}

impl<'a> Iterator for DatabaseIterator<'a> {
	type Item = (Box<[u8]>, Box<[u8]>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.iter.next() {
			None => None,
			Some(Err(_)) => None,
			Some(Ok((key, value))) => {
				let key: Vec<u8> = key.into();
				let value = match value {
					Value::Raw(data) => data.into(),
					Value::Record(record) => {
						let mut array = Vec::new();
						array.resize(record.value_len(), 0u8);
						record.read_value(&mut array);
						array
					},
				};

				Some((key.into_boxed_slice(), value.into_boxed_slice()))
			}
		}
	}
}
