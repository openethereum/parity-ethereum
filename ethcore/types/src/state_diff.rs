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

//! State diff module.

use account_diff::*;
use ethereum_types::Address;
use std::{collections::BTreeMap, fmt, ops::*};

/// Expression for the delta between two system states. Encoded the
/// delta of every altered account.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StateDiff {
    /// Raw diff key-value
    pub raw: BTreeMap<Address, AccountDiff>,
}

impl StateDiff {
    /// Get the actual data.
    pub fn get(&self) -> &BTreeMap<Address, AccountDiff> {
        &self.raw
    }
}

impl fmt::Display for StateDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (add, acc) in &self.raw {
            write!(f, "{} {}: {}", acc.existance(), add, acc)?;
        }
        Ok(())
    }
}

impl Deref for StateDiff {
    type Target = BTreeMap<Address, AccountDiff>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
