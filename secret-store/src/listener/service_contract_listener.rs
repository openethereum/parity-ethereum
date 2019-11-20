// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use bytes::Bytes;
use crypto::publickey::{Public, public_to_address};
use ethereum_types::{H256, U256, Address, BigEndianHash as _};
use key_server_set::KeyServerSet;
use key_server_cluster::{NodeId, ClusterClient, ClusterSessionsListener, ClusterSession};
use key_server_cluster::math;
use key_server_cluster::generation_session::SessionImpl as GenerationSession;
use key_server_cluster::encryption_session::{check_encrypted_data, update_encrypted_data};
use key_server_cluster::decryption_session::SessionImpl as DecryptionSession;
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSession,
	IsolatedSessionTransport as KeyVersionNegotiationTransport, FailedContinueAction};
use key_storage::KeyStorage;
use parking_lot::Mutex;
use acl_storage::AclStorage;
use listener::service_contract::ServiceContract;
use listener::tasks_queue::TasksQueue;
use {ServerKeyId, Error};
use blockchain::{NewBlocksNotify, SigningKeyPair};

/// Retry interval (in blocks). Every RETRY_INTERVAL_BLOCKS blocks each KeyServer reads pending requests from
/// service contract && tries to re-execute. The reason to have this mechanism is primarily because keys
/// servers set change takes a lot of time + there could be some races, when blocks are coming to different
/// KS at different times. This isn't intended to fix && respond to general session errors!
const RETRY_INTERVAL_BLOCKS: usize = 30;

/// Max failed retry requests (in single retry interval). The reason behind this constant is that if several
/// pending requests have failed, then most probably other will fail too.
const MAX_FAILED_RETRY_REQUESTS: usize = 1;

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

/// Service contract listener parameters.
pub struct ServiceContractListenerParams {
	/// Service contract.
	pub contract: Arc<dyn ServiceContract>,
	/// This node key pair.
	pub self_key_pair: Arc<dyn SigningKeyPair>,
	/// Key servers set.
	pub key_server_set: Arc<dyn KeyServerSet>,
	/// ACL storage reference.
	pub acl_storage: Arc<dyn AclStorage>,
	/// Cluster reference.
	pub cluster: Arc<dyn ClusterClient>,
	/// Key storage reference.
	pub key_storage: Arc<dyn KeyStorage>,
}

/// Service contract listener data.
struct ServiceContractListenerData {
	/// Blocks since last retry.
	pub last_retry: AtomicUsize,
	/// Retry-related data.
	pub retry_data: Mutex<ServiceContractRetryData>,
	/// Service tasks queue.
	pub tasks_queue: Arc<TasksQueue<ServiceTask>>,
	/// Service contract.
	pub contract: Arc<dyn ServiceContract>,
	/// ACL storage reference.
	pub acl_storage: Arc<dyn AclStorage>,
	/// Cluster client reference.
	pub cluster: Arc<dyn ClusterClient>,
	/// This node key pair.
	pub self_key_pair: Arc<dyn SigningKeyPair>,
	/// Key servers set.
	pub key_server_set: Arc<dyn KeyServerSet>,
	/// Key storage reference.
	pub key_storage: Arc<dyn KeyStorage>,

}

/// Retry-related data.
#[derive(Default)]
struct ServiceContractRetryData {
	/// Server keys, which we have 'touched' since last retry.
	pub affected_server_keys: HashSet<ServerKeyId>,
	/// Document keys + requesters, which we have 'touched' since last retry.
	pub affected_document_keys: HashSet<(ServerKeyId, Address)>,
}

/// Service task.
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceTask {
	/// Retry all 'stalled' tasks.
	Retry,
	/// Generate server key (origin, server_key_id, author, threshold).
	GenerateServerKey(Address, ServerKeyId, Address, usize),
	/// Retrieve server key (origin, server_key_id).
	RetrieveServerKey(Address, ServerKeyId),
	/// Store document key (origin, server_key_id, author, common_point, encrypted_point).
	StoreDocumentKey(Address, ServerKeyId, Address, Public, Public),
	/// Retrieve common data of document key (origin, server_key_id, requester).
	RetrieveShadowDocumentKeyCommon(Address, ServerKeyId, Address),
	/// Retrieve personal data of document key (origin, server_key_id, requester).
	RetrieveShadowDocumentKeyPersonal(Address, ServerKeyId, Public),
	/// Shutdown listener.
	Shutdown,
}

