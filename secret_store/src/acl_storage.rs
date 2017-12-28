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

use std::sync::{Arc, Weak};
use std::collections::{HashMap, HashSet};
use futures::{future, Future};
use parking_lot::{Mutex, RwLock};
use ethkey::public_to_address;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify};
use native_contracts::SecretStoreAclStorage;
use bigint::hash::H256;
use util::Address;
use bytes::Bytes;
use types::all::{Error, ServerKeyId, Public};

const ACL_CHECKER_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_acl_checker";

/// ACL storage of Secret Store
pub trait AclStorage: Send + Sync {
	/// Check if requestor with `public` key can access document with hash `document`
	fn check(&self, public: &Public, document: &ServerKeyId) -> Result<bool, Error>;
}

/// On-chain ACL storage implementation.
pub struct OnChainAclStorage {
	/// Cached on-chain contract.
	contract: Mutex<CachedContract>,
}

/// Cached on-chain ACL storage contract.
struct CachedContract {
	/// Blockchain client.
	client: Weak<Client>,
	/// Contract address.
	contract_addr: Option<Address>,
	/// Contract at given address.
	contract: Option<SecretStoreAclStorage>,
}

/// Dummy ACL storage implementation (check always passed).
#[derive(Default, Debug)]
pub struct DummyAclStorage {
	prohibited: RwLock<HashMap<Public, HashSet<ServerKeyId>>>,
}

impl OnChainAclStorage {
	pub fn new(client: &Arc<Client>) -> Arc<Self> {
		let acl_storage = Arc::new(OnChainAclStorage {
			contract: Mutex::new(CachedContract::new(client)),
		});
		client.add_notify(acl_storage.clone());
		acl_storage
	}
}

impl AclStorage for OnChainAclStorage {
	fn check(&self, public: &Public, document: &ServerKeyId) -> Result<bool, Error> {
		self.contract.lock().check(public, document)
	}
}

impl ChainNotify for OnChainAclStorage {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		if !enacted.is_empty() || !retracted.is_empty() {
			self.contract.lock().update()
		}
	}
}

impl CachedContract {
	pub fn new(client: &Arc<Client>) -> Self {
		CachedContract {
			client: Arc::downgrade(client),
			contract_addr: None,
			contract: None,
		}
	}

	pub fn update(&mut self) {
		if let Some(client) = self.client.upgrade() {
			let new_contract_addr = client.registry_address(ACL_CHECKER_CONTRACT_REGISTRY_NAME.to_owned());
			if self.contract_addr.as_ref() != new_contract_addr.as_ref() {
				self.contract = new_contract_addr.map(|contract_addr| {
					trace!(target: "secretstore", "Configuring for ACL checker contract from {}", contract_addr);

					SecretStoreAclStorage::new(contract_addr)
				});

				self.contract_addr = new_contract_addr;
			}
		}
	}

	pub fn check(&mut self, public: &Public, document: &ServerKeyId) -> Result<bool, Error> {
		match self.contract.as_ref() {
			Some(contract) => {
				let address = public_to_address(&public);
				let do_call = |a, d| future::done(
					self.client
						.upgrade()
						.ok_or_else(|| "Calling contract without client".into())
						.and_then(|c| c.call_contract(BlockId::Latest, a, d)));
				contract.check_permissions(do_call, address, document.clone())
					.map_err(|err| Error::Internal(err))
					.wait()
			},
			None => Err(Error::Internal("ACL checker contract is not configured".to_owned())),
		}
	}
}

impl DummyAclStorage {
	/// Prohibit given requestor access to given documents
	#[cfg(test)]
	pub fn prohibit(&self, public: Public, document: ServerKeyId) {
		self.prohibited.write()
			.entry(public)
			.or_insert_with(Default::default)
			.insert(document);
	}
}

impl AclStorage for DummyAclStorage {
	fn check(&self, public: &Public, document: &ServerKeyId) -> Result<bool, Error> {
		Ok(self.prohibited.read()
			.get(public)
			.map(|docs| !docs.contains(document))
			.unwrap_or(true))
	}
}
