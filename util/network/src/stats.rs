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

//! Network Statistics
use std::sync::atomic::*;

/// Network statistics structure
#[derive(Default, Debug)]
pub struct NetworkStats {
	/// Bytes received
	recv: AtomicUsize,
	/// Bytes sent
	send: AtomicUsize,
	/// Total number of sessions created
	sessions: AtomicUsize,
}

impl NetworkStats {
	/// Increase bytes received.
	#[inline]
	pub fn inc_recv(&self, size: usize) {
		self.recv.fetch_add(size, Ordering::Relaxed);
	}

	/// Increase bytes sent.
	#[inline]
	pub fn inc_send(&self, size: usize) {
		self.send.fetch_add(size, Ordering::Relaxed);
	}

	/// Increase number of sessions.
	#[inline]
	pub fn inc_sessions(&self) {
		self.sessions.fetch_add(1, Ordering::Relaxed);
	}

	/// Get bytes sent.
	#[inline]
	pub fn send(&self) -> usize {
		self.send.load(Ordering::Relaxed)
	}

	/// Get bytes received.
	#[inline]
	pub fn recv(&self) -> usize {
		self.recv.load(Ordering::Relaxed)
	}

	/// Get total number of sessions created.
	#[inline]
	pub fn sessions(&self) -> usize {
		self.sessions.load(Ordering::Relaxed)
	}

	/// Create a new empty instance.
	pub fn new() -> NetworkStats {
		NetworkStats {
			recv: AtomicUsize::new(0),
			send: AtomicUsize::new(0),
			sessions: AtomicUsize::new(0),
		}
	}
}
