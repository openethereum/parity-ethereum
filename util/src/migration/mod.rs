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

//! DB Migration module.
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ::kvdb::{CompactionProfile, Database, DatabaseConfig, DBTransaction};

/// Migration config.
pub struct Config {
	/// Defines how many elements should be migrated at once.
	pub batch_size: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			batch_size: 1024,
		}
	}
}

/// Migration error.
#[derive(Debug)]
pub enum Error {
	/// Error returned when it is impossible to add new migration rules.
	CannotAddMigration,
	/// Error returned when migration from specific version can not be performed.
	MigrationImpossible,
	/// Io Error.
	Io(::std::io::Error),
	/// Custom error.
	Custom(String),
}

impl From<::std::io::Error> for Error {
	fn from(e: ::std::io::Error) -> Self {
		Error::Io(e)
	}
}

/// Migration source.
pub trait Source {
	/// Get an iterator over the key, value pairs.
	fn iter(&self) -> Box<Iterator<Item=(Vec<u8>, Vec<u8>)>>;

	/// Query a specific key.
	fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
}

impl Source for Database {
	fn iter(&self) -> Box<Iterator<Item=(Vec<u8>, Vec<u8>)>> {
		Box::new(self.iter().map(|(k, v)| (k.to_vec(), v.to_vec())))
	}

	fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.get(key).ok().and_then(|x| x).map(|x| x.to_owned())
	}
}



/// Object being migrated
pub trait Migrateable {
	/// Called on destination to commit batch of migrated entries.
	fn commit(&mut self, batch: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), Error>;
}

impl Migrateable for Database {
	fn commit(&mut self, batch: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), Error> {
		let transaction = DBTransaction::new();

		for keypair in batch {
			try!(transaction.put(&keypair.0, &keypair.1).map_err(Error::Custom))
		}

		self.write(transaction).map_err(Error::Custom)
	}
}

/// A generalized migration from the given db to a destination db.
pub trait Migration: 'static {
	/// Version of the database after the migration.
	fn version(&self) -> u32;
	/// Migrate a source to a destination.
	fn migrate(&self, source: &Source, config: &Config, destination: &mut Destination) -> Result<(), Error>;
}

/// A simple migration over key-value pairs.
pub trait SimpleMigration: 'static {
	/// Version of database after the migration.
	fn version(&self) -> u32;
	/// Should migrate existing object to new database.
	/// Returns `None` if the object does not exist in new version of database.
	fn simple_migrate(&self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)>;
}

impl<T: SimpleMigration> Migration for T {
	fn version(&self) -> u32 { SimpleMigration::version(self) }

	fn migrate(&self, source: &Source, config: &Config, destination: &mut Destination) -> Result<(), Error> {
		let mut batch: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

		for (key, value) in source.iter() {

			if let Some((key, value)) = self.simple_migrate(key.to_vec(), value.to_vec()) {
				batch.insert(key, value);
			}

			if batch.len() == config.batch_size {
				try!(destination.commit(&batch));
				batch.clear();
			}
		}

		try!(destination.commit(&batch));

		Ok(())
	}
}

/// Get the path where all databases reside.
fn database_path(path: &Path) -> PathBuf {
	let mut temp_path = path.to_owned();
	temp_path.pop();
	temp_path
}

// which temp database we are currently using
enum TempDB {
	One,
	Two,
}

impl TempDB {
	// given the path that all databases reside in,
	// return the path of this one.
	fn path(&self, db_path: &Path) -> PathBuf {
		let mut path = db_path.to_owned();
		match *self {
			TempDB::One => path.push("migration_1"),
			TempDB::Two => path.push("migration_2"),
		}

		path
	}

	fn swap(&mut self) {
		*self = match *self {
			TempDB::One => TempDB::Two,
			TempDB::Two => TempDB::One,
		};
	}
}

/// Manages database migration.
pub struct Manager {
	config: Config,
	migrations: Vec<Box<Migration>>,
}

impl Manager {
	/// Creates new migration manager with given configuration.
	pub fn new(config: Config) -> Self {
		Manager {
			config: config,
			migrations: vec![],
		}
	}

	/// Adds new migration rules.
	pub fn add_migration<T>(&mut self, migration: T) -> Result<(), Error> where T: Migration {
		let version_match = match self.migrations.last() {
			Some(last) => last.version() + 1 == migration.version(),
			None => true,
		};

		match version_match {
			true => Ok(self.migrations.push(Box::new(migration))),
			false => Err(Error::CannotAddMigration),
		}
	}

	/// Performs migrations in order. This alternates migrating between two
	/// temporary databases, and returns the path to the final one used.
	/// The other is deleted by this function.
	pub fn execute(&self, db_path: PathBuf, version: u32) -> Result<PathBuf, Error> {
		let migrations = try!(self.migrations_from(version).ok_or(Error::MigrationImpossible));
		let db_config = DatabaseConfig {
			prefix_size: None,
			max_open_files: 64,
			cache_size: None,
			compaction: CompactionProfile::default(),
		};

		let mut temp_db = TempDB::One;
		let mut temp_path = temp_db.path(&db_path);

		// start with the old db.
		let old_path_str = try!(old_path.to_str().ok_or(Error::MigrationImpossible));
		let mut cur_db = try!(Database::open(&db_config, old_path_str).map_err(|s| Error::Custom(s)));
		for migration in migrations {
			// open the target temporary database.
			temp_path = temp_db.path(&db_path);
			let temp_path_str = try!(temp_path.to_str().ok_or(Error::MigrationImpossible));
			let mut new_db = try!(Database::open(&db_config, temp_path_str).map_err(|s| Error::Custom(s)));

			// perform the migration from cur_db to new_db.
			try!(migration.migrate(&cur_db, &self.config, &mut new_db));
			// next iteration, we will migrate from this db into the other temp.
			cur_db = new_db;
			temp_db.swap();

			// remove the other temporary migration database.
			let _ = fs::remove_dir_all(temp_db.path(&db_path));
		}
		Ok(temp_path)
	}

	/// Returns true if migration is needed.
	pub fn is_needed(&self, version: u32) -> bool {
		match self.migrations.last() {
			Some(last) => version < last.version(),
			None => false,
		}
	}

	fn migrations_from(&self, version: u32) -> Option<&[Box<Migration>]> {
		// index of the first required migration
		let position = self.migrations.iter().position(|m| m.version() == version + 1);
		position.map(|p| &self.migrations[p..])
	}
}

