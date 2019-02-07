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

//! Local Accounts checker

use std::collections::HashSet;

use ethereum_types::Address;

/// Local accounts checker
pub trait LocalAccounts: Send + Sync {
	/// Returns true if given address should be considered local account.
	fn is_local(&self, &Address) -> bool;
}

impl LocalAccounts for HashSet<Address> {
	fn is_local(&self, address: &Address) -> bool {
		self.contains(address)
	}
}

impl<A, B> LocalAccounts for (A, B) where
	A: LocalAccounts,
	B: LocalAccounts,
{
	fn is_local(&self, address: &Address) -> bool {
		self.0.is_local(address) || self.1.is_local(address)
	}
}

