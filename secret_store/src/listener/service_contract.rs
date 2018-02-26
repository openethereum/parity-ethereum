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
use ethabi::RawLog;
use ethcore::filter::Filter;
use ethcore::log_entry::LocalizedLogEntry;
use ethcore::client::{Client, BlockChainClient, BlockId};
use ethkey::{Public, Signature, Secret, public_to_address};
use hash::keccak;
use bytes::Bytes;
use ethereum_types::{H256, U256, Address};
use listener::service_contract_listener::ServiceTask;
use trusted_client::TrustedClient;
use {ServerKeyId, NodeKeyPair, ContractAddress, EncryptedDocumentKey};

use_contract!(service, "Service", "res/service.json");

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Server key generation has been requested.
const SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyGenerationRequested(bytes32,address,uint256)";
/// Server key retrieval has been requested.
const SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRetrievalRequested(bytes32)";
/// Document key store has been requested.
const DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyStoreRequested(bytes32,address,bytes,bytes)";
/// Document key common part retrieval has been requested.
const DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyCommonRetrievalRequested(bytes32,address)";
/// Document key personal part retrieval has been requested.
const DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME: &'static [u8] = &*b"DocumentKeyPersonalRetrievalRequested(bytes32,bytes)";

/// Number of confirmations required before request can be processed.
const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

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
	fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String>;
	/// Publish server key generation error.
	fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish retrieved server key.
	fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String>;
	/// Publish server key retrieval error.
	fn publish_server_key_retrieval_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish stored document key.
	fn publish_stored_document_key(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish document key store error.
	fn publish_document_key_store_error(&self, server_key_id: &ServerKeyId) -> Result<(), String>;
	/// Publish retrieved document key common.
	fn publish_retrieved_document_key_common(&self, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String>;
	/// Publish retrieved document key personal.
	fn publish_retrieved_document_key_personal(&self, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String>;
	/// Publish document key store error.
	fn publish_document_key_retrieval_error(&self, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String>;
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

	/// Send transaction to the service contract.
	fn send_contract_transaction<C, P>(&self, server_key_id: &ServerKeyId, check: C, prepare_tx: P) -> Result<(), String>
		where C: FnOnce(&Client, &Address, &service::Service, &ServerKeyId, &Address) -> bool,
			P: FnOnce(&service::Service) -> Result<Bytes, String> {
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
		let self_address = public_to_address(self.self_key_pair.public());
		if !check(&*client, &contract_address, &data.contract, server_key_id, &self_address) {
			return Ok(());
		}

		// prepare transaction data
		let transaction_data = prepare_tx(&data.contract)?;

		// send transaction
		client.transact_contract(
			data.contract_address,
			transaction_data
		).map_err(|e| format!("{}", e))?;

		Ok(())
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
					*SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH,
					*SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH,
					*DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH,
					*DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH,
					*DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH,
				]),
			],
			limit: None,
		});

		let data = self.data.read();
		Box::new(request_logs.into_iter()
			.filter_map(|log| {
				let raw_log: RawLog = (log.entry.topics.into_iter().map(|t| t.0.into()).collect(), log.entry.data).into();
				if raw_log.topics[0] == *SERVER_KEY_GENERATION_REQUESTED_EVENT_NAME_HASH {
					ServerKeyGenerationService::parse_log(&data.contract, raw_log)
				} else if raw_log.topics[0] == *SERVER_KEY_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					ServerKeyRetrievalService::parse_log(&data.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_STORE_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyStoreService::parse_log(&data.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_COMMON_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_common_request_log(&data.contract, raw_log)
				} else if raw_log.topics[0] == *DOCUMENT_KEY_PERSONAL_PART_RETRIEVAL_REQUESTED_EVENT_NAME_HASH {
					DocumentKeyShadowRetrievalService::parse_personal_request_log(&data.contract, raw_log)
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

	fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, ServerKeyGenerationService::is_confirmed, |service|
			Ok(ServerKeyGenerationService::prepare_pubish_tx_data(service, server_key_id, &server_key))
		)
	}

	fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, ServerKeyGenerationService::is_confirmed, |service|
			Ok(ServerKeyGenerationService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, ServerKeyRetrievalService::is_confirmed, |service|
			Ok(ServerKeyRetrievalService::prepare_pubish_tx_data(service, server_key_id, server_key, threshold))
		)
	}

	fn publish_server_key_retrieval_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, ServerKeyRetrievalService::is_confirmed, |service|
			Ok(ServerKeyRetrievalService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_stored_document_key(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, DocumentKeyStoreService::is_confirmed, |service|
			Ok(DocumentKeyStoreService::prepare_pubish_tx_data(service, server_key_id))
		)
	}

	fn publish_document_key_store_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, DocumentKeyStoreService::is_confirmed, |service|
			Ok(DocumentKeyStoreService::prepare_error_tx_data(service, server_key_id))
		)
	}

	fn publish_retrieved_document_key_common(&self, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, |client, contract_address, contract, server_key_id, authority|
			DocumentKeyShadowRetrievalService::is_confirmed(client, contract_address, contract, server_key_id, requester, authority),
		|service|
			Ok(DocumentKeyShadowRetrievalService::prepare_pubish_common_tx_data(service, server_key_id, requester, common_point, threshold))
		)
	}

	fn publish_retrieved_document_key_personal(&self, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, |_, _, _, _, _| true,
		move |service|
			Ok(DocumentKeyShadowRetrievalService::prepare_pubish_personal_tx_data(service, server_key_id, requester, participants, decrypted_secret, shadow))
		)
	}

	fn publish_document_key_retrieval_error(&self, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
		self.send_contract_transaction(server_key_id, |client, contract_address, contract, server_key_id, authority|
			DocumentKeyShadowRetrievalService::is_confirmed(client, contract_address, contract, server_key_id, requester, authority),
		|service|
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

/// Get hash of the last block with at least n confirmations.
fn get_confirmed_block_hash(client: &Client, confirmations: u64) -> Option<H256> {
	client.block_number(BlockId::Latest)
		.map(|b| b.saturating_sub(confirmations))
		.and_then(|b| client.block_hash(BlockId::Number(b)))
}

impl ServerKeyGenerationService {
	/// Parse request log entry.
	pub fn parse_log(contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().server_key_generation_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::GenerateServerKey(l.server_key_id, l.author, parse_threshold(l.threshold)?)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if request is confirmed by authority.
	pub fn is_confirmed(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, authority: &Address) -> bool {
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.get_server_key_generation_request_confirmation_status()
			.call(*server_key_id, authority.clone(), &do_call)
			.unwrap_or(false)
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
}

impl ServerKeyRetrievalService {
	/// Parse request log entry.
	pub fn parse_log(contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().server_key_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveServerKey(l.server_key_id)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if request is confirmed by authority.
	pub fn is_confirmed(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, authority: &Address) -> bool {
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.get_server_key_retrieval_request_confirmation_status()
			.call(*server_key_id, authority.clone(), &do_call)
			.unwrap_or(false)
	}

	/// Prepare publish key transaction data.
	pub fn prepare_pubish_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, server_key_public: Public, threshold: usize) -> Bytes {
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
}

impl DocumentKeyStoreService {
	/// Parse request log entry.
	pub fn parse_log(contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_store_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::StoreDocumentKey(l.server_key_id, l.author, (*l.common_point).into(), (*l.encrypted_point).into())),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if request is confirmed by authority.
	pub fn is_confirmed(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, authority: &Address) -> bool {
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.get_document_key_store_request_confirmation_status()
			.call(*server_key_id, authority.clone(), &do_call)
			.unwrap_or(false)
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
}

impl DocumentKeyShadowRetrievalService {
	/// Parse common request log entry.
	pub fn parse_common_request_log(contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_common_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyCommon(l.server_key_id, l.requester)),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Parse personal request log entry.
	pub fn parse_personal_request_log(contract: &service::Service, raw_log: RawLog) -> Result<ServiceTask, String> {
		let event = contract.events().document_key_personal_retrieval_requested();
		match event.parse_log(raw_log) {
			Ok(l) => Ok(ServiceTask::RetrieveShadowDocumentKeyPersonal(l.server_key_id, (*l.requester_public).into())),
			Err(e) => Err(format!("{}", e)),
		}
	}

	/// Check if request is confirmed by authority.
	pub fn is_confirmed(client: &Client, contract_address: &Address, contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, authority: &Address) -> bool {
		let do_call = |data| client.call_contract(BlockId::Latest, *contract_address, data);
		contract.functions()
			.get_document_key_shadow_retrieval_request_confirmation_status()
			.call(*server_key_id, *requester, authority.clone(), &do_call)
			.unwrap_or(false)
	}

	/// Prepare publish common key transaction data.
	pub fn prepare_pubish_common_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Bytes {
		contract.functions()
			.document_key_common_retrieved()
			.input(*server_key_id, *requester, common_point.to_vec(), threshold)
	}

	/// Prepare publish personal key transaction data.
	pub fn prepare_pubish_personal_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Bytes {
		let participants: Vec<_> = participants.iter().cloned().collect();
		contract.functions()
			.document_key_personal_retrieved()
			.input(*server_key_id, *requester, participants, decrypted_secret.to_vec(), shadow)
	}

	/// Prepare error transaction data.
	pub fn prepare_error_tx_data(contract: &service::Service, server_key_id: &ServerKeyId, requester: &Address) -> Bytes {
		contract.functions()
			.document_key_shadow_retrieval_error()
			.input(*server_key_id, *requester)
	}
}

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
	use bytes::Bytes;
	use ethkey::{Public, Secret};
	use ethereum_types::{Address, H256};
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

		fn publish_generated_server_key(&self, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String> {
			self.generated_server_keys.lock().push((server_key_id.clone(), server_key.clone()));
			Ok(())
		}

		fn publish_server_key_generation_error(&self, server_key_id: &ServerKeyId) -> Result<(), String> {
			self.server_keys_generation_failures.lock().push(server_key_id.clone());
			Ok(())
		}

		fn publish_retrieved_server_key(&self, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
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

		fn publish_retrieved_document_key_common(&self, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
			unimplemented!()
		}

		fn publish_retrieved_document_key_personal(&self, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
			unimplemented!()
		}

		fn publish_document_key_retrieval_error(&self, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
			unimplemented!()
		}
	}
}
