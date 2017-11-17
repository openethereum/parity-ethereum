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

use std::collections::VecDeque;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use parking_lot::{RwLock, Mutex, Condvar};	
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify};
use native_contracts::SecretStoreService;
use ethkey::{Random, Generator, Public, Signature, sign};
use bytes::Bytes;
use hash::keccak;
use bigint::hash::H256;
use util::Address;
use {ServerKeyId, NodeKeyPair, KeyServer};

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Key server has been added to the set.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32)";

lazy_static! {
	static ref SERVER_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_REQUESTED_EVENT_NAME);
}

/// SecretStore <-> Authority connector responsible for:
/// 1. listening for new requests on SecretStore contract
/// 2. redirecting requests to key server
/// 3. publishing response on SecretStore contract
pub struct ServiceContractListener {
	/// Service contract listener data.
	data: Arc<ServiceContractListenerData>,
	/// Service thread handle.
	service_handle: Option<thread::JoinHandle<()>>,
}

/// Service contract listener data.
struct ServiceContractListenerData {
	/// Contract (currently used for parameters encoding only).
	pub contract: RwLock<SecretStoreService>,
	/// Blockchain client.
	pub client: Weak<Client>,
	/// Key server reference.
	pub key_server: Arc<KeyServer>,
	/// This node key pair.
	pub self_key_pair: Arc<NodeKeyPair>,
	/// Service tasks queue.
	pub tasks_queue: Arc<TasksQueue>,
}

/// Service tasks queue.
struct TasksQueue {
	/// Are we closing currently.
	is_shutdown: AtomicBool,
	/// Service event.
	service_event: Condvar,
	/// Service tasks queue.
	service_tasks: Mutex<VecDeque<ServiceTask>>,
}

/// Service task.
enum ServiceTask {
	/// Generate server key (server_key_id, threshold).
	GenerateServerKey(H256, H256),
	/// Shutdown listener.
	Shutdown,
}

impl ServiceContractListener {
	pub fn new(client: &Arc<Client>, key_server: Arc<KeyServer>, self_key_pair: Arc<NodeKeyPair>) -> Arc<ServiceContractListener> {
		let contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()).unwrap_or_default();
		let data = Arc::new(ServiceContractListenerData {
			contract: RwLock::new(SecretStoreService::new(contract_addr)),
			client: Arc::downgrade(client),
			key_server: key_server,
			self_key_pair: self_key_pair,
			tasks_queue: Arc::new(TasksQueue::new()),
		});

