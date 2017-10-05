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

use std::time;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use parking_lot::{Mutex, RwLock};
use ethkey::{Public, Secret, Signature};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage, DocumentKeyShare, EncryptedDocumentKeyShadow, SessionMeta};
use key_server_cluster::cluster::{Cluster, ClusterData, ClusterConfiguration};
use key_server_cluster::message::{self, Message, GenerationMessage, EncryptionMessage, DecryptionMessage, SigningMessage,
	ShareAddMessage, ShareMoveMessage, ShareRemoveMessage, ServersSetChangeMessage};
use key_server_cluster::generation_session::{Session as GenerationSession, SessionImpl as GenerationSessionImpl,
	SessionParams as GenerationSessionParams, SessionState as GenerationSessionState};
use key_server_cluster::decryption_session::{Session as DecryptionSession, SessionImpl as DecryptionSessionImpl,
	DecryptionSessionId, SessionParams as DecryptionSessionParams};
use key_server_cluster::encryption_session::{Session as EncryptionSession, SessionImpl as EncryptionSessionImpl,
	SessionParams as EncryptionSessionParams, SessionState as EncryptionSessionState};
use key_server_cluster::signing_session::{Session as SigningSession, SessionImpl as SigningSessionImpl,
	SigningSessionId, SessionParams as SigningSessionParams};
use key_server_cluster::share_add_session::{Session as ShareAddSession, SessionImpl as ShareAddSessionImpl,
	SessionParams as ShareAddSessionParams, IsolatedSessionTransport as ShareAddTransport};
use key_server_cluster::share_move_session::{Session as ShareMoveSession, SessionImpl as ShareMoveSessionImpl,
	SessionParams as ShareMoveSessionParams, IsolatedSessionTransport as ShareMoveTransport};
use key_server_cluster::share_remove_session::{Session as ShareRemoveSession, SessionImpl as ShareRemoveSessionImpl,
	SessionParams as ShareRemoveSessionParams, IsolatedSessionTransport as ShareRemoveTransport};
use key_server_cluster::servers_set_change_session::{Session as ServersSetChangeSession, SessionImpl as ServersSetChangeSessionImpl,
	SessionParams as ServersSetChangeSessionParams};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// When there are no session-related messages for SESSION_TIMEOUT_INTERVAL seconds,
/// we must treat this session as stalled && finish it with an error.
/// This timeout is for cases when node is responding to KeepAlive messages, but intentionally ignores
/// session messages.
const SESSION_TIMEOUT_INTERVAL: u64 = 60;

lazy_static! {
	/// Servers set change session id (there could be at most 1 session => hardcoded id).
	static ref SERVERS_SET_CHANGE_SESSION_ID: SessionId = "10b7af423bb551d5dc8645db754163a2145d37d78d468fa7330435ed77064c1c"
		.parse()
		.expect("hardcoded id should parse without errors; qed");
}

/// Generic cluster session.
pub trait ClusterSession {
	/// If session is finished (either with succcess or not).
	fn is_finished(&self) -> bool;
	/// When it takes too much time to complete session.
	fn on_session_timeout(&self);
	/// When it takes too much time to receive response from the node.
	fn on_node_timeout(&self, node_id: &NodeId);
}

/// Administrative session.
pub enum AdminSession {
	/// Share add session.
	ShareAdd(ShareAddSessionImpl<ShareAddTransport>),
	/// Share move session.
	ShareMove(ShareMoveSessionImpl<ShareMoveTransport>),
	/// Share remove session.
	ShareRemove(ShareRemoveSessionImpl<ShareRemoveTransport>),
	/// Servers set change session.
	ServersSetChange(ServersSetChangeSessionImpl),
}

