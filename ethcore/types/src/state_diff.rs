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

//! State diff module.

use std::collections::BTreeMap;
use account_diff::AccountDiff;
use ethereum_types::Address;

/// Expression for the delta between two system states. Encoded the
/// delta of every altered account.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StateDiff {
	/// Raw diff key-value
	pub raw: BTreeMap<Address, AccountDiff>
}
