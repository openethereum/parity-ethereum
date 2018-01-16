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

//! Dapps settings de/serialization.

use hash;

/// Settings for specific dapp.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DappsSettings {
	/// A list of accounts this Dapp can see.
	pub accounts: Option<Vec<hash::Address>>,
	/// Default account
	pub default: Option<hash::Address>,
}

impl_serialization!(String => DappsSettings);

/// History for specific dapp.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DappsHistory {
	/// Last accessed timestamp
	pub last_accessed: u64,
}

impl_serialization!(String => DappsHistory);

/// Accounts policy for new dapps.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NewDappsPolicy {
	/// All accounts are exposed by default.
	AllAccounts {
		/// Default account, which should be returned as the first one.
		default: hash::Address,
	},
	/// Only accounts listed here are exposed by default for new dapps.
	Whitelist(Vec<hash::Address>),
}

impl_serialization!(String => NewDappsPolicy);
