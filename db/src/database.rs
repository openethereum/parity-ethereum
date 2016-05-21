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
use rocksdb::{DB, Writable, WriteBatch, IteratorMode, DBIterator,
	IndexType, Options, DBCompactionStyle, BlockBasedOptions, Direction};
use std::collections::BTreeMap;
use std::sync::{RwLock};
use std::convert::From;
use ipc::IpcConfig;
use std::ops::*;
use std::mem;
use ipc::binary::BinaryConvertError;
use std::collections::VecDeque;

impl From<String> for Error {
	fn from(s: String) -> Error {
		Error::RocksDb(s)
	}
}

pub struct Database {
	db: RwLock<Option<DB>>,
	transactions: RwLock<BTreeMap<TransactionHandle, WriteBatch>>,
	iterators: RwLock<BTreeMap<IteratorHandle, DBIterator>>,
}

impl Database {
	pub fn new() -> Database {
		Database {
			db: RwLock::new(None),
			transactions: RwLock::new(BTreeMap::new()),
			iterators: RwLock::new(BTreeMap::new()),
		}
	}
}

#[derive(Ipc)]
impl DatabaseService for Database {
	fn open(&self, config: DatabaseConfig, path: String) -> Result<(), Error> {
		let mut db = self.db.write().unwrap();
		if db.is_some() { return Err(Error::AlreadyOpen); }

		let mut opts = Options::new();
		opts.set_max_open_files(256);
		opts.create_if_missing(true);
		opts.set_use_fsync(false);
		opts.set_compaction_style(DBCompactionStyle::DBUniversalCompaction);
		if let Some(size) = config.prefix_size {
			let mut block_opts = BlockBasedOptions::new();
			block_opts.set_index_type(IndexType::HashSearch);
			opts.set_block_based_table_factory(&block_opts);
			opts.set_prefix_extractor_fixed_size(size);
		}
		*db = Some(try!(DB::open(&opts, &path)));

		Ok(())
	}

	fn close(&self) -> Result<(), Error> {
		let mut db = self.db.write().unwrap();
		if db.is_none() { return Err(Error::IsClosed); }

		// TODO: wait for transactions to expire/close here?
		if self.transactions.read().unwrap().len() > 0 { return Err(Error::UncommitedTransactions); }

		*db = None;
		Ok(())
	}

	fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		try!(db.put(key, value));
		Ok(())
	}

	fn delete(&self, key: &[u8]) -> Result<(), Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		try!(db.delete(key));
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

	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		match try!(db.get(key)) {
			Some(db_vec) => Ok(Some(db_vec.to_vec())),
			None => Ok(None),
		}
	}

	fn get_by_prefix(&self, prefix: &[u8]) -> Result<Option<Vec<u8>>, Error> {
		let db_lock = self.db.read().unwrap();
		let db = try!(db_lock.as_ref().ok_or(Error::IsClosed));

		let mut iter = db.iterator(IteratorMode::From(prefix, Direction::Forward));
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

	fn transaction_put(&self, transaction: TransactionHandle, key: &[u8], value: &[u8]) -> Result<(), Error>
	{
		let mut transactions = self.transactions.write().unwrap();
		let batch = try!(
			transactions.get_mut(&transaction).ok_or(Error::TransactionUnknown)
		);
		try!(batch.put(&key, &value));
		Ok(())
	}

	fn transaction_delete(&self, transaction: TransactionHandle, key: &[u8]) -> Result<(), Error> {
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

// TODO : put proper at compile-time
impl IpcConfig for Database {}


#[cfg(test)]
mod test {

	use super::Database;
	use traits::*;
	use devtools::*;

	#[test]
	fn can_be_created() {
		let db = Database::new();
		assert!(db.is_empty().is_err());
	}

	#[test]
	fn can_be_open_empty() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();
		db.open(DatabaseConfig { prefix_size: Some(8) }, path.as_str().to_owned()).unwrap();

		assert!(db.is_empty().is_ok());
	}

	#[test]
	fn can_store_key() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();
		db.open(DatabaseConfig { prefix_size: None }, path.as_str().to_owned()).unwrap();

		db.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		assert!(!db.is_empty().unwrap());
	}

	#[test]
	fn can_retrieve() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();
		db.open(DatabaseConfig { prefix_size: None }, path.as_str().to_owned()).unwrap();
		db.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		db.close().unwrap();

		db.open(DatabaseConfig { prefix_size: None }, path.as_str().to_owned()).unwrap();
		assert_eq!(db.get("xxx".as_bytes()).unwrap().unwrap(), "1".as_bytes().to_vec());
	}
}

