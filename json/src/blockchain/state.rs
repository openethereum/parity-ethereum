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

//! Blockchain test state deserializer.

use std::collections::BTreeMap;
use hash::Address;
use blockchain::account::Account;

/// Blockchain test state deserializer.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct State(BTreeMap<Address, Account>);

impl IntoIterator for State {
	type Item = <BTreeMap<Address, Account> as IntoIterator>::Item;
	type IntoIter = <BTreeMap<Address, Account> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
