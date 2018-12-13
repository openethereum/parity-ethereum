// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! This crate allows automatic caching of `T.len()` with an api that 
//! allows drop in replacement for `parking_lot`
//! [`Mutex`](../lock_api/struct.Mutex.html)
//! and [`RwLock`](../lock_api/struct.RwLock.html) for most common use-cases.
//!
//! This crate implements `Len` for the following types: 
//! `std::collections::{VecDeque, LinkedList, HashMap, BTreeMap, HashSet, BTreeSet, BinaryHeap}`
//!
//! ## Example
//!
//! ```rust
//! extern crate len_caching_lock;
//! use len_caching_lock::LenCachingMutex;
//!
//! fn main() {
//!		let vec: Vec<i32> = Vec::new();
//!		let len_caching_mutex = LenCachingMutex::new(vec);
//!		assert_eq!(len_caching_mutex.lock().len(), len_caching_mutex.load_len());
//!		len_caching_mutex.lock().push(0);
//!		assert_eq!(1, len_caching_mutex.load_len());
//!	}
//!	```

extern crate parking_lot;
use std::collections::{VecDeque, LinkedList, HashMap, BTreeMap, HashSet, BTreeSet, BinaryHeap};
use std::hash::Hash;

pub mod mutex;
pub mod rwlock;

pub use mutex::LenCachingMutex;
pub use rwlock::LenCachingRwLock;

/// Implement to allow a type with a len() method to be used
/// with [`LenCachingMutex`](mutex/struct.LenCachingMutex.html)
/// or  [`LenCachingRwLock`](rwlock/struct.LenCachingRwLock.html)
pub trait Len {
	fn len(&self) -> usize;
}

impl<T> Len for Vec<T> {
	fn len(&self) -> usize { Vec::len(self) }
}

impl<T> Len for VecDeque<T> {
	fn len(&self) -> usize { VecDeque::len(self) }
}

impl<T> Len for LinkedList<T> {
	fn len(&self) -> usize { LinkedList::len(self) }
}

impl<K: Eq + Hash, V> Len for HashMap<K, V> {
	fn len(&self) -> usize { HashMap::len(self) }
}

impl<K, V> Len for BTreeMap<K, V> {
	fn len(&self) -> usize { BTreeMap::len(self) }
}

impl<T: Eq + Hash> Len for HashSet<T> {
	fn len(&self) -> usize { HashSet::len(self) }
}

impl<T> Len for BTreeSet<T> {
	fn len(&self) -> usize { BTreeSet::len(self) }
}

impl<T: Ord> Len for BinaryHeap<T> {
	fn len(&self) -> usize { BinaryHeap::len(self) }
}
