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

/// Transaction readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
	/// The transaction is stalled (and should/will be removed from the pool).
	Stalled,
	/// The transaction is ready to be included in pending set.
	Ready,
	/// The transaction is not yet ready.
	Future,
}

/// A readiness indicator.
pub trait Ready<T> {
	/// Returns true if transaction is ready to be included in pending block,
	/// given all previous transactions that were ready are already included.
	///
	/// NOTE: readiness of transactions will be checked according to `Score` ordering,
	/// the implementation should maintain a state of already checked transactions.
	fn is_ready(&mut self, tx: &T) -> Readiness;
}

impl<T, F> Ready<T> for F where F: FnMut(&T) -> Readiness {
	fn is_ready(&mut self, tx: &T) -> Readiness {
		(*self)(tx)
	}
}
