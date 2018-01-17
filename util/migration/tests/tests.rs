// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Tests for migrations.
//! A random temp directory is created. A database is created within it, and migrations
//! are performed in temp sub-directories.

#[macro_use]
extern crate macros;
extern crate tempdir;
extern crate kvdb_rocksdb;
extern crate migration;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempdir::TempDir;
use kvdb_rocksdb::Database;
use migration::{Batch, Config, Error, SimpleMigration, Migration, Manager, ChangeColumns};

#[inline]
fn db_path(path: &Path) -> PathBuf {
	path.join("db")
}

// initialize a database at the given directory with the given values.
fn make_db(path: &Path, pairs: BTreeMap<Vec<u8>, Vec<u8>>) {
	let db = Database::open_default(path.to_str().unwrap()).expect("failed to open temp database");
	{
		let mut transaction = db.transaction();
		for (k, v) in pairs {
			transaction.put(None, &k, &v);
		}

		db.write(transaction).expect("failed to write db transaction");
	}
}

// helper for verifying a migrated database.
fn verify_migration(path: &Path, pairs: BTreeMap<Vec<u8>, Vec<u8>>) {
	let db = Database::open_default(path.to_str().unwrap()).unwrap();

	for (k, v) in pairs {
		let x = db.get(None, &k).unwrap().unwrap();

		assert_eq!(&x[..], &v[..]);
	}
}

struct Migration0;

impl SimpleMigration for Migration0 {
	fn columns(&self) -> Option<u32> { None }

	fn version(&self) -> u32 { 1 }

	fn simple_migrate(&mut self, mut key: Vec<u8>, mut value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		key.push(0x11);
		value.push(0x22);

		Some((key, value))
	}
}

struct Migration1;

impl SimpleMigration for Migration1 {
	fn columns(&self) -> Option<u32> { None }

	fn version(&self) -> u32 { 2 }

	fn simple_migrate(&mut self, key: Vec<u8>, _value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		Some((key, vec![]))
	}
}

struct AddsColumn;

impl Migration for AddsColumn {
	fn pre_columns(&self) -> Option<u32> { None }

	fn columns(&self) -> Option<u32> { Some(1) }

	fn version(&self) -> u32 { 1 }

	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, col);

		for (key, value) in source.iter(col).into_iter().flat_map(|inner| inner) {
			batch.insert(key.into_vec(), value.into_vec(), dest)?;
		}


		if col == Some(1) {
			batch.insert(vec![1, 2, 3], vec![4, 5, 6], dest)?;
		}

		batch.commit(dest)
	}
}

#[test]
fn one_simple_migration() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);
	let expected = map![vec![0x11] => vec![0x22], vec![1, 0x11] => vec![1, 0x22]];

	manager.add_migration(Migration0).unwrap();
	let end_path = manager.execute(&db_path, 0).unwrap();

	verify_migration(&end_path, expected);
}

#[test]
#[should_panic]
fn no_migration_needed() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);

	manager.add_migration(Migration0).unwrap();
	manager.execute(&db_path, 1).unwrap();
}

#[test]
#[should_panic]
fn wrong_adding_order() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);

	manager.add_migration(Migration1).unwrap();
	manager.add_migration(Migration0).unwrap();
}

#[test]
fn multiple_migrations() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);
	let expected = map![vec![0x11] => vec![], vec![1, 0x11] => vec![]];

	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();
	let end_path = manager.execute(&db_path, 0).unwrap();

	verify_migration(&end_path, expected);
}

#[test]
fn second_migration() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);
	let expected = map![vec![] => vec![], vec![1] => vec![]];

	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();
	let end_path = manager.execute(&db_path, 1).unwrap();

	verify_migration(&end_path, expected);
}

#[test]
fn first_and_noop_migration() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);
	let expected = map![vec![0x11] => vec![0x22], vec![1, 0x11] => vec![1, 0x22]];

	manager.add_migration(Migration0).unwrap();
	let end_path = manager.execute(&db_path, 0).unwrap();

	verify_migration(&end_path, expected);
}

#[test]
fn noop_and_second_migration() {
	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());
	let mut manager = Manager::new(Config::default());
	make_db(&db_path, map![vec![] => vec![], vec![1] => vec![1]]);
	let expected = map![vec![] => vec![], vec![1] => vec![]];

	manager.add_migration(Migration1).unwrap();
	let end_path = manager.execute(&db_path, 0).unwrap();

	verify_migration(&end_path, expected);
}

#[test]
fn is_migration_needed() {
	let mut manager = Manager::new(Config::default());
	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();

	assert!(manager.is_needed(0));
	assert!(manager.is_needed(1));
	assert!(!manager.is_needed(2));
}

#[test]
fn pre_columns() {
	let mut manager = Manager::new(Config::default());
	manager.add_migration(AddsColumn).unwrap();

	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());

	// this shouldn't fail to open the database even though it's one column
	// short of the one before it.
	manager.execute(&db_path, 0).unwrap();
}

#[test]
fn change_columns() {
	use kvdb_rocksdb::DatabaseConfig;

	let mut manager = Manager::new(Config::default());
	manager.add_migration(ChangeColumns {
		pre_columns: None,
		post_columns: Some(4),
		version: 1,
	}).unwrap();

	let tempdir = TempDir::new("").unwrap();
	let db_path = db_path(tempdir.path());

	let new_path = manager.execute(&db_path, 0).unwrap();

	assert_eq!(db_path, new_path, "Changing columns is an in-place migration.");

	let config = DatabaseConfig::with_columns(Some(4));
	let db = Database::open(&config, new_path.to_str().unwrap()).unwrap();
	assert_eq!(db.num_columns(), 4);
}
