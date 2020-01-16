// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use parking_lot::{Mutex, MutexGuard};

use Len;

/// Can be used in place of a [`Mutex`](../../lock_api/struct.Mutex.html) where reading `T`'s `len()` without 
/// needing to lock, is advantageous. 
/// When the Guard is released, `T`'s `len()` will be cached.
/// The cached `len()` may be at most 1 lock behind current state.
#[derive(Debug)]
pub struct LenCachingMutex<T: ?Sized> {
	len: AtomicUsize,
	data: Mutex<T>,
}

impl<T: Len + Default> Default for LenCachingMutex<T> {
	fn default() -> Self {
		LenCachingMutex::new(T::default())
	}
}

impl<T: Len> From<T> for LenCachingMutex<T> {
	fn from(data: T) -> Self {
		LenCachingMutex::new(data)
	}
}

impl<T: Len> LenCachingMutex<T> {
	/// Constructs a new LenCachingMutex
	pub fn new(data: T) -> Self {
		LenCachingMutex {
			len: AtomicUsize::new(data.len()),
			data: Mutex::new(data),
		}
	}
}

impl<T: Len + ?Sized> LenCachingMutex<T> {
	/// Load the cached value that was returned from your `T`'s `len()`
	/// subsequent to the most recent lock being released.
	pub fn load_len(&self) -> usize {
		self.len.load(Ordering::SeqCst)
	}

	/// Delegates to `parking_lot::Mutex`
	/// [`lock()`](../../lock_api/struct.Mutex.html#method.lock).
	pub fn lock(&self) -> CachingMutexGuard<T> {
		CachingMutexGuard {
			mutex_guard: self.data.lock(),
			len: &self.len,
		}
	}

	/// Delegates to `parking_lot::Mutex`
	/// [`try_lock()`](../../lock_api/struct.Mutex.html#method.try_lock).
	pub fn try_lock(&self) -> Option<CachingMutexGuard<T>> {
		Some(CachingMutexGuard {
			mutex_guard: self.data.try_lock()?,
			len: &self.len,
		})
	}
}

/// Guard comprising `MutexGuard` and `AtomicUsize` for cache
pub struct CachingMutexGuard<'a, T: Len + 'a + ?Sized> {
	mutex_guard: MutexGuard<'a, T>,
	len: &'a AtomicUsize,
}

impl<'a, T: Len + ?Sized> CachingMutexGuard<'a, T> {
	/// Returns a mutable reference to the contained
	/// [`MutexGuard`](../../parking_lot/mutex/type.MutexGuard.html)
	pub fn inner_mut(&mut self) -> &mut MutexGuard<'a, T> {
		&mut self.mutex_guard
	}

	/// Returns a non-mutable reference to the contained
	/// [`MutexGuard`](../../parking_lot/mutex/type.MutexGuard.html)
	pub fn inner(&self) -> &MutexGuard<'a, T> {
		&self.mutex_guard
	}
}

impl<'a, T: Len + ?Sized> Drop for CachingMutexGuard<'a, T> {
	fn drop(&mut self) {
		self.len.store(self.mutex_guard.len(), Ordering::SeqCst);
	}
}

impl<'a, T: Len + ?Sized> Deref for CachingMutexGuard<'a, T> {
	type Target = T;
	fn deref(&self)	-> &T {
		self.mutex_guard.deref()
	}
}

impl<'a, T: Len + ?Sized> DerefMut for CachingMutexGuard<'a, T> {
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
