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
use parking_lot::RwLock;
use ethabi::RawLog;
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId, RegistryInfo, CallContract};
use ethkey::{Public, public_to_address};
use hash::keccak;
use bytes::Bytes;
use ethereum_types::{H256, U256, Address};
use listener::ApiMask;
use listener::service_contract_listener::ServiceTask;
use trusted_client::TrustedClient;
use helpers::{get_confirmed_block_hash, REQUEST_CONFIRMATIONS_REQUIRED};
use {ServerKeyId, NodeKeyPair, ContractAddress};

use_contract!(service, "Service", "res/service.json");

/// Name of the general SecretStore contract in the registry.
pub const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";
/// Name of the server key generation SecretStore contract in the registry.
pub const SRV_KEY_GEN_SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service_srv_gen";
/// Name of the server key retrieval SecretStore contract in the registry.
pub const SRV_KEY_RETR_SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service_srv_retr";
/// Name of the document key store SecretStore contract in the registry.
pub const DOC_KEY_STORE_SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service_doc_store";
/// Name of the document key retrieval SecretStore contract in the registry.
pub const DOC_KEY_SRETR_SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service_doc_sretr";

/// Server key generation has been requested.
const SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyGenerationRequested(bytes32,address,uint8)";
/// Server key retrieval has been requested.
const SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRetrievalRequested(bytes32)";
/// Document key store has been requested.
const DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyStoreRequested(bytes32,address,bytes,bytes)";
/// Document key common part retrieval has been requested.
const DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyCommonRetrievalRequested(bytes32,address)";
/// Document key personal part retrieval has been requested.
const DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyPersonalRetrievalRequested(bytes32,bytes)";

