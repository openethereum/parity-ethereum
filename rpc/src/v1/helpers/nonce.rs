// Copyright 2015-2017 harity Technologies (UK) Ltd.
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

use std::{cmp, mem};
use std::sync::{atomic, Arc};
use std::sync::atomic::AtomicUsize;

use bigint::prelude::U256;
use futures::{Future, future, Poll, Async};
use futures::future::Either;
use futures::sync::oneshot;
use futures_cpupool::CpuPool;

/// Manages currently reserved and prospective nonces.
#[derive(Debug)]
pub struct Reservations {
	previous: Option<oneshot::Receiver<U256>>,
	pool: CpuPool,
	prospective_value: U256,
	dropped: Arc<AtomicUsize>,
}

impl Reservations {
	/// Create new nonces manager and spawn a single-threaded cpu pool
	/// for progressing execution of dropped nonces.
	pub fn new() -> Self {
		Self::with_pool(CpuPool::new(1))
	}

	/// Create new nonces manager with given cpu pool.
	pub fn with_pool(pool: CpuPool) -> Self {
		Reservations {
			previous: None,
			pool,
			prospective_value: Default::default(),
			dropped: Default::default(),
		}
	}

	/// Reserves a prospective nonce.
	/// The caller should provide a minimal nonce that needs to be reserved (taken from state/txqueue).
	/// If there were any previous reserved nonces the returned future will be resolved when those are finished
	/// (confirmed that the nonce were indeed used).
	/// The caller can use `prospective_nonce` and perform some heavy computation anticipating
	/// that the `prospective_nonce` will be equal to the one he will get.
	pub fn reserve_nonce(&mut self, minimal: U256) -> Reserved {
		// Update prospective value
		let dropped = self.dropped.swap(0, atomic::Ordering::SeqCst);
		let prospective_value = cmp::max(minimal, self.prospective_value - dropped.into());
		self.prospective_value = prospective_value + 1.into();

		let (next, rx) = oneshot::channel();
		let next = Some(next);
		let pool = self.pool.clone();
		let dropped = self.dropped.clone();
		match mem::replace(&mut self.previous, Some(rx)) {
			Some(previous) => Reserved {
				previous: Either::A(previous),
				next,
				minimal,
				prospective_value,
				pool,
				dropped,
			},
			None => Reserved {
				previous: Either::B(future::ok(minimal)),
				next,
				minimal,
				prospective_value,
				pool,
				dropped,
			},
		}
	}
}

/// Represents a future nonce.
#[derive(Debug)]
pub struct Reserved {
	previous: Either<
		oneshot::Receiver<U256>,
		future::FutureResult<U256, oneshot::Canceled>
	>,
	next: Option<oneshot::Sender<U256>>,
	minimal: U256,
	prospective_value: U256,
	pool: CpuPool,
	dropped: Arc<AtomicUsize>,
}

impl Reserved {
	/// Returns a prospective value of the nonce.
	/// NOTE: This might be different than the one we resolve to.
	/// Make sure to check if both nonces match or use the latter one.
	pub fn prospective_value(&self) -> &U256 {
		&self.prospective_value
	}
}

impl Future for Reserved {
    type Item = Ready;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let mut value = try_ready!(self.previous.poll().map_err(|e| {
			warn!("Unexpected nonce cancellation: {}", e);
		}));

		if value < self.minimal {
			value = self.minimal
		}
		let matches_prospective = value == self.prospective_value;

		Ok(Async::Ready(Ready {
			value,
			matches_prospective,
			next: self.next.take(),
			dropped: self.dropped.clone(),
		}))
	}
}

impl Drop for Reserved {
	fn drop(&mut self) {
		if let Some(next) = self.next.take() {
			self.dropped.fetch_add(1, atomic::Ordering::SeqCst);
			// If Reserved is dropped just pipe previous and next together.
			let previous = mem::replace(&mut self.previous, Either::B(future::ok(U256::default())));
			self.pool.spawn(previous.map(|nonce| {
				next.send(nonce).expect(Ready::RECV_PROOF)
			})).forget()
		}
	}
}

/// Represents a valid reserved nonce.
/// This can be used to dispatch the transaction.
///
/// After this nonce is used it should be marked as such
/// using `mark_used` method.
#[derive(Debug)]
pub struct Ready {
	value: U256,
	matches_prospective: bool,
	next: Option<oneshot::Sender<U256>>,
	dropped: Arc<AtomicUsize>,
}

impl Ready {
	const RECV_PROOF: &'static str = "Receiver never dropped.";

	/// Returns a value of the nonce.
	pub fn value(&self) -> &U256 {
		&self.value
	}

	/// Returns true if current value matches the prospective nonce.
	pub fn matches_prospective(&self) -> bool {
		self.matches_prospective
	}

	/// Marks this nonce as used.
	/// Make sure to call that method after this nonce has been consumed.
	pub fn mark_used(mut self) {
		let next = self.next.take().expect("Nonce can be marked as used only once; qed");
		next.send(self.value + 1.into()).expect(Self::RECV_PROOF);
	}
}

impl Drop for Ready {
	fn drop(&mut self) {
		if let Some(send) = self.next.take() {
			self.dropped.fetch_add(1, atomic::Ordering::SeqCst);
			send.send(self.value).expect(Self::RECV_PROOF);
		}
	}
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
	fn should_reserve_a_set_of_nonces_and_resolve_them() {
		let mut nonces = Reservations::new();

		let n1 = nonces.reserve_nonce(5.into());
		let n2 = nonces.reserve_nonce(5.into());
		let n3 = nonces.reserve_nonce(5.into());
		let n4 = nonces.reserve_nonce(5.into());

		// Check first nonce
		let r = n1.wait().unwrap();
		assert_eq!(r.value(), &U256::from(5));
		assert!(r.matches_prospective());
		r.mark_used();

		// Drop second nonce
		drop(n2);

		// Drop third without marking as used
		let r = n3.wait().unwrap();
		drop(r);

		// Last nonce should be resolved to 6
		let r = n4.wait().unwrap();
		assert_eq!(r.value(), &U256::from(6));
		assert!(!r.matches_prospective());
		r.mark_used();

		// Next nonce should be immediately available.
		let n5 = nonces.reserve_nonce(5.into());
		let r = n5.wait().unwrap();
		assert_eq!(r.value(), &U256::from(7));
		assert!(r.matches_prospective());
		r.mark_used();

		// Should use start number if it's greater
		let n6 = nonces.reserve_nonce(10.into());
		let r = n6.wait().unwrap();
		assert_eq!(r.value(), &U256::from(10));
		assert!(r.matches_prospective());
		r.mark_used();
	}

	#[test]
	fn should_return_prospective_nonce() {
		let mut nonces = Reservations::new();

		let n1 = nonces.reserve_nonce(5.into());
		let n2 = nonces.reserve_nonce(5.into());

		assert_eq!(n1.prospective_value(), &U256::from(5));
		assert_eq!(n2.prospective_value(), &U256::from(6));
	}
}
