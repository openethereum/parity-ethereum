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

use std::time::{Duration, Instant};
use std::sync::{Arc, Weak};
use std::sync::atomic::AtomicBool;
use std::collections::{VecDeque, BTreeMap, BTreeSet};
use parking_lot::{Mutex, RwLock, Condvar};
use ethereum_types::H256;
use ethkey::Secret;
use key_server_cluster::{Error, NodeId, SessionId, Requester};
use key_server_cluster::cluster::{Cluster, ClusterData, ClusterConfiguration, ClusterView};
use key_server_cluster::connection_trigger::ServersSetChangeSessionCreatorConnector;
use key_server_cluster::message::{self, Message};
use key_server_cluster::generation_session::{SessionImpl as GenerationSessionImpl};
use key_server_cluster::decryption_session::{SessionImpl as DecryptionSessionImpl};
use key_server_cluster::encryption_session::{SessionImpl as EncryptionSessionImpl};
use key_server_cluster::signing_session_ecdsa::{SessionImpl as EcdsaSigningSessionImpl};
use key_server_cluster::signing_session_schnorr::{SessionImpl as SchnorrSigningSessionImpl};
use key_server_cluster::share_add_session::{SessionImpl as ShareAddSessionImpl, IsolatedSessionTransport as ShareAddTransport};
use key_server_cluster::servers_set_change_session::{SessionImpl as ServersSetChangeSessionImpl};
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSessionImpl,
	IsolatedSessionTransport as VersionNegotiationTransport};

use key_server_cluster::cluster_sessions_creator::{GenerationSessionCreator, EncryptionSessionCreator, DecryptionSessionCreator,
	SchnorrSigningSessionCreator, KeyVersionNegotiationSessionCreator, AdminSessionCreator, SessionCreatorCore,
	EcdsaSigningSessionCreator, ClusterSessionCreator};

/// When there are no session-related messages for SESSION_TIMEOUT_INTERVAL seconds,
/// we must treat this session as stalled && finish it with an error.
/// This timeout is for cases when node is responding to KeepAlive messages, but intentionally ignores
/// session messages.
const SESSION_TIMEOUT_INTERVAL: Duration = Duration::from_secs(60);
/// Interval to send session-level KeepAlive-messages.
const SESSION_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);

lazy_static! {
	/// Servers set change session id (there could be at most 1 session => hardcoded id).
	pub static ref SERVERS_SET_CHANGE_SESSION_ID: SessionId = "10b7af423bb551d5dc8645db754163a2145d37d78d468fa7330435ed77064c1c"
		.parse()
		.expect("hardcoded id should parse without errors; qed");
}

/// Session id with sub session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIdWithSubSession {
	/// Key id.
	pub id: SessionId,
	/// Sub session id.
	pub access_key: Secret,
}

/// Generic cluster session.
pub trait ClusterSession {
	/// Session identifier type.
	type Id: ::std::fmt::Debug + Ord + Clone;

	/// Session type name.
	fn type_name() -> &'static str;
	/// Get session id.
	fn id(&self) -> Self::Id;
	/// If session is finished (either with succcess or not).
	fn is_finished(&self) -> bool;
	/// When it takes too much time to complete session.
	fn on_session_timeout(&self);
	/// When it takes too much time to receive response from the node.
	fn on_node_timeout(&self, node_id: &NodeId);
	/// Process error that has occured during session + propagate this error to required nodes.
	fn on_session_error(&self, sender: &NodeId, error: Error);
	/// Process session message.
	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error>;

	/// 'Wait for session completion' helper.
	fn wait_session<T, U, F: Fn(&U) -> Option<Result<T, Error>>>(completion_event: &Condvar, session_data: &Mutex<U>, timeout: Option<Duration>, result_reader: F) -> Option<Result<T, Error>> {
		let mut locked_data = session_data.lock();
		match result_reader(&locked_data) {
			Some(result) => Some(result),
			None => {
				match timeout {
					None => completion_event.wait(&mut locked_data),
					Some(timeout) => {
						completion_event.wait_for(&mut locked_data, timeout);
					},
				}

				result_reader(&locked_data)
			},
		}
	}
}

