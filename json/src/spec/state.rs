// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Blockchain test state deserializer.

use bytes::Bytes;
use hash::Address;
use spec::{Account, Builtin};
use std::collections::BTreeMap;

/// Blockchain test state deserializer.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State(BTreeMap<Address, Account>);

impl State {
    /// Returns all builtins.
    pub fn builtins(&self) -> BTreeMap<Address, Builtin> {
        self.0
            .iter()
            .filter_map(|(add, ref acc)| acc.builtin.clone().map(|b| (add.clone(), b.into())))
            .collect()
    }

    /// Returns all constructors.
    pub fn constructors(&self) -> BTreeMap<Address, Bytes> {
        self.0
            .iter()
            .filter_map(|(add, ref acc)| acc.constructor.clone().map(|b| (add.clone(), b)))
            .collect()
    }
}

impl IntoIterator for State {
    type Item = <BTreeMap<Address, Account> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Address, Account> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
