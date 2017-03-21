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

//! DB Migration module.
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ::kvdb::{CompactionProfile, Database, DatabaseConfig, DBTransaction};

/// Migration config.
#[derive(Clone)]
pub struct Config {
	/// Defines how many elements should be migrated at once.
	pub batch_size: usize,
	/// Database compaction profile.
	pub compaction_profile: CompactionProfile,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			batch_size: 1024,
			compaction_profile: Default::default(),
		}
	}
}

/// A batch of key-value pairs to be written into the database.
pub struct Batch {
	inner: BTreeMap<Vec<u8>, Vec<u8>>,
	batch_size: usize,
	column: Option<u32>,
}

impl Batch {
	/// Make a new batch with the given config.
	pub fn new(config: &Config, col: Option<u32>) -> Self {
		Batch {
			inner: BTreeMap::new(),
			batch_size: config.batch_size,
			column: col,
		}
	}

	/// Insert a value into the batch, committing if necessary.
	pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>, dest: &mut Database) -> Result<(), Error> {
		self.inner.insert(key, value);
		if self.inner.len() == self.batch_size {
			self.commit(dest)?;
		}
		Ok(())
	}

	/// Commit all the items in the batch to the given database.
	pub fn commit(&mut self, dest: &mut Database) -> Result<(), Error> {
		if self.inner.is_empty() { return Ok(()) }

		let mut transaction = DBTransaction::new();

		for keypair in &self.inner {
			transaction.put(self.column, &keypair.0, &keypair.1);
		}

		self.inner.clear();
		dest.write(transaction).map_err(Error::Custom)
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

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::CannotAddMigration => write!(f, "Cannot add migration"),
			Error::MigrationImpossible => write!(f, "Migration impossible"),
			Error::Io(ref err) => write!(f, "{}", err),
			Error::Custom(ref err) => write!(f, "{}", err),
		}
	}
}

impl From<::std::io::Error> for Error {
	fn from(e: ::std::io::Error) -> Self {
		Error::Io(e)
	}
}

impl From<String> for Error {
	fn from(e: String) -> Self {
		Error::Custom(e)
	}
}

/// A generalized migration from the given db to a destination db.
pub trait Migration: 'static {
	/// Number of columns in the database before the migration.
	fn pre_columns(&self) -> Option<u32> { self.columns() }
	/// Number of columns in database after the migration.
	fn columns(&self) -> Option<u32>;
	/// Whether this migration alters any existing columns.
	/// if not, then column families will simply be added and `migrate` will never be called.
	fn alters_existing(&self) -> bool { true }
	/// Version of the database after the migration.
	fn version(&self) -> u32;
	/// Migrate a source to a destination.
	fn migrate(&mut self, source: Arc<Database>, config: &Config, destination: &mut Database, col: Option<u32>) -> Result<(), Error>;
}

/// A simple migration over key-value pairs.
pub trait SimpleMigration: 'static {
	/// Number of columns in database after the migration.
	fn columns(&self) -> Option<u32>;
	/// Version of database after the migration.
	fn version(&self) -> u32;
	/// Should migrate existing object to new database.
	/// Returns `None` if the object does not exist in new version of database.
	fn simple_migrate(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)>;
}

impl<T: SimpleMigration> Migration for T {
	fn columns(&self) -> Option<u32> { SimpleMigration::columns(self) }

	fn version(&self) -> u32 { SimpleMigration::version(self) }

	fn alters_existing(&self) -> bool { true }

	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, col);

		for (key, value) in source.iter(col) {
			if let Some((key, value)) = self.simple_migrate(key.to_vec(), value.to_vec()) {
				batch.insert(key, value, dest)?;
			}
		}

		batch.commit(dest)
	}
}

/// An even simpler migration which just changes the number of columns.
pub struct ChangeColumns {
	/// The amount of columns before this migration.
	pub pre_columns: Option<u32>,
	/// The amount of columns after this migration.
	pub post_columns: Option<u32>,
	/// The version after this migration.
	pub version: u32,
}

impl Migration for ChangeColumns {
	fn pre_columns(&self) -> Option<u32> { self.pre_columns }
	fn columns(&self) -> Option<u32> { self.post_columns }
	fn version(&self) -> u32 { self.version }
	fn alters_existing(&self) -> bool { false }
	fn migrate(&mut self, _: Arc<Database>, _: &Config, _: &mut Database, _: Option<u32>) -> Result<(), Error> {
		Ok(())
	}
}

