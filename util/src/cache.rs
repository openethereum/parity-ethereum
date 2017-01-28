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

//! Lru-cache related utilities as quick-and-dirty wrappers around the lru-cache
//! crate.
// TODO: push changes upstream in a clean way.

use heapsize::HeapSizeOf;
use lru_cache::LruCache;

use std::hash::Hash;

const INITIAL_CAPACITY: usize = 4;

/// An LRU-cache which operates on memory used.
pub struct MemoryLruCache<K: Eq + Hash, V: HeapSizeOf> {
	inner: LruCache<K, V>,
	cur_size: usize,
	max_size: usize,
}

impl<K: Eq + Hash, V: HeapSizeOf> MemoryLruCache<K, V> {
	/// Create a new cache with a maximum size in bytes.
	pub fn new(max_size: usize) -> Self {
		MemoryLruCache {
			inner: LruCache::new(INITIAL_CAPACITY),
			max_size: max_size,
			cur_size: 0,
		}
	}

	/// Insert an item.
	pub fn insert(&mut self, key: K, val: V) {
		let cap = self.inner.capacity();

		// grow the cache as necessary; it operates on amount of items
		// but we're working based on memory usage.
		if self.inner.len() == cap && self.cur_size < self.max_size {
			self.inner.set_capacity(cap * 2);
		}

		// account for any element displaced from the cache.
		if let Some(lru) = self.inner.insert(key, val) {
			self.cur_size -= lru.heap_size_of_children();
		}

		// remove elements until we are below the memory target.
		while self.cur_size > self.max_size {
			match self.inner.remove_lru() {
				Some((_, v)) => self.cur_size -= v.heap_size_of_children(),
				_ => break,
			}
		}
	}

	/// Get a reference to an item in the cache. It is a logic error for its
	/// heap size to be altered while borrowed.
	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		self.inner.get_mut(key)
	}

	/// Currently-used size of values in bytes.
	pub fn current_size(&self) -> usize {
		self.cur_size
	}
}