/// Active sessions on this cluster.
pub struct ClusterSessions {
	/// Key generation sessions.
	pub generation_sessions: ClusterSessionsContainer<SessionId, GenerationSessionImpl, GenerationMessage>,
	/// Encryption sessions.
	pub encryption_sessions: ClusterSessionsContainer<SessionId, EncryptionSessionImpl, EncryptionMessage>,
	/// Decryption sessions.
	pub decryption_sessions: ClusterSessionsContainer<DecryptionSessionId, DecryptionSessionImpl, DecryptionMessage>,
	/// Signing sessions.
	pub signing_sessions: ClusterSessionsContainer<SigningSessionId, SigningSessionImpl, SigningMessage>,
	/// Administrative sessions.
	pub admin_sessions: ClusterSessionsContainer<SessionId, AdminSession, Message>,
	/// Self node id.
	self_node_id: NodeId,
	/// All nodes ids.
	nodes: BTreeSet<NodeId>,
	/// Reference to key storage
	key_storage: Arc<KeyStorage>,
	/// Reference to ACL storage
	acl_storage: Arc<AclStorage>,
	/// Administrator public.
	admin_public: Option<Public>,
	/// Make faulty generation sessions.
	make_faulty_generation_sessions: AtomicBool,
	/// Always-increasing sessions counter. Is used as session nonce to prevent replay attacks:
	/// 1) during handshake, KeyServers generate new random key to encrypt messages
	/// => there's no way to use messages from previous connections for replay attacks
	/// 2) when session (of any type) is started, master node increases its own session counter and broadcasts it
	/// 3) when slave KeyServer receives session initialization message, it checks that new nonce is larger than previous (from the same master)
	/// => there's no way to use messages from previous sessions for replay attacks
	/// 4) KeyServer checks that each session message contains the same nonce that initialization message
	/// Given that: (A) handshake is secure and (B) session itself is initially replay-protected
	/// => this guarantees that sessions are replay-protected.
	session_counter: AtomicUsize,
	/// Maximal session nonce, received from given connection.
	max_nonce: RwLock<BTreeMap<NodeId, u64>>,
}

/// Active sessions container.
pub struct ClusterSessionsContainer<K, V, M> {
	/// Active sessions.
	pub sessions: RwLock<BTreeMap<K, QueuedSession<V, M>>>,
	/// Sessions container state.
	container_state: Arc<Mutex<ClusterSessionsContainerState>>
}

/// Session and its message queue.
pub struct QueuedSession<V, M> {
	/// Session master.
	pub master: NodeId,
	/// Cluster view.
	pub cluster_view: Arc<Cluster>,
	/// Last received message time.
	pub last_message_time: time::Instant,
	/// Generation session.
	pub session: Arc<V>,
	/// Messages queue.
	pub queue: VecDeque<(NodeId, M)>,
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

/// Generation session implementation, which removes session from cluster on drop.
pub struct GenerationSessionWrapper {
	/// Wrapped session.
	session: Arc<GenerationSession>,
	/// Session Id.
	session_id: SessionId,
	/// Cluster data reference.
	cluster: Weak<ClusterData>,
}

/// Encryption session implementation, which removes session from cluster on drop.
pub struct EncryptionSessionWrapper {
	/// Wrapped session.
	session: Arc<EncryptionSession>,
	/// Session Id.
	session_id: SessionId,
	/// Cluster data reference.
	cluster: Weak<ClusterData>,
}

/// Decryption session implementation, which removes session from cluster on drop.
pub struct DecryptionSessionWrapper {
	/// Wrapped session.
	session: Arc<DecryptionSession>,
	/// Session Id.
	session_id: DecryptionSessionId,
	/// Cluster data reference.
	cluster: Weak<ClusterData>,
}

/// Signing session implementation, which removes session from cluster on drop.
pub struct SigningSessionWrapper {
	/// Wrapped session.
	session: Arc<SigningSession>,
	/// Session Id.
	session_id: SigningSessionId,
	/// Cluster data reference.
	cluster: Weak<ClusterData>,
}

/// Admin session implementation, which removes session from cluster on drop.
pub struct AdminSessionWrapper {
	/// Wrapped session.
	session: Arc<AdminSession>,
	/// Session Id.
	session_id: SessionId,
	/// Cluster data reference.
	cluster: Weak<ClusterData>,
}

impl ClusterSessions {
	/// Create new cluster sessions container.
	pub fn new(config: &ClusterConfiguration) -> Self {
		let container_state = Arc::new(Mutex::new(ClusterSessionsContainerState::Idle));
		ClusterSessions {
			self_node_id: config.self_key_pair.public().clone(),
			nodes: config.key_server_set.get().keys().cloned().collect(),
			acl_storage: config.acl_storage.clone(),
			key_storage: config.key_storage.clone(),
			admin_public: config.admin_public.clone(),
			generation_sessions: ClusterSessionsContainer::new(container_state.clone()),
			encryption_sessions: ClusterSessionsContainer::new(container_state.clone()),
			decryption_sessions: ClusterSessionsContainer::new(container_state.clone()),
			signing_sessions: ClusterSessionsContainer::new(container_state.clone()),
			admin_sessions: ClusterSessionsContainer::new(container_state),
			make_faulty_generation_sessions: AtomicBool::new(false),
			session_counter: AtomicUsize::new(0),
			max_nonce: RwLock::new(BTreeMap::new()),
		}
	}

