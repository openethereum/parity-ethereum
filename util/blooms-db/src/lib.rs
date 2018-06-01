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

use std::io;
use std::path::Path;
use parking_lot::Mutex;

pub struct Database {
	database: Mutex<db::Database>,
}

impl Database {
	pub fn open<P>(path: P) -> io::Result<Database> where P: AsRef<Path> {
		let result = Database {
			database: Mutex::new(db::Database::open(path)?),
		};

		Ok(result)
	}

	pub fn insert_blooms<'a, I, B>(&self, from: u64, blooms: I) -> io::Result<()>
	where ethbloom::BloomRef<'a>: From<B>, I: Iterator<Item = B> {
		self.database.lock().insert_blooms(from, blooms)
	}

	pub fn filter<'a, B>(&'a self, from: u64, to: u64, bloom: B) -> io::Result<Vec<u64>>
	where ethbloom::BloomRef<'a>: From<B> {
		self.database.lock()
			.iterate_matching(from, to, bloom)?
			.collect::<Result<Vec<u64>, _>>()
	}
}
