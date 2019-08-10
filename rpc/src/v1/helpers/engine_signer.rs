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

use accounts::AccountProvider;
use ethkey::{self, Address, Password};

/// An implementation of EngineSigner using internal account management.
pub struct EngineSigner {
	accounts: Arc<AccountProvider>,
	address: Address,
	password: Password,
}

impl EngineSigner {
	/// Creates new `EngineSigner` given account manager and account details.
	pub fn new(accounts: Arc<AccountProvider>, address: Address, password: Password) -> Self {
		EngineSigner { accounts, address, password }
	}
}

impl engine::signer::EngineSigner for EngineSigner {
	fn sign(&self, message: ethkey::Message) -> Result<ethkey::Signature, ethkey::Error> {
		match self.accounts.sign(self.address, Some(self.password.clone()), message) {
			Ok(ok) => Ok(ok),
			Err(_) => Err(ethkey::Error::InvalidSecret),
		}
	}

	fn address(&self) -> Address {
		self.address
	}
}

