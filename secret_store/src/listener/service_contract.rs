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
use futures::{future, Future};
use parking_lot::RwLock;
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId};
use ethkey::{Public, Signature, public_to_address};
use ethsync::SyncProvider;
use native_contracts::SecretStoreService;
use hash::keccak;
use bigint::hash::H256;
use bigint::prelude::U256;
use listener::service_contract_listener::ServiceTask;
use {ServerKeyId, NodeKeyPair};

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Key server has been added to the set.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32,uint256)";

lazy_static! {
	static ref SERVER_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_REQUESTED_EVENT_NAME);
}

/// Service contract trait.
pub trait ServiceContract: Send + Sync {
	/// Update contract.
	fn update(&self);
	/// Is contract installed && up-to-date (i.e. chain is synced)?
	fn is_actual(&self) -> bool;
	/// Read contract logs from given blocks. Returns topics of every entry.
	fn read_logs(&self, first_block: H256, last_block: H256) -> Box<Iterator<Item=Vec<H256>>>;
	/// Publish generated key.
	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>>;
	/// Publish server key.
	fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String>;
}

/// On-chain service contract.
pub struct OnChainServiceContract {
	/// Blockchain client.
	client: Weak<Client>,
	/// Sync provider.
	sync: Weak<SyncProvider>,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Contract.
	contract: RwLock<Arc<SecretStoreService>>,
}

/// Pending requests iterator.
struct PendingRequestsIterator {
	/// Blockchain client.
	client: Arc<Client>,
	/// Contract.
	contract: Arc<SecretStoreService>,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Current request index.
	index: U256,
	/// Requests length.
	length: U256,
}

impl OnChainServiceContract {
	/// Create new on-chain service contract.
	pub fn new(client: &Arc<Client>, sync: &Arc<SyncProvider>, self_key_pair: Arc<NodeKeyPair>) -> Self {
		let contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned())
			.map(|address| {
				trace!(target: "secretstore", "{}: installing service contract from address {}",
					self_key_pair.public(), address);
				address
			})
			.unwrap_or_default();

		OnChainServiceContract {
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			self_key_pair: self_key_pair,
			contract: RwLock::new(Arc::new(SecretStoreService::new(contract_addr))),
		}
	}
}

impl ServiceContract for OnChainServiceContract {
	fn update(&self) {
		if let (Some(client), Some(sync)) = (self.client.upgrade(), self.sync.upgrade()) {
			// do nothing until synced
			if sync.status().is_syncing(client.queue_info()) {
				return;
			}

			// update contract address from registry
			let service_contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()).unwrap_or_default();
			if self.contract.read().address != service_contract_addr {
				trace!(target: "secretstore", "{}: installing service contract from address {}",
					self.self_key_pair.public(), service_contract_addr);
				*self.contract.write() = Arc::new(SecretStoreService::new(service_contract_addr));
			}
		}
	}

	fn is_actual(&self) -> bool {
		self.contract.read().address != Default::default()
			&& match (self.client.upgrade(), self.sync.upgrade()) {
				(Some(client), Some(sync)) => !sync.status().is_syncing(client.queue_info()),
				_ => false,
			}
	}

	fn read_logs(&self, first_block: H256, last_block: H256) -> Box<Iterator<Item=Vec<H256>>> {
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => {
				warn!(target: "secretstore", "{}: client is offline during read_pending_requests call",
					self.self_key_pair.public());
				return Box::new(::std::iter::empty());
			},
		};

		// read server key generation requests
		let contract_address = self.contract.read().address.clone();
		let request_logs = client.logs(Filter {
			from_block: BlockId::Hash(first_block),
			to_block: BlockId::Hash(last_block),
			address: Some(vec![contract_address]),
			topics: vec![
				Some(vec![*SERVER_KEY_REQUESTED_EVENT_NAME_HASH]),
				None,
				None,
				None,
			],
			limit: None,
		});

		Box::new(request_logs.into_iter().map(|log| log.entry.topics))
	}

	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => {
				warn!(target: "secretstore", "{}: client is offline during read_pending_requests call",
					self.self_key_pair.public());
				return Box::new(::std::iter::empty());
			},
		};

		let contract = self.contract.read();
		let length = match contract.address == Default::default() {
			true => 0.into(),
			false => {
				let do_call = |a, d| future::done(client.call_contract(BlockId::Latest, a, d));
				contract.server_key_generation_requests_count(&do_call).wait()
					.map_err(|error| {
						warn!(target: "secretstore", "{}: call to server_key_generation_requests_count failed: {}",
							self.self_key_pair.public(), error);
						error
					})
					.unwrap_or_default()
			},
		};

		Box::new(PendingRequestsIterator {
			client: client,
			contract: contract.clone(),
			self_key_pair: self.self_key_pair.clone(),
			index: 0.into(),
			length: length,
		})
	}

	fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
		// only publish if contract address is set && client is online
		let contract = self.contract.read();
		if contract.address == Default::default() {
			// it is not an error, because key could be generated even without contract
			return Ok(());
		}
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return Err("client is required to publish key".into()),
		};

		// only publish key if contract waits for publication
		// failing is ok here - it could be that enough confirmations have been recevied
		// or key has been requested using HTTP API
		let do_call = |a, d| future::done(client.call_contract(BlockId::Latest, a, d));
		let self_address = public_to_address(self.self_key_pair.public());
		if contract.get_server_key_confirmation_status(&do_call, server_key_id.clone(), self_address).wait().unwrap_or(false) {
			return Ok(());
		}

		// prepare transaction data
		let server_key_hash = keccak(server_key);
		let signed_server_key = self.self_key_pair.sign(&server_key_hash).map_err(|e| format!("{}", e))?;
		let signed_server_key: Signature = signed_server_key.into_electrum().into();
		let transaction_data = contract.encode_server_key_generated_input(server_key_id.clone(),
			server_key.to_vec(),
			signed_server_key.v(),
			signed_server_key.r().into(),
			signed_server_key.s().into()
		)?;

		// send transaction
		if contract.address != Default::default() {
			client.transact_contract(
				contract.address.clone(),
				transaction_data
			).map_err(|e| format!("{}", e))?;
		}

		Ok(())
	}
}

