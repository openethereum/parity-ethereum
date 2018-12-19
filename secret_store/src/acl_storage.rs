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
use std::collections::{HashMap, HashSet};
use parking_lot::{Mutex, RwLock};
use ethcore::client::{BlockId, ChainNotify, NewBlocks, CallContract};
use ethereum_types::Address;
use ethabi::FunctionOutputDecoder;
use trusted_client::TrustedClient;
use types::{Error, ServerKeyId, ContractAddress};

use_contract!(acl_storage, "res/acl_storage.json");

const ACL_CHECKER_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_acl_checker";

/// ACL storage of Secret Store
pub trait AclStorage: Send + Sync {
	/// Check if requestor can access document with hash `document`
	fn check(&self, requester: Address, document: &ServerKeyId) -> Result<bool, Error>;
}

/// On-chain ACL storage implementation.
pub struct OnChainAclStorage {
	/// Cached on-chain contract.
	contract: Mutex<CachedContract>,
}

/// Cached on-chain ACL storage contract.
struct CachedContract {
	/// Blockchain client.
	client: TrustedClient,
	/// Contract address source.
	address_source: ContractAddress,
	/// Current contract address.
	contract_address: Option<Address>,
}

/// Dummy ACL storage implementation (check always passed).
#[derive(Default, Debug)]
pub struct DummyAclStorage {
	prohibited: RwLock<HashMap<Address, HashSet<ServerKeyId>>>,
}

impl OnChainAclStorage {
	pub fn new(trusted_client: TrustedClient, address_source: ContractAddress) -> Result<Arc<Self>, Error> {
		let client = trusted_client.get_untrusted();
		let acl_storage = Arc::new(OnChainAclStorage {
			contract: Mutex::new(CachedContract::new(trusted_client, address_source)),
		});
		client
			.ok_or_else(|| Error::Internal("Constructing OnChainAclStorage without active Client".into()))?
			.add_notify(acl_storage.clone());
		Ok(acl_storage)
	}
}

impl AclStorage for OnChainAclStorage {
	fn check(&self, requester: Address, document: &ServerKeyId) -> Result<bool, Error> {
		self.contract.lock().check(requester, document)
	}
}

impl ChainNotify for OnChainAclStorage {
	fn new_blocks(&self, new_blocks: NewBlocks) {
		if new_blocks.has_more_blocks_to_import { return }
		if !new_blocks.route.enacted().is_empty() || !new_blocks.route.retracted().is_empty() {
			self.contract.lock().update_contract_address()
		}
	}
}

impl CachedContract {
	pub fn new(client: TrustedClient, address_source: ContractAddress) -> Self {
		let mut contract = CachedContract {
			client,
			address_source,
			contract_address: None,
		};
		contract.update_contract_address();
		contract
	}

	pub fn update_contract_address(&mut self) {
		let contract_address = self.client.read_contract_address(ACL_CHECKER_CONTRACT_REGISTRY_NAME.into(), &self.address_source);
		if contract_address != self.contract_address {
			trace!(target: "secretstore", "Configuring for ACL checker contract from address {:?}",
				contract_address);

			self.contract_address = contract_address;
		}
	}

	pub fn check(&mut self, requester: Address, document: &ServerKeyId) -> Result<bool, Error> {
		if let Some(client) = self.client.get() {
			// call contract to check accesss
			match self.contract_address {
				Some(contract_address) => {
					let (encoded, decoder) = acl_storage::functions::check_permissions::call(requester, document.clone());
					let d = client.call_contract(BlockId::Latest, contract_address, encoded)
						.map_err(|e| Error::Internal(format!("ACL checker call error: {}", e.to_string())))?;
					decoder.decode(&d)
						.map_err(|e| Error::Internal(format!("ACL checker call error: {}", e.to_string())))
				},
				None => Err(Error::Internal("ACL checker contract is not configured".to_owned())),
			}
		} else {
			Err(Error::Internal("Calling ACL contract without trusted blockchain client".into()))
		}
	}
}

impl DummyAclStorage {
	/// Prohibit given requestor access to given documents
	#[cfg(test)]
	pub fn prohibit(&self, requester: Address, document: ServerKeyId) {
		self.prohibited.write()
			.entry(requester)
			.or_insert_with(Default::default)
			.insert(document);
	}
}

impl AclStorage for DummyAclStorage {
	fn check(&self, requester: Address, document: &ServerKeyId) -> Result<bool, Error> {
		Ok(self.prohibited.read()
			.get(&requester)
			.map(|docs| !docs.contains(document))
			.unwrap_or(true))
	}
}