/// Administrative session.
pub enum AdminSession {
	/// Share add session.
	ShareAdd(ShareAddSessionImpl<ShareAddTransport>),
	/// Servers set change session.
	ServersSetChange(ServersSetChangeSessionImpl),
}

/// Administrative session creation data.
pub enum AdminSessionCreationData {
	/// Share add session (key id).
	ShareAdd(H256),
	/// Servers set change session (block id, new_server_set).
	ServersSetChange(Option<H256>, BTreeSet<NodeId>),
}

/// Active sessions on this cluster.
pub struct ClusterSessions {
	/// Key generation sessions.
	pub generation_sessions: ClusterSessionsContainer<GenerationSessionImpl, GenerationSessionCreator, ()>,
	/// Encryption sessions.
	pub encryption_sessions: ClusterSessionsContainer<EncryptionSessionImpl, EncryptionSessionCreator, ()>,
	/// Decryption sessions.
	pub decryption_sessions: ClusterSessionsContainer<DecryptionSessionImpl, DecryptionSessionCreator, Requester>,
	/// Schnorr signing sessions.
	pub schnorr_signing_sessions: ClusterSessionsContainer<SchnorrSigningSessionImpl, SchnorrSigningSessionCreator, Requester>,
	/// ECDSA signing sessions.
	pub ecdsa_signing_sessions: ClusterSessionsContainer<EcdsaSigningSessionImpl, EcdsaSigningSessionCreator, Requester>,
	/// Key version negotiation sessions.
	pub negotiation_sessions: ClusterSessionsContainer<KeyVersionNegotiationSessionImpl<VersionNegotiationTransport>, KeyVersionNegotiationSessionCreator, ()>,
	/// Administrative sessions.
	pub admin_sessions: ClusterSessionsContainer<AdminSession, AdminSessionCreator, AdminSessionCreationData>,
	/// Self node id.
	self_node_id: NodeId,
	/// Creator core.
	creator_core: Arc<SessionCreatorCore>,
}

/// Active sessions container listener.
pub trait ClusterSessionsListener<S: ClusterSession>: Send + Sync {
	/// When new session is inserted to the container.
	fn on_session_inserted(&self, _session: Arc<S>) {}
	/// When session is removed from the container.
	fn on_session_removed(&self, _session: Arc<S>) {}
}

/// Active sessions container.
pub struct ClusterSessionsContainer<S: ClusterSession, SC: ClusterSessionCreator<S, D>, D> {
	/// Sessions creator.
	pub creator: SC,
	/// Active sessions.
	sessions: RwLock<BTreeMap<S::Id, QueuedSession<S>>>,
	/// Listeners. Lock order: sessions -> listeners.
	listeners: Mutex<Vec<Weak<ClusterSessionsListener<S>>>>,
	/// Sessions container state.
	container_state: Arc<Mutex<ClusterSessionsContainerState>>,
	/// Phantom data.
	_pd: ::std::marker::PhantomData<D>,
}

/// Session and its message queue.
pub struct QueuedSession<S> {
	/// Session master.
	pub master: NodeId,
	/// Cluster view.
	pub cluster_view: Arc<Cluster>,
	/// Last keep alive time.
	pub last_keep_alive_time: Instant,
	/// Last received message time.
	pub last_message_time: Instant,
	/// Generation session.
	pub session: Arc<S>,
	/// Messages queue.
	pub queue: VecDeque<(NodeId, Message)>,
}

/// Cluster sessions container state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClusterSessionsContainerState {
	/// There's no active sessions => any session can be started.
	Idle,
	/// There are active sessions => exclusive session can't be started right now.
	Active(usize),
	/// Exclusive session is active => can't start any other sessions.
	Exclusive,
}

