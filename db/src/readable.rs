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

//! Readable trait mod

use std::ops::Deref;
use std::hash::Hash;
use std::sync::RwLock;
use std::collections::HashMap;
use util;
use util::rlp::{encode, Encodable, decode, Decodable};
use traits::{DatabaseService};
use cache::{Key, Cache};

/// Should be used to read values from database.
pub trait Readable {
	/// Returns value for given key.
	fn read<T, R>(&self, key: &Key<T, Target = R>) -> Option<T>
		where T: Decodable, R: Deref<Target = [u8]>;

	/// Returns value for given key either in cache or in database.
	fn read_with_cache<K, T, C>(&self, cache: &RwLock<C>, key: &K) -> Option<T>
		where K: Key<T> + Eq + Hash + Clone,
			T: Clone + Decodable,
			C: Cache<K, T>
	{
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
	fn exists_with_cache<K, T, R, C>(&self, cache: &RwLock<C>, key: &K) -> bool
		where K: Eq + Hash + Key<T, Target = R>,
			R: Deref<Target = [u8]>,
			C: Cache<K, T>
	{
		{
			let read = cache.read().unwrap();
			if read.get(key).is_some() {
				return true;
			}
		}

		self.exists::<T, R>(key)
	}
}


impl Readable for util::Database {
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


impl<D: DatabaseService> Readable for D {
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
