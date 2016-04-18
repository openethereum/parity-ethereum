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

use std::collections::HashMap;
use std::hash::Hash;
use util::DBTransaction;
use util::rlp::Encodable;
use super::{Key, Writable};

#[derive(Clone, Copy)]
pub enum CacheUpdatePolicy {
	Overwrite,
	Remove,
}

pub struct BatchWriter<'a, K, V> where K: 'a, V: 'a {
	batch: &'a DBTransaction,
	cache: &'a mut HashMap<K, V>,
}

impl<'a, K, V> BatchWriter<'a, K, V> {
	pub fn new(batch: &'a DBTransaction, cache: &'a mut HashMap<K, V>) -> Self {
		BatchWriter {
			batch: batch,
			cache: cache,
		}
	}

	pub fn extend(&mut self, map: HashMap<K, V>, policy: CacheUpdatePolicy) where
	K: Key<V> + Hash + Eq,
	V: Encodable {
		match policy {
			CacheUpdatePolicy::Overwrite => {
				for (key, value) in map.into_iter() {
					self.batch.write(&key, &value);
					self.cache.insert(key, value);
				}
			},
			CacheUpdatePolicy::Remove => {
				for (key, value) in &map {
					self.batch.write(key, value);
					self.cache.remove(key);
				}
			},
		}
	}
}
