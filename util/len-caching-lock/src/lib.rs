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
	fn len(&self) -> usize { self.len() }
}

impl<T> Len for VecDeque<T> {
	fn len(&self) -> usize { self.len() }
}

impl<T> Len for LinkedList<T> {
	fn len(&self) -> usize { self.len() }
}

impl<K: Eq + Hash, V> Len for HashMap<K, V> {
	fn len(&self) -> usize { self.len() }
}

impl<K, V> Len for BTreeMap<K, V> {
	fn len(&self) -> usize { self.len() }
}

impl<T: Eq + Hash> Len for HashSet<T> {
	fn len(&self) -> usize { self.len() }
}

impl<T> Len for BTreeSet<T> {
	fn len(&self) -> usize { self.len() }
}

impl<T: Ord> Len for BinaryHeap<T> {
	fn len(&self) -> usize { self.len() }
}
