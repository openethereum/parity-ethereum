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
use std::sync::RwLock;

pub struct DatabaseInstance {
	db: DB,
	is_open: bool,
	transactions: RwLock<HashMap<TransactionHandle, WriteBatch>>,
	iterators: RwLock<HashMap<IteratorHandle, DBIterator<'static>>>,
}

impl Database for DatabaseInstance {
	fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String> {
		self.db.put(&key, &value)
	}

	fn delete(&self, key: Vec<u8>) -> Result<(), String> {
		self.db.delete(&key)
	}

	fn write(&self, handle: TransactionHandle) -> Result<(), String> {
		let transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get(&handle).ok_or("Unknown transaction to write".to_owned())
		);
		self.db.write(*batch)
	}

	fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
		match try!(self.db.get(&key)) {
			Some(db_vec) => Ok(Some(db_vec.to_vec())),
			None => Ok(None),
		}
	}

	fn get_by_prefix(&self, prefix: Vec<u8>) -> Option<Vec<u8>> {
		let mut iter = self.db.iterator(IteratorMode::From(&prefix, Direction::forward));
		match iter.next() {
			// TODO: use prefix_same_as_start read option (not availabele in C API currently)
			Some((k, v)) => if k[0 .. prefix.len()] == prefix[..] { Some(v.to_vec()) } else { None },
			_ => None
		}
	}

	fn is_empty(&self) -> bool {
		self.db.iterator(IteratorMode::Start).next().is_none()
	}

	fn iter(&self) -> IteratorHandle {
		let iterators = self.iterators.write().unwrap();
		let next_iterator = iterators.keys().last().unwrap_or(&0) + 1;
		iterators.insert(next_iterator, self.db.iterator(IteratorMode::Start));

		next_iterator
	}

	fn transaction_put(&self, transaction: TransactionHandle, key: Vec<u8>, value: Vec<u8>) -> Result<(), String>
	{
		let transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get(&transaction).ok_or("Unknown transaction to write to".to_owned())
		);
		batch.put(&key, &value)
	}

	fn transaction_delete(&self, transaction: TransactionHandle, key: Vec<u8>) -> Result<(), String> {
		let transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get(&transaction).ok_or("Unknown transaction to delete from".to_owned())
		);
		batch.delete(&key)
	}

	fn new_transaction(&self) -> TransactionHandle {
		let transactions = self.transactions.write().unwrap();
		let next_transaction = transactions.keys().last().unwrap_or(&0) + 1;
		transactions.insert(next_transaction, WriteBatch::new());

		next_transaction
	}

	fn iter_next(&self, handle: IteratorHandle) -> Option<KeyValue>
	{
		let iterators = self.iterators.write().unwrap();
		let iterator = match iterators.get(&handle) {
			Some(ref some_iterator) => some_iterator,
			None => { return None; },
		};

		iterator.next().and_then(|(some_key, some_val)| {
			Some(KeyValue {
				key: some_key.to_vec(),
				value: some_val.to_vec(),
			})
		})
	}
}
