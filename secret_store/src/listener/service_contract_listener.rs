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

use std::collections::{VecDeque, HashSet};
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use futures::{future, Future};
use parking_lot::{RwLock, Mutex, Condvar};	
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify};
use ethkey::{Random, Generator, Public, Signature, sign, public_to_address};
use ethsync::SyncProvider;
use native_contracts::SecretStoreService;
use bytes::Bytes;
use hash::keccak;
use bigint::hash::H256;
use bigint::prelude::U256;
use util::Address;
use key_server_set::KeyServerSet;
use key_server_cluster::{ClusterClient, ClusterSessionsListener, ClusterSession};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, Session as GenerationSessionTrait};
use {ServerKeyId, NodeKeyPair, KeyServer};

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Key server has been added to the set.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32,uint256)";

/// Retry interval (in blocks). Every RETRY_INTEVAL_BLOCKS blocks each KeyServer reads pending requests from
/// service contract && tries to re-execute. The reason to have this mechanism is primarily because keys
/// servers set change takes a lot of time + there could be some races, when blocks are coming to different
/// KS at different times. This isn't intended to fix && respond to general session errors!
const RETRY_INTEVAL_BLOCKS: usize = 30;

/// Max failed retry requests (in single retry interval). The reason behind this constant is that if several
/// pending requests have failed, then most probably other will fail too.
const MAX_FAILED_RETRY_REQUESTS: usize = 1;

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
	/// Blocks since last retry.
	pub last_retry: AtomicUsize,
	/// Retry-related data.
	pub retry_data: Mutex<ServiceContractRetryData>,
	/// Contract.
	pub contract: RwLock<SecretStoreService>,
	/// Blockchain client.
	pub client: Weak<Client>,
	/// Sync provider.
	pub sync: Weak<SyncProvider>,
	/// Key server reference.
	pub key_server: Arc<KeyServer>,
	/// This node key pair.
	pub self_key_pair: Arc<NodeKeyPair>,
	/// Key servers set.
	pub key_servers_set: Arc<KeyServerSet>,
	/// Service tasks queue.
	pub tasks_queue: Arc<TasksQueue>,
}

/// Retry-related data.
#[derive(Default)]
struct ServiceContractRetryData {
	/// Server keys, which we have generated (or tried to generate) since last retry moment.
	pub generated_keys: HashSet<ServerKeyId>,
}

/// Service tasks queue.
struct TasksQueue {
	/// Service event.
	service_event: Condvar,
	/// Service tasks queue.
	service_tasks: Mutex<VecDeque<ServiceTask>>,
}

/// Service task.
#[derive(Debug)]
enum ServiceTask {
	/// Retry all 'stalled' tasks.
	Retry,
	/// Generate server key (server_key_id, threshold).
	GenerateServerKey(H256, H256),
	/// Confirm server key (server_key_id).
	ConfirmServerKey(H256),
	/// Shutdown listener.
	Shutdown,
}

impl ServiceContractListener {
	pub fn new(client: &Arc<Client>, sync: &Arc<SyncProvider>, key_server: Arc<KeyServer>, cluster: Arc<ClusterClient>, self_key_pair: Arc<NodeKeyPair>, key_servers_set: Arc<KeyServerSet>) -> Arc<ServiceContractListener> {
		let contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned())
			.map(|a| {
				trace!(target: "secretstore", "{}: installing service contract from address {}", self_key_pair.public(), a);
				a
			})
			.unwrap_or_default();

		let is_syncing = sync.status().is_syncing(client.queue_info());
		let data = Arc::new(ServiceContractListenerData {
			last_retry: AtomicUsize::new(0),
			retry_data: Default::default(),
			contract: RwLock::new(SecretStoreService::new(contract_addr)),
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			key_server: key_server,
			self_key_pair: self_key_pair,
			key_servers_set: key_servers_set,
			tasks_queue: Arc::new(TasksQueue::new()),
		});

		// retry on restart
		if !is_syncing {
			data.tasks_queue.push(::std::iter::once(ServiceTask::Retry));
		}

