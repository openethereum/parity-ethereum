// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use parking_lot::RwLock;
use ethabi::RawLog;
use ethabi::FunctionOutputDecoder;
use crypto::publickey::{Public, public_to_address};
use hash::keccak;
use bytes::Bytes;
use ethereum_types::{H256, U256, Address, H512};
use listener::ApiMask;
use listener::service_contract_listener::ServiceTask;
use blockchain::{SecretStoreChain, Filter, SigningKeyPair, ContractAddress, BlockId};
use ServerKeyId;

use_contract!(service, "res/service.json");

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
	fn read_logs(&self) -> Box<dyn Iterator<Item=ServiceTask>>;
	/// Publish generated key.
	fn read_pending_requests(&self) -> Box<dyn Iterator<Item=(bool, ServiceTask)>>;
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
	client: Arc<dyn SecretStoreChain>,
	/// This node key pair.
	self_key_pair: Arc<dyn SigningKeyPair>,
	/// Contract registry name (if any).
	name: String,
	/// Contract address source.
	address_source: ContractAddress,
	/// Contract.
	data: RwLock<ServiceData>,
}

/// On-chain service contract data.
struct ServiceData {
	/// Current contract address.
	pub contract_address: Option<Address>,
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
	pub fn new(mask: ApiMask, client: Arc<dyn SecretStoreChain>, name: String, address_source: ContractAddress, self_key_pair: Arc<dyn SigningKeyPair>) -> Self {
		let contract = OnChainServiceContract {
			mask: mask,
			client: client,
			self_key_pair: self_key_pair,
			name: name,
			address_source: address_source,
			data: RwLock::new(ServiceData {
				contract_address: None,
				last_log_block: None,
			}),
		};

		contract.update_contract_address();
		contract
	}

	/// Send transaction to the service contract.
	fn send_contract_transaction<C, P>(&self, tx_name: &str, origin: &Address, server_key_id: &ServerKeyId, is_response_required: C, prepare_tx: P) -> Result<(), String>
		where C: FnOnce(&dyn SecretStoreChain, &Address, &ServerKeyId, &Address) -> bool,
			P: FnOnce(&dyn SecretStoreChain, &Address) -> Result<Bytes, String> {
		// only publish if contract address is set && client is online
		if !self.client.is_trusted() {
			return Err("trusted client is required to publish key".into())
		}

		// only publish key if contract waits for publication
		// failing is ok here - it could be that enough confirmations have been recevied
		// or key has been requested using HTTP API
		let self_address = public_to_address(self.self_key_pair.public());
		if !is_response_required(&*self.client, origin, server_key_id, &self_address) {
			return Ok(());
		}

		// prepare transaction data
		let transaction_data = prepare_tx(&*self.client, origin)?;

		// send transaction
		self.client.transact_contract(
			origin.clone(),
			transaction_data
		).map_err(|e| format!("{}", e))?;

		trace!(target: "secretstore", "{}: transaction {} sent to service contract",
			self.self_key_pair.public(), tx_name);

		Ok(())
	}

	/// Create task-specific pending requests iterator.
	fn create_pending_requests_iterator<
		C: 'static + Fn(&dyn SecretStoreChain, &Address, &BlockId) -> Result<U256, String>,
		R: 'static + Fn(&dyn SigningKeyPair, &dyn SecretStoreChain, &Address, &BlockId, U256) -> Result<(bool, ServiceTask), String>
	>(&self, client: Arc<dyn SecretStoreChain>, contract_address: &Address, block: &BlockId, get_count: C, read_item: R) -> Box<dyn Iterator<Item=(bool, ServiceTask)>> {
		get_count(&*client, contract_address, block)
			.map(|count| {
				let client = client.clone();
				let self_key_pair = self.self_key_pair.clone();
				let contract_address = contract_address.clone();
				let block = block.clone();
				Box::new(PendingRequestsIterator {
					read_request: move |index| read_item(&*self_key_pair, &*client, &contract_address, &block, index)
						.map_err(|error| {
							warn!(target: "secretstore", "{}: reading pending request failed: {}",
								self_key_pair.public(), error);
							error
						})
						.ok(),
					index: 0.into(),
					length: count,
				}) as Box<dyn Iterator<Item=(bool, ServiceTask)>>
			})
			.map_err(|error| {
				warn!(target: "secretstore", "{}: creating pending requests iterator failed: {}",
					self.self_key_pair.public(), error);
				error
			})
			.ok()
			.unwrap_or_else(|| Box::new(::std::iter::empty()))
	}

