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
use parking_lot::Mutex;
use ethcore::client::{Client, BlockChainClient};
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
	contract: Mutex<Option<provider::Contract>>,
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

					Some(provider::Contract::new(self.client.clone(), contract_addr))
				})
		}
		if let Some(ref contract) = *contract {
			contract.check_permission(public, document)
				.map_err(|err| Error::Internal(err))
		} else {
			Err(Error::Internal("ACL checker contract is not configured".to_owned()))
		}
	}
}

mod provider {
	use std::sync::Arc;
	use ethcore::client::Client;
	use util;

	pub struct Contract;

	impl Contract {
		pub fn new(_client: Arc<Client>, _contract_addr: util::Address) -> Self {
			unimplemented!()
		}
	}

	impl Contract {
		pub fn check_permission(&self, _public: &util::H512, _document: &util::H256) -> Result<bool, String> {
			unimplemented!()
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::{HashMap, HashSet};
	use parking_lot::RwLock;
	use types::all::{Error, DocumentAddress, Public};

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
