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

//! Blockchain state deserializer.

use std::collections::BTreeMap;
use crate::{
	bytes::Bytes,
	hash::{Address, H256},
	spec::{Account, Builtin}
};
use serde::Deserialize;

/// Recent JSON tests can be either a map or a hash (represented by a string).
/// See https://github.com/ethereum/tests/issues/637
#[cfg_attr(any(test, feature = "test-helpers"), derive(Clone))]
#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum HashOrMap {
	/// When the `postState` is large, tests sometimes just include the state root of the last
	/// successful block here.
	Hash(H256),
	/// The expected `postState` of a test
	Map(BTreeMap<Address, Account>),
}

/// Blockchain state deserializer.
#[cfg_attr(any(test, feature = "test-helpers"), derive(Clone))]
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State(pub HashOrMap);

impl State {
	/// Returns all builtins.
	pub fn builtins(&self) -> BTreeMap<Address, Builtin> {
		match &self.0 {
			HashOrMap::Hash(_) => BTreeMap::default(),
			HashOrMap::Map(map) => {
				map.iter().filter_map(|(add, ref acc)| {
					acc.builtin.clone().map(|b| (add.clone(), b))
				}).collect()
			}

		}
	}

	/// Returns all constructors.
	pub fn constructors(&self) -> BTreeMap<Address, Bytes> {
		match &self.0 {
			HashOrMap::Hash(_) => BTreeMap::default(),
			HashOrMap::Map(map) => {
				map.iter().filter_map(|(add, ref acc)| {
					acc.constructor.clone().map(|b| (add.clone(), b))
				}).collect()
			}

		}
	}
}

impl IntoIterator for State {
	type Item = <BTreeMap<Address, Account> as IntoIterator>::Item;
	type IntoIter = <BTreeMap<Address, Account> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		if let HashOrMap::Map(m) = self.0 {
			m.into_iter()
		} else {
			BTreeMap::default().into_iter()
		}
	}
}