		let service_thread_data = data.clone();
		let service_handle = thread::spawn(move || Self::run_service_thread(service_thread_data));
		let contract = Arc::new(ServiceContractListener {
			data: data,
			service_handle: Some(service_handle),
		});
		client.add_notify(contract.clone());
		cluster.add_generation_listener(contract.clone());
		contract
	}

	fn process_service_contract_events(&self, client: &Client, service_contract: Address, blocks: Vec<H256>) {
		debug_assert!(!blocks.is_empty());

		// read server key generation requests
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

		// schedule correct requests if they're intended to be processed by this KeyServer
		self.data.tasks_queue.push(request_logs.into_iter()
			.filter_map(|r| match r.entry.topics.len() {
				3 if is_processed_by_this_key_server(&*self.data.key_servers_set, &*self.data.self_key_pair, &r.entry.topics[1]) => {
					Some(ServiceTask::GenerateServerKey(
						r.entry.topics[1],
						r.entry.topics[2],
					))
				},
				3 => None,
				l @ _ => {
					warn!(target: "secretstore", "Ignoring ServerKeyRequested event with wrong number of params {}", l);
					None
				},
			}));
	}

	fn run_service_thread(data: Arc<ServiceContractListenerData>) {
		loop {
			let task = data.tasks_queue.wait();
			trace!(target: "secretstore", "{}: processing {:?} task",data.self_key_pair.public(), task);

			match task {
				ServiceTask::Shutdown => break,
				task @ _ => {
					// the only possible reaction to an error is a trace && it is already happened
					let _ = Self::process_service_task(&data, task);
				},
			};
		}
	}

	fn process_service_task(data: &Arc<ServiceContractListenerData>, task: ServiceTask) -> Result<(), String> {
		match task {
			ServiceTask::Retry =>
				Self::retry_pending_requests(&data)
					.map(|processed_requests| {
						if processed_requests != 0 {
							trace!(target: "secretstore", "{}: successfully retried {} pending requests",
								data.self_key_pair.public(), processed_requests);
						}
						()
					})
					.map_err(|error| {
						warn!(target: "secretstore", "{}: retrying pending requests has failed with: {}",
							data.self_key_pair.public(), error);
						error
					}),
			ServiceTask::ConfirmServerKey(_) => Err("not implemented".to_owned()), // TODO
			ServiceTask::GenerateServerKey(server_key_id, threshold) => {
				data.retry_data.lock().generated_keys.insert(server_key_id.clone());
				Self::generate_server_key(&data, &server_key_id, &threshold)
					.and_then(|server_key| Self::publish_server_key(&data, &server_key_id, &server_key))
					.map(|_| {
						trace!(target: "secretstore", "{}: started processing GenerateServerKey({}, {}) request",
							data.self_key_pair.public(), server_key_id, threshold);
						()
					})
					.map_err(|error| {
						warn!(target: "secretstore", "{}: failed to start processing GenerateServerKey({}, {}) request with: {}",
							data.self_key_pair.public(), server_key_id, threshold, error);
						error
					})
			},
			ServiceTask::Shutdown => unreachable!("it must be checked outside"),
		}
	}

	fn retry_pending_requests(data: &Arc<ServiceContractListenerData>) -> Result<usize, String> {
		let client = data.client.upgrade().ok_or("client is required".to_owned())?;
		let retry_data = ::std::mem::replace(&mut *data.retry_data.lock(), Default::default());
		let contract = data.contract.read();

		// it is only possible when contract address is set
		if contract.address == Default::default() {
			return Ok(0);
		}

		let do_call = |a, d| future::done(client.call_contract(BlockId::Latest, a, d));
		let generate_server_key_requests_count = contract.server_key_generation_requests_count(&do_call).wait()?;
		let mut generate_server_key_request_index = 0.into();
		let mut failed_requests = 0;
		let mut processed_requests = 0;
		loop {
			if generate_server_key_request_index >= generate_server_key_requests_count {
				break;
			}

			// read request from the contract
			let (server_key_id, threshold, is_confirmed) = contract.get_server_key_generation_request(&do_call,
				public_to_address(data.self_key_pair.public()),
				generate_server_key_request_index).wait()?;
			generate_server_key_request_index = generate_server_key_request_index + 1.into();

			// only process requests, which we haven't confirmed yet
			if is_confirmed {
				continue;
			}

			// only process request, which haven't been processed recently
			// there could be a lag when we've just generated server key && retrying on the same block
			// (or before our tx is mined) - state is not updated yet
			if retry_data.generated_keys.contains(&server_key_id){
				continue;
			}

			// process request
			let is_own_request = is_processed_by_this_key_server(&*data.key_servers_set, &*data.self_key_pair, &server_key_id);
			let request_result = Self::process_service_task(data, match is_own_request {
				true => ServiceTask::GenerateServerKey(server_key_id, threshold.into()),
				false => ServiceTask::ConfirmServerKey(server_key_id),
			});

			// process request result
			match request_result {
				Ok(_) => processed_requests += 1,
				Err(_) => {
					failed_requests += 1;
					if failed_requests > MAX_FAILED_RETRY_REQUESTS {
						return Err("too many failed requests".into());
					}
				},
			}
		}
		Ok(processed_requests)
	}

	fn generate_server_key(data: &Arc<ServiceContractListenerData>, server_key_id: &ServerKeyId, threshold: &H256) -> Result<Public, String> {
		let threshold_num = threshold.low_u64();
		if threshold != &threshold_num.into() || threshold_num >= ::std::usize::MAX as u64 {
			return Err(format!("invalid threshold {:?}", threshold));
		}

		// key server expects signed server_key_id in server_key_generation procedure
		// only signer could store document key for this server key later
		// => this API (server key generation) is not suitable for usage in encryption via contract endpoint
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
		let enacted_len = enacted.len();
		if enacted_len != 0 {
			if let (Some(client), Some(sync)) = (self.data.client.upgrade(), self.data.sync.upgrade()) {
				// do nothing until synced
				if sync.status().is_syncing(client.queue_info()) {
					return;
				}

				// update contract address from registry
				if let Some(service_contract_addr) = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()) {
					if self.data.contract.read().address != service_contract_addr {
						trace!(target: "secretstore", "{}: installing service contract from address {}", self.data.self_key_pair.public(), service_contract_addr);
						*self.data.contract.write() = SecretStoreService::new(service_contract_addr.clone());
					}

					// and process contract events
					self.process_service_contract_events(&*client, service_contract_addr, enacted);
				}

				// schedule retry if received enough blocks since last retry
				// it maybe inaccurate when switching syncing/synced states, but that's ok
				if self.data.last_retry.fetch_add(enacted_len, Ordering::Relaxed) >= RETRY_INTEVAL_BLOCKS {
					self.data.tasks_queue.push(::std::iter::once(ServiceTask::Retry));
					self.data.last_retry.store(0, Ordering::Relaxed);
				}
			}
		}
	}
}

