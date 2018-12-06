
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
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard as InnerWriteGuard};

use Len;

/// Can be used in place of a `Mutex` where reading `T`'s `len()` without 
/// needing to lock, is advantageous. 
/// When the Guard is released, `T`'s `len()` will be cached.
/// The cached `len()` may be at most 1 lock behind current state.
pub struct LenCachingRwLock<T> {
  data: RwLock<T>,
  len: AtomicUsize,
}

impl<T: Len> LenCachingRwLock<T> {
	/// Constructs a new LenCachingRwLock
	pub fn new(data: T) -> LenCachingRwLock<T> {
		LenCachingRwLock {
			len: AtomicUsize::new(data.len()),
			data: RwLock::new(data),
		}
	}

	/// Load the value returned from your `T`'s `len()`
	/// subsequent to the most recent lock being released.
	pub fn load_len(&self) -> usize {
		self.len.load(Ordering::SeqCst)
	}

	/// Convenience method to check if collection T `is_empty()`
	pub fn load_is_empty(&self) -> bool {
		self.len.load(Ordering::SeqCst) == 0
	}

	/// Delegates to `parking_lot::Mutex` `lock()`
	pub fn write(&self) -> RwLockWriteGuard<T> {
		RwLockWriteGuard {
			write_guard: self.data.write(),
			len: &self.len,
		}
	}

	/// Delegates to `parking_lot::Mutex` `try_lock()`
	pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
		Some( RwLockWriteGuard {
			write_guard: self.data.try_write()?,
			len: &self.len,
		})
	}

	pub fn read(&self) -> RwLockReadGuard<T> {
		self.data.read()
	}

	pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
		self.data.try_read()
	}
}

/// Guard comprising `MutexGuard` and `AtomicUsize` for cache
pub struct RwLockWriteGuard<'a, T: Len + 'a> {
	write_guard: InnerWriteGuard<'a, T>,
	len: &'a AtomicUsize,
}

impl<'a, T: Len> RwLockWriteGuard<'a, T> {
	pub fn inner_mut(&mut self) -> &mut InnerWriteGuard<'a, T> {
		&mut self.write_guard
	}

	pub fn inner(&self) -> &InnerWriteGuard<'a, T> {
		&self.write_guard
	}
}

impl<'a, T: Len> Drop for RwLockWriteGuard<'a, T> {
	fn drop(&mut self) {
		self.len.store(self.write_guard.len(), Ordering::SeqCst);
	}
}

impl<'a, T: Len> Deref for RwLockWriteGuard<'a, T> {
	type Target = T;
	fn deref(&self)	-> &T {
		self.write_guard.deref()
	}
}

impl<'a, T: Len> DerefMut for RwLockWriteGuard<'a, T> {
	fn deref_mut(&mut self)	-> &mut T {
		self.write_guard.deref_mut()
	}
}

#[cfg(test)]
mod tests {}