lazy_static! {
	pub static ref SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME);
	pub static ref SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME);
	pub static ref DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH: H256 = keccak(DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME);
	pub static ref DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH: H256 = keccak(DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME);
	pub static ref DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH: H256 = keccak(DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME);
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
	fn publish_generated_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String>;
	/// Publish server key generation error.
	fn publish_server_key_generation_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish retrieved server key.
	fn publish_retrieved_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String>;
	/// Publish server key retrieval error.
	fn publish_server_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish stored document key.
	fn publish_stored_document_key(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish document key store error.
	fn publish_document_key_store_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish retrieved document key common.
	fn publish_retrieved_document_key_common(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String>;
	/// Publish retrieved document key personal.
	fn publish_retrieved_document_key_personal(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String>;
	/// Publish document key store error.
	fn publish_document_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String>;
}

/// On-chain service contract.
pub struct OnChainServiceContract {
	/// Requests mask.
	mask: ApiMask,
	/// Blockchain client.
	client: TrustedClient,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Contract registry name (if any).
	name: String,
	/// Contract address.
	address: ContractAddress,
	/// Contract.
	contract: service::Service,
	/// Contract.
	data: RwLock<ServiceData>,
}

/// On-chain service contract data.
struct ServiceData {
	/// Actual contract address.
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

/// Server key generation related functions.
struct ServerKeyGenerationService;
/// Server key retrieval related functions.
struct ServerKeyRetrievalService;
/// Document key store related functions.
struct DocumentKeyStoreService;
/// Document key shadow retrievalrelated functions.
struct DocumentKeyShadowRetrievalService;

impl OnChainServiceContract {
	/// Create new on-chain service contract.
	pub fn new(mask: ApiMask, client: TrustedClient, name: String, address: ContractAddress, self_key_pair: Arc<NodeKeyPair>) -> Self {
		let contract_addr = match address {
			ContractAddress::Registry => client.get().and_then(|c| c.registry_address(name.clone(), BlockId::Latest)
				.map(|address| {
					trace!(target: "secretstore", "{}: installing {} service contract from address {}",
						self_key_pair.public(), name, address);
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
			mask: mask,
			client: client,
			self_key_pair: self_key_pair,
			name: name,
			address: address,
			contract: service::Service::default(),
			data: RwLock::new(ServiceData {
				contract_address: contract_addr,
				last_log_block: None,
			}),
		}
	}

	/// Send transaction to the service contract.
	fn send_contract_transaction<C, P>(&self, origin: &Address, server_key_id: &ServerKeyId, is_response_required: C, prepare_tx: P) -> Result<(), String>
		where C: FnOnce(&Client, &Address, &service::Service, &ServerKeyId, &Address) -> bool,
			P: FnOnce(&Client, &Address, &service::Service) -> Result<Bytes, String> {
		// only publish if contract address is set && client is online
		let client = match self.client.get() {
			Some(client) => client,
			None => return Err("trusted client is required to publish key".into()),
		};

		// only publish key if contract waits for publication
		// failing is ok here - it could be that enough confirmations have been recevied
		// or key has been requested using HTTP API
		let self_address = public_to_address(self.self_key_pair.public());
		if !is_response_required(&*client, origin, &self.contract, server_key_id, &self_address) {
			return Ok(());
		}

		// prepare transaction data
		let transaction_data = prepare_tx(&*client, origin, &self.contract)?;

		// send transaction
		self.client.transact_contract(
			origin.clone(),
			transaction_data
		).map_err(|e| format!("{}", e))?;

		Ok(())
	}

	/// Create task-specific pending requests iterator.
	fn create_pending_requests_iterator<
		C: 'static + Fn(&Client, &Address, &service::Service, &BlockId) -> Result<U256, String>,
		R: 'static + Fn(&NodeKeyPair, &Client, &Address, &service::Service, &BlockId, U256) -> Result<(bool, ServiceTask), String>
	>(&self, client: Arc<Client>, contract_address: &Address, block: &BlockId, get_count: C, read_item: R) -> Box<Iterator<Item=(bool, ServiceTask)>> {
		let contract = service::Service::default();
		get_count(&*client, contract_address, &contract, block)
			.map(|count| {
				let client = client.clone();
				let self_key_pair = self.self_key_pair.clone();
				let contract_address = contract_address.clone();
				let block = block.clone();
				Box::new(PendingRequestsIterator {
					read_request: move |index| read_item(&*self_key_pair, &*client, &contract_address, &contract, &block, index)
						.map_err(|error| {
							warn!(target: "secretstore", "{}: reading pending request failed: {}",
								self_key_pair.public(), error);
							error
						})
						.ok(),
					index: 0.into(),
					length: count,
				}) as Box<Iterator<Item=(bool, ServiceTask)>>
			})
			.map_err(|error| {
				warn!(target: "secretstore", "{}: creating pending requests iterator failed: {}",
					self.self_key_pair.public(), error);
				error
			})
			.ok()
			.unwrap_or_else(|| Box::new(::std::iter::empty()))
	}
}

impl ServiceContract for OnChainServiceContract {
	fn update(&self) -> bool {
		if let &ContractAddress::Registry = &self.address {
			if let Some(client) = self.client.get() {
				if let Some(block_hash) = get_confirmed_block_hash(&*client, REQUEST_CONFIRMATIONS_REQUIRED) {
					// update contract address from registry
					let service_contract_addr = client.registry_address(self.name.clone(), BlockId::Hash(block_hash)).unwrap_or_default();
					if self.data.read().contract_address != service_contract_addr {
						trace!(target: "secretstore", "{}: installing {} service contract from address {}",
							   self.self_key_pair.public(), self.name, service_contract_addr);
						self.data.write().contract_address = service_contract_addr;
					}
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
			topics: vec![Some(mask_topics(&self.mask))],
			limit: None,
		});

		Box::new(request_logs.into_iter()
			.filter_map(|log| {
				let raw_log: RawLog = (log.entry.topics.into_iter().map(|t| t.0.into()).collect(), log.entry.data).into();
				if raw_log.topics[0] == *SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH {
					ServerKeyGenerationService::parse_log(&address, &self.contract, raw_log)
				} else if raw_log.topics[0] == *SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					ServerKeyRetrievalService::parse_log(&address, &self.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyStoreService::parse_log(&address, &self.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_common_request_log(&address, &self.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_personal_request_log(&address, &self.contract, raw_log)
				} else {
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
				.map(|b| {
					let block = BlockId::Hash(b);
					let iter = match self.mask.server_key_generation_requests {
						true => Box::new(self.create_pending_requests_iterator(client.clone(), &data.contract_address, &block,
							&ServerKeyGenerationService::read_pending_requests_count,
							&ServerKeyGenerationService::read_pending_request)) as Box<Iterator<Item=(bool, ServiceTask)>>,
						false => Box::new(::std::iter::empty()),
					};
					let iter = match self.mask.server_key_retrieval_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(client.clone(), &data.contract_address, &block,
							&ServerKeyRetrievalService::read_pending_requests_count,
							&ServerKeyRetrievalService::read_pending_request))),
						false => iter,
					};
					let iter = match self.mask.document_key_store_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(client.clone(), &data.contract_address, &block,
							&DocumentKeyStoreService::read_pending_requests_count,
							&DocumentKeyStoreService::read_pending_request))),
						false => iter,
					};
					let iter = match self.mask.document_key_shadow_retrieval_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(client, &data.contract_address, &block,
							&DocumentKeyShadowRetrievalService::read_pending_requests_count,
							&DocumentKeyShadowRetrievalService::read_pending_request))),
						false => iter
					};

					iter
				})
				.unwrap_or_else(|| Box::new(::std::iter::empty()))
		}
	}

	fn publish_generated_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, ServerKeyGenerationService::is_response_required, |_, _, service|
			Ok(ServerKeyGenerationService::prepare_pubish_tx_data(service, server_key_id, &server_key))
		)
	}

	fn publish_server_key_generation_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, ServerKeyGenerationService::is_response_required, |_, _, service|
			Ok(ServerKeyGenerationService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_retrieved_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
		let threshold = serialize_threshold(threshold)?;
		self.send_contract_transaction(origin, server_key_id, ServerKeyRetrievalService::is_response_required, |_, _, service|
			Ok(ServerKeyRetrievalService::prepare_pubish_tx_data(service, server_key_id, server_key, threshold))
		)
	}

	fn publish_server_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, ServerKeyRetrievalService::is_response_required, |_, _, service|
			Ok(ServerKeyRetrievalService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_stored_document_key(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, DocumentKeyStoreService::is_response_required, |_, _, service|
			Ok(DocumentKeyStoreService::prepare_pubish_tx_data(service, server_key_id))
		)
	}

	fn publish_document_key_store_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, DocumentKeyStoreService::is_response_required, |_, _, service|
			Ok(DocumentKeyStoreService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_retrieved_document_key_common(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
		let threshold = serialize_threshold(threshold)?;
		self.send_contract_transaction(origin, server_key_id, |client, contract_address, contract, server_key_id, key_server|
			DocumentKeyShadowRetrievalService::is_response_required(client, contract_address, contract, server_key_id, requester, key_server),
		|_, _, service|
			Ok(DocumentKeyShadowRetrievalService::prepare_pubish_common_tx_data(service, server_key_id, requester, common_point, threshold))
		)
	}

	fn publish_retrieved_document_key_personal(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, |_, _, _, _, _| true,
		move |client, address, service|
			DocumentKeyShadowRetrievalService::prepare_pubish_personal_tx_data(client, address, service, server_key_id, requester, participants, decrypted_secret, shadow)
		)
	}

	fn publish_document_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
		self.send_contract_transaction(origin, server_key_id, |client, contract_address, contract, server_key_id, key_server|
			DocumentKeyShadowRetrievalService::is_response_required(client, contract_address, contract, server_key_id, requester, key_server),
		|_, _, service|
			Ok(DocumentKeyShadowRetrievalService::prepare_error_tx_data(service, server_key_id, requester))
		)
	}
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

/// Returns vector of logs topics to listen to.
pub fn mask_topics(mask: &ApiMask) -> Vec<H256> {
	let mut topics = Vec::new();
	if mask.server_key_generation_requests {
		topics.push(*SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH);
	}
	if mask.server_key_retrieval_requests {
		topics.push(*SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH);
	}
	if mask.document_key_store_requests {
		topics.push(*DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH);
	}
	if mask.document_key_shadow_retrieval_requests {
		topics.push(*DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH);
		topics.push(*DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH);
	}
	topics
}

impl ServerKeyGenerationService {
	/// Parse request log entry.
	pub fn parse_log(origin: &Address, contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().server_key_generation_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::GenerateServerKey(origin.clone(), l.server_key_id, l.author, parse_threshold(l.threshold)?)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.is_server_key_generation_response_required()
			.call(*server_key_id, key_server.clone(), &do_call)
			.unwrap_or(true)
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, server_key_public: &Public) -> Bytes {
		contract.functions()
			.server_key_generated()
			.input(*server_key_id, server_key_public.to_vec())
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(contract: &service::Service, server_key_id: &ServerKeyId) -> Bytes {
		contract.functions()
			.server_key_generation_error()
			.input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &Client, contract_address: &Address, _contract: &service::Service, block: &BlockId) -> Result<U256, String> {
		let do_call = |data| client.call_contract(block.clone(), contract_address.clone(), data);
		let contract = service::Service::default();
		contract.functions()
			.server_key_generation_requests_count()
			.call(&do_call)
			.map_err(|error| format!("{}", error))
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &NodeKeyPair, client: &Client, contract_address: &Address, contract: &service::Service, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());
		let do_call = |d| client.call_contract(block.clone(), contract_address.clone(), d);
		contract.functions()
			.get_server_key_generation_request()
			.call(index, &do_call)
			.map_err(|error| format!("{}", error))
			.and_then(|(server_key_id, author, threshold)| parse_threshold(threshold)
				.map(|threshold| (server_key_id, author, threshold)))
			.and_then(|(server_key_id, author, threshold)| contract.functions()
				.is_server_key_generation_response_required()
				.call(server_key_id.clone(), self_address, &do_call)
				.map(|not_confirmed| (
					not_confirmed,
					ServiceTask::GenerateServerKey(
						contract_address.clone(),
						server_key_id,
						author,
						threshold,
					)))
				.map_err(|error| format!("{}", error)))
	}
}

impl ServerKeyRetrievalService {
	/// Parse request log entry.
	pub fn parse_log(origin: &Address, contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().server_key_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveServerKey(origin.clone(), l.server_key_id)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.is_server_key_retrieval_response_required()
			.call(*server_key_id, key_server.clone(), &do_call)
			.unwrap_or(true)
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, server_key_public: Public, threshold: U256) -> Bytes {
		contract.functions()
			.server_key_retrieved()
			.input(*server_key_id, server_key_public.to_vec(), threshold)
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(contract: &service::Service, server_key_id: &ServerKeyId) -> Bytes {
		contract.functions()
			.server_key_retrieval_error()
			.input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &Client, contract_address: &Address, _contract: &service::Service, block: &BlockId) -> Result<U256, String> {
		let do_call = |data| client.call_contract(block.clone(), contract_address.clone(), data);
		let contract = service::Service::default();
		contract.functions()
			.server_key_retrieval_requests_count()
			.call(&do_call)
			.map_err(|error| format!("{}", error))
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &NodeKeyPair, client: &Client, contract_address: &Address, contract: &service::Service, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());
		let do_call = |d| client.call_contract(block.clone(), contract_address.clone(), d);
		contract.functions()
			.get_server_key_retrieval_request()
			.call(index, &do_call)
			.map_err(|error| format!("{}", error))
			.and_then(|server_key_id| contract.functions()
				.is_server_key_retrieval_response_required()
				.call(server_key_id.clone(), self_address, &do_call)
				.map(|not_confirmed| (
					not_confirmed,
					ServiceTask::RetrieveServerKey(
						contract_address.clone(),
						server_key_id,
					)))
				.map_err(|error| format!("{}", error)))
	}
}

impl DocumentKeyStoreService {
	/// Parse request log entry.
	pub fn parse_log(origin: &Address, contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_store_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::StoreDocumentKey(origin.clone(), l.server_key_id, l.author, (*l.common_point).into(), (*l.encrypted_point).into())),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.is_document_key_store_response_required()
			.call(*server_key_id, key_server.clone(), &do_call)
			.unwrap_or(true)
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(contract: &service::Service, server_key_id: &ServerKeyId) -> Bytes {
		contract.functions()
			.document_key_stored()
			.input(*server_key_id)
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(contract: &service::Service, server_key_id: &ServerKeyId) -> Bytes {
		contract.functions()
			.document_key_store_error()
			.input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &Client, contract_address: &Address, _contract: &service::Service, block: &BlockId) -> Result<U256, String> {
		let do_call = |data| client.call_contract(block.clone(), contract_address.clone(), data);
		let contract = service::Service::default();
		contract.functions()
			.document_key_store_requests_count()
			.call(&do_call)
			.map_err(|error| format!("{}", error))
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &NodeKeyPair, client: &Client, contract_address: &Address, contract: &service::Service, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());
		let do_call = |d| client.call_contract(block.clone(), contract_address.clone(), d);
		contract.functions()
			.get_document_key_store_request()
			.call(index, &do_call)
			.map_err(|error| format!("{}", error))
			.and_then(|(server_key_id, author, common_point, encrypted_point)| contract.functions()
				.is_document_key_store_response_required()
				.call(server_key_id.clone(), self_address, &do_call)
				.map(|not_confirmed| (
					not_confirmed,
					ServiceTask::StoreDocumentKey(
						contract_address.clone(),
						server_key_id,
						author,
						Public::from_slice(&common_point),
						Public::from_slice(&encrypted_point),
					)))
				.map_err(|error| format!("{}", error)))
	}
}

impl DocumentKeyShadowRetrievalService {
	/// Parse common request log entry.
	pub fn parse_common_request_log(origin: &Address, contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_common_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyCommon(origin.clone(), l.server_key_id, l.requester)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Parse personal request log entry.
	pub fn parse_personal_request_log(origin: &Address, contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_personal_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyPersonal(origin.clone(), l.server_key_id, (*l.requester_public).into())),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.is_document_key_shadow_retrieval_response_required()
			.call(*server_key_id, *requester, key_server.clone(), &do_call)
			.unwrap_or(true)
	}

	/// Prepare publish common key transaction data.
	pub fn prepare_pubish_common_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: U256) -> Bytes {
		contract.functions()
			.document_key_common_retrieved()
			.input(*server_key_id, *requester, common_point.to_vec(), threshold)
	}

	/// Prepare publish personal key transaction data.
	pub fn prepare_pubish_personal_tx_data(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<Bytes, String> {
		let mut participants_mask = U256::default();
		for participant in participants {
			let participant_index = Self::map_key_server_address(client, contract_address, contract, participant.clone())
				.map_err(|e| format!("Error searching for {} participant: {}", participant, e))?;
			participants_mask = participants_mask | (U256::one() << participant_index.into());
		}
		Ok(contract.functions()
			.document_key_personal_retrieved()
			.input(*server_key_id, *requester, participants_mask, decrypted_secret.to_vec(), shadow))
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address) -> Bytes {
		contract.functions()
			.document_key_shadow_retrieval_error()
			.input(*server_key_id, *requester)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &Client, contract_address: &Address, _contract: &service::Service, block: &BlockId) -> Result<U256, String> {
		let do_call = |data| client.call_contract(block.clone(), contract_address.clone(), data);
		let contract = service::Service::default();
		contract.functions()
			.document_key_shadow_retrieval_requests_count()
			.call(&do_call)
			.map_err(|error| format!("{}", error))
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &NodeKeyPair, client: &Client, contract_address: &Address, contract: &service::Service, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());
		let do_call = |d| client.call_contract(block.clone(), contract_address.clone(), d);
		contract.functions()
			.get_document_key_shadow_retrieval_request()
			.call(index, &do_call)
			.map_err(|error| format!("{}", error))
			.and_then(|(server_key_id, requester, is_common_retrieval_completed)| {
				let requester = Public::from_slice(&requester);
				contract.functions()
					.is_document_key_shadow_retrieval_response_required()
					.call(server_key_id.clone(), public_to_address(&requester), self_address, &do_call)
					.map(|not_confirmed| (
						not_confirmed,
						match is_common_retrieval_completed {
							true => ServiceTask::RetrieveShadowDocumentKeyCommon(
								contract_address.clone(),
								server_key_id,
								public_to_address(&requester),
							),
							false => ServiceTask::RetrieveShadowDocumentKeyPersonal(
								contract_address.clone(),
								server_key_id,
								requester,
							)
						},
					))
					.map_err(|error| format!("{}", error))
			})
	}

	/// Map from key server address to key server index.
	fn map_key_server_address(client: &Client, contract_address: &Address, contract: &service::Service, key_server: Address) -> Result<u8, String> {
		// we're checking confirmation in Latest block, because tx ,ust be appended to the latest state
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.require_key_server()
			.call(key_server, &do_call)
			.map_err(|e| format!("{}", e))
			.and_then(|index| if index > ::std::u8::MAX.into() {
				Err(format!("key server index is too big: {}", index))
			} else {
				let index: u32 = index.into();
				Ok(index as u8)
			})
	}
}