impl ServiceContractListener {
	/// Create new service contract listener.
	pub fn new(params: ServiceContractListenerParams) -> Result<Arc<ServiceContractListener>, Error> {
		let data = Arc::new(ServiceContractListenerData {
			last_retry: AtomicUsize::new(0),
			retry_data: Default::default(),
			tasks_queue: Arc::new(TasksQueue::new()),
			contract: params.contract,
			acl_storage: params.acl_storage,
			cluster: params.cluster,
			self_key_pair: params.self_key_pair,
			key_server_set: params.key_server_set,
			key_storage: params.key_storage,
		});

		// we are not starting thread when in test mode
		let service_handle = if cfg!(test) {
			None
		} else {
			let service_thread_data = data.clone();
			Some(thread::Builder::new().name("ServiceContractListener".into()).spawn(move ||
				Self::run_service_thread(service_thread_data)).map_err(|e| Error::Internal(format!("{}", e)))?)
		};
		let contract = Arc::new(ServiceContractListener {
			data: data,
			service_handle: service_handle,
		});
		contract.data.cluster.add_generation_listener(contract.clone());
		contract.data.cluster.add_decryption_listener(contract.clone());
		contract.data.cluster.add_key_version_negotiation_listener(contract.clone());
		Ok(contract)
	}

	/// Process incoming events of service contract.
	fn process_service_contract_events(&self) {
		// shortcut: do not process events if we're isolated from the cluster
		if self.data.key_server_set.is_isolated() {
			return;
		}

		self.data.tasks_queue.push_many(self.data.contract.read_logs()
			.filter_map(|task| Self::filter_task(&self.data, task)));
	}

	/// Filter service task. Only returns Some if task must be executed by this server.
	fn filter_task(data: &Arc<ServiceContractListenerData>, task: ServiceTask) -> Option<ServiceTask> {
		match task {
			// when this node should be master of this server key generation session
			ServiceTask::GenerateServerKey(origin, server_key_id, author, threshold) if is_processed_by_this_key_server(
				&*data.key_server_set, data.self_key_pair.public(), &server_key_id) =>
				Some(ServiceTask::GenerateServerKey(origin, server_key_id, author, threshold)),
			// when server key is not yet generated and generation must be initiated by other node
			ServiceTask::GenerateServerKey(_, _, _, _) => None,

			// when server key retrieval is requested
			ServiceTask::RetrieveServerKey(origin, server_key_id) =>
				Some(ServiceTask::RetrieveServerKey(origin, server_key_id)),

			// when document key store is requested
			ServiceTask::StoreDocumentKey(origin, server_key_id, author, common_point, encrypted_point) =>
				Some(ServiceTask::StoreDocumentKey(origin, server_key_id, author, common_point, encrypted_point)),

			// when common document key data retrieval is requested
			ServiceTask::RetrieveShadowDocumentKeyCommon(origin, server_key_id, requester) =>
				Some(ServiceTask::RetrieveShadowDocumentKeyCommon(origin, server_key_id, requester)),

			// when this node should be master of this document key decryption session
			ServiceTask::RetrieveShadowDocumentKeyPersonal(origin, server_key_id, requester) if is_processed_by_this_key_server(
				&*data.key_server_set, data.self_key_pair.public(), &server_key_id) =>
				Some(ServiceTask::RetrieveShadowDocumentKeyPersonal(origin, server_key_id, requester)),
			// when server key is not yet generated and generation must be initiated by other node
			ServiceTask::RetrieveShadowDocumentKeyPersonal(_, _, _) => None,

			ServiceTask::Retry | ServiceTask::Shutdown => unreachable!("must be filtered outside"),
		}
	}

	/// Service thread procedure.
	fn run_service_thread(data: Arc<ServiceContractListenerData>) {
		loop {
			let task = data.tasks_queue.wait();
			trace!(target: "secretstore", "{}: processing {:?} task", data.self_key_pair.public(), task);

			match task {
				ServiceTask::Shutdown => break,
				task => {
					// the only possible reaction to an error is a tx+trace && it is already happened
					let _ = Self::process_service_task(&data, task);
				},
			};
		}

		trace!(target: "secretstore", "{}: ServiceContractListener thread stopped", data.self_key_pair.public());
	}

