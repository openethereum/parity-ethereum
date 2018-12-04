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

use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use parking_lot::{Mutex, MutexGuard};

/// Implement to allow a type with a len() method to be used
/// with [`LenCachingMutex`](struct.LenCachingMutex.html)
pub trait Len {
	fn len(&self) -> usize;
}

impl<T> Len for Vec<T> {
	fn len(&self) -> usize { self.len() }
}

impl<T> Len for VecDeque<T> {
	fn len(&self) -> usize { self.len() }
}

/// Can be used in place of a `Mutex` where reading `T`'s `len()` without 
/// needing to lock, is advantageous. 
/// When the Guard is released, `T`'s `len()` will be cached.
/// The cached `len()` may be at most 1 lock behind current state.
pub struct LenCachingMutex<T> {
  data: Mutex<T>,
  len: AtomicUsize,
}

impl<T: Len> LenCachingMutex<T> {
	pub fn new(data: T) -> LenCachingMutex<T> {
		LenCachingMutex {
			len: AtomicUsize::new(data.len()),
			data: Mutex::new(data),
		}
	}

	/// Load the most recent value returned from your `T`'s `len()`
	pub fn load_len(&self) -> usize {
		self.len.load(Ordering::SeqCst)
	}

	pub fn lock(&self) -> Guard<T> {
		Guard {
			mutex_guard: self.data.lock(),
			len: &self.len,
		}
	}

	pub fn try_lock(&self) -> Option<Guard<T>> {
		Some( Guard {
			mutex_guard: self.data.try_lock()?,
			len: &self.len,
		})
	}
}

pub struct Guard<'a, T: Len + 'a> {
	mutex_guard: MutexGuard<'a, T>,
	len: &'a AtomicUsize,
}

impl<'a, T: Len> Guard<'a, T> {
	pub fn inner_mut(&mut self) -> &mut MutexGuard<'a, T> {
		&mut self.mutex_guard
	}

	pub fn inner(&self) -> &MutexGuard<'a, T> {
		&self.mutex_guard
	}
}

impl<'a, T: Len> Drop for Guard<'a, T> {
	fn drop(&mut self) {
		self.len.store(self.mutex_guard.len(), Ordering::SeqCst);
	}
}

impl<'a, T: Len> Deref for Guard<'a, T> {
	type Target = T;
	fn deref(&self)	-> &T {
		self.mutex_guard.deref()
	}
}

impl<'a, T: Len> DerefMut for Guard<'a, T> {
	fn deref_mut(&mut self)	-> &mut T {
		self.mutex_guard.deref_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::VecDeque;

	#[test]
	fn caches_len() {
		let v = vec![1,2,3];
		let lcm = LenCachingMutex::new(v);
		assert_eq!(lcm.load_len(), 3);
		lcm.lock().push(4);
		assert_eq!(lcm.load_len(), 4);
	}

	#[test]
	fn works_with_vec() {
		let v: Vec<i32> = Vec::new();
		let lcm = LenCachingMutex::new(v);
		assert!(lcm.lock().is_empty());
	}

	#[test]
	fn works_with_vecdeque() {
		let v: VecDeque<i32> = VecDeque::new();
		let lcm = LenCachingMutex::new(v);
		lcm.lock().push_front(4);
		assert_eq!(lcm.load_len(), 1);
	}
}
