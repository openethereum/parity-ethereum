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

//! Helper module that should be used to randomly squeeze
//! caches to a given size in bytes
//!
//! ```
//! extern crate heapsize;
//! extern crate ethcore_util as util;
//! use std::collections::HashMap;
//! use std::mem::size_of;
//! use heapsize::HeapSizeOf;
//! use util::squeeze::Squeeze;
//!
//! fn main() {
//!     let initial_size = 60;
//! 	let mut map: HashMap<u8, u8> = HashMap::with_capacity(initial_size);
//! 	assert!(map.capacity() >= initial_size);
//! 	for i in 0..initial_size {
//! 		map.insert(i as u8, i as u8);
//! 	}
//!
//! 	assert_eq!(map.heap_size_of_children(), map.capacity() * 2 * size_of::<u8>());
//! 	assert_eq!(map.len(), initial_size);
//! 	let initial_heap_size = map.heap_size_of_children();
//!
//! 	// squeeze it to size of key and value
//! 	map.squeeze(2 * size_of::<u8>());
//! 	assert_eq!(map.len(), 1);
//!
//! 	// its likely that heap size was reduced, but we can't be 100% sure
//! 	assert!(initial_heap_size >= map.heap_size_of_children());
//! }
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use heapsize::HeapSizeOf;

/// Should be used to squeeze collections to certain size in bytes
pub trait Squeeze {
	/// Try to reduce collection size to `size` bytes
	fn squeeze(&mut self, size: usize);
}

impl<K, T> Squeeze for HashMap<K, T>
	where K: Eq + Hash + Clone + HeapSizeOf,
	      T: HeapSizeOf,
{
	fn squeeze(&mut self, size: usize) {
		if self.is_empty() {
			return;
		}

		let size_of_entry = self.heap_size_of_children() / self.capacity();
		let all_entries = size_of_entry * self.len();
		let mut shrinked_size = all_entries;

		while !self.is_empty() && shrinked_size > size {
			// could be optimized
			let key = self.keys().next().unwrap().clone();
			self.remove(&key);
			shrinked_size -= size_of_entry;
		}

		self.shrink_to_fit();

		// if we squeezed something, but not enough, squeeze again
		if all_entries != shrinked_size && self.heap_size_of_children() > size {
			self.squeeze(size);
		}
	}
}
