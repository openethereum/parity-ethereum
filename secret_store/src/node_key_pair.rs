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
use ethcrypto::ecdh::agree;
use ethkey::{KeyPair, Public, Signature, Error as EthKeyError, sign};
use ethcore::account_provider::AccountProvider;
use util::H256;
use traits::NodeKeyPair;

pub struct PlainNodeKeyPair {
	key_pair: KeyPair,
}

pub struct KeyStoreNodeKeyPair {
	_account_provider: Arc<AccountProvider>,
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

	fn sign(&self, data: &H256) -> Result<Signature, EthKeyError> {
		sign(self.key_pair.secret(), data)
	}

	fn compute_shared_key(&self, peer_public: &Public) -> Result<KeyPair, EthKeyError> {
		agree(self.key_pair.secret(), peer_public).map_err(|e| EthKeyError::Custom(e.into()))
			.and_then(KeyPair::from_secret)
	}
}

impl NodeKeyPair for KeyStoreNodeKeyPair {
	fn public(&self) -> &Public {
		unimplemented!()
	}

	fn sign(&self, _data: &H256) -> Result<Signature, EthKeyError> {
		unimplemented!()
	}

	fn compute_shared_key(&self, _peer_public: &Public) -> Result<KeyPair, EthKeyError> {
		unimplemented!()
	}
}
