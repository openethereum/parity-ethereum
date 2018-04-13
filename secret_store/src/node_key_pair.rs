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

use std::sync::Arc;
use crypto::ecdh::agree;
use ethkey::{KeyPair, Public, Signature, Error as EthKeyError, sign, public_to_address};
use ethcore::account_provider::AccountProvider;
use ethereum_types::{H256, Address};
use traits::NodeKeyPair;

pub struct PlainNodeKeyPair {
	key_pair: KeyPair,
}

pub struct KeyStoreNodeKeyPair {
	account_provider: Arc<AccountProvider>,
	address: Address,
	public: Public,
	password: String,
}

impl PlainNodeKeyPair {
	pub fn new(key_pair: KeyPair) -> Self {
		PlainNodeKeyPair {
			key_pair: key_pair,
		}
	}
}

impl NodeKeyPair for PlainNodeKeyPair {
	fn public(&self) -> &Public {
		self.key_pair.public()
	}

	fn address(&self) -> Address {
		public_to_address(self.key_pair.public())
	}

	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError> {
		sign(self.key_pair.secret(), data)
	}

	fn compute_shared_key(&self, peer_public: &Public) -> Result<KeyPair, EthKeyError> {
		agree(self.key_pair.secret(), peer_public).map_err(|e| EthKeyError::Custom(e.into()))
			.and_then(KeyPair::from_secret)
	}
}

impl KeyStoreNodeKeyPair {
	pub fn new(account_provider: Arc<AccountProvider>, address: Address, password: String) -> Result<Self, EthKeyError> {
		let public = account_provider.account_public(address.clone(), &password).map_err(|e| EthKeyError::Custom(format!("{}", e)))?;
		Ok(KeyStoreNodeKeyPair {
			account_provider: account_provider,
			address: address,
			public: public,
			password: password,
		})
	}
}

impl NodeKeyPair for KeyStoreNodeKeyPair {
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

	fn compute_shared_key(&self, peer_public: &Public) -> Result<KeyPair, EthKeyError> {
		KeyPair::from_secret(self.account_provider.agree(self.address.clone(), Some(self.password.clone()), peer_public)
			.map_err(|e| EthKeyError::Custom(format!("{}", e)))?)
	}
}
