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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use parking_lot::RwLock;
use ethkey::Secret;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage};
use key_server_cluster::cluster::{Cluster, ClusterView, ClusterConfiguration};
use key_server_cluster::message::{self, Message, GenerationMessage, EncryptionMessage, DecryptionMessage};
use key_server_cluster::generation_session::{SessionImpl as GenerationSessionImpl, SessionParams as GenerationSessionParams};
use key_server_cluster::decryption_session::{SessionImpl as DecryptionSessionImpl, DecryptionSessionId,
	SessionParams as DecryptionSessionParams};
use key_server_cluster::encryption_session::{SessionImpl as EncryptionSessionImpl, SessionParams as EncryptionSessionParams};

/// When there are no session-related messages for SESSION_TIMEOUT_INTERVAL seconds,
/// we must treat this session as stalled && finish it with an error.
/// This timeout is for cases when node is responding to KeepAlive messages, but intentionally ignores
/// session messages.
const SESSION_TIMEOUT_INTERVAL: u64 = 60;

/// Generic cluster session.
pub trait ClusterSession {
	/// If session is finished (either with succcess or not).
	fn is_finished(&self) -> bool;
	/// When it takes too much time to complete session.
	fn on_session_timeout(&self);
	/// When it takes too much time to receive response from the node.
	fn on_node_timeout(&self, node_id: &NodeId);
}
/// Active sessions on this cluster.
pub struct ClusterSessions {
	/// Key generation sessions.
	pub generation_sessions: ClusterSessionsContainer<SessionId, GenerationSessionImpl, GenerationMessage>,
	/// Encryption sessions.
	pub encryption_sessions: ClusterSessionsContainer<SessionId, EncryptionSessionImpl, EncryptionMessage>,
	/// Decryption sessions.
	pub decryption_sessions: ClusterSessionsContainer<DecryptionSessionId, DecryptionSessionImpl, DecryptionMessage>,
	/// Self node id.
	self_node_id: NodeId,
	/// All nodes ids.
	nodes: BTreeSet<NodeId>,
	/// Reference to key storage
	key_storage: Arc<KeyStorage>,
	/// Reference to ACL storage
	acl_storage: Arc<AclStorage>,
	/// Make faulty generation sessions.
	make_faulty_generation_sessions: AtomicBool,
}

/// Active sessions container.
pub struct ClusterSessionsContainer<K, V, M> {
	/// Active sessions.
	pub sessions: RwLock<BTreeMap<K, QueuedSession<V, M>>>,
}

/// Session and its message queue.
pub struct QueuedSession<V, M> {
	/// Session master.
	pub master: NodeId,
	/// Cluster view.
	pub cluster_view: Arc<ClusterView>,
	/// Last received message time.
	pub last_message_time: time::Instant,
	/// Generation session.
	pub session: Arc<V>,
	/// Messages queue.
	pub queue: VecDeque<(NodeId, M)>,
}

impl ClusterSessions {
	/// Create new cluster sessions container.
	pub fn new(config: &ClusterConfiguration) -> Self {
		ClusterSessions {
			self_node_id: config.self_key_pair.public().clone(),
			nodes: config.nodes.keys().cloned().collect(),
			acl_storage: config.acl_storage.clone(),
			key_storage: config.key_storage.clone(),
			generation_sessions: ClusterSessionsContainer::new(),
			encryption_sessions: ClusterSessionsContainer::new(),
			decryption_sessions: ClusterSessionsContainer::new(),
			make_faulty_generation_sessions: AtomicBool::new(false),
		}
	}

	#[cfg(test)]
	pub fn make_faulty_generation_sessions(&self) {
		self.make_faulty_generation_sessions.store(true, Ordering::Relaxed);
	}

