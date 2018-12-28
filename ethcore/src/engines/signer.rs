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

//! A signer used by Engines which need to sign messages.

use std::sync::Arc;
use ethereum_types::{H256, Address};
use ethkey::{Password, Signature};
use account_provider::{self, AccountProvider};

/// Everything that an Engine needs to sign messages.
pub struct EngineSigner {
	account_provider: Arc<AccountProvider>,
	address: Option<Address>,
	password: Option<Password>,
}

impl Default for EngineSigner {
	fn default() -> Self {
		EngineSigner {
			account_provider: Arc::new(AccountProvider::transient_provider()),
			address: Default::default(),
			password: Default::default(),
		}
	}
}

impl EngineSigner {
	/// Set up the signer to sign with given address and password.
	pub fn set(&mut self, ap: Arc<AccountProvider>, address: Address, password: Password) {
		self.account_provider = ap;
		self.address = Some(address);
		self.password = Some(password);
		debug!(target: "poa", "Setting Engine signer to {}", address);
	}

	/// Sign a consensus message hash.
	pub fn sign(&self, hash: H256) -> Result<Signature, account_provider::SignError> {
		self.account_provider.sign(self.address.unwrap_or_else(Default::default), self.password.clone(), hash)
	}

	/// Signing address.
	pub fn address(&self) -> Option<Address> {
		self.address.clone()
	}

	/// Check if the signing address was set.
	pub fn is_some(&self) -> bool {
		self.address.is_some()
	}
}