impl ClusterSessions {
	/// Create new cluster sessions container.
	pub fn new(config: &ClusterConfiguration, servers_set_change_session_creator_connector: Arc<ServersSetChangeSessionCreatorConnector>) -> Self {
		let container_state = Arc::new(Mutex::new(ClusterSessionsContainerState::Idle));
		let creator_core = Arc::new(SessionCreatorCore::new(config));
		ClusterSessions {
			self_node_id: config.self_key_pair.public().clone(),
			generation_sessions: ClusterSessionsContainer::new(GenerationSessionCreator {
				core: creator_core.clone(),
				make_faulty_generation_sessions: AtomicBool::new(false),
			}, container_state.clone()),
			encryption_sessions: ClusterSessionsContainer::new(EncryptionSessionCreator {
				core: creator_core.clone(),
			}, container_state.clone()),
			decryption_sessions: ClusterSessionsContainer::new(DecryptionSessionCreator {
				core: creator_core.clone(),
			}, container_state.clone()),
			schnorr_signing_sessions: ClusterSessionsContainer::new(SchnorrSigningSessionCreator {
				core: creator_core.clone(),
			}, container_state.clone()),
			ecdsa_signing_sessions: ClusterSessionsContainer::new(EcdsaSigningSessionCreator {
				core: creator_core.clone(),
			}, container_state.clone()),
			negotiation_sessions: ClusterSessionsContainer::new(KeyVersionNegotiationSessionCreator {
				core: creator_core.clone(),
			}, container_state.clone()),
			admin_sessions: ClusterSessionsContainer::new(AdminSessionCreator {
				core: creator_core.clone(),
				servers_set_change_session_creator_connector: servers_set_change_session_creator_connector,
				admin_public: config.admin_public.clone(),
			}, container_state),
			creator_core: creator_core,
		}
	}

	#[cfg(test)]
	pub fn make_faulty_generation_sessions(&self) {
		self.generation_sessions.creator.make_faulty_generation_sessions();
	}

	/// Send session-level keep-alive messages.
	pub fn sessions_keep_alive(&self) {
		self.admin_sessions.send_keep_alive(&*SERVERS_SET_CHANGE_SESSION_ID, &self.self_node_id);
	}

	/// When session-level keep-alive response is received.
	pub fn on_session_keep_alive(&self, sender: &NodeId, session_id: SessionId) {
		if session_id == *SERVERS_SET_CHANGE_SESSION_ID {
			self.admin_sessions.on_keep_alive(&session_id, sender);
		}
	}

	/// Stop sessions that are stalling.
	pub fn stop_stalled_sessions(&self) {
		self.generation_sessions.stop_stalled_sessions();
		self.encryption_sessions.stop_stalled_sessions();
		self.decryption_sessions.stop_stalled_sessions();
		self.schnorr_signing_sessions.stop_stalled_sessions();
		self.ecdsa_signing_sessions.stop_stalled_sessions();
		self.negotiation_sessions.stop_stalled_sessions();
		self.admin_sessions.stop_stalled_sessions();
	}

	/// When connection to node is lost.
	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		self.generation_sessions.on_connection_timeout(node_id);
		self.encryption_sessions.on_connection_timeout(node_id);
		self.decryption_sessions.on_connection_timeout(node_id);
		self.schnorr_signing_sessions.on_connection_timeout(node_id);
		self.ecdsa_signing_sessions.on_connection_timeout(node_id);
		self.negotiation_sessions.on_connection_timeout(node_id);
		self.admin_sessions.on_connection_timeout(node_id);
		self.creator_core.on_connection_timeout(node_id);
	}
}

impl<S, SC, D> ClusterSessionsContainer<S, SC, D> where S: ClusterSession, SC: ClusterSessionCreator<S, D> {
	pub fn new(creator: SC, container_state: Arc<Mutex<ClusterSessionsContainerState>>) -> Self {
		ClusterSessionsContainer {
			creator: creator,
			sessions: RwLock::new(BTreeMap::new()),
			listeners: Mutex::new(Vec::new()),
			container_state: container_state,
			_pd: Default::default(),
		}
	}

	pub fn add_listener(&self, listener: Arc<ClusterSessionsListener<S>>) {
		self.listeners.lock().push(Arc::downgrade(&listener));
	}

	#[cfg(test)]
	pub fn is_empty(&self) -> bool {
		self.sessions.read().is_empty()
	}

	pub fn get(&self, session_id: &S::Id, update_last_message_time: bool) -> Option<Arc<S>> {
		let mut sessions = self.sessions.write();
		sessions.get_mut(session_id)
			.map(|s| {
				if update_last_message_time {
					s.last_message_time = Instant::now();
				}
				s.session.clone()
			})
	}

	#[cfg(test)]
	pub fn first(&self) -> Option<Arc<S>> {
		self.sessions.read().values().nth(0).map(|s| s.session.clone())
	}