	#[cfg(test)]
	pub fn make_faulty_generation_sessions(&self) {
		self.make_faulty_generation_sessions.store(true, Ordering::Relaxed);
	}

	/// Create new generation session.
	pub fn new_generation_session(&self, master: NodeId, session_id: SessionId, nonce: Option<u64>, cluster: Arc<Cluster>) -> Result<Arc<GenerationSessionImpl>, Error> {
		// check that there's no finished encryption session with the same id
		if self.key_storage.contains(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// communicating to all other nodes is crucial for encryption session
		// => check that we have connections to all cluster nodes
		if self.nodes.iter().any(|n| !cluster.is_connected(n)) {
			return Err(Error::NodeDisconnected);
		}

		// check that there's no active encryption session with the same id
		let nonce = self.check_session_nonce(&master, nonce)?;
		self.generation_sessions.insert(master, session_id, cluster.clone(), false, move ||
			Ok(GenerationSessionImpl::new(GenerationSessionParams {
				id: session_id.clone(),
				self_node_id: self.self_node_id.clone(),
				key_storage: Some(self.key_storage.clone()),
				cluster: cluster,
				nonce: Some(nonce),
			})))
			.map(|session| {
				if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
					session.simulate_faulty_behaviour();
				}
				session
			})
	}

	/// Send generation session error.
	pub fn respond_with_generation_error(&self, session_id: &SessionId, error: message::SessionError) {
		self.generation_sessions.sessions.read().get(session_id)
			.map(|s| {
				// error in generation session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::Generation(GenerationMessage::SessionError(error)));
			});
	}

	/// Create new encryption session.
	pub fn new_encryption_session(&self, master: NodeId, session_id: SessionId, nonce: Option<u64>, cluster: Arc<Cluster>) -> Result<Arc<EncryptionSessionImpl>, Error> {
		let encrypted_data = self.read_key_share(&session_id, &cluster)?;
		let nonce = self.check_session_nonce(&master, nonce)?;

		self.encryption_sessions.insert(master, session_id, cluster.clone(), false, move || EncryptionSessionImpl::new(EncryptionSessionParams {
			id: session_id.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: encrypted_data,
			key_storage: self.key_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}))
	}

	/// Send encryption session error.
	pub fn respond_with_encryption_error(&self, session_id: &SessionId, error: message::EncryptionSessionError) {
		self.encryption_sessions.sessions.read().get(session_id)
			.map(|s| {
				// error in encryption session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::Encryption(EncryptionMessage::EncryptionSessionError(error)));
			});
	}

