// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Key pair with signing ability

use std::sync::Arc;
use accounts::AccountProvider;
use ethkey::Password;
use parity_crypto::publickey::public_to_address;
use ethereum_types::{H256, Address, Public};
use parity_crypto::publickey::{Signature, Error as EthKeyError};
use ethcore_secretstore::SigningKeyPair;

pub struct KeyStoreNodeKeyPair {
	account_provider: Arc<AccountProvider>,
	address: Address,
	public: Public,
	password: Password,
}

impl KeyStoreNodeKeyPair {
	pub fn new(account_provider: Arc<AccountProvider>, address: Address, password: Password) -> Result<Self, EthKeyError> {
		let public = account_provider.account_public(address.clone(), &password).map_err(|e| EthKeyError::Custom(format!("{}", e)))?;
		Ok(KeyStoreNodeKeyPair {
			account_provider,
			address,
			public,
			password,
		})
	}
}

impl SigningKeyPair for KeyStoreNodeKeyPair {
	fn public(&self) -> &Public {
		&self.public
	}

	fn address(&self) -> Address {
		public_to_address(&self.public)
	}

	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError> {
		self.account_provider.sign(self.address.clone(), Some(self.password.clone()), data.clone())
			.map_err(|e| EthKeyError::Custom(format!("{}", e)))
	}
}
