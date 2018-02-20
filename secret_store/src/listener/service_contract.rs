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
use ethcore::log_entry::LocalizedLogEntry;
use ethcore::client::{Client, BlockChainClient, BlockId};
use ethkey::{Public, Signature, public_to_address};
use hash::keccak;
use ethereum_types::{H256, U256, Address};
use listener::service_contract_listener::ServiceTask;
use trusted_client::TrustedClient;
use {ServerKeyId, NodeKeyPair, ContractAddress, EncryptedDocumentKey};

use_contract!(service, "Service", "res/service.json");

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Server key generation has been requested.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32,uint256)";
/// Document key generation has been requested.
const DOCUMENT_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyRequested(bytes32,uint256,uint8,bytes32,bytes32)";

/// Number of confirmations required before request can be processed.
const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

lazy_static! {
	pub static ref SERVER_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_REQUESTED_EVENT_NAME);
	pub static ref DOCUMENT_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(DOCUMENT_KEY_REQUESTED_EVENT_NAME);
}

/// Service contract trait.
pub trait ServiceContract: Send + Sync {
	/// Update contract when new blocks are enacted. Returns true if contract is installed && up-to-date (i.e. chain is synced).
	fn update(&self) -> bool;
	/// Read recent contract logs. Returns topics of every entry.
	fn read_logs(&self) -> Box<Iterator<Item=ServiceTask>>;
	/// Publish generated key.
	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>>;
	/// Publish generated server key.
	fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String>;
	/// Publish server key generation error.
	fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish retrieved server key.
	fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String>;
	/// Publish server key retrieval error.
	fn publish_server_key_retrieval_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish stored document key.
	fn publish_stored_document_key(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish document key store error.
	fn publish_document_key_store_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
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
struct PendingRequestsIterator<F: Fn(U256) -> Option<(bool, ServiceTask)>> {
	/// Pending request read function.
	read_request: F,
	/// Current request index.
	index: U256,
	/// Requests length.
	length: U256,
}

impl OnChainServiceContract {
	/// Create new on-chain service contract.
	pub fn new(client: TrustedClient, address: ContractAddress, self_key_pair: Arc<NodeKeyPair>) -> Self {
		let contract_addr = match address {
			ContractAddress::Registry => client.get().and_then(|c| c.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest)
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
				let service_contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest).unwrap_or_default();
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

	fn read_logs(&self) -> Box<Iterator<Item=ServiceTask>> {
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
				Some(vec![
					*SERVER_KEY_REQUESTED_EVENT_NAME_HASH,
					*DOCUMENT_KEY_REQUESTED_EVENT_NAME_HASH,
				]),
			],
			limit: None,
		});

		let data = self.data.read();
		Box::new(request_logs.into_iter()
			.filter_map(|log| {
				if log.entry.topics[0] == *SERVER_KEY_REQUESTED_EVENT_NAME_HASH {
					parse_server_key_generation_request(&data.contract, log)
				}/* else if log.entry.topics[0] == *DOCUMENT_KEY_REQUESTED_EVENT_NAME_HASH {
					parse_document_key_generation_request(&data.contract, log)
				}*/ else {
					Err("unknown type of log entry".into())
				}
				.map_err(|error| {
					warn!(target: "secretstore", "{}: error parsing log entry from service contract: {}",
						self.self_key_pair.public(), error);
					error
				})
				.ok()
			}).collect::<Vec<_>>().into_iter())
	}

	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
return Box::new(::std::iter::empty()); // TODO: remove me
/*		let client = match self.client.get() {
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
							warn!(target: "secretstore", "{}: call to key_generation_requests_count failed: {}",
								self.self_key_pair.public(), error);
							error
						})
						.map(|(s_l, d_l)| (b, (s_l, d_l)))
						.ok()
				})
				.map(|(b, (s_l, d_l))| {
					let c_client = client.clone();
					let contract = data.contract.clone();
					let self_key_pair = self.self_key_pair.clone();
					let server_key_generation_requests = PendingRequestsIterator {
						read_request: move |index| read_pending_server_key_generation_request(
							&*c_client, &*contract, &BlockId::Hash(b.clone()), &*self_key_pair, index),
						index: 0.into(),
						length: s_l,
					};

					let c_client = client.clone();
					let contract = data.contract.clone();
					let self_key_pair = self.self_key_pair.clone();
					let document_key_generation_requests = PendingRequestsIterator {
						read_request: move |index| read_pending_document_key_generation_request(
							&*c_client, &*contract, &BlockId::Hash(b.clone()), &*self_key_pair, index),
						index: 0.into(),
						length: d_l,
					};

					Box::new(server_key_generation_requests.chain(document_key_generation_requests))
						as Box<Iterator<Item=(bool, ServiceTask)>>
				})
				/*.map(|(b, l)| Box::new(PendingRequestsIterator {
					client: client,
					contract: service::Service::default(),
					contract_address: data.contract_address,
					self_key_pair: self.self_key_pair.clone(),
					block: b,
					index: 0.into(),
					length: l,
				}) as Box<Iterator<Item=(bool, ServiceTask)>>)*/
				.unwrap_or_else(|| Box::new(::std::iter::empty()))
		}*/
	}

	fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
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

	fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		unimplemented!()
	}

	fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
		unimplemented!()
	}

	fn publish_server_key_retrieval_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		unimplemented!()
	}

	fn publish_stored_document_key(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		unimplemented!()
	}

	fn publish_document_key_store_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		unimplemented!()
	}

	/*fn publish_document_key(&self, server_key_id: &ServerKeyId, document_key: &EncryptedDocumentKey) -> Result<(), String> {
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
		let do_call = |d| client.call_contract(BlockId::Latest, d);
		let self_address = public_to_address(self.self_key_pair.public());
		if data.contract.functions().get_server_key_confirmation_status().call(*server_key_id, self_address, &do_call).unwrap_or(false) {
			return Ok(());
		}

		// prepare transaction data
		let document_key_hash = keccak(document_key);
		let signed_document_key = self.self_key_pair.sign(&document_key_hash).map_err(|e| format!("{}", e))?;
		let signed_document_key: Signature = signed_document_key.into_electrum().into();
		let transaction_data = data.contract.encode_document_key_generated_input(server_key_id.clone(),
			document_key.to_vec(),
			signed_document_key.v(),
			signed_document_key.r().into(),
			signed_document_key.s().into()
		)?;

		// send transaction
		client.transact_contract(
			data.contract_address.clone(),
			transaction_data
		).map_err(|e| format!("{}", e))?;

		Ok(())
	}*/
}