/// Parse threshold (we only supposrt 256 KS at max).
fn parse_threshold(threshold: U256) -> Result<usize, String> {
	let threshold_num = threshold.low_u64();
	if threshold != threshold_num.into() || threshold_num >= ::std::u8::MAX as u64 {
		return Err(format!("invalid threshold to use in service contract: {}", threshold));
	}

	Ok(threshold_num as usize)
}

/// Serialize threshold (we only support 256 KS at max).
fn serialize_threshold(threshold: usize) -> Result<U256, String> {
	if threshold > ::std::u8::MAX as usize {
		return Err(format!("invalid threshold to use in service contract: {}", threshold));
	}
	Ok(threshold.into())
}

#[cfg(test)]
pub mod tests {
	use parking_lot::Mutex;
	use bytes::Bytes;
	use ethkey::Public;
	use ethereum_types::Address;
	use listener::service_contract_listener::ServiceTask;
	use {ServerKeyId};
	use super::ServiceContract;

	#[derive(Default)]
	pub struct DummyServiceContract {
		pub is_actual: bool,
		pub logs: Vec<ServiceTask>,
		pub pending_requests: Vec<(bool, ServiceTask)>,
		pub generated_server_keys: Mutex<Vec<(ServerKeyId, Public)>>,
		pub server_keys_generation_failures: Mutex<Vec<ServerKeyId>>,
		pub retrieved_server_keys: Mutex<Vec<(ServerKeyId, Public, usize)>>,
		pub server_keys_retrieval_failures: Mutex<Vec<ServerKeyId>>,
		pub stored_document_keys: Mutex<Vec<ServerKeyId>>,
		pub document_keys_store_failures: Mutex<Vec<ServerKeyId>>,
		pub common_shadow_retrieved_document_keys: Mutex<Vec<(ServerKeyId, Address, Public, usize)>>,
		pub personal_shadow_retrieved_document_keys: Mutex<Vec<(ServerKeyId, Address, Vec<Address>, Public, Bytes)>>,
		pub document_keys_shadow_retrieval_failures: Mutex<Vec<(ServerKeyId, Address)>>,
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

