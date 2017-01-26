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

//! Ethcore rocksdb ipc service

use traits::*;
use rocksdb::{DB, Writable, WriteBatch, IteratorMode, DBIterator, IndexType, Options, DBCompactionStyle, BlockBasedOptions, Direction};
use std::sync::{RwLock, Arc};
use std::convert::From;
use ipc::IpcConfig;
use std::mem;
use ipc::binary::BinaryConvertError;
use std::collections::{VecDeque, HashMap, BTreeMap};

enum WriteCacheEntry {
	Remove,
	Write(Vec<u8>),
}

pub struct WriteCache {
	entries: HashMap<Vec<u8>, WriteCacheEntry>,
	preferred_len: usize,
}

const FLUSH_BATCH_SIZE: usize = 4096;

impl WriteCache {
	fn new(cache_len: usize) -> WriteCache {
		WriteCache {
			entries: HashMap::new(),
			preferred_len: cache_len,
		}
	}

	fn write(&mut self, key: Vec<u8>, val: Vec<u8>) {
		self.entries.insert(key, WriteCacheEntry::Write(val));
	}

	fn remove(&mut self, key: Vec<u8>) {
		self.entries.insert(key, WriteCacheEntry::Remove);
	}

	fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.entries.get(key).and_then(
			|vec_ref| match vec_ref {
				&WriteCacheEntry::Write(ref val) => Some(val.clone()),
				&WriteCacheEntry::Remove => None
			})
	}

	/// WriteCache should be locked for this
	fn flush(&mut self, db: &DB, amount: usize) -> Result<(), Error> {
		let batch = WriteBatch::new();
		let mut removed_so_far = 0;
		while removed_so_far < amount {
			if self.entries.len() == 0 { break; }
			let removed_key = {
				let (key, cache_entry) = self.entries.iter().nth(0)
				.expect("if entries.len == 0, we should have break in the loop, still we got here somehow");

				match *cache_entry {
					WriteCacheEntry::Write(ref val) => {
						batch.put(&key, val)?;
					},
					WriteCacheEntry::Remove => {
						batch.delete(&key)?;
					},
				}
				key.clone()
			};

			self.entries.remove(&removed_key);

			removed_so_far = removed_so_far + 1;
		}
		if removed_so_far > 0 {
			db.write(batch)?;
		}
		Ok(())
	}

	/// flushes until cache is empty
	fn flush_all(&mut self, db: &DB) -> Result<(), Error> {
		while !self.is_empty() { self.flush(db, FLUSH_BATCH_SIZE)?; }
		Ok(())
	}

	fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	fn try_shrink(&mut self, db: &DB) -> Result<(), Error> {
		if self.entries.len() > self.preferred_len {
			self.flush(db, FLUSH_BATCH_SIZE)?;
		}
		Ok(())
	}
}

pub struct Database {
	db: RwLock<Option<DB>>,
	/// Iterators - dont't use between threads!
	iterators: RwLock<BTreeMap<IteratorHandle, DBIterator>>,
	write_cache: RwLock<WriteCache>,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
	pub fn new() -> Database {
		Database {
			db: RwLock::new(None),
			iterators: RwLock::new(BTreeMap::new()),
			write_cache: RwLock::new(WriteCache::new(DEFAULT_CACHE_LEN)),
		}
	}

	pub fn flush(&self) -> Result<(), Error> {
		let mut cache_lock = self.write_cache.write();
		let db_lock = self.db.read();
		if db_lock.is_none() { return Ok(()); }
		let db = db_lock.as_ref().unwrap();

		cache_lock.try_shrink(&db)?;
		Ok(())
	}

	pub fn flush_all(&self) -> Result<(), Error> {
		let mut cache_lock = self.write_cache.write();
		let db_lock = self.db.read();
		if db_lock.is_none() { return Ok(()); }
		let db = db_lock.as_ref().expect("we should have exited with Ok(()) on the previous step");

		cache_lock.flush_all(&db)?;
		Ok(())

	}
}

impl Drop for Database {
	fn drop(&mut self) {
		self.flush().unwrap();
	}
}

#[ipc]
impl DatabaseService for Database {
	fn open(&self, config: DatabaseConfig, path: String) -> Result<(), Error> {
		let mut db = self.db.write();
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
		*db = Some(DB::open(&opts, &path)?);

		Ok(())
	}

	/// Opens database in the specified path with the default config
	fn open_default(&self, path: String) -> Result<(), Error> {
		self.open(DatabaseConfig::default(), path)
	}