	/// Create new decryption session.
	pub fn new_decryption_session(&self, master: NodeId, session_id: SessionId, sub_session_id: Secret, nonce: Option<u64>, cluster: Arc<Cluster>, requester_signature: Option<Signature>) -> Result<Arc<DecryptionSessionImpl>, Error> {
		let session_id = DecryptionSessionId::new(session_id, sub_session_id);
		let encrypted_data = self.read_key_share(&session_id.id, &cluster)?;
		let nonce = self.check_session_nonce(&master, nonce)?;

		self.decryption_sessions.insert(master, session_id.clone(), cluster.clone(), false, move || DecryptionSessionImpl::new(DecryptionSessionParams {
			meta: SessionMeta {
				id: session_id.id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.threshold,
			},
			access_key: session_id.access_key,
			key_share: encrypted_data,
			acl_storage: self.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester_signature))
	}

	/// Send decryption session error.
	pub fn respond_with_decryption_error(&self, session_id: &SessionId, sub_session_id: &Secret, to: &NodeId, error: message::DecryptionSessionError) {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in decryption session is non-fatal, if occurs on slave node
				// => either respond with error
				// => or broadcast error

				// do not bother processing send error, as we already processing error
				if s.master == self.self_node_id {
					let _ = s.cluster_view.broadcast(Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				} else {
					let _ = s.cluster_view.send(to, Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				}
			});
	}

	/// Create new signing session.
	pub fn new_signing_session(&self, master: NodeId, session_id: SessionId, sub_session_id: Secret, nonce: Option<u64>, cluster: Arc<Cluster>, requester_signature: Option<Signature>) -> Result<Arc<SigningSessionImpl>, Error> {
		let session_id = SigningSessionId::new(session_id, sub_session_id);
		let encrypted_data = self.read_key_share(&session_id.id, &cluster)?;
		let nonce = self.check_session_nonce(&master, nonce)?;

		self.signing_sessions.insert(master, session_id.clone(), cluster.clone(), false, move || SigningSessionImpl::new(SigningSessionParams {
			meta: SessionMeta {
				id: session_id.id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.threshold,
			},
			access_key: session_id.access_key,
			key_share: encrypted_data,
			acl_storage: self.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester_signature))
	}

	/// Send signing session error.
	pub fn respond_with_signing_error(&self, session_id: &SessionId, sub_session_id: &Secret, to: &NodeId, error: message::SigningSessionError) {
		let session_id = SigningSessionId::new(session_id.clone(), sub_session_id.clone());
		self.signing_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in signing session is non-fatal, if occurs on slave node
				// => either respond with error
				// => or broadcast error

				// do not bother processing send error, as we already processing error
				if s.master == self.self_node_id {
					let _ = s.cluster_view.broadcast(Message::Signing(SigningMessage::SigningSessionError(error)));
				} else {
					let _ = s.cluster_view.send(to, Message::Signing(SigningMessage::SigningSessionError(error)));
				}
			});
	}

	/// Create new share add session.
	pub fn new_share_add_session(&self, master: NodeId, session_id: SessionId, nonce: Option<u64>, cluster: Arc<Cluster>) -> Result<Arc<AdminSession>, Error> {
		let nonce = self.check_session_nonce(&master, nonce)?;
		let admin_public = self.admin_public.clone().ok_or(Error::AccessDenied)?;

		self.admin_sessions.insert(master, session_id.clone(), cluster.clone(), false, move || ShareAddSessionImpl::new(ShareAddSessionParams {
			meta: ShareChangeSessionMeta {
				id: session_id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
			},
			transport: ShareAddTransport::new(session_id.clone(), nonce, cluster),
			key_storage: self.key_storage.clone(),
			admin_public: Some(admin_public),
			nonce: nonce,
		}).map(AdminSession::ShareAdd))
	}