	/// Process single service task.
	fn process_service_task(data: &Arc<ServiceContractListenerData>, task: ServiceTask) -> Result<(), String> {
		match &task {
			&ServiceTask::GenerateServerKey(origin, server_key_id, author, threshold) => {
				data.retry_data.lock().affected_server_keys.insert(server_key_id.clone());
				log_service_task_result(&task, data.self_key_pair.public(),
					Self::generate_server_key(&data, origin, &server_key_id, author, threshold))
			},
			&ServiceTask::RetrieveServerKey(origin, server_key_id) => {
				data.retry_data.lock().affected_server_keys.insert(server_key_id.clone());
				log_service_task_result(&task, data.self_key_pair.public(),
					Self::retrieve_server_key(&data, origin, &server_key_id))
			},
			&ServiceTask::StoreDocumentKey(origin, server_key_id, author, common_point, encrypted_point) => {
				data.retry_data.lock().affected_document_keys.insert((server_key_id.clone(), author.clone()));
				log_service_task_result(&task, data.self_key_pair.public(),
					Self::store_document_key(&data, origin, &server_key_id, &author, &common_point, &encrypted_point))
			},
			&ServiceTask::RetrieveShadowDocumentKeyCommon(origin, server_key_id, requester) => {
				data.retry_data.lock().affected_document_keys.insert((server_key_id.clone(), requester.clone()));
				log_service_task_result(&task, data.self_key_pair.public(),
					Self::retrieve_document_key_common(&data, origin, &server_key_id, &requester))
			},
			&ServiceTask::RetrieveShadowDocumentKeyPersonal(origin, server_key_id, requester) => {
				data.retry_data.lock().affected_server_keys.insert(server_key_id.clone());
				log_service_task_result(&task, data.self_key_pair.public(),
					Self::retrieve_document_key_personal(&data, origin, &server_key_id, requester))
			},
			&ServiceTask::Retry => {
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
					})
			},
			&ServiceTask::Shutdown => unreachable!("must be filtered outside"),
		}
	}

	/// Retry processing pending requests.
	fn retry_pending_requests(data: &Arc<ServiceContractListenerData>) -> Result<usize, String> {
		let mut failed_requests = 0;
		let mut processed_requests = 0;
		let retry_data = ::std::mem::replace(&mut *data.retry_data.lock(), Default::default());
		let pending_tasks = data.contract.read_pending_requests()
			.filter_map(|(is_confirmed, task)| Self::filter_task(data, task)
				.map(|t| (is_confirmed, t)));
		for (is_response_required, task) in pending_tasks {
			// only process requests, which we haven't confirmed yet
			if !is_response_required {
				continue;
			}

			match task {
				ServiceTask::GenerateServerKey(_, ref key, _, _) | ServiceTask::RetrieveServerKey(_, ref key)
					if retry_data.affected_server_keys.contains(key) => continue,
				ServiceTask::StoreDocumentKey(_, ref key, ref author, _, _) |
					ServiceTask::RetrieveShadowDocumentKeyCommon(_, ref key, ref author)
					if retry_data.affected_document_keys.contains(&(key.clone(), author.clone())) => continue,
				ServiceTask::RetrieveShadowDocumentKeyPersonal(_, ref key, ref requester)
					if retry_data.affected_document_keys.contains(&(key.clone(), public_to_address(requester))) => continue,
				_ => (),
			}

			// process request result
			let request_result = Self::process_service_task(data, task);
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

	/// Generate server key (start generation session).
	fn generate_server_key(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, author: Address, threshold: usize) -> Result<(), String> {
		Self::process_server_key_generation_result(data, origin, server_key_id, data.cluster.new_generation_session(
			server_key_id.clone(), Some(origin), author, threshold).map(|_| None).map_err(Into::into))
	}

	/// Process server key generation result.
	fn process_server_key_generation_result(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, result: Result<Option<Public>, Error>) -> Result<(), String> {
		match result {
			Ok(None) => Ok(()),
			Ok(Some(server_key)) => {
				data.contract.publish_generated_server_key(&origin, server_key_id, server_key)
			},
			Err(ref error) if error.is_non_fatal() => Err(format!("{}", error)),
			Err(ref error) => {
				// ignore error as we're already processing an error
				let _ = data.contract.publish_server_key_generation_error(&origin, server_key_id)
					.map_err(|error| warn!(target: "secretstore", "{}: failed to publish GenerateServerKey({}) error: {}",
						data.self_key_pair.public(), server_key_id, error));
				Err(format!("{}", error))
			}
		}
	}

	/// Retrieve server key.
	fn retrieve_server_key(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		match data.key_storage.get(server_key_id) {
			Ok(Some(server_key_share)) => {
				data.contract.publish_retrieved_server_key(&origin, server_key_id, server_key_share.public, server_key_share.threshold)
			},
			Ok(None) => {
				data.contract.publish_server_key_retrieval_error(&origin, server_key_id)
			}
			Err(ref error) if error.is_non_fatal() => Err(format!("{}", error)),
			Err(ref error) => {
				// ignore error as we're already processing an error
				let _ = data.contract.publish_server_key_retrieval_error(&origin, server_key_id)
					.map_err(|error| warn!(target: "secretstore", "{}: failed to publish RetrieveServerKey({}) error: {}",
						data.self_key_pair.public(), server_key_id, error));
				Err(format!("{}", error))
			}
		}
	}

	/// Store document key.
	fn store_document_key(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, author: &Address, common_point: &Public, encrypted_point: &Public) -> Result<(), String> {
		let store_result = data.key_storage.get(server_key_id)
			.and_then(|key_share| key_share.ok_or(Error::ServerKeyIsNotFound))
			.and_then(|key_share| check_encrypted_data(Some(&key_share)).map(|_| key_share).map_err(Into::into))
			.and_then(|key_share| update_encrypted_data(&data.key_storage, server_key_id.clone(), key_share,
				author.clone(), common_point.clone(), encrypted_point.clone()).map_err(Into::into));
		match store_result {
			Ok(()) => {
				data.contract.publish_stored_document_key(&origin, server_key_id)
			},
			Err(ref error) if error.is_non_fatal() => Err(format!("{}", error)),
			Err(ref error) => {
				// ignore error as we're already processing an error
				let _ = data.contract.publish_document_key_store_error(&origin, server_key_id)
					.map_err(|error| warn!(target: "secretstore", "{}: failed to publish StoreDocumentKey({}) error: {}",
						data.self_key_pair.public(), server_key_id, error));
				Err(format!("{}", error))
			},
		}
	}

	/// Retrieve common part of document key.
	fn retrieve_document_key_common(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
		let retrieval_result = data.acl_storage.check(requester.clone(), server_key_id)
			.and_then(|is_allowed| if !is_allowed { Err(Error::AccessDenied) } else { Ok(()) })
			.and_then(|_| data.key_storage.get(server_key_id).and_then(|key_share| key_share.ok_or(Error::ServerKeyIsNotFound)))
			.and_then(|key_share| key_share.common_point
				.ok_or(Error::DocumentKeyIsNotFound)
				.and_then(|common_point| math::make_common_shadow_point(key_share.threshold, common_point))
				.map(|common_point| (common_point, key_share.threshold)));
		match retrieval_result {
			Ok((common_point, threshold)) => {
				data.contract.publish_retrieved_document_key_common(&origin, server_key_id, requester, common_point, threshold)
			},
			Err(ref error) if error.is_non_fatal() => Err(format!("{}", error)),
			Err(ref error) => {
				// ignore error as we're already processing an error
				let _ = data.contract.publish_document_key_retrieval_error(&origin, server_key_id, requester)
					.map_err(|error| warn!(target: "secretstore", "{}: failed to publish RetrieveDocumentKey({}) error: {}",
						data.self_key_pair.public(), server_key_id, error));
				Err(format!("{}", error))
			},
		}
	}

	/// Retrieve personal part of document key (start decryption session).
	fn retrieve_document_key_personal(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, requester: Public) -> Result<(), String> {
		Self::process_document_key_retrieval_result(data, origin, server_key_id, &public_to_address(&requester), data.cluster.new_decryption_session(
			server_key_id.clone(), Some(origin), requester.clone().into(), None, true, true).map(|_| None).map_err(Into::into))
	}

	/// Process document key retrieval result.
	fn process_document_key_retrieval_result(data: &Arc<ServiceContractListenerData>, origin: Address, server_key_id: &ServerKeyId, requester: &Address, result: Result<Option<(Vec<Address>, Public, Bytes)>, Error>) -> Result<(), String> {
		match result {
			Ok(None) => Ok(()),
			Ok(Some((participants, decrypted_secret, shadow))) => {
				data.contract.publish_retrieved_document_key_personal(&origin, server_key_id, &requester, &participants, decrypted_secret, shadow)
			},
			Err(ref error) if error.is_non_fatal() => Err(format!("{}", error)),
			Err(ref error) => {
				// ignore error as we're already processing an error
				let _ = data.contract.publish_document_key_retrieval_error(&origin, server_key_id, &requester)
					.map_err(|error| warn!(target: "secretstore", "{}: failed to publish RetrieveDocumentKey({}) error: {}",
						data.self_key_pair.public(), server_key_id, error));
				Err(format!("{}", error))
			}
		}
	}
}

impl Drop for ServiceContractListener {
	fn drop(&mut self) {
		if let Some(service_handle) = self.service_handle.take() {
			self.data.tasks_queue.push_front(ServiceTask::Shutdown);
			// ignore error as we are already closing
			let _ = service_handle.join();
		}
	}
}

impl NewBlocksNotify for ServiceContractListener {
	fn new_blocks(&self, new_enacted_len: usize) {
		if !self.data.contract.update() {
			return;
		}

		self.process_service_contract_events();

		// schedule retry if received enough blocks since last retry
		// it maybe inaccurate when switching syncing/synced states, but that's ok
		if self.data.last_retry.fetch_add(new_enacted_len, Ordering::Relaxed) >= RETRY_INTERVAL_BLOCKS {
			// shortcut: do not retry if we're isolated from the cluster
			if !self.data.key_server_set.is_isolated() {
				self.data.tasks_queue.push(ServiceTask::Retry);
				self.data.last_retry.store(0, Ordering::Relaxed);
			}
		}
	}
}

impl ClusterSessionsListener<GenerationSession> for ServiceContractListener {
	fn on_session_removed(&self, session: Arc<GenerationSession>) {
		// by this time sesion must already be completed - either successfully, or not
		assert!(session.is_finished());

		// ignore result - the only thing that we can do is to log the error
		let server_key_id = session.id();
		if let Some(origin) = session.origin() {
			if let Some(generation_result) = session.result() {
				let generation_result = generation_result.map(Some).map_err(Into::into);
				let _ = Self::process_server_key_generation_result(&self.data, origin, &server_key_id, generation_result);
			}
		}
	}
}

impl ClusterSessionsListener<DecryptionSession> for ServiceContractListener {
	fn on_session_removed(&self, session: Arc<DecryptionSession>) {
		// by this time sesion must already be completed - either successfully, or not
		assert!(session.is_finished());

		// ignore result - the only thing that we can do is to log the error
		let session_id = session.id();
		let server_key_id = session_id.id;
		if let (Some(requester), Some(origin)) = (session.requester().and_then(|r| r.address(&server_key_id).ok()), session.origin()) {
			if let Some(retrieval_result) = session.result() {
				let retrieval_result = retrieval_result.map(|key_shadow|
					session.broadcast_shadows()
						.and_then(|broadcast_shadows|
							broadcast_shadows.get(self.data.self_key_pair.public())
								.map(|self_shadow| (
									broadcast_shadows.keys().map(public_to_address).collect(),
									key_shadow.decrypted_secret,
									self_shadow.clone()
								)))
				).map_err(Into::into);
				let _ = Self::process_document_key_retrieval_result(&self.data, origin, &server_key_id, &requester, retrieval_result);
			}
		}
	}
}

impl ClusterSessionsListener<KeyVersionNegotiationSession<KeyVersionNegotiationTransport>> for ServiceContractListener {
	fn on_session_removed(&self, session: Arc<KeyVersionNegotiationSession<KeyVersionNegotiationTransport>>) {
		// by this time sesion must already be completed - either successfully, or not
		assert!(session.is_finished());

		// we're interested in:
		// 1) sessions failed with fatal error
		// 2) with decryption continue action
		let error = match session.result() {
			Some(Err(ref error)) if !error.is_non_fatal() => error.clone(),
			_ => return,
		};

		let (origin, requester) = match session.take_failed_continue_action() {
			Some(FailedContinueAction::Decrypt(Some(origin), requester)) => (origin, requester),
			_ => return,
		};

		// check if master node is responsible for processing key requests
		let meta = session.meta();
		if !is_processed_by_this_key_server(&*self.data.key_server_set, &meta.master_node_id, &meta.id) {
			return;
		}

		// ignore result as we're already processing an error
		let _ = Self::process_document_key_retrieval_result(&self.data, origin, &meta.id, &requester, Err(error));
	}
}

impl ::std::fmt::Display for ServiceTask {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			ServiceTask::Retry => write!(f, "Retry"),
			ServiceTask::GenerateServerKey(_, ref server_key_id, ref author, ref threshold) =>
				write!(f, "GenerateServerKey({}, {}, {})", server_key_id, author, threshold),
			ServiceTask::RetrieveServerKey(_, ref server_key_id) =>
				write!(f, "RetrieveServerKey({})", server_key_id),
			ServiceTask::StoreDocumentKey(_, ref server_key_id, ref author, _, _) =>
				write!(f, "StoreDocumentKey({}, {})", server_key_id, author),
			ServiceTask::RetrieveShadowDocumentKeyCommon(_, ref server_key_id, ref requester) =>
				write!(f, "RetrieveShadowDocumentKeyCommon({}, {})", server_key_id, requester),
			ServiceTask::RetrieveShadowDocumentKeyPersonal(_, ref server_key_id, ref requester) =>
				write!(f, "RetrieveShadowDocumentKeyPersonal({}, {})", server_key_id, public_to_address(requester)),
			ServiceTask::Shutdown => write!(f, "Shutdown"),
		}
	}
}