		fn publish_generated_server_key(&self, _origin: &Address, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String> {
			self.generated_server_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}

		fn publish_server_key_generation_error(&self, _origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.server_keys_generation_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_retrieved_server_key(&self, _origin: &Address, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
			self.retrieved_server_keys.lock().push((server_key_id.clone(), server_key.clone(), threshold));
			Ok(())
		}

		fn publish_server_key_retrieval_error(&self, _origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.server_keys_retrieval_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_stored_document_key(&self, _origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.stored_document_keys.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_document_key_store_error(&self, _origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.document_keys_store_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_retrieved_document_key_common(&self, _origin: &Address, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
			self.common_shadow_retrieved_document_keys.lock().push((server_key_id.clone(), requester.clone(), common_point.clone(), threshold));
			Ok(())
		}

		fn publish_retrieved_document_key_personal(&self, _origin: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
			self.personal_shadow_retrieved_document_keys.lock().push((server_key_id.clone(), requester.clone(), participants.iter().cloned().collect(), decrypted_secret, shadow));
			Ok(())
		}

		fn publish_document_key_retrieval_error(&self, _origin: &Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
			self.document_keys_shadow_retrieval_failures.lock().push((server_key_id.clone(), requester.clone()));
			Ok(())
		}
	}
}