	pub fn insert(&self, cluster: Arc<Cluster>, master: NodeId, session_id: S::Id, session_nonce: Option<u64>, is_exclusive_session: bool, creation_data: Option<D>) -> Result<Arc<S>, Error> {
		let mut sessions = self.sessions.write();
		if sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// create cluster
		// let cluster = create_cluster_view(data, requires_all_connections)?;
		// create session
		let session = self.creator.create(cluster.clone(), master.clone(), session_nonce, session_id.clone(), creation_data)?;
		// check if session can be started
		self.container_state.lock().on_session_starting(is_exclusive_session)?;

		// insert session
		let queued_session = QueuedSession {
			master: master,
			cluster_view: cluster,
			last_keep_alive_time: Instant::now(),
			last_message_time: Instant::now(),
			session: session.clone(),
			queue: VecDeque::new(),
		};
		sessions.insert(session_id, queued_session);
		self.notify_listeners(|l| l.on_session_inserted(session.clone()));

		Ok(session)
	}

	pub fn remove(&self, session_id: &S::Id) {
		if let Some(session) = self.sessions.write().remove(session_id) {
			self.container_state.lock().on_session_completed();
			self.notify_listeners(|l| l.on_session_removed(session.session.clone()));
		}
	}

	pub fn enqueue_message(&self, session_id: &S::Id, sender: NodeId, message: Message, is_queued_message: bool) {
		self.sessions.write().get_mut(session_id)
			.map(|session| if is_queued_message { session.queue.push_front((sender, message)) }
				else { session.queue.push_back((sender, message)) });
	}

	pub fn dequeue_message(&self, session_id: &S::Id) -> Option<(NodeId, Message)> {
		self.sessions.write().get_mut(session_id)
			.and_then(|session| session.queue.pop_front())
	}

	pub fn stop_stalled_sessions(&self) {
		let mut sessions = self.sessions.write();
		for sid in sessions.keys().cloned().collect::<Vec<_>>() {
			let remove_session = {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				if Instant::now() - session.last_message_time > SESSION_TIMEOUT_INTERVAL {
					session.session.on_session_timeout();
					session.session.is_finished()
				} else {
					false
				}
			};

			if remove_session {
				sessions.remove(&sid);
			}
		}
	}

	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		let mut sessions = self.sessions.write();
		for sid in sessions.keys().cloned().collect::<Vec<_>>() {
			let remove_session = {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				session.session.on_node_timeout(node_id);
				session.session.is_finished()
			};
			if remove_session {
				sessions.remove(&sid);
			}
		}
	}

	fn notify_listeners<F: Fn(&ClusterSessionsListener<S>) -> ()>(&self, callback: F) {
		let mut listeners = self.listeners.lock();
		let mut listener_index = 0;
		while listener_index < listeners.len() {
			match listeners[listener_index].upgrade() {
				Some(listener) => {
					callback(&*listener);
					listener_index += 1;
				},
				None => {
					listeners.swap_remove(listener_index);
				},
			}
		}
	}
}

impl<S, SC, D> ClusterSessionsContainer<S, SC, D> where S: ClusterSession, SC: ClusterSessionCreator<S, D>, SessionId: From<S::Id> {
	pub fn send_keep_alive(&self, session_id: &S::Id, self_node_id: &NodeId) {
		if let Some(session) = self.sessions.write().get_mut(session_id) {
			let now = Instant::now();
			if self_node_id == &session.master && now - session.last_keep_alive_time > SESSION_KEEP_ALIVE_INTERVAL {
				session.last_keep_alive_time = now;
				// since we send KeepAlive message to prevent nodes from disconnecting
				// && worst thing that can happen if node is disconnected is that session is failed
				// => ignore error here, because probably this node is not need for the rest of the session at all
				let _ = session.cluster_view.broadcast(Message::Cluster(message::ClusterMessage::KeepAliveResponse(message::KeepAliveResponse {
					session_id: Some(session_id.clone().into()),
				})));
			}
		}
	}

	pub fn on_keep_alive(&self, session_id: &S::Id, sender: &NodeId) {
		if let Some(session) = self.sessions.write().get_mut(session_id) {
			let now = Instant::now();
			// we only accept keep alive from master node of ServersSetChange session
			if sender == &session.master {
				session.last_keep_alive_time = now;
			}
		}
	}
}

