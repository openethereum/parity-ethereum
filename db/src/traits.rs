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

//! Ethcore database trait

use std::cell::RefCell;

pub type IteratorHandle = u32;

pub const DEFAULT_CACHE_LEN: usize = 12288;

#[derive(Binary)]
pub struct KeyValue {
	pub key: Vec<u8>,
	pub value: Vec<u8>,
}

#[derive(Debug, Binary)]
pub enum Error {
	AlreadyOpen,
	IsClosed,
	RocksDb(String),
	TransactionUnknown,
	IteratorUnknown,
	UncommitedTransactions,
}

impl From<String> for Error {
	fn from(s: String) -> Error {
		Error::RocksDb(s)
	}
}

/// Database configuration
#[derive(Binary)]
pub struct DatabaseConfig {
	/// Optional prefix size in bytes. Allows lookup by partial key.
	pub prefix_size: Option<usize>,
	/// write cache length
	pub cache: usize,
}

impl Default for DatabaseConfig {
	fn default() -> DatabaseConfig {
		DatabaseConfig {
			prefix_size: None,
			cache: DEFAULT_CACHE_LEN,
		}
	}
}

impl DatabaseConfig {
	fn with_prefix(prefix: usize) -> DatabaseConfig {
		DatabaseConfig {
			prefix_size: Some(prefix),
			cache: DEFAULT_CACHE_LEN,
		}
	}
}

pub trait DatabaseService : Sized {
	/// Opens database in the specified path
	fn open(&self, config: DatabaseConfig, path: String) -> Result<(), Error>;

	/// Opens database in the specified path with the default config
	fn open_default(&self, path: String) -> Result<(), Error>;

	/// Closes database
	fn close(&self) -> Result<(), Error>;

	/// Insert a key-value pair in the transaction. Any existing value value will be overwritten.
	fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;

	/// Delete value by key.
	fn delete(&self, key: &[u8]) -> Result<(), Error>;

	/// Get value by key.
	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;

	/// Get value by partial key. Prefix size should match configured prefix size.
	fn get_by_prefix(&self, prefix: &[u8]) -> Result<Option<Vec<u8>>, Error>;

	/// Check if there is anything in the database.
	fn is_empty(&self) -> Result<bool, Error>;

	/// Get handle to iterate through keys
	fn iter(&self) -> Result<IteratorHandle, Error>;

	/// Next key-value for the the given iterator
	fn iter_next(&self, iterator: IteratorHandle) -> Option<KeyValue>;

	/// Dispose iteration that is no longer needed
	fn dispose_iter(&self, handle: IteratorHandle) -> Result<(), Error>;

	/// Write client transaction
	fn write(&self, transaction: DBTransaction) -> Result<(), Error>;
}

#[derive(Binary)]
pub struct DBTransaction {
	pub writes: RefCell<Vec<KeyValue>>,
	pub removes: RefCell<Vec<Vec<u8>>>,
}

impl DBTransaction {
	pub fn new() -> DBTransaction {
		DBTransaction {
			writes: RefCell::new(Vec::new()),
			removes: RefCell::new(Vec::new()),
		}
	}

	pub fn put(&self, key: &[u8], value: &[u8]) {
		let mut brw = self.writes.borrow_mut();
		brw.push(KeyValue { key: key.to_vec(), value: value.to_vec() });
	}

	pub fn delete(&self, key: &[u8]) {
		let mut brw = self.removes.borrow_mut();
		brw.push(key.to_vec());
	}
}