	fn close(&self) -> Result<(), Error> {
		self.flush_all()?;

		let mut db = self.db.write();
		if db.is_none() { return Err(Error::IsClosed); }

		*db = None;
		Ok(())
	}

	fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
		let mut cache_lock = self.write_cache.write();
		cache_lock.write(key.to_vec(), value.to_vec());
		Ok(())
	}

	fn delete(&self, key: &[u8]) -> Result<(), Error> {
		let mut cache_lock = self.write_cache.write();
		cache_lock.remove(key.to_vec());
		Ok(())
	}

	fn write(&self, transaction: DBTransaction) -> Result<(), Error> {
		let mut cache_lock = self.write_cache.write();

		let mut writes = transaction.writes.borrow_mut();
		for kv in writes.drain(..) {
			cache_lock.write(kv.key, kv.value);
		}

		let mut removes = transaction.removes.borrow_mut();
		for k in removes.drain(..) {
			cache_lock.remove(k);
		}
		Ok(())
	}

	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
		{
			let key_vec = key.to_vec();
			let cache_hit = self.write_cache.read().get(&key_vec);

			if cache_hit.is_some() {
				return Ok(Some(cache_hit.expect("cache_hit.is_some() = true, still there is none somehow here")))
			}
		}
		let db_lock = self.db.read();
		let db = db_lock.as_ref().ok_or(Error::IsClosed)?;

		match db.get(key)? {
			Some(db_vec) => {
				Ok(Some(db_vec.to_vec()))
			},
			None => Ok(None),
		}
	}

	fn get_by_prefix(&self, prefix: &[u8]) -> Result<Option<Vec<u8>>, Error> {
		let db_lock = self.db.read();
		let db = db_lock.as_ref().ok_or(Error::IsClosed)?;

		let mut iter = db.iterator(IteratorMode::From(prefix, Direction::Forward));
		match iter.next() {
			// TODO: use prefix_same_as_start read option (not availabele in C API currently)
			Some((k, v)) => if k[0 .. prefix.len()] == prefix[..] { Ok(Some(v.to_vec())) } else { Ok(None) },
			_ => Ok(None)
		}
	}

	fn is_empty(&self) -> Result<bool, Error> {
		let db_lock = self.db.read();
		let db = db_lock.as_ref().ok_or(Error::IsClosed)?;

		Ok(db.iterator(IteratorMode::Start).next().is_none())
	}

	fn iter(&self) -> Result<IteratorHandle, Error> {
		let db_lock = self.db.read();
		let db = db_lock.as_ref().ok_or(Error::IsClosed)?;

		let mut iterators = self.iterators.write();
		let next_iterator = iterators.keys().last().unwrap_or(&0) + 1;
		iterators.insert(next_iterator, db.iterator(IteratorMode::Start));
		Ok(next_iterator)
	}

	fn iter_next(&self, handle: IteratorHandle) -> Option<KeyValue> {
		let mut iterators = self.iterators.write();
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

	fn dispose_iter(&self, handle: IteratorHandle) -> Result<(), Error> {
		let mut iterators = self.iterators.write();
		iterators.remove(&handle);
		Ok(())
	}
}

// TODO : put proper at compile-time
impl IpcConfig for Database {}

/// Database iterator
pub struct DatabaseIterator {
	client: Arc<DatabaseClient<::nanomsg::Socket>>,
	handle: IteratorHandle,
}

impl Iterator for DatabaseIterator {
	type Item = (Vec<u8>, Vec<u8>);

	fn next(&mut self) -> Option<Self::Item> {
		self.client.iter_next(self.handle).and_then(|kv| Some((kv.key, kv.value)))
	}
}

impl Drop for DatabaseIterator {
	fn drop(&mut self) {
		self.client.dispose_iter(self.handle).unwrap();
	}
}

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
		db.open_default(path.as_str().to_owned()).unwrap();

		assert!(db.is_empty().is_ok());
	}

	#[test]
	fn can_store_key() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();
		db.open_default(path.as_str().to_owned()).unwrap();

		db.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		db.flush_all().unwrap();
		assert!(!db.is_empty().unwrap());
	}

	#[test]
	fn can_retrieve() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();
		db.open_default(path.as_str().to_owned()).unwrap();
		db.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
		db.close().unwrap();

		db.open_default(path.as_str().to_owned()).unwrap();
		assert_eq!(db.get("xxx".as_bytes()).unwrap().unwrap(), "1".as_bytes().to_vec());
	}
}

#[cfg(test)]
mod write_cache_tests {
	use super::Database;
	use traits::*;
	use devtools::*;