impl ClusterSessionsContainerState {
	/// When session is starting.
	pub fn on_session_starting(&mut self, is_exclusive_session: bool) -> Result<(), Error> {
		match *self {
			ClusterSessionsContainerState::Idle if is_exclusive_session => {
				::std::mem::replace(self, ClusterSessionsContainerState::Exclusive);
			},
			ClusterSessionsContainerState::Idle => {
				::std::mem::replace(self, ClusterSessionsContainerState::Active(1));
			},
			ClusterSessionsContainerState::Active(_) if is_exclusive_session =>
				return Err(Error::HasActiveSessions),
			ClusterSessionsContainerState::Active(sessions_count) => {
				::std::mem::replace(self, ClusterSessionsContainerState::Active(sessions_count + 1));
			},
			ClusterSessionsContainerState::Exclusive =>
				return Err(Error::ExclusiveSessionActive),
		}
		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&mut self) {
		match *self {
			ClusterSessionsContainerState::Idle =>
				unreachable!("idle means that there are no active sessions; on_session_completed is only called once after active session is completed; qed"),
			ClusterSessionsContainerState::Active(sessions_count) if sessions_count == 1 => {
				::std::mem::replace(self, ClusterSessionsContainerState::Idle);
			},
			ClusterSessionsContainerState::Active(sessions_count) => {
				::std::mem::replace(self, ClusterSessionsContainerState::Active(sessions_count - 1));
			}
			ClusterSessionsContainerState::Exclusive => {
				::std::mem::replace(self, ClusterSessionsContainerState::Idle);
			},
		}
	}
}

impl SessionIdWithSubSession {
	/// Create new decryption session Id.
	pub fn new(session_id: SessionId, sub_session_id: Secret) -> Self {
		SessionIdWithSubSession {
			id: session_id,
			access_key: sub_session_id,
		}
	}
}

impl PartialOrd for SessionIdWithSubSession {
	fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for SessionIdWithSubSession {
	fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
		match self.id.cmp(&other.id) {
			::std::cmp::Ordering::Equal => self.access_key.cmp(&other.access_key),
			r @ _ => r,
		}
	}
}

impl AdminSession {
	pub fn as_servers_set_change(&self) -> Option<&ServersSetChangeSessionImpl> {
		match *self {
			AdminSession::ServersSetChange(ref session) => Some(session),
			_ => None
		}
	}
}

impl ClusterSession for AdminSession {
	type Id = SessionId;

