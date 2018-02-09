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
use parking_lot::RwLock;
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId};
use ethkey::{Public, Signature, public_to_address};
use hash::keccak;
use ethereum_types::{H256, U256, Address};
use listener::service_contract_listener::ServiceTask;
use trusted_client::TrustedClient;
use {ServerKeyId, NodeKeyPair, ContractAddress};

use_contract!(service, "Service", "res/service.json");

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Key server has been added to the set.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32,uint256)";

/// Number of confirmations required before request can be processed.
const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

lazy_static! {
	static ref SERVER_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_REQUESTED_EVENT_NAME);
}

/// Service contract trait.
pub trait ServiceContract: Send + Sync {
	/// Update contract when new blocks are enacted. Returns true if contract is installed && up-to-date (i.e. chain is synced).
	fn update(&self) -> bool;
	/// Read recent contract logs. Returns topics of every entry.
	fn read_logs(&self) -> Box<Iterator<Item=Vec<H256>>>;
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
	data: RwLock<ServiceData>,
}

/// On-chain service contract data.
struct ServiceData {
	/// Contract.
	pub contract: service::Service,
	/// Contract address.
	pub contract_address: Address,
	/// Last block we have read logs from.
	pub last_log_block: Option<H256>,
}

/// Pending requests iterator.
struct PendingRequestsIterator {
	/// Blockchain client.
	client: Arc<Client>,
	/// Contract.
	contract: service::Service,
	/// Contract address.
	contract_address: Address,
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
			data: RwLock::new(ServiceData {
				contract: service::Service::default(),
				contract_address: contract_addr,
				last_log_block: None,
			}),
		}
	}
}

impl ServiceContract for OnChainServiceContract {
	fn update(&self) -> bool {
		// TODO [Sec]: registry_address currently reads from BlockId::Latest, instead of
		// from block with REQUEST_CONFIRMATIONS_REQUIRED confirmations
		if let &ContractAddress::Registry = &self.address {
			if let Some(client) = self.client.get() {
				// update contract address from registry
				let service_contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()).unwrap_or_default();
				if self.data.read().contract_address != service_contract_addr {
					trace!(target: "secretstore", "{}: installing service contract from address {}",
						self.self_key_pair.public(), service_contract_addr);
					self.data.write().contract_address = service_contract_addr;
				}
			}
		}

		self.data.read().contract_address != Default::default()
			&& self.client.get().is_some()
	}

	fn read_logs(&self) -> Box<Iterator<Item=Vec<H256>>> {
		let client = match self.client.get() {
			Some(client) => client,
			None => {
				warn!(target: "secretstore", "{}: client is offline during read_logs call",
					self.self_key_pair.public());
				return Box::new(::std::iter::empty());
			},
		};

		// prepare range of blocks to read logs from
		let (address, first_block, last_block) = {
			let mut data = self.data.write();
			let address = data.contract_address;
			let confirmed_block = match get_confirmed_block_hash(&*client, REQUEST_CONFIRMATIONS_REQUIRED) {
				Some(confirmed_block) => confirmed_block,
				None => return Box::new(::std::iter::empty()), // no block with enough confirmations
			};
			let first_block = match data.last_log_block.take().and_then(|b| client.tree_route(&b, &confirmed_block)) {
				// if we have a route from last_log_block to confirmed_block => search for logs on this route
				//
				// potentially this could lead us to reading same logs twice when reorganizing to the fork, which
				// already has been canonical previosuly
				// the worst thing that can happen in this case is spending some time reading unneeded data from SS db
				Some(ref route) if route.index < route.blocks.len() => route.blocks[route.index],
				// else we care only about confirmed block
				_ => confirmed_block.clone(),
			};

			data.last_log_block = Some(confirmed_block.clone());
			(address, first_block, confirmed_block)
		};

		// read server key generation requests
		let request_logs = client.logs(Filter {
			from_block: BlockId::Hash(first_block),
			to_block: BlockId::Hash(last_block),
			address: Some(vec![address]),
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
		let client = match self.client.get() {
			Some(client) => client,
			None => return Box::new(::std::iter::empty()),
		};

		// we only need requests that are here for more than REQUEST_CONFIRMATIONS_REQUIRED blocks
		// => we're reading from Latest - (REQUEST_CONFIRMATIONS_REQUIRED + 1) block
		let data = self.data.read();
		match data.contract_address == Default::default() {
			true => Box::new(::std::iter::empty()),
			false => get_confirmed_block_hash(&*client, REQUEST_CONFIRMATIONS_REQUIRED + 1)
				.and_then(|b| {
					let contract_address = data.contract_address;
					let do_call = |data| client.call_contract(BlockId::Hash(b), contract_address, data);
					data.contract.functions().server_key_generation_requests_count().call(&do_call)
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
					contract: service::Service::default(),
					contract_address: data.contract_address,
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
		let data = self.data.read();
		if data.contract_address == Default::default() {
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
		let contract_address = data.contract_address;
		let do_call = |data| client.call_contract(BlockId::Latest, contract_address, data);
		let self_address = public_to_address(self.self_key_pair.public());
		if data.contract.functions()
			.get_server_key_confirmation_status()
			.call(*server_key_id, self_address, &do_call)
			.unwrap_or(false) {
			return Ok(());
		}

		// prepare transaction data
		let server_key_hash = keccak(server_key);
		let signed_server_key = self.self_key_pair.sign(&server_key_hash).map_err(|e| format!("{}", e))?;
		let signed_server_key: Signature = signed_server_key.into_electrum().into();
		let transaction_data = data.contract.functions()
			.server_key_generated()
			.input(*server_key_id,
				server_key.to_vec(),
				signed_server_key.v(),
				signed_server_key.r(),
				signed_server_key.s(),
			);

		// send transaction
		client.transact_contract(
			data.contract_address,
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
		let contract_address = self.contract_address;
		let do_call = |data| self.client.call_contract(BlockId::Hash(self.block.clone()), contract_address, data);
		self.contract.functions().get_server_key_id().call(index, &do_call)
			.and_then(|server_key_id|
				self.contract.functions().get_server_key_threshold().call(server_key_id, &do_call)
					.map(|threshold| (server_key_id, threshold)))
			.and_then(|(server_key_id, threshold)|
				self.contract.functions().get_server_key_confirmation_status().call(server_key_id, self_address, &do_call)
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

/// Get hash of the last block with at least n confirmations.
fn get_confirmed_block_hash(client: &Client, confirmations: u64) -> Option<H256> {
	client.block_number(BlockId::Latest)
		.map(|b| b.saturating_sub(confirmations))
		.and_then(|b| client.block_hash(BlockId::Number(b)))
}

#[cfg(test)]
pub mod tests {
	use parking_lot::Mutex;
	use ethkey::Public;
	use ethereum_types::H256;
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
		fn update(&self) -> bool {
			true
		}

		fn read_logs(&self) -> Box<Iterator<Item=Vec<H256>>> {
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
