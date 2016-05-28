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

//! Migration manager

use std::collections::BTreeMap;
use migration::{Migration, Destination};

/// Migration error.
#[derive(Debug)]
pub enum Error {
	/// Error returned when it is impossible to add new migration rules.
	CannotAddMigration,
	/// Error returned when migration from specific version can not be performed.
	MigrationImpossible,
	/// Custom error.
	Custom(String),
}

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
			migrations: vec![]
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

	/// Performs migration to destination.
	pub fn execute<D>(&self, db_iter: D, version: u32, destination: &mut Destination) -> Result<(), Error> where
		D: Iterator<Item = (Vec<u8>, Vec<u8>)> {

		let migrations = try!(self.migrations_from(version).ok_or(Error::MigrationImpossible));

		let mut batch: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

		for keypair in db_iter {
			let migrated = migrations.iter().fold(Some(keypair), |migrated, migration| {
				migrated.and_then(|(key, value)| migration.simple_migrate(key, value))
			});

			if let Some((key, value)) = migrated {
				batch.insert(key, value);
			}

			if batch.len() == self.config.batch_size {
				try!(destination.commit(batch));
				batch = BTreeMap::new();
			}
		}

		try!(destination.commit(batch));

		Ok(())
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