#[cfg(test)]
mod client_tests {
	use super::{DatabaseClient, Database};
	use traits::*;
	use devtools::*;
	use nanoipc;
	use std::sync::Arc;
	use std::sync::atomic::{Ordering, AtomicBool};

	fn init_worker(addr: &str) -> nanoipc::Worker<Database> {
		let mut worker = nanoipc::Worker::<Database>::new(&Arc::new(Database::new()));
		worker.add_duplex(addr).unwrap();
		worker
	}

	#[test]
	fn can_call_handshake() {
		let url = "ipc:///tmp/parity-db-ipc-test-10.ipc";
		let worker_should_exit = Arc::new(AtomicBool::new(false));
		let worker_is_ready = Arc::new(AtomicBool::new(false));
		let c_worker_should_exit = worker_should_exit.clone();
		let c_worker_is_ready = worker_is_ready.clone();

		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !c_worker_should_exit.load(Ordering::Relaxed) {
				worker.poll();
				c_worker_is_ready.store(true, Ordering::Relaxed);
			}
		});

		while !worker_is_ready.load(Ordering::Relaxed) { }
		let client = nanoipc::init_duplex_client::<DatabaseClient<_>>(url).unwrap();

		let hs = client.handshake();

		worker_should_exit.store(true, Ordering::Relaxed);
		assert!(hs.is_ok());
	}

	#[test]
	fn can_open_db() {
		let url = "ipc:///tmp/parity-db-ipc-test-20.ipc";
		let path = RandomTempPath::create_dir();

		let worker_should_exit = Arc::new(AtomicBool::new(false));
		let worker_is_ready = Arc::new(AtomicBool::new(false));
		let c_worker_should_exit = worker_should_exit.clone();
		let c_worker_is_ready = worker_is_ready.clone();

		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !c_worker_should_exit.load(Ordering::Relaxed) {
				worker.poll();
				c_worker_is_ready.store(true, Ordering::Relaxed);
			}
		});

		while !worker_is_ready.load(Ordering::Relaxed) { }
		let client = nanoipc::init_duplex_client::<DatabaseClient<_>>(url).unwrap();

		client.open(DatabaseConfig { prefix_size: Some(8) }, path.as_str().to_owned()).unwrap();
		assert!(client.is_empty().unwrap());
		worker_should_exit.store(true, Ordering::Relaxed);
	}

	#[test]
	fn can_put() {
		let url = "ipc:///tmp/parity-db-ipc-test-30.ipc";
		let path = RandomTempPath::create_dir();

		let worker_should_exit = Arc::new(AtomicBool::new(false));
		let worker_is_ready = Arc::new(AtomicBool::new(false));
		let c_worker_should_exit = worker_should_exit.clone();
		let c_worker_is_ready = worker_is_ready.clone();

		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !c_worker_should_exit.load(Ordering::Relaxed) {
				worker.poll();
				c_worker_is_ready.store(true, Ordering::Relaxed);
			}
		});

		while !worker_is_ready.load(Ordering::Relaxed) { }
		let client = nanoipc::init_duplex_client::<DatabaseClient<_>>(url).unwrap();

		client.open(DatabaseConfig { prefix_size: Some(8) }, path.as_str().to_owned()).unwrap();
		client.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		client.close().unwrap();

		worker_should_exit.store(true, Ordering::Relaxed);
	}

	#[test]
	fn can_put_and_read() {
		let url = "ipc:///tmp/parity-db-ipc-test-40.ipc";
		let path = RandomTempPath::create_dir();

		let worker_should_exit = Arc::new(AtomicBool::new(false));
		let worker_is_ready = Arc::new(AtomicBool::new(false));
		let c_worker_should_exit = worker_should_exit.clone();
		let c_worker_is_ready = worker_is_ready.clone();

		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !c_worker_should_exit.load(Ordering::Relaxed) {
				worker.poll();
				c_worker_is_ready.store(true, Ordering::Relaxed);
			}
		});

		while !worker_is_ready.load(Ordering::Relaxed) { }
		let client = nanoipc::init_duplex_client::<DatabaseClient<_>>(url).unwrap();

		client.open(DatabaseConfig { prefix_size: Some(8) }, path.as_str().to_owned()).unwrap();
		client.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		client.close().unwrap();

		client.open(DatabaseConfig { prefix_size: Some(8) }, path.as_str().to_owned()).unwrap();
		assert_eq!(client.get("xxx".as_bytes()).unwrap().unwrap(), "1".as_bytes().to_vec());

		worker_should_exit.store(true, Ordering::Relaxed);
	}
}