/// Get the path where all databases reside.
fn database_path(path: &Path) -> PathBuf {
	let mut temp_path = path.to_owned();
	temp_path.pop();
	temp_path
}

enum TempIndex {
	One,
	Two,
}

impl TempIndex {
	fn swap(&mut self) {
		match *self {
			TempIndex::One => *self = TempIndex::Two,
			TempIndex::Two => *self = TempIndex::One,
		}
	}

	// given the path to the old database, get the path of this one.
	fn path(&self, db_root: &Path) -> PathBuf {
		let mut buf = db_root.to_owned();

		match *self {
			TempIndex::One => buf.push("temp_migration_1"),
			TempIndex::Two => buf.push("temp_migration_2"),
		};

		buf
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
		let is_new = match self.migrations.last() {
			Some(last) => migration.version() > last.version(),
			None => true,
		};

		match is_new {
			true => Ok(self.migrations.push(Box::new(migration))),
			false => Err(Error::CannotAddMigration),
		}
	}

	/// Performs migration in order, starting with a source path, migrating between two temporary databases,
	/// and producing a path where the final migration lives.
	pub fn execute(&mut self, old_path: &Path, version: u32) -> Result<PathBuf, Error> {
		let config = self.config.clone();
		let migrations = self.migrations_from(version);
		trace!(target: "migration", "Total migrations to execute for version {}: {}", version, migrations.len());
		if migrations.is_empty() { return Err(Error::MigrationImpossible) };

		let columns = migrations.get(0).and_then(|m| m.pre_columns());

		trace!(target: "migration", "Expecting database to contain {:?} columns", columns);
		let mut db_config = DatabaseConfig {
			max_open_files: 64,
			cache_sizes: Default::default(),
			compaction: config.compaction_profile,
			columns: columns,
			wal: true,
		};

		let db_root = database_path(old_path);
		let mut temp_idx = TempIndex::One;
		let mut temp_path = old_path.to_path_buf();

		// start with the old db.
		let old_path_str = old_path.to_str().ok_or(Error::MigrationImpossible)?;
		let mut cur_db = Arc::new(Database::open(&db_config, old_path_str).map_err(Error::Custom)?);

		for migration in migrations {
			trace!(target: "migration", "starting migration to version {}", migration.version());
			// Change number of columns in new db
			let current_columns = db_config.columns;
			db_config.columns = migration.columns();

			// slow migrations: alter existing data.
			if migration.alters_existing() {
				temp_path = temp_idx.path(&db_root);

				// open the target temporary database.
				let temp_path_str = temp_path.to_str().ok_or(Error::MigrationImpossible)?;
				let mut new_db = Database::open(&db_config, temp_path_str).map_err(Error::Custom)?;

				match current_columns {
					// migrate only default column
					None => migration.migrate(cur_db.clone(), &config, &mut new_db, None)?,
					Some(v) => {
						// Migrate all columns in previous DB
						for col in 0..v {
							migration.migrate(cur_db.clone(), &config, &mut new_db, Some(col))?
						}
					}
				}
				// next iteration, we will migrate from this db into the other temp.
				cur_db = Arc::new(new_db);
				temp_idx.swap();

				// remove the other temporary migration database.
				let _ = fs::remove_dir_all(temp_idx.path(&db_root));
			} else {
				// migrations which simply add or remove column families.
				// we can do this in-place.
				let goal_columns = migration.columns().unwrap_or(0);
				while cur_db.num_columns() < goal_columns {
					cur_db.add_column().map_err(Error::Custom)?;
				}

				while cur_db.num_columns() > goal_columns {
					cur_db.drop_column().map_err(Error::Custom)?;
				}
			}
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

	/// Find all needed migrations.
	fn migrations_from(&mut self, version: u32) -> Vec<&mut Box<Migration>> {
		self.migrations.iter_mut().filter(|m| m.version() > version).collect()
	}
}

/// Prints a dot every `max` ticks
pub struct Progress {
	current: usize,
	max: usize,
}

impl Default for Progress {
	fn default() -> Self {
		Progress {
			current: 0,
			max: 100_000,
		}
	}
}

impl Progress {
	/// Tick progress meter.
	pub fn tick(&mut self) {
		self.current += 1;
		if self.current == self.max {
			self.current = 0;
			flush!(".");
		}
	}
}