	/// Update service contract address.
	fn update_contract_address(&self) -> bool {
		let contract_address = self.client.read_contract_address(&self.name, &self.address_source);
		let mut data = self.data.write();
		if contract_address != data.contract_address {
			trace!(target: "secretstore", "{}: installing {} service contract from address {:?}",
				self.self_key_pair.public(), self.name, contract_address);

			data.contract_address = contract_address;
		}

		data.contract_address.is_some()
	}
}

impl ServiceContract for OnChainServiceContract {
	fn update(&self) -> bool {
		self.update_contract_address() && self.client.is_trusted()
	}

	fn read_logs(&self) -> Box<dyn Iterator<Item=ServiceTask>> {
		if !self.client.is_trusted() {
			warn!(target: "secretstore", "{}: client is offline during read_logs call",
				self.self_key_pair.public());
			return Box::new(::std::iter::empty());
		}

		let address = match self.data.read().contract_address {
			Some(address) => address,
			None => return Box::new(::std::iter::empty()), // no contract installed
		};
		let confirmed_block = match self.client.get_confirmed_block_hash() {
			Some(confirmed_block) => confirmed_block,
			None => return Box::new(::std::iter::empty()), // no block with enough confirmations
		};

		let request_logs = self.client.retrieve_last_logs(Filter {
			from_block: BlockId::Hash(self.data.read().last_log_block.unwrap_or_else(|| confirmed_block)),
			address: Some(vec![address]),
			topics: vec![Some(mask_topics(&self.mask))],
		}).unwrap_or_default();

		let mut data = self.data.write();
		data.last_log_block = Some(confirmed_block.clone());

		Box::new(request_logs.into_iter()
			.filter_map(|log| {
				if log.topics[0] == *SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH {
					ServerKeyGenerationService::parse_log(&address, log)
				} else if log.topics[0] == *SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					ServerKeyRetrievalService::parse_log(&address, log)
				} else if log.topics[0] == *DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyStoreService::parse_log(&address, log)
				} else if log.topics[0] == *DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_common_request_log(&address, log)
				} else if log.topics[0] == *DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_personal_request_log(&address, log)
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

	fn read_pending_requests(&self) -> Box<dyn Iterator<Item=(bool, ServiceTask)>> {
		if !self.client.is_trusted() {
			return Box::new(::std::iter::empty())
		}

		// we only need requests that are here from the last confirm block
		let data = self.data.read();
		match data.contract_address {
			None => Box::new(::std::iter::empty()),
			Some(contract_address) => self.client.get_confirmed_block_hash()
				.map(|b| {
					let block = BlockId::Hash(b);
					let iter = match self.mask.server_key_generation_requests {
						true => Box::new(self.create_pending_requests_iterator(self.client.clone(), &contract_address, &block,
							&ServerKeyGenerationService::read_pending_requests_count,
							&ServerKeyGenerationService::read_pending_request)) as Box<dyn Iterator<Item=(bool, ServiceTask)>>,
						false => Box::new(::std::iter::empty()),
					};
					let iter = match self.mask.server_key_retrieval_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(self.client.clone(), &contract_address, &block,
							&ServerKeyRetrievalService::read_pending_requests_count,
							&ServerKeyRetrievalService::read_pending_request))),
						false => iter,
					};
					let iter = match self.mask.document_key_store_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(self.client.clone(), &contract_address, &block,
							&DocumentKeyStoreService::read_pending_requests_count,
							&DocumentKeyStoreService::read_pending_request))),
						false => iter,
					};
					let iter = match self.mask.document_key_shadow_retrieval_requests {
						true => Box::new(iter.chain(self.create_pending_requests_iterator(self.client.clone(), &contract_address, &block,
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
		self.send_contract_transaction("publish_generated_server_key", origin, server_key_id, ServerKeyGenerationService::is_response_required,
			|_, _| Ok(ServerKeyGenerationService::prepare_pubish_tx_data(server_key_id, &server_key)))
	}

	fn publish_server_key_generation_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction("publish_server_key_generation_error", origin, server_key_id, ServerKeyGenerationService::is_response_required,
			|_, _| Ok(ServerKeyGenerationService::prepare_error_tx_data(server_key_id)))
	}

	fn publish_retrieved_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
		let threshold = serialize_threshold(threshold)?;
		self.send_contract_transaction("publish_retrieved_server_key", origin, server_key_id, ServerKeyRetrievalService::is_response_required,
			|_, _| Ok(ServerKeyRetrievalService::prepare_pubish_tx_data(server_key_id, server_key, threshold)))
	}

	fn publish_server_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction("publish_server_key_retrieval_error", origin, server_key_id, ServerKeyRetrievalService::is_response_required,
			|_, _| Ok(ServerKeyRetrievalService::prepare_error_tx_data(server_key_id)))
	}

	fn publish_stored_document_key(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction("publish_stored_document_key", origin, server_key_id, DocumentKeyStoreService::is_response_required,
			|_, _| Ok(DocumentKeyStoreService::prepare_pubish_tx_data(server_key_id)))
	}

	fn publish_document_key_store_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction("publish_document_key_store_error", origin, server_key_id, DocumentKeyStoreService::is_response_required,
			|_, _| Ok(DocumentKeyStoreService::prepare_error_tx_data(server_key_id)))
	}

	fn publish_retrieved_document_key_common(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
		let threshold = serialize_threshold(threshold)?;
		self.send_contract_transaction("publish_retrieved_document_key_common", origin, server_key_id,
			|client, contract_address, server_key_id, key_server|
				DocumentKeyShadowRetrievalService::is_response_required(client, contract_address, server_key_id, requester, key_server),
			|_, _|
				Ok(DocumentKeyShadowRetrievalService::prepare_pubish_common_tx_data(server_key_id, requester, common_point, threshold))
		)
	}

	fn publish_retrieved_document_key_personal(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
		self.send_contract_transaction("publish_retrieved_document_key_personal", origin, server_key_id, |_, _, _, _| true,
		move |client, address|
			DocumentKeyShadowRetrievalService::prepare_pubish_personal_tx_data(client, address, server_key_id, requester, participants, decrypted_secret, shadow)
		)
	}

	fn publish_document_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
		self.send_contract_transaction("publish_document_key_retrieval_error", origin, server_key_id,
			|client, contract_address, server_key_id, key_server|
				DocumentKeyShadowRetrievalService::is_response_required(client, contract_address, server_key_id, requester, key_server),
			|_, _|
				Ok(DocumentKeyShadowRetrievalService::prepare_error_tx_data(server_key_id, requester))
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
		self.index = self.index + 1;

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
	pub fn parse_log(origin: &Address, raw_log: RawLog) -> Result<ServiceTask, String> {
		match service::events::server_key_generation_requested::parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::GenerateServerKey(origin.clone(), l.server_key_id, l.author, parse_threshold(l.threshold)?)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &dyn SecretStoreChain, contract_address: &Address, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let (encoded, decoder) = service::functions::is_server_key_generation_response_required::call(*server_key_id, *key_server);
		match client.call_contract(BlockId::Latest, *contract_address, encoded) {
			Err(_) => true,
			Ok(data) => decoder.decode(&data).unwrap_or(true)
		}
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(server_key_id: &ServerKeyId, server_key_public: &Public) -> Bytes {
		service::functions::server_key_generated::encode_input(*server_key_id, server_key_public.as_bytes().to_vec())
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(server_key_id: &ServerKeyId) -> Bytes {
		service::functions::server_key_generation_error::encode_input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId) -> Result<U256, String> {
		let (encoded, decoder) = service::functions::server_key_generation_requests_count::call();
		decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &dyn SigningKeyPair, client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());

		let (encoded, decoder) = service::functions::get_server_key_generation_request::call(index);
		let (server_key_id, author, threshold) = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;
		let threshold = parse_threshold(threshold)?;

		let (encoded, decoder) = service::functions::is_server_key_generation_response_required::call(server_key_id, self_address);
		let not_confirmed = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let task = ServiceTask::GenerateServerKey(
			contract_address.clone(),
			server_key_id,
			author,
			threshold,
		);

		Ok((not_confirmed, task))
	}
}

impl ServerKeyRetrievalService {
	/// Parse request log entry.
	pub fn parse_log(origin: &Address, raw_log: RawLog) -> Result<ServiceTask, String> {
		match service::events::server_key_retrieval_requested::parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveServerKey(*origin, l.server_key_id)),
			Err(e) => Err(e.to_string())
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &dyn SecretStoreChain, contract_address: &Address, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let (encoded, decoder) = service::functions::is_server_key_retrieval_response_required::call(*server_key_id, *key_server);
		match client.call_contract(BlockId::Latest, *contract_address, encoded) {
			Err(_) => true,
			Ok(data) => decoder.decode(&data).unwrap_or(true)
		}
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(server_key_id: &ServerKeyId, server_key_public: Public, threshold: U256) -> Bytes {
		service::functions::server_key_retrieved::encode_input(*server_key_id, server_key_public.as_bytes().to_vec(), threshold)
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(server_key_id: &ServerKeyId) -> Bytes {
		service::functions::server_key_retrieval_error::encode_input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId) -> Result<U256, String> {
		let (encoded, decoder) = service::functions::server_key_retrieval_requests_count::call();
		decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &dyn SigningKeyPair, client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());

		let (encoded, decoder) = service::functions::get_server_key_retrieval_request::call(index);
		let server_key_id = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let (encoded, decoder) = service::functions::is_server_key_retrieval_response_required::call(server_key_id, self_address);
		let not_confirmed = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let task = ServiceTask::RetrieveServerKey(
			*contract_address,
			server_key_id,
		);

		Ok((not_confirmed, task))
	}
}

impl DocumentKeyStoreService {
	/// Parse request log entry.
	pub fn parse_log(origin: &Address, raw_log: RawLog) -> Result<ServiceTask, String> {
		match service::events::document_key_store_requested::parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::StoreDocumentKey(
				origin.clone(),
				l.server_key_id,
				l.author,
				H512::from_slice(&*l.common_point),
				H512::from_slice(&*l.encrypted_point),
			)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &dyn SecretStoreChain, contract_address: &Address, server_key_id: &ServerKeyId, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let (encoded, decoder) = service::functions::is_document_key_store_response_required::call(*server_key_id, *key_server);
		match client.call_contract(BlockId::Latest, *contract_address, encoded) {
			Err(_) => true,
			Ok(data) => decoder.decode(&data).unwrap_or(true)
		}
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(server_key_id: &ServerKeyId) -> Bytes {
		service::functions::document_key_stored::encode_input(*server_key_id)
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(server_key_id: &ServerKeyId) -> Bytes {
		service::functions::document_key_store_error::encode_input(*server_key_id)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId) -> Result<U256, String> {
		let (encoded, decoder) = service::functions::document_key_store_requests_count::call();
		decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &dyn SigningKeyPair, client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());
		let (encoded, decoder) = service::functions::get_document_key_store_request::call(index);
		let (server_key_id, author, common_point, encrypted_point) = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let (encoded, decoder) = service::functions::is_document_key_store_response_required::call(server_key_id, self_address);
		let not_confirmed = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let task = ServiceTask::StoreDocumentKey(
			*contract_address,
			server_key_id,
			author,
			Public::from_slice(&common_point),
			Public::from_slice(&encrypted_point),
		);

		Ok((not_confirmed, task))
	}
}

impl DocumentKeyShadowRetrievalService {
	/// Parse common request log entry.
	pub fn parse_common_request_log(origin: &Address, raw_log: RawLog) -> Result<ServiceTask, String> {
		match service::events::document_key_common_retrieval_requested::parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyCommon(origin.clone(), l.server_key_id, l.requester)),
			Err(e) => Err(e.to_string())
		}
	}

	/// Parse personal request log entry.
	pub fn parse_personal_request_log(origin: &Address, raw_log: RawLog) -> Result<ServiceTask, String> {
		match service::events::document_key_personal_retrieval_requested::parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyPersonal(origin.clone(), l.server_key_id, H512::from_slice(&*l.requester_public))),
			Err(e) => Err(e.to_string())
		}
	}

	/// Check if response from key server is required.
	pub fn is_response_required(client: &dyn SecretStoreChain, contract_address: &Address, server_key_id: &ServerKeyId, requester: &Address, key_server: &Address) -> bool {
		// we're checking confirmation in Latest block, because we're interested in latest contract state here
		let (encoded, decoder) = service::functions::is_document_key_shadow_retrieval_response_required::call(*server_key_id, *requester, *key_server);
		match client.call_contract(BlockId::Latest, *contract_address, encoded) {
			Err(_) => true,
			Ok(data) => decoder.decode(&data).unwrap_or(true)
		}
	}

	/// Prepare publish common key transaction data.
	pub fn prepare_pubish_common_tx_data(server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: U256) -> Bytes {
		service::functions::document_key_common_retrieved::encode_input(*server_key_id, *requester, common_point.as_bytes().to_vec(), threshold)
	}

	/// Prepare publish personal key transaction data.
	pub fn prepare_pubish_personal_tx_data(client: &dyn SecretStoreChain, contract_address: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<Bytes, String> {
		let mut participants_mask = U256::default();
		for participant in participants {
			let participant_index = Self::map_key_server_address(client, contract_address, participant.clone())
				.map_err(|e| format!("Error searching for {} participant: {}", participant, e))?;
			participants_mask = participants_mask | (U256::one() << participant_index);
		}
		Ok(service::functions::document_key_personal_retrieved::encode_input(
			*server_key_id, *requester, participants_mask, decrypted_secret.as_bytes().to_vec(), shadow
		))
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(server_key_id: &ServerKeyId, requester: &Address) -> Bytes {
		service::functions::document_key_shadow_retrieval_error::encode_input(*server_key_id, *requester)
	}

	/// Read pending requests count.
	fn read_pending_requests_count(client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId) -> Result<U256, String> {
		let (encoded, decoder) = service::functions::document_key_shadow_retrieval_requests_count::call();
		decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())
	}

	/// Read pending request.
	fn read_pending_request(self_key_pair: &dyn SigningKeyPair, client: &dyn SecretStoreChain, contract_address: &Address, block: &BlockId, index: U256) -> Result<(bool, ServiceTask), String> {
		let self_address = public_to_address(self_key_pair.public());

		let (encoded, decoder) = service::functions::get_document_key_shadow_retrieval_request::call(index);
		let (server_key_id, requester, is_common_retrieval_completed) =
			decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let requester = Public::from_slice(&requester);
		let (encoded, decoder) = service::functions::is_document_key_shadow_retrieval_response_required::call(server_key_id, public_to_address(&requester), self_address);
		let not_confirmed = decoder.decode(&client.call_contract(*block, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		let task = match is_common_retrieval_completed {
			true => ServiceTask::RetrieveShadowDocumentKeyPersonal(
				*contract_address,
				server_key_id,
				requester,
			),
			false => ServiceTask::RetrieveShadowDocumentKeyCommon(
				*contract_address,
				server_key_id,
				public_to_address(&requester),
			),
		};

		Ok((not_confirmed, task))
	}

	/// Map from key server address to key server index.
	fn map_key_server_address(client: &dyn SecretStoreChain, contract_address: &Address, key_server: Address) -> Result<u8, String> {
		// we're checking confirmation in Latest block, because tx ,ust be appended to the latest state
		let (encoded, decoder) = service::functions::require_key_server::call(key_server);
		let index = decoder.decode(&client.call_contract(BlockId::Latest, *contract_address, encoded)?)
			.map_err(|e| e.to_string())?;

		if index > u8::max_value().into() {
			Err(format!("key server index is too big: {}", index))
		} else {
			let index: u32 = index.low_u32();
			Ok(index as u8)
		}
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
	use crypto::publickey::Public;
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

		fn read_logs(&self) -> Box<dyn Iterator<Item=ServiceTask>> {
			Box::new(self.logs.clone().into_iter())
		}

		fn read_pending_requests(&self) -> Box<dyn Iterator<Item=(bool, ServiceTask)>> {
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
