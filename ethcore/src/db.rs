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

//! Extras db utils.

use std::ops::Deref;
use std::hash::Hash;
use std::sync::RwLock;
use std::collections::HashMap;
use util::{DBTransaction, Database};
use util::rlp::{encode, Encodable, decode, Decodable};

#[derive(Clone, Copy)]
pub enum CacheUpdatePolicy {
	Overwrite,
	Remove,
}

pub trait Cache<K, V> {
	fn insert(&mut self, k: K, v: V) -> Option<V>;

	fn remove(&mut self, k: &K) -> Option<V>;

	fn get(&self, k: &K) -> Option<&V>;
}

impl<K, V> Cache<K, V> for HashMap<K, V> where K: Hash + Eq {
	fn insert(&mut self, k: K, v: V) -> Option<V> {
		HashMap::insert(self, k, v)
	}

	fn remove(&mut self, k: &K) -> Option<V> {
		HashMap::remove(self, k)
	}

	fn get(&self, k: &K) -> Option<&V> {
		HashMap::get(self, k)
	}
}

/// Should be used to get database key associated with given value.
pub trait Key<T> {
	type Target: Deref<Target = [u8]>;

	/// Returns db key.
	fn key(&self) -> Self::Target;
}

/// Should be used to write value into database.
pub trait Writable {
	/// Writes the value into the database.
	fn write<T, R>(&self, key: &Key<T, Target = R>, value: &T) where T: Encodable, R: Deref<Target = [u8]>;

	/// Writes the value into the database and updates the cache.
	fn write_with_cache<K, T, R>(&self, cache: &mut Cache<K, T>, key: K, value: T, policy: CacheUpdatePolicy) where
	K: Key<T, Target = R> + Hash + Eq,
	T: Encodable,
	R: Deref<Target = [u8]> {
		self.write(&key, &value);
		match policy {
			CacheUpdatePolicy::Overwrite => {
				cache.insert(key, value);
			},
			CacheUpdatePolicy::Remove => {
				cache.remove(&key);
			}
		}
	}

	/// Writes the values into the database and updates the cache.
	fn extend_with_cache<K, T, R>(&self, cache: &mut Cache<K, T>, values: HashMap<K, T>, policy: CacheUpdatePolicy) where
	K: Key<T, Target = R> + Hash + Eq,
	T: Encodable,
	R: Deref<Target = [u8]> {
		match policy {
			CacheUpdatePolicy::Overwrite => {
				for (key, value) in values.into_iter() {
					self.write(&key, &value);
					cache.insert(key, value);
				}
			},
			CacheUpdatePolicy::Remove => {
				for (key, value) in &values {
					self.write(key, value);
					cache.remove(key);
				}
			},
		}
	}
}

/// Should be used to read values from database.
pub trait Readable {
	/// Returns value for given key.
	fn read<T, R>(&self, key: &Key<T, Target = R>) -> Option<T> where
	T: Decodable,
	R: Deref<Target = [u8]>;

	/// Returns value for given key either in cache or in database.
	fn read_with_cache<K, T, C>(&self, cache: &RwLock<C>, key: &K) -> Option<T> where
		K: Key<T> + Eq + Hash + Clone,
		T: Clone + Decodable,
		C: Cache<K, T> {
		{
			let read = cache.read().unwrap();
			if let Some(v) = read.get(key) {
				return Some(v.clone());
			}
		}

		self.read(key).map(|value: T|{
			let mut write = cache.write().unwrap();
			write.insert(key.clone(), value.clone());
			value
		})
	}

	/// Returns true if given value exists.
	fn exists<T, R>(&self, key: &Key<T, Target = R>) -> bool where R: Deref<Target= [u8]>;

	/// Returns true if given value exists either in cache or in database.
	fn exists_with_cache<K, T, R, C>(&self, cache: &RwLock<C>, key: &K) -> bool where
	K: Eq + Hash + Key<T, Target = R>,
	R: Deref<Target = [u8]>,
	C: Cache<K, T> {
		{
			let read = cache.read().unwrap();
			if read.get(key).is_some() {
				return true;
			}
		}

		self.exists::<T, R>(key)
	}
}

impl Writable for DBTransaction {
	fn write<T, R>(&self, key: &Key<T, Target = R>, value: &T) where T: Encodable, R: Deref<Target = [u8]> {
		let result = self.put(&key.key(), &encode(value));
		if let Err(err) = result {
			panic!("db put failed, key: {:?}, err: {:?}", &key.key() as &[u8], err);
		}
	}
}

impl Readable for Database {
	fn read<T, R>(&self, key: &Key<T, Target = R>) -> Option<T> where T: Decodable, R: Deref<Target = [u8]> {
		let result = self.get(&key.key());

		match result {
			Ok(option) => option.map(|v| decode(&v)),
			Err(err) => {
				panic!("db get failed, key: {:?}, err: {:?}", &key.key() as &[u8], err);
			}
		}
	}

	fn exists<T, R>(&self, key: &Key<T, Target = R>) -> bool where R: Deref<Target = [u8]> {
		let result = self.get(&key.key());

		match result {
			Ok(v) => v.is_some(),
			Err(err) => {
				panic!("db get failed, key: {:?}, err: {:?}", &key.key() as &[u8], err);
			}
		}
	}
}
