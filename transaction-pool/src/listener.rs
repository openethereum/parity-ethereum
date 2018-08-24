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

use std::sync::Arc;
use error::ErrorKind;

/// Transaction pool listener.
///
/// Listener is being notified about status of every transaction in the pool.
pub trait Listener<T> {
	/// The transaction has been successfuly added to the pool.
	/// If second argument is `Some` the transaction has took place of some other transaction
	/// which was already in pool.
	/// NOTE: You won't be notified about drop of `old` transaction separately.
	fn added(&mut self, _tx: &Arc<T>, _old: Option<&Arc<T>>) {}

	/// The transaction was rejected from the pool.
	/// It means that it was too cheap to replace any transaction already in the pool.
	fn rejected(&mut self, _tx: &Arc<T>, _reason: &ErrorKind) {}

	/// The transaction was pushed out from the pool because of the limit.
	fn dropped(&mut self, _tx: &Arc<T>, _by: Option<&T>) {}

	/// The transaction was marked as invalid by executor.
	fn invalid(&mut self, _tx: &Arc<T>) {}

	/// The transaction has been canceled.
	fn canceled(&mut self, _tx: &Arc<T>) {}

	/// The transaction has been culled from the pool.
	fn culled(&mut self, _tx: &Arc<T>) {}
}

/// A no-op implementation of `Listener`.
#[derive(Debug)]
pub struct NoopListener;
impl<T> Listener<T> for NoopListener {}

impl<T, A, B> Listener<T> for (A, B) where
	A: Listener<T>,
	B: Listener<T>,
{
	fn added(&mut self, tx: &Arc<T>, old: Option<&Arc<T>>) {
		self.0.added(tx, old);
		self.1.added(tx, old);
	}

	fn rejected(&mut self, tx: &Arc<T>, reason: &ErrorKind) {
		self.0.rejected(tx, reason);
		self.1.rejected(tx, reason);
	}

	fn dropped(&mut self, tx: &Arc<T>, by: Option<&T>) {
		self.0.dropped(tx, by);
		self.1.dropped(tx, by);
	}

	fn invalid(&mut self, tx: &Arc<T>) {
		self.0.invalid(tx);
		self.1.invalid(tx);
	}

	fn canceled(&mut self, tx: &Arc<T>) {
		self.0.canceled(tx);
		self.1.canceled(tx);
	}

	fn culled(&mut self, tx: &Arc<T>) {
		self.0.culled(tx);
		self.1.culled(tx);
	}
}
