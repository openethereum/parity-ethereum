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

//! Cache for in-memory chained operations

use std::ops::Deref;
use std::hash::Hash;
use std::collections::HashMap;

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