impl Iterator for PendingRequestsIterator {
	type Item = (bool, ServiceTask);

	fn next(&mut self) -> Option<(bool, ServiceTask)> {
		if self.index >= self.length {
			return None;
		}

		let index = self.index.clone();
		self.index = self.index + 1.into();

		let self_address = public_to_address(self.self_key_pair.public());
		let do_call = |a, d| future::done(self.client.call_contract(BlockId::Latest, a, d));
		self.contract.get_server_key_id(&do_call, index).wait()
			.and_then(|server_key_id|
				self.contract.get_server_key_threshold(&do_call, server_key_id.clone()).wait()
					.map(|threshold| (server_key_id, threshold)))
			.and_then(|(server_key_id, threshold)|
				self.contract.get_server_key_confirmation_status(&do_call, server_key_id.clone(), self_address).wait()
					.map(|is_confirmed| (server_key_id, threshold, is_confirmed)))
			.map(|(server_key_id, threshold, is_confirmed)|
				Some((is_confirmed, ServiceTask::GenerateServerKey(server_key_id, threshold.into()))))
			.map_err(|error| {
				warn!(target: "secretstore", "{}: reading service contract request failed: {}",
					self.self_key_pair.public(), error);
				()
			})
			.unwrap_or(None)
	}
}

#[cfg(test)]
pub mod tests {
	use parking_lot::Mutex;	
	use ethkey::Public;
	use bigint::hash::H256;
	use listener::service_contract_listener::ServiceTask;
	use ServerKeyId;
	use super::ServiceContract;

	#[derive(Default)]
	pub struct DummyServiceContract {
		pub is_actual: bool,
		pub logs: Vec<Vec<H256>>,
		pub pending_requests: Vec<(bool, ServiceTask)>,
		pub published_keys: Mutex<Vec<(ServerKeyId, Public)>>,
	}

	impl ServiceContract for DummyServiceContract {
		fn update(&self) {
		}

		fn is_actual(&self) -> bool {
			self.is_actual
		}

		fn read_logs(&self, _first_block: H256, _last_block: H256) -> Box<Iterator<Item=Vec<H256>>> {
			Box::new(self.logs.clone().into_iter())
		}

		fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
			Box::new(self.pending_requests.clone().into_iter())
		}

		fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
			self.published_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}
	}
}
