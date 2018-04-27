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
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use ethcore::client::{BlockId, ChainNotify, CallContract, RegistryInfo};
use ethereum_types::{H256, Address};
use bytes::Bytes;
use trusted_client::TrustedClient;
use types::all::{Error, ServerKeyId};

use_contract!(acl_storage, "AclStorage", "res/acl_storage.json");

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
	/// Contract address.
	contract_addr: Option<Address>,
	/// Contract at given address.
	contract: acl_storage::AclStorage,
}

/// Dummy ACL storage implementation (check always passed).
#[derive(Default, Debug)]
pub struct DummyAclStorage {
	prohibited: RwLock<HashMap<Address, HashSet<ServerKeyId>>>,
}

impl OnChainAclStorage {
	pub fn new(trusted_client: TrustedClient) -> Result<Arc<Self>, Error> {
		let client = trusted_client.get_untrusted();
		let acl_storage = Arc::new(OnChainAclStorage {
			contract: Mutex::new(CachedContract::new(trusted_client)),
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
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: Duration) {
		if !enacted.is_empty() || !retracted.is_empty() {
			self.contract.lock().update()
		}
	}
}

impl CachedContract {
	pub fn new(client: TrustedClient) -> Self {
		CachedContract {
			client,
			contract_addr: None,
			contract: acl_storage::AclStorage::default(),
		}
	}

	pub fn update(&mut self) {
		if let Some(client) = self.client.get() {
			match client.registry_address(ACL_CHECKER_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest) {
				Some(new_contract_addr) if Some(new_contract_addr).as_ref() != self.contract_addr.as_ref() => {
					trace!(target: "secretstore", "Configuring for ACL checker contract from {}", new_contract_addr);
					self.contract_addr = Some(new_contract_addr);
				},
				Some(_) | None => ()
			}
		}
	}

	pub fn check(&mut self, requester: Address, document: &ServerKeyId) -> Result<bool, Error> {
		if let Some(client) = self.client.get() {
			// call contract to check accesss
			match self.contract_addr {
				Some(contract_address) => {
					let do_call = |data| client.call_contract(BlockId::Latest, contract_address, data);
					self.contract.functions()
						.check_permissions()
						.call(requester, document.clone(), &do_call)
						.map_err(|e| Error::Internal(e.to_string()))
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
