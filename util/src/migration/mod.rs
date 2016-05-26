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

mod db_impl;
mod manager;

#[cfg(test)]
mod tests;

pub use self::manager::{Error, Config, Manager};
pub use self::db_impl::MigrationIterator;
use std::collections::BTreeMap;

/// Single migration.
pub trait Migration: 'static {
	/// Version of database after the migration.
	fn version(&self) -> u32;
	/// Should migrate existing object to new database.
	/// Returns `None` if the object does not exist in new version of database.
	fn simple_migrate(&self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)>;
}

/// Migration destination.
pub trait Destination {
	/// Called on destination to commit batch of migrated entries.
	fn commit(&mut self, batch: BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), Error>;
}