impl ClusterSessionsListener<GenerationSession> for ServiceContractListener {
	fn on_session_inserted(&self, _session: Arc<GenerationSession>) {
	}

	fn on_session_removed(&self, session: Arc<GenerationSession>) {
		// TODO: only start if session started via the contract
		// only publish when the session is started by another node
		if !is_processed_by_this_key_server(&*self.data.key_servers_set, &*self.data.self_key_pair, &session.id()) {
			session.wait(Some(Default::default()))
				.map_err(|e| format!("{}", e))
				.and_then(|server_key| Self::publish_server_key(&self.data, &session.id(), &server_key))
				.map(|_| {
					trace!(target: "secretstore", "{}: completed foreign GenerateServerKey({}) request",
						self.data.self_key_pair.public(), session.id());
					()
				})
				.map_err(|error| {
					warn!(target: "secretstore", "{}: failed to process GenerateServerKey({}) request with: {}",
						self.data.self_key_pair.public(), session.id(), error);
					error
				});
		}
	}
}

impl TasksQueue {
	pub fn new() -> Self {
		TasksQueue {
			service_event: Condvar::new(),
			service_tasks: Mutex::new(VecDeque::new()),
		}
	}

	pub fn shutdown(&self) {
		let mut service_tasks = self.service_tasks.lock();
		service_tasks.push_front(ServiceTask::Shutdown);
		self.service_event.notify_all();
	}

