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

//! Ethcore rocksdb ipc service

use traits::*;
use rocksdb::{DB, Writable, WriteBatch, IteratorMode, DBVector, DBIterator,
	IndexType, Options, DBCompactionStyle, BlockBasedOptions, Direction};
use std::collections::HashMap;
use std::sync::{RwLock, Mutex};
use std::path::Path;
use std::convert::From;
use std::ops::Deref;

impl From<String> for Error {
	fn from(s: String) -> Error {
		Error::RocksDb(s)
	}
}

pub struct Database {
	db: RwLock<Option<DB>>,
	is_open: Mutex<bool>,
	transactions: RwLock<HashMap<TransactionHandle, WriteBatch>>,
	iterators: RwLock<HashMap<IteratorHandle, DBIterator<'static>>>,
}

impl DatabaseService for Database {
	fn open(&self, path: String) -> Result<(), Error> {
		if *self.is_open.lock().unwrap() { return Err(Error::AlreadyOpen); }
		Ok(())
	}

	fn close(&self) -> Result<(), Error> {
		if *self.is_open.lock().unwrap() { return Err(Error::IsClosed); }
		Ok(())
	}

	fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		try!(db.put(&key, &value));
		Ok(())
	}

	fn delete(&self, key: Vec<u8>) -> Result<(), Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		try!(db.delete(&key));
		Ok(())
	}

	fn write(&self, handle: TransactionHandle) -> Result<(), Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		let mut transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.remove(&handle).ok_or(Error::TransactionUnknown)
		);
		try!(db.write(batch));
		Ok(())
	}

	fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		match try!(db.get(&key)) {
			Some(db_vec) => Ok(Some(db_vec.to_vec())),
			None => Ok(None),
		}
	}

	fn get_by_prefix(&self, prefix: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		let mut iter = db.iterator(IteratorMode::From(&prefix, Direction::forward));
		match iter.next() {
			// TODO: use prefix_same_as_start read option (not availabele in C API currently)
			Some((k, v)) => if k[0 .. prefix.len()] == prefix[..] { Ok(Some(v.to_vec())) } else { Ok(None) },
			_ => Ok(None)
		}
	}

	fn is_empty(&self) -> Result<bool, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		Ok(db.iterator(IteratorMode::Start).next().is_none())
	}

	fn iter(&self) -> Result<IteratorHandle, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		let mut iterators = self.iterators.write().unwrap();
		let next_iterator = iterators.keys().last().unwrap_or(&0) + 1;
		iterators.insert(next_iterator, db.iterator(IteratorMode::Start));
		Ok(next_iterator)
	}

	fn iter_next(&self, handle: IteratorHandle) -> Option<KeyValue>
	{
		let mut iterators = self.iterators.write().unwrap();
		let mut iterator = match iterators.get_mut(&handle) {
			Some(some_iterator) => some_iterator,
			None => { return None; },
		};

		iterator.next().and_then(|(some_key, some_val)| {
			Some(KeyValue {
				key: some_key.to_vec(),
				value: some_val.to_vec(),
			})
		})
	}

	fn transaction_put(&self, transaction: TransactionHandle, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error>
	{
		let mut transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get_mut(&transaction).ok_or(Error::TransactionUnknown)
		);
		try!(batch.put(&key, &value));
		Ok(())
	}

	fn transaction_delete(&self, transaction: TransactionHandle, key: Vec<u8>) -> Result<(), Error> {
		let mut transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get_mut(&transaction).ok_or(Error::TransactionUnknown)
		);
		try!(batch.delete(&key));
		Ok(())
	}

	fn new_transaction(&self) -> TransactionHandle {
		let mut transactions = self.transactions.write().unwrap();
		let next_transaction = transactions.keys().last().unwrap_or(&0) + 1;
		transactions.insert(next_transaction, WriteBatch::new());

		next_transaction
	}
}