	fn type_name() -> &'static str {
		"admin"
	}

	fn id(&self) -> SessionId {
		match *self {
			AdminSession::ShareAdd(ref session) => session.id().clone(),
			AdminSession::ServersSetChange(ref session) => session.id().clone(),
		}
	}

	fn is_finished(&self) -> bool {
		match *self {
			AdminSession::ShareAdd(ref session) => session.is_finished(),
			AdminSession::ServersSetChange(ref session) => session.is_finished(),
		}
	}

	fn on_session_timeout(&self) {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_session_timeout(),
			AdminSession::ServersSetChange(ref session) => session.on_session_timeout(),
		}
	}

	fn on_node_timeout(&self, node_id: &NodeId) {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_node_timeout(node_id),
			AdminSession::ServersSetChange(ref session) => session.on_node_timeout(node_id),
		}
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_session_error(node, error),
			AdminSession::ServersSetChange(ref session) => session.on_session_error(node, error),
		}
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_message(sender, message),
			AdminSession::ServersSetChange(ref session) => session.on_message(sender, message),
		}
	}
}
pub fn create_cluster_view(data: &Arc<ClusterData>, requires_all_connections: bool) -> Result<Arc<Cluster>, Error> {
	let disconnected_nodes_count = data.connections.disconnected_nodes().len();
	if requires_all_connections {
		if disconnected_nodes_count != 0 {
			return Err(Error::NodeDisconnected);
		}
	}

	let mut connected_nodes = data.connections.connected_nodes();
	connected_nodes.insert(data.self_key_pair.public().clone());

	let connected_nodes_count = connected_nodes.len();
	Ok(Arc::new(ClusterView::new(data.clone(), connected_nodes, connected_nodes_count + disconnected_nodes_count)))
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use ethkey::{Random, Generator};
	use key_server_cluster::{Error, DummyAclStorage, DummyKeyStorage, MapKeyServerSet, PlainNodeKeyPair};
	use key_server_cluster::cluster::ClusterConfiguration;
	use key_server_cluster::connection_trigger::SimpleServersSetChangeSessionCreatorConnector;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::{SessionImpl as GenerationSession};
	use super::{ClusterSessions, AdminSessionCreationData, ClusterSessionsListener};

	pub fn make_cluster_sessions() -> ClusterSessions {
		let key_pair = Random.generate().unwrap();
		let config = ClusterConfiguration {
			threads: 1,
			self_key_pair: Arc::new(PlainNodeKeyPair::new(key_pair.clone())),
			listen_address: ("127.0.0.1".to_owned(), 100_u16),
			key_server_set: Arc::new(MapKeyServerSet::new(vec![(key_pair.public().clone(), format!("127.0.0.1:{}", 100).parse().unwrap())].into_iter().collect())),
			allow_connecting_to_higher_nodes: false,
			key_storage: Arc::new(DummyKeyStorage::default()),
			acl_storage: Arc::new(DummyAclStorage::default()),
			admin_public: Some(Random.generate().unwrap().public().clone()),
			auto_migrate_enabled: false,
		};
		ClusterSessions::new(&config, Arc::new(SimpleServersSetChangeSessionCreatorConnector {
			admin_public: Some(Random.generate().unwrap().public().clone()),
		}))
	}

	#[test]
	fn cluster_session_cannot_be_started_if_exclusive_session_is_active() {
		let sessions = make_cluster_sessions();
		sessions.generation_sessions.insert(Arc::new(DummyCluster::new(Default::default())), Default::default(), Default::default(), None, false, None).unwrap();
		match sessions.admin_sessions.insert(Arc::new(DummyCluster::new(Default::default())), Default::default(), Default::default(), None, true, Some(AdminSessionCreationData::ShareAdd(Default::default()))) {
			Err(Error::HasActiveSessions) => (),
			Err(e) => unreachable!(format!("{}", e)),
			Ok(_) => unreachable!("OK"),
		}
	}

	#[test]
	fn exclusive_session_cannot_be_started_if_other_session_is_active() {
		let sessions = make_cluster_sessions();

		sessions.admin_sessions.insert(Arc::new(DummyCluster::new(Default::default())), Default::default(), Default::default(), None, true, Some(AdminSessionCreationData::ShareAdd(Default::default()))).unwrap();
		match sessions.generation_sessions.insert(Arc::new(DummyCluster::new(Default::default())), Default::default(), Default::default(), None, false, None) {
			Err(Error::ExclusiveSessionActive) => (),
			Err(e) => unreachable!(format!("{}", e)),
			Ok(_) => unreachable!("OK"),
		}
	}

	#[test]
	fn session_listener_works() {
		#[derive(Default)]
		struct GenerationSessionListener {
			inserted: AtomicUsize,
			removed: AtomicUsize,
		}

		impl ClusterSessionsListener<GenerationSession> for GenerationSessionListener {
			fn on_session_inserted(&self, _session: Arc<GenerationSession>) {
				self.inserted.fetch_add(1, Ordering::Relaxed);
			}

			fn on_session_removed(&self, _session: Arc<GenerationSession>) {
				self.removed.fetch_add(1, Ordering::Relaxed);
			}
		}

		let listener = Arc::new(GenerationSessionListener::default());
		let sessions = make_cluster_sessions();
		sessions.generation_sessions.add_listener(listener.clone());

		sessions.generation_sessions.insert(Arc::new(DummyCluster::new(Default::default())), Default::default(), Default::default(), None, false, None).unwrap();
		assert_eq!(listener.inserted.load(Ordering::Relaxed), 1);
		assert_eq!(listener.removed.load(Ordering::Relaxed), 0);

		sessions.generation_sessions.remove(&Default::default());
		assert_eq!(listener.inserted.load(Ordering::Relaxed), 1);
		assert_eq!(listener.removed.load(Ordering::Relaxed), 1);
	}
}