		let service_thread_data = data.clone();
		let service_handle = thread::spawn(move || Self::run_service_thread(service_thread_data));
		let contract = Arc::new(ServiceContractListener {
			data: data,
			service_handle: Some(service_handle),
		});
		client.add_notify(contract.clone());
		contract
	}

	fn process_service_contract_events(&self, client: &Client, service_contract: Address, blocks: Vec<H256>) {
		debug_assert!(!blocks.is_empty());

		// TODO: is blocks guaranteed to be ordered here?
		// TODO: logs() is called from notify() thread - is it ok?
		let request_logs = client.logs(Filter {
			from_block: BlockId::Hash(blocks.first().expect("!block.is_empty(); qed").clone()),
			to_block: BlockId::Hash(blocks.last().expect("!block.is_empty(); qed").clone()),
			address: Some(vec![service_contract]),
			topics: vec![
				Some(vec![*SERVER_KEY_REQUESTED_EVENT_NAME_HASH]),
				None,
				None,
				None,
			],
			limit: None,
		});

		self.data.tasks_queue.push(request_logs.into_iter()
			.filter_map(|r| match r.entry.topics.len() {
				3 => Some(ServiceTask::GenerateServerKey(
					r.entry.topics[1],
					r.entry.topics[2],
				)),
				l @ _ => {
					warn!(target: "secretstore", "Ignoring ServerKeyRequested event with wrong number of params {}", l);
					None
				},
			}));
	}

	fn run_service_thread(data: Arc<ServiceContractListenerData>) {
		loop {
			let task = data.tasks_queue.wait();

			match task {
				ServiceTask::GenerateServerKey(server_key_id, threshold) => {
					match Self::generate_server_key(&data, &server_key_id, &threshold)
						.and_then(|server_key| Self::publish_server_key(&data, &server_key_id, &server_key)) {
						Ok(_) => trace!(target: "secretstore", "GenerateServerKey({}, {}) request has completed",
							server_key_id, threshold),
						Err(error) => warn!(target: "secretstore", "GenerateServerKey({}, {}) request has failed with: {}",
							server_key_id, threshold, error),
					}
				},
				ServiceTask::Shutdown => break,
			}
		}
	}

	fn generate_server_key(data: &Arc<ServiceContractListenerData>, server_key_id: &ServerKeyId, threshold: &H256) -> Result<Public, String> {
		let threshold_num = threshold.low_u64();
		if threshold != &threshold_num.into() || threshold_num >= ::std::usize::MAX as u64 {
			return Err(format!("invalid threshold {:?}", threshold));
		}

		// TODO: if this server key is going to be used for document key generation later, author must
		// be specified from outside
		let author_key = Random.generate().map_err(|e| format!("{}", e))?;
		let server_key_id_signature = sign(author_key.secret(), server_key_id).map_err(|e| format!("{}", e))?;
		data.key_server.generate_key(server_key_id, &server_key_id_signature, threshold_num as usize)
			.map_err(Into::into)

	}

	fn publish_server_key(data: &Arc<ServiceContractListenerData>, server_key_id: &ServerKeyId, server_key: &Public) -> Result<(), String> {
		let server_key_hash = keccak(server_key);
		let signed_server_key = data.self_key_pair.sign(&server_key_hash).map_err(|e| format!("{}", e))?;
		let signed_server_key: Signature = signed_server_key.into_electrum().into();
		let transaction_data = data.contract.read().encode_server_key_generated_input(server_key_id.clone(),
			server_key.to_vec(),
			signed_server_key.v(),
			signed_server_key.r().into(),
			signed_server_key.s().into()
		)?;

		let contract = data.contract.read();
		if contract.address != Default::default() {
			if let Some(client) = data.client.upgrade() {
				client.transact_contract(
					contract.address.clone(),
					transaction_data
				).map_err(|e| format!("{}", e))?;
			} // else we will read this in the next refresh cycle
		}

		Ok(())
	}
}

impl Drop for ServiceContractListener {
	fn drop(&mut self) {
		if let Some(service_handle) = self.service_handle.take() {
			self.data.tasks_queue.shutdown();
			// ignore error as we are already closing
			let _ = service_handle.join();
		}
	}
}

impl ChainNotify for ServiceContractListener {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		if !enacted.is_empty() {
			if let Some(client) = self.data.client.upgrade() {
				if let Some(service_contract_addr) = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()) {
					if self.data.contract.read().address != service_contract_addr {
						*self.data.contract.write() = SecretStoreService::new(service_contract_addr.clone());
					}
					self.process_service_contract_events(&*client, service_contract_addr, enacted);
				}
			}

			//self.contract.lock().update(enacted)
		}
	}
}

impl TasksQueue {
	pub fn new() -> Self {
		TasksQueue {
			is_shutdown: AtomicBool::new(false),
			service_event: Condvar::new(),
			service_tasks: Mutex::new(VecDeque::new()),
		}
	}

	pub fn shutdown(&self) {
		self.is_shutdown.store(true, Ordering::Release);
		self.service_event.notify_all();
	}

	pub fn push<I>(&self, tasks: I) where I: Iterator<Item=ServiceTask> {
		let mut service_tasks = self.service_tasks.lock();
		service_tasks.extend(tasks);
		self.service_event.notify_all();
	}

	pub fn wait(&self) -> ServiceTask {
		if self.is_shutdown.load(Ordering::Release) {
			return ServiceTask::Shutdown;
		}

		let mut service_tasks = self.service_tasks.lock();
		if service_tasks.is_empty() {
			self.service_event.wait(&mut service_tasks);
			if self.is_shutdown.load(Ordering::Release) {
				return ServiceTask::Shutdown;
			}
		}

		service_tasks.pop_front()
			.expect("service_event is only fired when there are new tasks or is_shutdown == true; is_shutdown == false; qed")
	}
}