/// Log service task result.
fn log_service_task_result(task: &ServiceTask, self_id: &Public, result: Result<(), String>) -> Result<(), String> {
	match result {
		Ok(_) => trace!(target: "secretstore", "{}: processed {} request", self_id, task),
		Err(ref error) => warn!(target: "secretstore", "{}: failed to process {} request with: {}", self_id, task, error),
	}

	result
}

/// Returns true when session, related to `server_key_id` must be started on `node`.
fn is_processed_by_this_key_server(key_server_set: &dyn KeyServerSet, node: &NodeId, server_key_id: &H256) -> bool {
	let servers = key_server_set.snapshot().current_set;
	let total_servers_count = servers.len();
	match total_servers_count {
		0 => return false,
		1 => return true,
		_ => (),
	}

	let this_server_index = match servers.keys().enumerate().find(|&(_, s)| s == node) {
		Some((index, _)) => index,
		None => return false,
	};

	let server_key_id_value: U256 = server_key_id.into_uint();
	let range_interval = U256::max_value() / total_servers_count;
	let range_begin = (range_interval + 1) * this_server_index as u32;
	let range_end = range_begin.saturating_add(range_interval);

	server_key_id_value >= range_begin && server_key_id_value <= range_end
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::Ordering;
	use crypto::publickey::{Random, Generator, KeyPair};
	use listener::service_contract::ServiceContract;
	use listener::service_contract::tests::DummyServiceContract;
	use key_server_cluster::DummyClusterClient;
	use acl_storage::{AclStorage, DummyAclStorage};
	use key_storage::{KeyStorage, DocumentKeyShare};
	use key_storage::tests::DummyKeyStorage;
	use key_server_set::KeyServerSet;
	use key_server_set::tests::MapKeyServerSet;
	use blockchain::SigningKeyPair;
	use {PlainNodeKeyPair, ServerKeyId};
	use super::{ServiceTask, ServiceContractListener, ServiceContractListenerParams, is_processed_by_this_key_server};
	use ethereum_types::Address;

	fn create_non_empty_key_storage(has_doc_key: bool) -> Arc<DummyKeyStorage> {
		let key_storage = Arc::new(DummyKeyStorage::default());
		let mut key_share = DocumentKeyShare::default();
		key_share.public = KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000001"
			.parse().unwrap()).unwrap().public().clone();
		if has_doc_key {
			key_share.common_point = Some(Default::default());
			key_share.encrypted_point = Some(Default::default());
		}
		key_storage.insert(Default::default(), key_share.clone()).unwrap();
		key_storage
	}

	fn make_servers_set(is_isolated: bool) -> Arc<dyn KeyServerSet> {
		Arc::new(MapKeyServerSet::new(is_isolated, vec![
			("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee51ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			("f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
		].into_iter().collect()))
	}

	fn make_service_contract_listener(contract: Option<Arc<dyn ServiceContract>>, cluster: Option<Arc<DummyClusterClient>>, key_storage: Option<Arc<dyn KeyStorage>>, acl_storage: Option<Arc<dyn AclStorage>>, servers_set: Option<Arc<dyn KeyServerSet>>) -> Arc<ServiceContractListener> {
		let contract = contract.unwrap_or_else(|| Arc::new(DummyServiceContract::default()));
		let cluster = cluster.unwrap_or_else(|| Arc::new(DummyClusterClient::default()));
		let key_storage = key_storage.unwrap_or_else(|| Arc::new(DummyKeyStorage::default()));
		let acl_storage = acl_storage.unwrap_or_else(|| Arc::new(DummyAclStorage::default()));
		let servers_set = servers_set.unwrap_or_else(|| make_servers_set(false));
		let self_key_pair = Arc::new(PlainNodeKeyPair::new(KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap()).unwrap()));
		ServiceContractListener::new(ServiceContractListenerParams {
			contract: contract,
			self_key_pair: self_key_pair,
			key_server_set: servers_set,
			acl_storage: acl_storage,
			cluster: cluster,
			key_storage: key_storage,
		}).unwrap()
	}

	#[test]
	fn is_not_processed_by_this_key_server_with_zero_servers() {
		assert_eq!(is_processed_by_this_key_server(
			&MapKeyServerSet::default(),
			Random.generate().unwrap().public(),
			&Default::default()), false);
	}

	#[test]
	fn is_processed_by_this_key_server_with_single_server() {
		let self_key_pair = Random.generate().unwrap();
		assert_eq!(is_processed_by_this_key_server(
			&MapKeyServerSet::new(false, vec![
				(self_key_pair.public().clone(), "127.0.0.1:8080".parse().unwrap())
			].into_iter().collect()),
			self_key_pair.public(),
			&Default::default()), true);
	}

	#[test]
	fn is_not_processed_by_this_key_server_when_not_a_part_of_servers_set() {
		assert!(is_processed_by_this_key_server(
			&MapKeyServerSet::new(false, vec![
				(Random.generate().unwrap().public().clone(), "127.0.0.1:8080".parse().unwrap())
			].into_iter().collect()),
			Random.generate().unwrap().public(),
			&Default::default()));
	}

	#[test]
	fn is_processed_by_this_key_server_in_set_of_3() {
		// servers set is ordered && server range depends on index of this server
		let servers_set = MapKeyServerSet::new(false, vec![
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
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"3000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"5555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"5555555555555555555555555555555555555555555555555555555555555556".parse().unwrap()), false);

		// 2nd server: process hashes from 0x555...556 to 0xaaa...aab
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"5555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"5555555555555555555555555555555555555555555555555555555555555556".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"7555555555555555555555555555555555555555555555555555555555555555".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), false);

		// 3rd server: process hashes from 0x800...000 to 0xbff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"daaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
	}

	#[test]
	fn is_processed_by_this_key_server_in_set_of_4() {
		// servers set is ordered && server range depends on index of this server
		let servers_set = MapKeyServerSet::new(false, vec![
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
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"2000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"4000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 2nd server: process hashes from 0x400...000 to 0x7ff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"4000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"6000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"8000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 3rd server: process hashes from 0x800...000 to 0xbff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000004".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"8000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"a000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"bfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"c000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), false);

		// 4th server: process hashes from 0xc00...000 to 0xfff...ff
		let key_pair = PlainNodeKeyPair::new(KeyPair::from_secret(
			"0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap()).unwrap());
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"bfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), false);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"c000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"e000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()), true);
		assert_eq!(is_processed_by_this_key_server(&servers_set, key_pair.public(),
			&"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap()), true);
	}

	#[test]
	fn no_tasks_scheduled_when_no_contract_events() {
		let listener = make_service_contract_listener(None, None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
	}

	#[test]
	fn tasks_are_not_scheduled_on_isolated_node() {
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::GenerateServerKey(Default::default(), Default::default(), Default::default(), 0));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, Some(make_servers_set(true)));
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
	}

	// server key generation tests

	#[test]
	fn server_key_generation_is_scheduled_when_requested() {
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::GenerateServerKey(Default::default(), Default::default(), Default::default(), 0));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::GenerateServerKey(
			Default::default(), Default::default(), Default::default(), 0)));
	}

	#[test]
	fn no_new_tasks_scheduled_when_server_key_generation_requested_and_request_belongs_to_other_key_server() {
		let server_key_id = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap();
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::GenerateServerKey(Default::default(), server_key_id, Default::default(), 0));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
	}

	#[test]
	fn generation_session_is_created_when_processing_generate_server_key_task() {
		let cluster = Arc::new(DummyClusterClient::default());
		let listener = make_service_contract_listener(None, Some(cluster.clone()), None, None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::GenerateServerKey(
			Default::default(), Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(cluster.generation_requests_count.load(Ordering::Relaxed), 1);
	}

	#[test]
	fn server_key_generation_is_not_retried_if_tried_in_the_same_cycle() {
		let mut contract = DummyServiceContract::default();
		contract.pending_requests.push((false, ServiceTask::GenerateServerKey(Default::default(),
			Default::default(), Default::default(), Default::default())));
		let cluster = Arc::new(DummyClusterClient::default());
		let listener = make_service_contract_listener(Some(Arc::new(contract)), Some(cluster.clone()), None, None, None);
		listener.data.retry_data.lock().affected_server_keys.insert(Default::default());
		ServiceContractListener::retry_pending_requests(&listener.data).unwrap();
		assert_eq!(cluster.generation_requests_count.load(Ordering::Relaxed), 0);
	}

	// server key retrieval tests

	#[test]
	fn server_key_retrieval_is_scheduled_when_requested() {
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::RetrieveServerKey(Default::default(), Default::default()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::RetrieveServerKey(
			Default::default(), Default::default())));
	}

	#[test]
	fn server_key_retrieval_is_scheduled_when_requested_and_request_belongs_to_other_key_server() {
		let server_key_id: ServerKeyId = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap();
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::RetrieveServerKey(Default::default(), server_key_id.clone()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::RetrieveServerKey(
			Default::default(), server_key_id)));
	}

	#[test]
	fn server_key_is_retrieved_when_processing_retrieve_server_key_task() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(false);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveServerKey(
			Default::default(), Default::default())).unwrap();
		assert_eq!(*contract.retrieved_server_keys.lock(), vec![(Default::default(),
			KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap()).unwrap().public().clone(), 0)]);
	}

	#[test]
	fn server_key_retrieval_failure_is_reported_when_processing_retrieve_server_key_task_and_key_is_unknown() {
		let contract = Arc::new(DummyServiceContract::default());
		let listener = make_service_contract_listener(Some(contract.clone()), None, None, None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveServerKey(
			Default::default(), Default::default())).unwrap();
		assert_eq!(*contract.server_keys_retrieval_failures.lock(), vec![Default::default()]);
	}

	#[test]
	fn server_key_retrieval_is_not_retried_if_tried_in_the_same_cycle() {
		let mut contract = DummyServiceContract::default();
		contract.pending_requests.push((false, ServiceTask::RetrieveServerKey(Default::default(), Default::default())));
		let cluster = Arc::new(DummyClusterClient::default());
		let listener = make_service_contract_listener(Some(Arc::new(contract)), Some(cluster.clone()), None, None, None);
		listener.data.retry_data.lock().affected_server_keys.insert(Default::default());
		ServiceContractListener::retry_pending_requests(&listener.data).unwrap();
		assert_eq!(cluster.generation_requests_count.load(Ordering::Relaxed), 0);
	}

	// document key store tests

	#[test]
	fn document_key_store_is_scheduled_when_requested() {
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::StoreDocumentKey(Default::default(), Default::default(),
			Default::default(), Default::default(), Default::default()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::StoreDocumentKey(
			Default::default(), Default::default(), Default::default(), Default::default(), Default::default())));
	}

	#[test]
	fn document_key_store_is_scheduled_when_requested_and_request_belongs_to_other_key_server() {
		let server_key_id: ServerKeyId = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap();
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::StoreDocumentKey(Default::default(), server_key_id.clone(),
			Default::default(), Default::default(), Default::default()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::StoreDocumentKey(
			Default::default(), server_key_id, Default::default(), Default::default(), Default::default())));
	}

	#[test]
	fn document_key_is_stored_when_processing_store_document_key_task() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(false);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage.clone()), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::StoreDocumentKey(
			Default::default(), Default::default(), Default::default(), Default::default(), Default::default())).unwrap();
		assert_eq!(*contract.stored_document_keys.lock(), vec![Default::default()]);

		let key_share = key_storage.get(&Default::default()).unwrap().unwrap();
		assert_eq!(key_share.common_point, Some(Default::default()));
		assert_eq!(key_share.encrypted_point, Some(Default::default()));
	}

	#[test]
	fn document_key_store_failure_reported_when_no_server_key() {
		let contract = Arc::new(DummyServiceContract::default());
		let listener = make_service_contract_listener(Some(contract.clone()), None, None, None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::StoreDocumentKey(
			Default::default(), Default::default(), Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_store_failures.lock(), vec![Default::default()]);
	}

	#[test]
	fn document_key_store_failure_reported_when_document_key_already_set() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(true);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::StoreDocumentKey(
			Default::default(), Default::default(), Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_store_failures.lock(), vec![Default::default()]);
	}

	#[test]
	fn document_key_store_failure_reported_when_author_differs() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(false);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::StoreDocumentKey(
			Default::default(), Default::default(), Address::from_low_u64_be(1), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_store_failures.lock(), vec![Default::default()]);
	}

	// document key shadow common retrieval tests

	#[test]
	fn document_key_shadow_common_retrieval_is_scheduled_when_requested() {
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::RetrieveShadowDocumentKeyCommon(Default::default(), Default::default(), Default::default()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), Default::default(), Default::default())));
	}

	#[test]
	fn document_key_shadow_common_retrieval_is_scheduled_when_requested_and_request_belongs_to_other_key_server() {
		let server_key_id: ServerKeyId = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".parse().unwrap();
		let mut contract = DummyServiceContract::default();
		contract.logs.push(ServiceTask::RetrieveShadowDocumentKeyCommon(Default::default(), server_key_id.clone(), Default::default()));
		let listener = make_service_contract_listener(Some(Arc::new(contract)), None, None, None, None);
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 0);
		listener.process_service_contract_events();
		assert_eq!(listener.data.tasks_queue.snapshot().len(), 1);
		assert_eq!(listener.data.tasks_queue.snapshot().pop_back(), Some(ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), server_key_id, Default::default())));
	}

	#[test]
	fn document_key_shadow_common_is_retrieved_when_processing_document_key_shadow_common_retrieval_task() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(true);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage.clone()), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), Default::default(), Default::default())).unwrap();
		assert_eq!(*contract.common_shadow_retrieved_document_keys.lock(), vec![(Default::default(), Default::default(),
			Default::default(), 0)]);
	}

	#[test]
	fn document_key_shadow_common_retrieval_failure_reported_when_access_denied() {
		let acl_storage = DummyAclStorage::default();
		acl_storage.prohibit(Default::default(), Default::default());
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(true);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage.clone()), Some(Arc::new(acl_storage)), None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_shadow_retrieval_failures.lock(), vec![(Default::default(), Default::default())]);
	}

	#[test]
	fn document_key_shadow_common_retrieval_failure_reported_when_no_server_key() {
		let contract = Arc::new(DummyServiceContract::default());
		let listener = make_service_contract_listener(Some(contract.clone()), None, None, None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_shadow_retrieval_failures.lock(), vec![(Default::default(), Default::default())]);
	}

	#[test]
	fn document_key_shadow_common_retrieval_failure_reported_when_no_document_key() {
		let contract = Arc::new(DummyServiceContract::default());
		let key_storage = create_non_empty_key_storage(false);
		let listener = make_service_contract_listener(Some(contract.clone()), None, Some(key_storage.clone()), None, None);
		ServiceContractListener::process_service_task(&listener.data, ServiceTask::RetrieveShadowDocumentKeyCommon(
			Default::default(), Default::default(), Default::default())).unwrap_err();
		assert_eq!(*contract.document_keys_shadow_retrieval_failures.lock(), vec![(Default::default(), Default::default())]);
	}
}
