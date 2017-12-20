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
use parking_lot::RwLock;
use ethcore::client::{Client, BlockChainClient, BlockId};
use ethkey::{Public, Signature, public_to_address};
use native_contracts::SecretStoreService;
use hash::keccak;
use bigint::hash::H256;
use bigint::prelude::U256;
use listener::service_contract_listener::ServiceTask;
use trusted_client::TrustedClient;
use {ServerKeyId, NodeKeyPair, ContractAddress};

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Number of confirmations required before request can be processed.
const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

/// Service contract trait.
pub trait ServiceContract: Send + Sync {
	/// Update contract.
	fn update(&self);
	/// Is contract installed && up-to-date (i.e. chain is synced)?
	fn is_actual(&self) -> bool;
	/// Publish generated key.
	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>>;
	/// Publish server key.
	fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String>;
}

/// On-chain service contract.
pub struct OnChainServiceContract {
	/// Blockchain client.
	client: TrustedClient,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Contract addresss.
	address: ContractAddress,
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
	/// Block, this iterator is created for.
	block: H256,
	/// Current request index.
	index: U256,
	/// Requests length.
	length: U256,
}

impl OnChainServiceContract {
	/// Create new on-chain service contract.
	pub fn new(client: TrustedClient, address: ContractAddress, self_key_pair: Arc<NodeKeyPair>) -> Self {
		let contract_addr = match address {
			ContractAddress::Registry => client.get().and_then(|c| c.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned())
				.map(|address| {
					trace!(target: "secretstore", "{}: installing service contract from address {}",
						self_key_pair.public(), address);
					address
				}))
				.unwrap_or_default(),
			ContractAddress::Address(ref address) => {
				trace!(target: "secretstore", "{}: installing service contract from address {}",
					self_key_pair.public(), address);
				address.clone()
			},
		};

		OnChainServiceContract {
			client: client,
			self_key_pair: self_key_pair,
			address: address,
			contract: RwLock::new(Arc::new(SecretStoreService::new(contract_addr))),
		}
	}
}

impl ServiceContract for OnChainServiceContract {
	fn update(&self) {
		if let &ContractAddress::Registry = &self.address {
			if let Some(client) = self.client.get() {
				// update contract address from registry
				let service_contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()).unwrap_or_default();
				if self.contract.read().address != service_contract_addr {
					trace!(target: "secretstore", "{}: installing service contract from address {}",
						self.self_key_pair.public(), service_contract_addr);
					*self.contract.write() = Arc::new(SecretStoreService::new(service_contract_addr));
				}
			}
		}
	}

	fn is_actual(&self) -> bool {
		self.contract.read().address != Default::default()
			&& self.client.get().is_some()
	}

	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
		let client = match self.client.get() {
			Some(client) => client,
			None => {
				warn!(target: "secretstore", "{}: client is untrusted during read_pending_requests call",
					self.self_key_pair.public());
				return Box::new(::std::iter::empty());
			},
		};

		let contract = self.contract.read();
		match contract.address == Default::default() {
			true => Box::new(::std::iter::empty()),
			false => client.block_number(BlockId::Latest)
				.and_then(|b| b.checked_sub(REQUEST_CONFIRMATIONS_REQUIRED))
				.and_then(|b| client.block_hash(BlockId::Number(b)))
				.and_then(|b| {
					let do_call = |a, d| future::done(client.call_contract(BlockId::Hash(b.clone()), a, d));
					contract.server_key_generation_requests_count(&do_call).wait()
						.map_err(|error| {
							warn!(target: "secretstore", "{}: call to server_key_generation_requests_count failed: {}",
								self.self_key_pair.public(), error);
							error
						})
						.map(|l| (b, l))
						.ok()
				})
				.map(|(b, l)| Box::new(PendingRequestsIterator {
					client: client,
					contract: contract.clone(),
					self_key_pair: self.self_key_pair.clone(),
					block: b,
					index: 0.into(),
					length: l,
				}) as Box<Iterator<Item=(bool, ServiceTask)>>)
				.unwrap_or_else(|| Box::new(::std::iter::empty()))
		}
	}

	fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
		// only publish if contract address is set && client is online
		let contract = self.contract.read();
		if contract.address == Default::default() {
			// it is not an error, because key could be generated even without contract
			return Ok(());
		}

		let client = match self.client.get() {
			Some(client) => client,
			None => return Err("trusted client is required to publish key".into()),
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
		client.transact_contract(
			contract.address.clone(),
			transaction_data
		).map_err(|e| format!("{}", e))?;

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
		let do_call = |a, d| future::done(self.client.call_contract(BlockId::Hash(self.block.clone()), a, d));
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

		fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
			Box::new(self.pending_requests.clone().into_iter())
		}

		fn publish_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
			self.published_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}
	}
}