	/// Create new generation session.
	pub fn new_generation_session(&self, master: NodeId, session_id: SessionId, cluster: Arc<ClusterView>) -> Result<Arc<GenerationSessionImpl>, Error> {
		// check that there's no active encryption session with the same id
		if self.generation_sessions.contains(&session_id) { // TODO: possible race here and below
			return Err(Error::DuplicateSessionId);
		}
		// check that there's no finished encryption session with the same id
		if self.key_storage.contains(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// communicating to all other nodes is crucial for encryption session
		// => check that we have connections to all cluster nodes
		if self.nodes.iter().any(|n| !cluster.is_connected(n)) {
			return Err(Error::NodeDisconnected);
		}

		let session = self.generation_sessions.insert(master, session_id, cluster.clone(), GenerationSessionImpl::new(GenerationSessionParams {
			id: session_id.clone(),
			self_node_id: self.self_node_id.clone(),
			key_storage: self.key_storage.clone(),
			cluster: cluster,
		}));
		if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
			session.simulate_faulty_behaviour();
		}

		Ok(session)
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
	pub fn new_encryption_session(&self, master: NodeId, session_id: SessionId, cluster: Arc<ClusterView>) -> Result<Arc<EncryptionSessionImpl>, Error> {
		if self.encryption_sessions.contains(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// some of nodes, which were generating the key may be down
		// => do not use these in encryption session
		let mut encrypted_data = self.key_storage.get(&session_id).map_err(|e| Error::KeyStorage(e.into()))?;
		let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		let disconnected_nodes: BTreeSet<_> = disconnected_nodes.difference(&cluster.nodes()).cloned().collect();
		for disconnected_node in disconnected_nodes {
			encrypted_data.id_numbers.remove(&disconnected_node);
		}

		Ok(self.encryption_sessions.insert(master, session_id, cluster.clone(), EncryptionSessionImpl::new(EncryptionSessionParams {
			id: session_id.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: encrypted_data,
			key_storage: self.key_storage.clone(),
			cluster: cluster,
		})?))
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
	pub fn new_decryption_session(&self, master: NodeId, session_id: SessionId, sub_session_id: Secret, cluster: Arc<ClusterView>) -> Result<Arc<DecryptionSessionImpl>, Error> {
		let session_id = DecryptionSessionId::new(session_id, sub_session_id);
		if self.decryption_sessions.contains(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// some of nodes, which were encrypting secret may be down
		// => do not use these in decryption session
		let mut encrypted_data = self.key_storage.get(&session_id.id).map_err(|e| Error::KeyStorage(e.into()))?;
		let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		let disconnected_nodes: BTreeSet<_> = disconnected_nodes.difference(&cluster.nodes()).cloned().collect();
		for disconnected_node in disconnected_nodes {
			encrypted_data.id_numbers.remove(&disconnected_node);
		}

		Ok(self.decryption_sessions.insert(master, session_id.clone(), cluster.clone(), DecryptionSessionImpl::new(DecryptionSessionParams {
			id: session_id.id,
			access_key: session_id.access_key,
			self_node_id: self.self_node_id.clone(),
			encrypted_data: encrypted_data,
			acl_storage: self.acl_storage.clone(),
			cluster: cluster,
		})?))
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
				if &s.master == s.session.node() {
					let _ = s.cluster_view.broadcast(Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				} else {
					let _ = s.cluster_view.send(to, Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				}
			});
	}

	/// Stop sessions that are stalling.
	pub fn stop_stalled_sessions(&self) {
		self.generation_sessions.stop_stalled_sessions();
		self.encryption_sessions.stop_stalled_sessions();
		self.decryption_sessions.stop_stalled_sessions();
	}

	/// When connection to node is lost.
	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		self.generation_sessions.on_connection_timeout(node_id);
		self.encryption_sessions.on_connection_timeout(node_id);
		self.decryption_sessions.on_connection_timeout(node_id);
	}
}

impl<K, V, M> ClusterSessionsContainer<K, V, M> where K: Clone + Ord, V: ClusterSession {
	pub fn new() -> Self {
		ClusterSessionsContainer {
			sessions: RwLock::new(BTreeMap::new()),
		}
	}

	pub fn contains(&self, session_id: &K) -> bool {
		self.sessions.read().contains_key(session_id)
	}

	pub fn get(&self, session_id: &K) -> Option<Arc<V>> {
		self.sessions.read().get(session_id).map(|s| s.session.clone())
	}

	pub fn insert(&self, master: NodeId, session_id: K, cluster: Arc<ClusterView>, session: V) -> Arc<V> {
		let session = Arc::new(session);
		let queued_session = QueuedSession {
			master: master,
			cluster_view: cluster,
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new(),
		};
		self.sessions.write().insert(session_id, queued_session);
		session
	}

	pub fn remove(&self, session_id: &K) {
		self.sessions.write().remove(session_id);
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

/*
impl Clu

	pub fn respond_with_generation_error(&self, session_id: &SessionId, error: message::SessionError) {
		self.generation_sessions.read().get(session_id)
			.map(|s| {
				// error in generation session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::Generation(GenerationMessage::SessionError(error)));
			});
	}






	pub fn new_session(&self, master: NodeId, session_id: SessionId, cluster: Arc<ClusterView>) -> Result<Arc<GenerationSessionImpl>, Error> {
		let mut generation_sessions = self.generation_sessions.write();
		// check that there's no active encryption session with the same id
		if generation_sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}
		// check that there's no finished encryption session with the same id
		if self.key_storage.contains(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// communicating to all other nodes is crucial for encryption session
		// => check that we have connections to all cluster nodes
		if self.nodes.iter().any(|n| !cluster.is_connected(n)) {
			return Err(Error::NodeDisconnected);
		}

		let session = Arc::new(GenerationSessionImpl::new(GenerationSessionParams {
			id: session_id.clone(),
			self_node_id: self.self_node_id.clone(),
			key_storage: self.key_storage.clone(),
			cluster: cluster.clone(),
		}));
		let generation_session = QueuedGenerationSession {
			master: master,
			cluster_view: cluster,
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new()
		};
		if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
			generation_session.session.simulate_faulty_behaviour();
		}
		generation_sessions.insert(session_id, generation_session);
		Ok(session)
	}

	pub fn remove_generation_session(&self, session_id: &SessionId) {
		self.generation_sessions.write().remove(session_id);
	}

	pub fn generation_session(&self, session_id: &SessionId) -> Option<Arc<GenerationSessionImpl>> {
		self.generation_sessions.read().get(session_id).map(|s| s.session.clone())
	}

	pub fn enqueue_generation_message(&self, session_id: &SessionId, sender: NodeId, message: GenerationMessage, is_queued_message: bool) {
		self.generation_sessions.write().get_mut(session_id)
			.map(|session| if is_queued_message { session.queue.push_front((sender, message)) }
				else { session.queue.push_back((sender, message)) });
	}

	pub fn dequeue_generation_message(&self, session_id: &SessionId) -> Option<(NodeId, GenerationMessage)> {
		self.generation_sessions.write().get_mut(session_id)
			.and_then(|session| session.queue.pop_front())
	}

	pub fn respond_with_generation_error(&self, session_id: &SessionId, error: message::SessionError) {
		self.generation_sessions.read().get(session_id)
			.map(|s| {
				// error in generation session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::Generation(GenerationMessage::SessionError(error)));
			});
	}

	pub fn new_encryption_session(&self, master: NodeId, session_id: SessionId, cluster: Arc<ClusterView>) -> Result<Arc<EncryptionSessionImpl>, Error> {
		let mut encryption_sessions = self.encryption_sessions.write();
		if encryption_sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// some of nodes, which were generating the key may be down
		// => do not use these in encryption session
		let mut encrypted_data = self.key_storage.get(&session_id).map_err(|e| Error::KeyStorage(e.into()))?;
		let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		let disconnected_nodes: BTreeSet<_> = disconnected_nodes.difference(&cluster.nodes()).cloned().collect();
		for disconnected_node in disconnected_nodes {
			encrypted_data.id_numbers.remove(&disconnected_node);
		}

		let session = Arc::new(EncryptionSessionImpl::new(EncryptionSessionParams {
			id: session_id.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: encrypted_data,
			key_storage: self.key_storage.clone(),
			cluster: cluster.clone(),
		})?);
		let encryption_session = QueuedEncryptionSession {
			master: master,
			cluster_view: cluster,
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new()
		};
		encryption_sessions.insert(session_id, encryption_session);
		Ok(session)
	}

	pub fn remove_encryption_session(&self, session_id: &SessionId) {
		self.encryption_sessions.write().remove(session_id);
	}

	pub fn encryption_session(&self, session_id: &SessionId) -> Option<Arc<EncryptionSessionImpl>> {
		self.encryption_sessions.read().get(session_id).map(|s| s.session.clone())
	}

	pub fn enqueue_encryption_message(&self, session_id: &SessionId, sender: NodeId, message: EncryptionMessage, is_queued_message: bool) {
		self.encryption_sessions.write().get_mut(session_id)
			.map(|session| if is_queued_message { session.queue.push_front((sender, message)) }
				else { session.queue.push_back((sender, message)) });
	}

	pub fn dequeue_encryption_message(&self, session_id: &SessionId) -> Option<(NodeId, EncryptionMessage)> {
		self.encryption_sessions.write().get_mut(session_id)
			.and_then(|session| session.queue.pop_front())
	}

	pub fn respond_with_encryption_error(&self, session_id: &SessionId, error: message::EncryptionSessionError) {
		self.encryption_sessions.read().get(session_id)
			.map(|s| {
				// error in encryption session is considered fatal
				// => broadcast error

				// do not bother processing send error, as we already processing error
				let _ = s.cluster_view.broadcast(Message::Encryption(EncryptionMessage::EncryptionSessionError(error)));
			});
	}

	#[cfg(test)]
	pub fn make_faulty_generation_sessions(&self) {
		self.make_faulty_generation_sessions.store(true, Ordering::Relaxed);
	}

	pub fn new_decryption_session(&self, master: NodeId, session_id: SessionId, sub_session_id: Secret, cluster: Arc<ClusterView>) -> Result<Arc<DecryptionSessionImpl>, Error> {
		let mut decryption_sessions = self.decryption_sessions.write();
		let session_id = DecryptionSessionId::new(session_id, sub_session_id);
		if decryption_sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// some of nodes, which were encrypting secret may be down
		// => do not use these in decryption session
		let mut encrypted_data = self.key_storage.get(&session_id.id).map_err(|e| Error::KeyStorage(e.into()))?;
		let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		let disconnected_nodes: BTreeSet<_> = disconnected_nodes.difference(&cluster.nodes()).cloned().collect();
		for disconnected_node in disconnected_nodes {
			encrypted_data.id_numbers.remove(&disconnected_node);
		}

		let session = Arc::new(DecryptionSessionImpl::new(DecryptionSessionParams {
			id: session_id.id.clone(),
			access_key: session_id.access_key.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: encrypted_data,
			acl_storage: self.acl_storage.clone(),
			cluster: cluster.clone(),
		})?);
		let decryption_session = QueuedDecryptionSession {
			master: master,
			cluster_view: cluster,
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new()
		};
		decryption_sessions.insert(session_id, decryption_session);
		Ok(session)
	}

	pub fn remove_decryption_session(&self, session_id: &SessionId, sub_session_id: &Secret) {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.write().remove(&session_id);
	}

	pub fn decryption_session(&self, session_id: &SessionId, sub_session_id: &Secret) -> Option<Arc<DecryptionSessionImpl>> {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.read().get(&session_id).map(|s| s.session.clone())
	}

	pub fn enqueue_decryption_message(&self, session_id: &SessionId, sub_session_id: &Secret, sender: NodeId, message: DecryptionMessage, is_queued_message: bool) {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.write().get_mut(&session_id)
			.map(|session| if is_queued_message { session.queue.push_front((sender, message)) }
				else { session.queue.push_back((sender, message)) });
	}

	pub fn dequeue_decryption_message(&self, session_id: &SessionId, sub_session_id: &Secret) -> Option<(NodeId, DecryptionMessage)> {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.write().get_mut(&session_id)
			.and_then(|session| session.queue.pop_front())
	}

	pub fn respond_with_decryption_error(&self, session_id: &SessionId, sub_session_id: &Secret, to: &NodeId, error: message::DecryptionSessionError) {
		let session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		self.decryption_sessions.read().get(&session_id)
			.map(|s| {
				// error in decryption session is non-fatal, if occurs on slave node
				// => either respond with error
				// => or broadcast error

				// do not bother processing send error, as we already processing error
				if &s.master == s.session.node() {
					let _ = s.cluster_view.broadcast(Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				} else {
					let _ = s.cluster_view.send(to, Message::Decryption(DecryptionMessage::DecryptionSessionError(error)));
				}
			});
	}

	fn stop_stalled_sessions(&self) {
		{
			let sessions = self.generation_sessions.write();
			for sid in sessions.keys().collect::<Vec<_>>() {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				if time::Instant::now() - session.last_message_time > time::Duration::from_secs(ENCRYPTION_SESSION_TIMEOUT_INTERVAL) {
					session.session.on_session_timeout();
					if session.session.state() == GenerationSessionState::Finished
						|| session.session.state() == GenerationSessionState::Failed {
						self.remove_generation_session(&sid);
					}
				}
			}
		}
		{
			let sessions = self.encryption_sessions.write();
			for sid in sessions.keys().collect::<Vec<_>>() {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				if time::Instant::now() - session.last_message_time > time::Duration::from_secs(ENCRYPTION_SESSION_TIMEOUT_INTERVAL) {
					session.session.on_session_timeout();
					if session.session.state() == EncryptionSessionState::Finished
						|| session.session.state() == EncryptionSessionState::Failed {
						self.remove_encryption_session(&sid);
					}
				}
			}
		}
		{
			let sessions = self.decryption_sessions.write();
			for sid in sessions.keys().collect::<Vec<_>>() {
				let session = sessions.get(&sid).expect("enumerating only existing sessions; qed");
				if time::Instant::now() - session.last_message_time > time::Duration::from_secs(DECRYPTION_SESSION_TIMEOUT_INTERVAL) {
					session.session.on_session_timeout();
					if session.session.state() == DecryptionSessionState::Finished
						|| session.session.state() == DecryptionSessionState::Failed {
						self.remove_decryption_session(&sid.id, &sid.access_key);
					}
				}
			}
		}
	}

	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		for (sid, session) in self.generation_sessions.read().iter() {
			session.session.on_node_timeout(node_id);
			if session.session.state() == GenerationSessionState::Finished
				|| session.session.state() == GenerationSessionState::Failed {
				self.remove_generation_session(sid);
			}
		}
		for (sid, session) in self.encryption_sessions.read().iter() {
			session.session.on_node_timeout(node_id);
			if session.session.state() == EncryptionSessionState::Finished
				|| session.session.state() == EncryptionSessionState::Failed {
				self.remove_encryption_session(sid);
			}
		}
		for (sid, session) in self.decryption_sessions.read().iter() {
			session.session.on_node_timeout(node_id);
			if session.session.state() == DecryptionSessionState::Finished
				|| session.session.state() == DecryptionSessionState::Failed {
				self.remove_decryption_session(&sid.id, &sid.access_key);
			}
		}
	}
}
*/