impl<F> Iterator for PendingRequestsIterator<F> where F: Fn(U256) -> Option<(bool, ServiceTask)> {
	type Item = (bool, ServiceTask);

	fn next(&mut self) -> Option<(bool, ServiceTask)> {
		if self.index >= self.length {
			return None;
		}

		let index = self.index.clone();
		self.index = self.index + 1.into();

		(self.read_request)(index)
	}
}

/// Get hash of the last block with at least n confirmations.
fn get_confirmed_block_hash(client: &Client, confirmations: u64) -> Option<H256> {
	client.block_number(BlockId::Latest)
		.map(|b| b.saturating_sub(confirmations))
		.and_then(|b| client.block_hash(BlockId::Number(b)))
}

/// Parse server key generation log entry.
fn parse_server_key_generation_request(contract: &service::Service, log: LocalizedLogEntry) -> Result<ServiceTask, String> {
	let event = contract.events().server_key_requested();
	let log_data = (log.entry.topics.into_iter().map(|t| t.0.into()).collect(), log.entry.data).into();
	match event.parse_log(log_data) {
		Ok(l) => Ok(ServiceTask::GenerateServerKey(l.server_key_id, l.requester, parse_threshold(l.threshold)?)),
		Err(e) => Err(format!("{}", e)),
	}
}
/*
/// Parse document key generation log entry.
fn parse_document_key_generation_request(contract: &service::Service, log: LocalizedLogEntry) -> Result<ServiceTask, String> {
	let event = contract.event("DocumentKeyRequested".into())
		.expect("Contract known ahead of time to have `DocumentKeyRequested` event; qed");
	let log_data = (log.entry.topics.into_iter().map(|t| t.0).collect(), log.entry.data).into();
	event.parse_log(log_data)
		.map(|l| ServiceTask::GenerateDocumentKey(
			(*l.params[0].value.clone().to_fixed_bytes().expect("TODO")).into(),
			l.params[1].value.clone().to_uint().expect("TODO").into(),
			Signature::from_rsv(
				&(*l.params[3].value.clone().to_fixed_bytes().expect("TODO")).into(),
				&(*l.params[4].value.clone().to_fixed_bytes().expect("TODO")).into(),
				l.params[2].value.clone().to_uint().expect("TODO")[0],
			),
		))
		.map_err(|e| format!("{}", e))
}
*/
/// Read pending server key generation request.
/*fn read_pending_server_key_generation_request(client: &Client, contract_address: Address, contract: &service::Service, block: &BlockId, self_key_pair: &NodeKeyPair, index: U256) -> Option<(bool, ServiceTask)> {
	let self_address = public_to_address(self_key_pair.public());
	let do_call = |d| client.call_contract(block.clone(), contract_address, d);
	contract.functions().get_server_key_id().call(index, &do_call).wait()
		.and_then(|server_key_id|
			contract.functions().get_server_key_threshold().call(server_key_id.clone(), &do_call).wait()
				.map(|threshold| (server_key_id, threshold)))
		.and_then(|(server_key_id, threshold)|
			contract.functions().get_server_key_confirmation_status().call(server_key_id.clone(), self_address, &do_call).wait()
				.map(|is_confirmed| (server_key_id, threshold, is_confirmed)))
		.map(|(server_key_id, threshold, is_confirmed)|
			Some((is_confirmed, ServiceTask::GenerateServerKey(server_key_id, threshold.into()))))
		.map_err(|error| {
			warn!(target: "secretstore", "{}: reading service contract request failed: {}",
				self_key_pair.public(), error);
			()
		})
		.unwrap_or(None)
}*/
/*
/// Read pending document key generation request.
fn read_pending_document_key_generation_request(client: &Client, contract: &service::Service, block: &BlockId, self_key_pair: &NodeKeyPair, index: U256) -> Option<(bool, ServiceTask)> {
	let self_address = public_to_address(self_key_pair.public());
	let do_call = |a, d| client.call_contract(block.clone(), a, d);
	contract.get_document_key_id(&do_call, index).wait()
		.and_then(|server_key_id|
			contract.get_document_key_threshold(&do_call, server_key_id.clone()).wait()
				.map(|threshold| (server_key_id, threshold)))
		.and_then(|(server_key_id, threshold)|
			contract.get_document_key_signature(&do_call, server_key_id.clone()).wait()
				.map(|(v, r, s)| (server_key_id, threshold, Signature::from_rsv(&r.into(), &s.into(), v))))
		.and_then(|(server_key_id, threshold, signature)|
			contract.get_document_key_confirmation_status(&do_call, server_key_id.clone(), self_address).wait()
				.map(|is_confirmed| (server_key_id, threshold, signature, is_confirmed)))
		.map(|(server_key_id, threshold, signature, is_confirmed)|
			Some((is_confirmed, ServiceTask::GenerateDocumentKey(server_key_id, threshold.into(), signature))))
		.map_err(|error| {
			warn!(target: "secretstore", "{}: reading service contract request failed: {}",
				self_key_pair.public(), error);
			()
		})
		.unwrap_or(None)
}
*/

