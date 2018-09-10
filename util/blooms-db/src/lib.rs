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

//! Ethereum blooms database

extern crate byteorder;
extern crate ethbloom;
extern crate parking_lot;
extern crate tiny_keccak;

#[cfg(test)]
extern crate tempdir;

mod db;
mod file;

use std::io;
use std::path::Path;
use parking_lot::Mutex;

/// Threadsafe API for blooms database.
///
/// # Warning
///
/// This database does not guarantee atomic writes.
pub struct Database {
	database: Mutex<db::Database>,
}

impl Database {
	/// Creates new database handle.
	///
	/// # Arguments
	///
	/// * `path` - database directory
	pub fn open<P>(path: P) -> io::Result<Database> where P: AsRef<Path> {
		let result = Database {
			database: Mutex::new(db::Database::open(path)?),
		};

		Ok(result)
	}

	/// Closes the inner database
	pub fn close(&self) -> io::Result<()> {
		self.database.lock().close()
	}

	/// Reopens database at the same location.
	pub fn reopen(&self) -> io::Result<()> {
		self.database.lock().reopen()
	}

	/// Inserts one or more blooms into database.
	///
	/// # Arguments
	///
	/// * `from` - index of the first bloom that needs to be inserted
	/// * `blooms` - iterator over blooms
	pub fn insert_blooms<'a, I, B>(&self, from: u64, blooms: I) -> io::Result<()>
	where ethbloom::BloomRef<'a>: From<B>, I: Iterator<Item = B> {
		self.database.lock().insert_blooms(from, blooms)
	}

	/// Returns indexes of all headers matching given bloom in a specified range.
	///
	/// # Arguments
	///
	/// * `from` - index of the first bloom that needs to be checked
	/// * `to` - index of the last bloom that needs to be checked (inclusive range)
	/// * `blooms` - searched pattern
	pub fn filter<'a, B, I, II>(&self, from: u64, to: u64, blooms: II) -> io::Result<Vec<u64>>
	where ethbloom::BloomRef<'a>: From<B>, II: IntoIterator<Item = B, IntoIter = I> + Copy, I: Iterator<Item = B> {
		self.database.lock()
			.iterate_matching(from, to, blooms)?
			.collect::<Result<Vec<u64>, _>>()
	}
}
