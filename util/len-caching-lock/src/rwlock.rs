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

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use Len;

/// Can be used in place of a [`RwLock`](../../lock_api/struct.RwLock.html) where 
/// reading `T`'s `len()` without needing to lock, is advantageous. 
/// When the WriteGuard is released, `T`'s `len()` will be cached.
#[derive(Debug)]
pub struct LenCachingRwLock<T: ?Sized> {
	len: AtomicUsize,
	data: RwLock<T>,
}

impl<T: Len + Default> Default for LenCachingRwLock<T> {
	fn default() -> Self {
		LenCachingRwLock::new(T::default())
	}
}

impl<T: Len> From<T> for LenCachingRwLock<T> {
	fn from(data: T) -> Self {
		LenCachingRwLock::new(data)
	}
}

impl<T: Len> LenCachingRwLock<T> {
	/// Constructs a new LenCachingRwLock
	pub fn new(data: T) -> Self {
		LenCachingRwLock {
			len: AtomicUsize::new(data.len()),
			data: RwLock::new(data),
		}
	}
}

impl<T: Len + ?Sized> LenCachingRwLock<T> {
	/// Load the cached value that was returned from your `T`'s `len()`
	/// subsequent to the most recent lock being released.
	pub fn load_len(&self) -> usize {
		self.len.load(Ordering::SeqCst)
	}

	/// Delegates to `parking_lot::RwLock`
	/// [`write()`](../../lock_api/struct.RwLock.html#method.write).
	pub fn write(&self) -> CachingRwLockWriteGuard<T> {
		CachingRwLockWriteGuard {
			write_guard: self.data.write(),
			len: &self.len,
		}
	}

	/// Delegates to `parking_lot::RwLock`
	/// [`try_write()`](../../lock_api/struct.RwLock.html#method.try_write).
	pub fn try_write(&self) -> Option<CachingRwLockWriteGuard<T>> {
		Some(CachingRwLockWriteGuard {
			write_guard: self.data.try_write()?,
			len: &self.len,
		})
	}

	/// Delegates to `parking_lot::RwLock`
	/// [`read()`](../../lock_api/struct.RwLock.html#method.read).
	pub fn read(&self) -> RwLockReadGuard<T> {
		self.data.read()
	}

	/// Delegates to `parking_lot::RwLock`
	/// [`try_read()`](../../lock_api/struct.RwLock.html#method.try_read).
	pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
		self.data.try_read()
	}
}

/// Guard that caches `T`'s `len()` in an `AtomicUsize` when dropped
pub struct CachingRwLockWriteGuard<'a, T: Len + 'a + ?Sized> {
	write_guard: RwLockWriteGuard<'a, T>,
	len: &'a AtomicUsize,
}

impl<'a, T: Len + ?Sized> CachingRwLockWriteGuard<'a, T> {
	/// Returns a mutable reference to the contained
	/// [`RwLockWriteGuard`](../../parking_lot/rwlock/type.RwLockWriteGuard.html)
	pub fn inner_mut(&mut self) -> &mut RwLockWriteGuard<'a, T> {
		&mut self.write_guard
	}

	/// Returns a non-mutable reference to the contained
	/// [`RwLockWriteGuard`](../../parking_lot/rwlock/type.RwLockWriteGuard.html)
	pub fn inner(&self) -> &RwLockWriteGuard<'a, T> {
		&self.write_guard
	}
}

impl<'a, T: Len + ?Sized> Drop for CachingRwLockWriteGuard<'a, T> {
	fn drop(&mut self) {
		self.len.store(self.write_guard.len(), Ordering::SeqCst);
	}
}

impl<'a, T: Len + ?Sized> Deref for CachingRwLockWriteGuard<'a, T> {
	type Target = T;
	fn deref(&self)	-> &T {
		self.write_guard.deref()
	}
}

impl<'a, T: Len + ?Sized> DerefMut for CachingRwLockWriteGuard<'a, T> {
	fn deref_mut(&mut self)	-> &mut T {
		self.write_guard.deref_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::VecDeque;

	#[test]
	fn caches_len() {
		let v = vec![1,2,3];
		let lcl = LenCachingRwLock::new(v);
		assert_eq!(lcl.load_len(), 3);
		lcl.write().push(4);
		assert_eq!(lcl.load_len(), 4);
	}

	#[test]
	fn works_with_vec() {
		let v: Vec<i32> = Vec::new();
		let lcl = LenCachingRwLock::new(v);
		assert!(lcl.write().is_empty());
	}

	#[test]
	fn works_with_vecdeque() {
		let v: VecDeque<i32> = VecDeque::new();
		let lcl = LenCachingRwLock::new(v);
		lcl.write().push_front(4);
		assert_eq!(lcl.load_len(), 1);
	}

	#[test]
	fn read_works() {
		let v = vec![1,2,3];
		let lcl = LenCachingRwLock::new(v);
		assert_eq!(lcl.read().len(), 3);
	}
}
