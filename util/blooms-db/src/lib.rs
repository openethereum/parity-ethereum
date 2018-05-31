//! Ethereum blooms database
//!
//! zero allocation
//! zero copying

extern crate byteorder;
extern crate ethbloom;
extern crate parking_lot;
extern crate tiny_keccak;

#[cfg(test)]
extern crate tempdir;

mod db;
mod file;
mod meta;
mod pending;

pub const VERSION: u64 = 1;

use std::io;
use std::path::Path;
use parking_lot::RwLock;

pub struct Database {
	database: RwLock<db::Database>,
}

impl Database {
	pub fn open<P>(path: P) -> io::Result<Database> where P: AsRef<Path> {
		let result = Database {
			database: RwLock::new(db::Database::open(path)?),
		};

		Ok(result)
	}

	pub fn insert_blooms<'a, B>(&self, from: u64, blooms: impl Iterator<Item = B>) -> io::Result<()>
	where ethbloom::BloomRef<'a>: From<B> {
		self.database.write().insert_blooms(from, blooms)
	}

	pub fn flush(&self) -> io::Result<()> {
		self.database.write().flush()
	}

	pub fn filter<'a, B>(&'a self, from: u64, to: u64, bloom: B) -> io::Result<Vec<u64>>
	where ethbloom::BloomRef<'a>: From<B> {
		let database = self.database.read();
			database
				.iterate_matching(from, to, bloom)?
				.collect::<Result<Vec<u64>, _>>()
	}
}
