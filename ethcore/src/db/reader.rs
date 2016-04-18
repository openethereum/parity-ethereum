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

use std::hash::Hash;
use std::sync::RwLock;
use std::collections::HashMap;
use util::Database;
use util::rlp::Decodable;
use super::{Readable, Key};

pub struct DatabaseReader<'a, K, V> where K: 'a, V: 'a {
	db: &'a Database,
	cache: &'a RwLock<HashMap<K, V>>,
}

impl<'a, K, V> DatabaseReader<'a, K, V> {
	pub fn new(db: &'a Database, cache: &'a RwLock<HashMap<K, V>>) -> Self {
		DatabaseReader {
			db: db,
			cache: cache,
		}
	}

	pub fn read(&self, key: &K) -> Option<V> where
	K: Eq + Hash + Clone + Key<V>,
	V: Clone + Decodable {
		{
			let read = self.cache.read().unwrap();
			if let Some(v) = read.get(key) {
				return Some(v.clone());
			}
		}

		self.db.read(key).map(|value: V|{
			let mut write = self.cache.write().unwrap();
			write.insert(key.clone(), value.clone());
			value
		})
	}

	pub fn exists(&self, key: &K) -> bool where
	K: Eq + Hash + Key<V> {
		{
			let read = self.cache.read().unwrap();
			if read.get(key).is_some() {
				return true;
			}
		}

		self.db.exists::<V>(key)
	}
}