	/// Send share add session error.
	pub fn respond_with_share_add_error(&self, session_id: &SessionId, error: message::ShareAddError) {
		self.admin_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in any share change session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::ShareAdd(ShareAddMessage::ShareAddError(error)));
			});
	}

	/// Create new share move session.
	pub fn new_share_move_session(&self, master: NodeId, session_id: SessionId, nonce: Option<u64>, cluster: Arc<Cluster>) -> Result<Arc<AdminSession>, Error> {
		let nonce = self.check_session_nonce(&master, nonce)?;
		let admin_public = self.admin_public.clone().ok_or(Error::AccessDenied)?;

		self.admin_sessions.insert(master, session_id.clone(), cluster.clone(), false, move || ShareMoveSessionImpl::new(ShareMoveSessionParams {
			meta: ShareChangeSessionMeta {
				id: session_id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
			},
			transport: ShareMoveTransport::new(session_id.clone(), nonce, cluster),
			key_storage: self.key_storage.clone(),
			admin_public: Some(admin_public),
			nonce: nonce,
		}).map(AdminSession::ShareMove))
	}

	/// Send share move session error.
	pub fn respond_with_share_move_error(&self, session_id: &SessionId, error: message::ShareMoveError) {
		self.admin_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in any share change session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::ShareMove(ShareMoveMessage::ShareMoveError(error)));
			});
	}

	/// Create new share remove session.
	pub fn new_share_remove_session(&self, master: NodeId, session_id: SessionId, nonce: Option<u64>, cluster: Arc<Cluster>) -> Result<Arc<AdminSession>, Error> {
		let nonce = self.check_session_nonce(&master, nonce)?;
		let admin_public = self.admin_public.clone().ok_or(Error::AccessDenied)?;

		self.admin_sessions.insert(master, session_id.clone(), cluster.clone(), false, move || ShareRemoveSessionImpl::new(ShareRemoveSessionParams {
			meta: ShareChangeSessionMeta {
				id: session_id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
			},
			transport: ShareRemoveTransport::new(session_id.clone(), nonce, cluster),
			key_storage: self.key_storage.clone(),
			admin_public: Some(admin_public),
			nonce: nonce,
		}).map(AdminSession::ShareRemove))
	}

	/// Send share remove session error.
	pub fn respond_with_share_remove_error(&self, session_id: &SessionId, error: message::ShareRemoveError) {
		self.admin_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in any share change session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::ShareRemove(ShareRemoveMessage::ShareRemoveError(error)));
			});
	}

	/// Create new servers set change session.
	pub fn new_servers_set_change_session(&self, master: NodeId, session_id: Option<SessionId>, nonce: Option<u64>, cluster: Arc<Cluster>, all_nodes_set: BTreeSet<NodeId>) -> Result<Arc<AdminSession>, Error> {
		let session_id = match session_id {
			Some(session_id) => if session_id == *SERVERS_SET_CHANGE_SESSION_ID {
				session_id
			} else {
				return Err(Error::InvalidMessage)
			},
			None => (*SERVERS_SET_CHANGE_SESSION_ID).clone(),
		};
		let nonce = self.check_session_nonce(&master, nonce)?;
		let admin_public = self.admin_public.clone().ok_or(Error::AccessDenied)?;

		self.admin_sessions.insert(master, session_id.clone(), cluster.clone(), true, move || ServersSetChangeSessionImpl::new(ServersSetChangeSessionParams {
			meta: ShareChangeSessionMeta {
				id: session_id,
				self_node_id: self.self_node_id.clone(),
				master_node_id: master,
			},
			cluster: cluster,
			key_storage: self.key_storage.clone(),
			admin_public: admin_public,
			nonce: nonce,
			all_nodes_set: all_nodes_set,
		}).map(AdminSession::ServersSetChange))
	}

	/// Send share remove session error.
	pub fn respond_with_servers_set_change_error(&self, session_id: &SessionId, error: message::ServersSetChangeError) {
		self.admin_sessions.sessions.read().get(&session_id)
			.map(|s| {
				// error in any share change session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeError(error)));
			});
	}

	/// Stop sessions that are stalling.
	pub fn stop_stalled_sessions(&self) {
		self.generation_sessions.stop_stalled_sessions();
		self.encryption_sessions.stop_stalled_sessions();
		self.decryption_sessions.stop_stalled_sessions();
		self.signing_sessions.stop_stalled_sessions();
		// TODO: servers set change session could take a lot of time
		// && during that session some nodes could not receive messages
		// => they could stop session as stalled. This must be handled
		self.admin_sessions.stop_stalled_sessions();
	}

	/// When connection to node is lost.
	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		self.generation_sessions.on_connection_timeout(node_id);
		self.encryption_sessions.on_connection_timeout(node_id);
		self.decryption_sessions.on_connection_timeout(node_id);
		self.signing_sessions.on_connection_timeout(node_id);
		self.admin_sessions.on_connection_timeout(node_id);
		self.max_nonce.write().remove(node_id);
	}

	/// Read key share && remove disconnected nodes.
	fn read_key_share(&self, key_id: &SessionId, cluster: &Arc<Cluster>) -> Result<DocumentKeyShare, Error> {
		let mut encrypted_data = self.key_storage.get(key_id).map_err(|e| Error::KeyStorage(e.into()))?;

		// some of nodes, which were encrypting secret may be down
		// => do not use these in session
		let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		for disconnected_node in disconnected_nodes.difference(&cluster.nodes()) {
			encrypted_data.id_numbers.remove(&disconnected_node);
		}
		Ok(encrypted_data)
	}

	/// Check or generate new session nonce.
	fn check_session_nonce(&self, master: &NodeId, nonce: Option<u64>) -> Result<u64, Error> {
		// if we're master node of the session, then nonce should be generated
		// if we're slave node of the session, then nonce should be passed from outside
		debug_assert!((master == &self.self_node_id) == nonce.is_none());

		match nonce {
			Some(nonce) => match nonce > *self.max_nonce.write().entry(master.clone()).or_insert(0) {
				true => Ok(nonce),
				false => Err(Error::ReplayProtection),
			},
			None => Ok(self.session_counter.fetch_add(1, Ordering::Relaxed) as u64 + 1),
		}
	}
}