/// Parse threshold.
fn parse_threshold(threshold: U256) -> Result<usize, String> {
	let threshold_num = threshold.low_u64();
	if threshold != threshold_num.into() || threshold_num >= ::std::usize::MAX as u64 {
		return Err(format!("invalid threshold {:?}", threshold));
	}

	Ok(threshold_num as usize)
}

#[cfg(test)]
pub mod tests {
	use parking_lot::Mutex;
	use ethkey::Public;
	use ethereum_types::H256;
	use listener::service_contract_listener::ServiceTask;
	use {ServerKeyId, EncryptedDocumentKey};
	use super::ServiceContract;

	#[derive(Default)]
	pub struct DummyServiceContract {
		pub is_actual: bool,
		pub logs: Vec<ServiceTask>,
		pub pending_requests: Vec<(bool, ServiceTask)>,
		pub generated_server_keys: Mutex<Vec<(ServerKeyId, Public)>>,
		pub server_keys_generation_failures: Mutex<Vec<ServerKeyId>>,
		pub retrieved_server_keys: Mutex<Vec<(ServerKeyId, Public)>>,
		pub server_keys_retrieval_failures: Mutex<Vec<ServerKeyId>>,
	}

	impl ServiceContract for DummyServiceContract {
		fn update(&self) -> bool {
			true
		}

		fn read_logs(&self) -> Box<Iterator<Item=ServiceTask>> {
			Box::new(self.logs.clone().into_iter())
		}

		fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
			Box::new(self.pending_requests.clone().into_iter())
		}

		fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
			self.generated_server_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}

		fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.server_keys_generation_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
			self.retrieved_server_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}

		fn publish_server_key_retrieval_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.server_keys_retrieval_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_stored_document_key(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
			unimplemented!()
		}

		fn publish_document_key_store_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
			unimplemented!()
		}

		/*fn publish_document_key(&self, server_key_id: &ServerKeyId, document_key: &EncryptedDocumentKey) -> Result<(), String> {
			self.published_document_keys.lock().push((server_key_id.clone(), document_key.clone()));
			Ok(())
		}*/
	}
}
