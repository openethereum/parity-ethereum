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

//! Wrapper around key server responsible for access keys processing.

use std::sync::Arc;
use parking_lot::RwLock;
use ethereum_types::{H256, Address};
use call_contract::CallContract;
use registrar::RegistrarClient;
use types::ids::BlockId;
use ethabi::FunctionOutputDecoder;

const ACL_CHECKER_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_acl_checker";

use_contract!(keys_acl_contract, "res/keys_acl.json");

/// Returns the address (of the contract), that corresponds to the key
pub fn key_to_address(key: &H256) -> Address {
	Address::from_slice(&key.as_bytes()[..10])
}

/// Returns the key from the key server associated with the contract
pub fn address_to_key(contract_address: &Address) -> H256 {
	// Current solution uses contract address extended with 0 as id
	let contract_address_extended: H256 = (*contract_address).into();

	H256::from_slice(contract_address_extended.as_bytes())
}

/// Trait for keys server keys provider.
pub trait KeyProvider: Send + Sync + 'static {
	/// Account, that is used for communication with key server
	fn key_server_account(&self) -> Option<Address>;

	/// List of keys available for the account
	fn available_keys(&self, block: BlockId, account: &Address) -> Option<Vec<Address>>;

	/// Update permissioning contract
	fn update_acl_contract(&self);
}

/// Secret Store keys provider
pub struct SecretStoreKeys<C> where C: CallContract + RegistrarClient + Send + Sync + 'static {
	client: Arc<C>,
	key_server_account: Option<Address>,
	keys_acl_contract: RwLock<Option<Address>>,
}

impl<C> SecretStoreKeys<C> where C: CallContract + RegistrarClient + Send + Sync + 'static {
	/// Create provider
	pub fn new(client: Arc<C>, key_server_account: Option<Address>) -> Self {
		SecretStoreKeys {
			client,
			key_server_account,
			keys_acl_contract: RwLock::new(None),
		}
	}
}

impl<C> KeyProvider for SecretStoreKeys<C>
	where C: CallContract + RegistrarClient + Send + Sync + 'static
{
	fn key_server_account(&self) -> Option<Address> {
		self.key_server_account
	}

	fn available_keys(&self, block: BlockId, account: &Address) -> Option<Vec<Address>> {
		match *self.keys_acl_contract.read() {
			Some(acl_contract_address) => {
				let (data, decoder) = keys_acl_contract::functions::available_keys::call(*account);
				if let Ok(value) = self.client.call_contract(block, acl_contract_address, data) {
					decoder.decode(&value).ok().map(|key_values| {
						key_values.iter().map(key_to_address).collect()
					})
				} else {
					None
				}
			}
			None => None,
		}
	}

	fn update_acl_contract(&self) {
		let contract_address = self.client.get_address(
			ACL_CHECKER_CONTRACT_REGISTRY_NAME,
			BlockId::Latest
		).unwrap_or(None);

		if *self.keys_acl_contract.read() != contract_address {
			trace!(target: "privatetx", "Configuring for ACL checker contract from address {:?}",
				contract_address);
			*self.keys_acl_contract.write() = contract_address;
		}
	}
}

/// Dummy keys provider.
pub struct StoringKeyProvider {
	available_keys: RwLock<Option<Vec<Address>>>,
	key_server_account: Option<Address>,
}

impl StoringKeyProvider {
	/// Store available keys
	pub fn set_available_keys(&self, keys: &Vec<Address>) {
		*self.available_keys.write() = Some(keys.clone())
	}
}

impl Default for StoringKeyProvider {
	fn default() -> Self {
		StoringKeyProvider {
			available_keys: RwLock::new(None),
			key_server_account: Some(Address::zero()),
		}
	}
}

impl KeyProvider for StoringKeyProvider {
	fn key_server_account(&self) -> Option<Address> {
		self.key_server_account
	}

	fn available_keys(&self, _block: BlockId, _account: &Address) -> Option<Vec<Address>> {
		self.available_keys.read().clone()
	}

	fn update_acl_contract(&self) {}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use crypto::publickey::{Secret, KeyPair};
	use bytes::Bytes;
	use super::*;
	use registrar::RegistrarClient;

	struct DummyRegistryClient {
		registry_address: Option<Address>,
	}

	impl DummyRegistryClient {
		pub fn new(registry_address: Option<Address>) -> Self {
			DummyRegistryClient {
				registry_address
			}
		}
	}

	impl RegistrarClient for DummyRegistryClient {
		fn registrar_address(&self) -> Option<Address> {
			unimplemented!()
		}

		fn get_address(&self, _name: &str, _block: BlockId) -> Result<Option<Address>, String> {
			Ok(self.registry_address)
		}
	}

	impl CallContract for DummyRegistryClient {
		fn call_contract(
			&self,
			_block_id: BlockId,
			_address: Address,
			_data: Bytes
		) -> Result<Bytes, String> {
			Ok(vec![])
		}
	}

	#[test]
	fn should_update_acl_contract() {
		let key = KeyPair::from_secret(Secret::from_str("0000000000000000000000000000000000000000000000000000000000000011").unwrap()).unwrap();
		let client = DummyRegistryClient::new(Some(key.address()));
		let keys_data = SecretStoreKeys::new(Arc::new(client), None);
		keys_data.update_acl_contract();
		assert_eq!(keys_data.keys_acl_contract.read().unwrap(), key.address());
	}
}