	#[test]
	fn cache_write_flush() {
		let db = Database::new();
		let path = RandomTempPath::create_dir();

		db.open_default(path.as_str().to_owned()).unwrap();
		db.put("100500".as_bytes(), "1".as_bytes()).unwrap();
		db.delete("100500".as_bytes()).unwrap();
		db.flush_all().unwrap();

		let val = db.get("100500".as_bytes()).unwrap();
		assert!(val.is_none());
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
	use crossbeam;
	use run_worker;

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

		client.open_default(path.as_str().to_owned()).unwrap();
		assert!(client.is_empty().unwrap());
		worker_should_exit.store(true, Ordering::Relaxed);
	}

	#[test]
	fn can_put() {
		let url = "ipc:///tmp/parity-db-ipc-test-30.ipc";
		let path = RandomTempPath::create_dir();

		crossbeam::scope(move |scope| {
			let stop = Arc::new(AtomicBool::new(false));
			run_worker(scope, stop.clone(), url);
			let client = nanoipc::generic_client::<DatabaseClient<_>>(url).unwrap();
			client.open_default(path.as_str().to_owned()).unwrap();
			client.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
			client.close().unwrap();

			stop.store(true, Ordering::Relaxed);
		});
	}

	#[test]
	fn can_put_and_read() {
		let url = "ipc:///tmp/parity-db-ipc-test-40.ipc";
		let path = RandomTempPath::create_dir();

		crossbeam::scope(move |scope| {
			let stop = Arc::new(AtomicBool::new(false));
			run_worker(scope, stop.clone(), url);
			let client = nanoipc::generic_client::<DatabaseClient<_>>(url).unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			client.put("xxx".as_bytes(), "1".as_bytes()).unwrap();
			client.close().unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			assert_eq!(client.get("xxx".as_bytes()).unwrap().unwrap(), "1".as_bytes().to_vec());

			stop.store(true, Ordering::Relaxed);
		});
	}

	#[test]
	fn can_read_empty() {
		let url = "ipc:///tmp/parity-db-ipc-test-45.ipc";
		let path = RandomTempPath::create_dir();

		crossbeam::scope(move |scope| {
			let stop = Arc::new(AtomicBool::new(false));
			run_worker(scope, stop.clone(), url);
			let client = nanoipc::generic_client::<DatabaseClient<_>>(url).unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			assert!(client.get("xxx".as_bytes()).unwrap().is_none());

			stop.store(true, Ordering::Relaxed);
		});
	}


	#[test]
	fn can_commit_client_transaction() {
		let url = "ipc:///tmp/parity-db-ipc-test-60.ipc";
		let path = RandomTempPath::create_dir();

		crossbeam::scope(move |scope| {
			let stop = Arc::new(AtomicBool::new(false));
			run_worker(scope, stop.clone(), url);
			let client = nanoipc::generic_client::<DatabaseClient<_>>(url).unwrap();
			client.open_default(path.as_str().to_owned()).unwrap();

			let transaction = DBTransaction::new();
			transaction.put("xxx".as_bytes(), "1".as_bytes());
			client.write(transaction).unwrap();

			client.close().unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			assert_eq!(client.get("xxx".as_bytes()).unwrap().unwrap(), "1".as_bytes().to_vec());

			stop.store(true, Ordering::Relaxed);
		});
	}

	#[test]
	fn key_write_read_ipc() {
		let url = "ipc:///tmp/parity-db-ipc-test-70.ipc";
		let path = RandomTempPath::create_dir();

		crossbeam::scope(|scope| {
			let stop = StopGuard::new();
			run_worker(&scope, stop.share(), url);

			let client = nanoipc::generic_client::<DatabaseClient<_>>(url).unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			let mut batch = Vec::new();
			for _ in 0..100 {
				batch.push((random_str(256).as_bytes().to_vec(), random_str(256).as_bytes().to_vec()));
				batch.push((random_str(256).as_bytes().to_vec(), random_str(2048).as_bytes().to_vec()));
				batch.push((random_str(2048).as_bytes().to_vec(), random_str(2048).as_bytes().to_vec()));
				batch.push((random_str(2048).as_bytes().to_vec(), random_str(256).as_bytes().to_vec()));
			}

			for &(ref k, ref v) in batch.iter() {
				client.put(k, v).unwrap();
			}
			client.close().unwrap();

			client.open_default(path.as_str().to_owned()).unwrap();
			for &(ref k, ref v) in batch.iter() {
				assert_eq!(v, &client.get(k).unwrap().unwrap());
			}
		});
	}
}
