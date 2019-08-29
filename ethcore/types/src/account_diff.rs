// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Diff between two accounts.

use std::collections::BTreeMap;
use bytes::Bytes;
use ethereum_types::{H256, U256};

#[derive(Debug, PartialEq, Eq, Clone)]
/// Diff type for specifying a change (or not).
pub enum Diff<T> {
	/// Both sides are the same.
	Same,
	/// Left (pre, source) side doesn't include value, right side (post, destination) does.
	Born(T),
	/// Both sides include data; it chaged value between them.
	Changed(T, T),
	/// Left (pre, source) side does include value, right side (post, destination) does not.
	Died(T),
}

impl<T> Diff<T> {
	/// Construct new object with given `pre` and `post`.
	pub fn new(pre: T, post: T) -> Self where T: Eq {
		if pre == post {
			Diff::Same
		} else {
			Diff::Changed(pre, post)
		}
	}

	/// Determine whether there was a change or not.
	pub fn is_same(&self) -> bool {
		match *self {
			Diff::Same => true,
			_ => false
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Account diff.
pub struct AccountDiff {
	/// Change in balance, allowed to be `Diff::Same`.
	pub balance: Diff<U256>,
	/// Change in nonce, allowed to be `Diff::Same`.
	pub nonce: Diff<U256>,
	/// Change in code, allowed to be `Diff::Same`.
	pub code: Diff<Bytes>,
	/// Change in storage, values are not allowed to be `Diff::Same`.
	pub storage: BTreeMap<H256, Diff<H256>>,
}