	pub fn push<I>(&self, tasks: I) where I: Iterator<Item=ServiceTask> {
		let mut service_tasks = self.service_tasks.lock();
		service_tasks.extend(tasks);
		if !service_tasks.is_empty() {
			self.service_event.notify_all();
		}
	}

	pub fn wait(&self) -> ServiceTask {
		let mut service_tasks = self.service_tasks.lock();
		if service_tasks.is_empty() {
			self.service_event.wait(&mut service_tasks);
		}

		service_tasks.pop_front()
			.expect("service_event is only fired when there are new tasks or is_shutdown == true; is_shutdown == false; qed")
	}
}

/// Returns true when session, related to `server_key_id` must be started on this KeyServer.
fn is_processed_by_this_key_server(key_servers_set: &KeyServerSet, self_key_pair: &NodeKeyPair, server_key_id: &H256) -> bool {
	let servers = key_servers_set.get();
	let total_servers_count = servers.len();
	if total_servers_count == 0 {
		return false;
	}
	let this_server_index = match servers.keys().enumerate().find(|&(_, s)| s == self_key_pair.public()) {
		Some((index, _)) => index,
		None => return false,
	};

	let server_key_id_value: U256 = server_key_id.into();
	let range_interval = U256::max_value() / total_servers_count.into();
	let range_begin = (range_interval + 1.into()) * this_server_index.into();
	let range_end = range_begin.saturating_add(range_interval);

	server_key_id_value >= range_begin && server_key_id_value <= range_end
}

#[cfg(test)]
mod tests {
	use ethkey::{Random, Generator, KeyPair};
	use key_server_set::tests::MapKeyServerSet;
	use PlainNodeKeyPair;
	use super::is_processed_by_this_key_server;

	#[test]
	fn is_not_processed_by_this_key_server_with_zero_servers() {
		assert_eq!(is_processed_by_this_key_server(
			&MapKeyServerSet::default(),
			&PlainNodeKeyPair::new(Random.generate().unwrap()),
			&Default::default()), false);
	}

	#[test]
	fn is_not_processed_by_this_key_server_when_not_a_part_of_servers_set() {
		assert_eq!(is_processed_by_this_key_server(
			&MapKeyServerSet::new(vec![
				(Random.generate().unwrap().public().clone(), "127.0.0.1:8080".parse().unwrap())
			].into_iter().collect()),
			&PlainNodeKeyPair::new(Random.generate().unwrap()),
			&Default::default()), false);
	}

	#[test]
	fn is_processed_by_this_key_server_in_set_of_3() {
		// servers set is ordered && server range depends on index of this server
		let servers_set = MapKeyServerSet::new(vec![
			// secret: 0000000000000000000000000000000000000000000000000000000000000001
			("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000002
			("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee51ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000003
			("f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
		].into_iter().collect());

		// 1st server: process hashes [0x0; 0x555...555]
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"3000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"5555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"5555555555555555555555555555555555555555555555555555555555555556".parse().unwrap()), false);

		// 2nd server: process hashes from 0x555...556 to 0xaaa...aab
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"5555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"5555555555555555555555555555555555555555555555555555555555555556".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"7555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), false);

		// 3rd server: process hashes from 0x800...000 to 0xbff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"daaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
	}

	#[test]
	fn is_processed_by_this_key_server_in_set_of_4() {
		// servers set is ordered && server range depends on index of this server
		let servers_set = MapKeyServerSet::new(vec![
			// secret: 0000000000000000000000000000000000000000000000000000000000000001
			("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000002
			("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee51ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000004
			("e493dbf1c10d80f3581e4904930b1404cc6c13900ee0758474fa94abe8c4cd1351ed993ea0d455b75642e2098ea51448d967ae33bfbdfe40cfe97bdc47739922".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000003
			("f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
		].into_iter().collect());

		// 1st server: process hashes [0x0; 0x3ff...ff]
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"2000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"4000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 2nd server: process hashes from 0x400...000 to 0x7ff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"4000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"6000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"8000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 3rd server: process hashes from 0x800...000 to 0xbff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000004".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"8000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"a000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"bfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"c000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 4th server: process hashes from 0xc00...000 to 0xfff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"bfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"c000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"e000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, &key_pair,
			&"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
	}
}