impl<K, V, M> ClusterSessionsContainer<K, V, M> where K: Clone + Ord, V: ClusterSession {
	pub fn new(container_state: Arc<Mutex<ClusterSessionsContainerState>>) -> Self {
		ClusterSessionsContainer {
			sessions: RwLock::new(BTreeMap::new()),
			container_state: container_state,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.sessions.read().is_empty()
	}

	pub fn get(&self, session_id: &K) -> Option<Arc<V>> {
		self.sessions.read().get(session_id).map(|s| s.session.clone())
	}

	pub fn insert<F: FnOnce() -> Result<V, Error>>(&self, master: NodeId, session_id: K, cluster: Arc<Cluster>, is_exclusive_session: bool, session: F) -> Result<Arc<V>, Error> {
		let mut sessions = self.sessions.write();
		if sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// create session
		let session = Arc::new(session()?);
		// check if session can be started
		self.container_state.lock().on_session_starting(is_exclusive_session)?;

		// insert session
		let queued_session = QueuedSession {
			master: master,
			cluster_view: cluster,
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new(),
		};
		sessions.insert(session_id, queued_session);
		Ok(session)
	}

	pub fn remove(&self, session_id: &K) {
		if self.sessions.write().remove(session_id).is_some() {
			self.container_state.lock().on_session_completed();
		}
	}

	pub fn enqueue_message(&self, session_id: &K, sender: NodeId, message: M, is_queued_message: bool) {
		self.sessions.write().get_mut(session_id)
			.map(|session| if is_queued_message { session.queue.push_front((sender, message)) }
				else { session.queue.push_back((sender, message)) });
	}

	pub fn dequeue_message(&self, session_id: &K) -> Option<(NodeId, M)> {
		self.sessions.write().get_mut(session_id)
			.and_then(|session| session.queue.pop_front())
	}

	pub fn stop_stalled_sessions(&self) {
		let mut sessions = self.sessions.write();
		for sid in sessions.keys().cloned().collect::<Vec<_>>() {
			let remove_session = {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				if time::Instant::now() - session.last_message_time > time::Duration::from_secs(SESSION_TIMEOUT_INTERVAL) {
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

impl AdminSession {
	pub fn as_share_add(&self) -> Option<&ShareAddSessionImpl<ShareAddTransport>> {
		match *self {
			AdminSession::ShareAdd(ref session) => Some(session),
			_ => None
		}
	}

	pub fn as_share_move(&self) -> Option<&ShareMoveSessionImpl<ShareMoveTransport>> {
		match *self {
			AdminSession::ShareMove(ref session) => Some(session),
			_ => None
		}
	}

	pub fn as_share_remove(&self) -> Option<&ShareRemoveSessionImpl<ShareRemoveTransport>> {
		match *self {
			AdminSession::ShareRemove(ref session) => Some(session),
			_ => None
		}
	}

	pub fn as_servers_set_change(&self) -> Option<&ServersSetChangeSessionImpl> {
		match *self {
			AdminSession::ServersSetChange(ref session) => Some(session),
			_ => None
		}
	}
}

impl ClusterSession for AdminSession {
	fn is_finished(&self) -> bool {
		match *self {
			AdminSession::ShareAdd(ref session) => session.is_finished(),
			AdminSession::ShareMove(ref session) => session.is_finished(),
			AdminSession::ShareRemove(ref session) => session.is_finished(),
			AdminSession::ServersSetChange(ref session) => session.is_finished(),
		}
	}

	fn on_session_timeout(&self) {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_session_timeout(),
			AdminSession::ShareMove(ref session) => session.on_session_timeout(),
			AdminSession::ShareRemove(ref session) => session.on_session_timeout(),
			AdminSession::ServersSetChange(ref session) => session.on_session_timeout(),
		}
	}

	fn on_node_timeout(&self, node_id: &NodeId) {
		match *self {
			AdminSession::ShareAdd(ref session) => session.on_node_timeout(node_id),
			AdminSession::ShareMove(ref session) => session.on_node_timeout(node_id),
			AdminSession::ShareRemove(ref session) => session.on_node_timeout(node_id),
			AdminSession::ServersSetChange(ref session) => session.on_node_timeout(node_id),
		}
	}
}

impl GenerationSessionWrapper {
	pub fn new(cluster: Weak<ClusterData>, session_id: SessionId, session: Arc<GenerationSession>) -> Arc<Self> {
		Arc::new(GenerationSessionWrapper {
			session: session,
			session_id: session_id,
			cluster: cluster,
		})
	}
}

impl GenerationSession for GenerationSessionWrapper {
	fn state(&self) -> GenerationSessionState {
		self.session.state()
	}

	fn wait(&self, timeout: Option<time::Duration>) -> Result<Public, Error> {
		self.session.wait(timeout)
	}

	fn joint_public_and_secret(&self) -> Option<Result<(Public, Secret), Error>> {
		self.session.joint_public_and_secret()
	}
}

impl Drop for GenerationSessionWrapper {
	fn drop(&mut self) {
		if let Some(cluster) = self.cluster.upgrade() {
			cluster.sessions().generation_sessions.remove(&self.session_id);
		}
	}
}

impl EncryptionSessionWrapper {
	pub fn new(cluster: Weak<ClusterData>, session_id: SessionId, session: Arc<EncryptionSession>) -> Arc<Self> {
		Arc::new(EncryptionSessionWrapper {
			session: session,
			session_id: session_id,
			cluster: cluster,
		})
	}
}

impl EncryptionSession for EncryptionSessionWrapper {
	fn state(&self) -> EncryptionSessionState {
		self.session.state()
	}

	fn wait(&self, timeout: Option<time::Duration>) -> Result<(), Error> {
		self.session.wait(timeout)
	}
}

impl Drop for EncryptionSessionWrapper {
	fn drop(&mut self) {
		if let Some(cluster) = self.cluster.upgrade() {
			cluster.sessions().encryption_sessions.remove(&self.session_id);
		}
	}
}

impl DecryptionSessionWrapper {
	pub fn new(cluster: Weak<ClusterData>, session_id: DecryptionSessionId, session: Arc<DecryptionSession>) -> Arc<Self> {
		Arc::new(DecryptionSessionWrapper {
			session: session,
			session_id: session_id,
			cluster: cluster,
		})
	}
}

impl DecryptionSession for DecryptionSessionWrapper {
	fn wait(&self) -> Result<EncryptedDocumentKeyShadow, Error> {
		self.session.wait()
	}
}

impl Drop for DecryptionSessionWrapper {
	fn drop(&mut self) {
		if let Some(cluster) = self.cluster.upgrade() {
			cluster.sessions().decryption_sessions.remove(&self.session_id);
		}
	}
}

impl SigningSessionWrapper {
	pub fn new(cluster: Weak<ClusterData>, session_id: SigningSessionId, session: Arc<SigningSession>) -> Arc<Self> {
		Arc::new(SigningSessionWrapper {
			session: session,
			session_id: session_id,
			cluster: cluster,
		})
	}
}

impl SigningSession for SigningSessionWrapper {
	fn wait(&self) -> Result<(Secret, Secret), Error> {
		self.session.wait()
	}
}

impl Drop for SigningSessionWrapper {
	fn drop(&mut self) {
		if let Some(cluster) = self.cluster.upgrade() {
			cluster.sessions().signing_sessions.remove(&self.session_id);
		}
	}
}

impl AdminSessionWrapper {
	pub fn new(cluster: Weak<ClusterData>, session_id: SessionId, session: Arc<AdminSession>) -> Arc<Self> {
		Arc::new(AdminSessionWrapper {
			session: session,
			session_id: session_id,
			cluster: cluster,
		})
	}
}

impl ShareAddSession for AdminSessionWrapper {
	fn wait(&self) -> Result<(), Error> {
		match *self.session {
			AdminSession::ShareAdd(ref session) => session.wait(),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl ShareMoveSession for AdminSessionWrapper {
	fn wait(&self) -> Result<(), Error> {
		match *self.session {
			AdminSession::ShareMove(ref session) => session.wait(),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl ShareRemoveSession for AdminSessionWrapper {
	fn wait(&self) -> Result<(), Error> {
		match *self.session {
			AdminSession::ShareRemove(ref session) => session.wait(),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl ServersSetChangeSession for AdminSessionWrapper {
	fn wait(&self) -> Result<(), Error> {
		match *self.session {
			AdminSession::ServersSetChange(ref session) => session.wait(),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl Drop for AdminSessionWrapper {
	fn drop(&mut self) {
		if let Some(cluster) = self.cluster.upgrade() {
			cluster.sessions().admin_sessions.remove(&self.session_id);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::BTreeSet;
	use ethkey::{Random, Generator};
	use key_server_cluster::{Error, DummyAclStorage, DummyKeyStorage, MapKeyServerSet, PlainNodeKeyPair};
	use key_server_cluster::cluster::ClusterConfiguration;
	use key_server_cluster::cluster::tests::DummyCluster;
	use super::ClusterSessions;

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
		};
		ClusterSessions::new(&config)
	}

	#[test]
	fn cluster_session_cannot_be_started_if_exclusive_session_is_active() {
		let sessions = make_cluster_sessions();

		sessions.new_generation_session(Default::default(), Default::default(), Some(1), Arc::new(DummyCluster::new(sessions.self_node_id.clone()))).unwrap();
		match sessions.new_servers_set_change_session(Default::default(), None, Some(1), Arc::new(DummyCluster::new(sessions.self_node_id.clone())), BTreeSet::new()) {
			Err(Error::HasActiveSessions) => (),
			Err(e) => unreachable!(format!("{}", e)),
			Ok(_) => unreachable!("OK"),
		}
	}

	#[test]
	fn exclusive_session_cannot_be_started_if_other_session_is_active() {
		let sessions = make_cluster_sessions();

		sessions.new_servers_set_change_session(Default::default(), None, Some(1), Arc::new(DummyCluster::new(sessions.self_node_id.clone())), BTreeSet::new()).unwrap();
		match sessions.new_generation_session(Default::default(), Default::default(), Some(1), Arc::new(DummyCluster::new(sessions.self_node_id.clone()))) {
			Err(Error::ExclusiveSessionActive) => (),
			Err(e) => unreachable!(format!("{}", e)),
			Ok(_) => unreachable!("OK"),
		}
	}
}
