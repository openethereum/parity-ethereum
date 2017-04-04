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
use futures::{future, Future};
use parking_lot::Mutex;
use ethkey::public_to_address;
use ethcore::client::{Client, BlockChainClient, BlockId};
use native_contracts::SecretStoreAclStorage;
use types::all::{Error, DocumentAddress, Public};

const ACL_CHECKER_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_acl_checker";

/// ACL storage of Secret Store
pub trait AclStorage: Send + Sync {
	/// Check if requestor with `public` key can access document with hash `document`
	fn check(&self, public: &Public, document: &DocumentAddress) -> Result<bool, Error>;
}

/// On-chain ACL storage implementation.
pub struct OnChainAclStorage {
	/// Blockchain client.
	client: Arc<Client>,
	/// On-chain contract.
	contract: Mutex<Option<SecretStoreAclStorage>>,
}

impl OnChainAclStorage {
	pub fn new(client: Arc<Client>) -> Self {
		OnChainAclStorage {
			client: client,
			contract: Mutex::new(None),
		}
	}
}

impl AclStorage for OnChainAclStorage {
	fn check(&self, public: &Public, document: &DocumentAddress) -> Result<bool, Error> {
		let mut contract = self.contract.lock();
		if !contract.is_some() {
			*contract = self.client.registry_address(ACL_CHECKER_CONTRACT_REGISTRY_NAME.to_owned())
				.and_then(|contract_addr| {
					trace!(target: "secretstore", "Configuring for ACL checker contract from {}", contract_addr);

					Some(SecretStoreAclStorage::new(contract_addr))
				})
		}
		if let Some(ref contract) = *contract {
			let address = public_to_address(&public);
			let do_call = |a, d| future::done(self.client.call_contract(BlockId::Latest, a, d));
			contract.check_permissions(do_call, address, document.clone())
				.map_err(|err| Error::Internal(err))
				.wait()
		} else {
			Err(Error::Internal("ACL checker contract is not configured".to_owned()))
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::{HashMap, HashSet};
	use parking_lot::RwLock;
	use types::all::{Error, DocumentAddress, Public};
	use super::AclStorage;

	#[derive(Default, Debug)]
	/// Dummy ACL storage implementation
	pub struct DummyAclStorage {
		prohibited: RwLock<HashMap<Public, HashSet<DocumentAddress>>>,
	}

	impl DummyAclStorage {
		#[cfg(test)]
		/// Prohibit given requestor access to given document
		pub fn prohibit(&self, public: Public, document: DocumentAddress) {
			self.prohibited.write()
				.entry(public)
				.or_insert_with(Default::default)
				.insert(document);
		}
	}

	impl AclStorage for DummyAclStorage {
		fn check(&self, public: &Public, document: &DocumentAddress) -> Result<bool, Error> {
			Ok(self.prohibited.read()
				.get(public)
				.map(|docs| !docs.contains(document))
				.unwrap_or(true))
		}
	}
}
