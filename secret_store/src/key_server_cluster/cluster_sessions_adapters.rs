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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::collections::{BTreeSet, BTreeMap, VecDeque};
use parking_lot::{Mutex, RwLock};
use ethkey::Public;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage};
use key_server_cluster::cluster::{Cluster, ClusterData, ClusterView};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{self, Message, GenerationMessage};
use key_server_cluster::generation_session::{Session as GenerationSession, SessionImpl as GenerationSessionImpl,
	SessionParams as GenerationSessionParams, SessionState as GenerationSessionState};

/// When there are no session-related messages for SESSION_TIMEOUT_INTERVAL seconds,
/// we must treat this session as stalled && finish it with an error.
/// This timeout is for cases when node is responding to KeepAlive messages, but intentionally ignores
/// session messages.
const SESSION_TIMEOUT_INTERVAL: u64 = 60;
/// Interval to send session-level KeepAlive-messages.
const SESSION_KEEP_ALIVE_INTERVAL: u64 = 30;

/*/// Generic cluster session.
pub trait ClusterSession {
	/// Session identifier type.
	type Id: Ord + Clone;

	/// If session is finished (either with succcess or not).
	fn is_finished(&self) -> bool { unimplemented!() }
	/// When it takes too much time to complete session.
	fn on_session_timeout(&self) { unimplemented!() }
	/// When it takes too much time to receive response from the node.
	fn on_node_timeout(&self, sender: &NodeId) { unimplemented!() }
	/// Process error that has occured during session + propagate this error to required nodes.
	fn on_session_error(&self, sender: &NodeId, error: Error) { unimplemented!() }
	/// Process session message.
	fn process_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> { unimplemented!() }
}*/

/// Generic cluster session creator.
pub trait ClusterSessionCreator<S: ClusterSession> {
	/// Create cluster session.
	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: S::Id, message: &Message) -> Result<Arc<S>, Error>;
}

/// Message with session id.
trait IntoSessionId<K> {
	/// Get session id.
	fn into_session_id(&self) -> Result<K, Error>;
}

/// Message with session nonce.
trait IntoSessionNonce {
	/// Get session nonce.
	fn into_session_nonce(&self) -> Result<u64, Error>;
}

/// Session creator.
pub struct SessionCreator {
	/// Cluster data reference.
	data: Arc<ClusterData>,
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
pub struct ClusterSessionsContainer<S: ClusterSession, SC: ClusterSessionCreator<S>> {
	/// Cluster data.
	data: Arc<ClusterData>,
	/// Sessions creator.
	creator: SC,
	/// Active sessions.
	sessions: RwLock<BTreeMap<S::Id, QueuedSession<S>>>,
	/// Sessions container state.
	container_state: Arc<Mutex<ClusterSessionsContainerState>>
}

/// Session and its message queue.
pub struct QueuedSession<S> {
	/// Session master.
	pub master: NodeId,
	/// Cluster view.
	pub cluster_view: Arc<Cluster>,
	/// Last keep alive time.
	pub last_keep_alive_time: time::Instant,
	/// Last received message time.
	pub last_message_time: time::Instant,
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

/// Process single session message.
fn process_message<S: ClusterSession, SC: ClusterSessionCreator<S>>(creator: &SessionCreator, sessions: ClusterSessionsContainer<S, SC>, mut sender: NodeId, mut message: Message) -> Result<(), Error>
	where Message: IntoSessionId<S::Id> {
	// get or create new session, if required
	let session_id = message.into_session_id()?;
	let session_nonce = message.into_session_nonce()?;
	let session = match is_initialization_message(&message) {
		false => sessions.get(&session_id, true).ok_or(Error::InvalidSessionId),
		true => sessions.insert(sender.clone(), session_id.clone(), &message),
	};

	let session = match session {
		Ok(session) => session,
		Err(error) => unimplemented!("TODO"),
	};

	let mut is_queued_message = false;
	loop {
		let message_result = session.on_message(&sender, &message);
		match message_result {
			Ok(_) => {
				// if session is completed => stop
				if session.is_finished() {
//					info!(target: "secretstore_net", "{}: {} session completed", data.self_key_pair.public(), "generation");
					sessions.remove(&session_id);
					return Ok(());
				}

				// try to dequeue message
				match sessions.dequeue_message(&session_id) {
					Some((msg_sender, msg)) => {
						is_queued_message = true;
						sender = msg_sender;
						message = msg;
					},
					None => return Ok(()),
				}
			},
			Err(Error::TooEarlyForRequest) => {
				sessions.enqueue_message(&session_id, sender, message, is_queued_message);
				return Ok(());
			},
			Err(err) => {
				/*warn!(target: "secretstore_net", "{}: {} session error '{}' when processing message {} from node {}",
					data.self_key_pair.public(),
					"generation",
					err,
					message,
					sender);*/
				let _ = session.on_session_error(&Public::default()/*data.self_key_pair.public()*/, err); // processing error => ignore error
				//session.cluster().broadcast(error_message.into());
				sessions.remove(&session_id);
				return Ok(());
			},
		}
	}
}

impl SessionCreator {
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

/*impl ClusterSession for GenerationSessionImpl {
	type Id = SessionId;
}*/

impl ClusterSessionCreator<GenerationSessionImpl> for SessionCreator {
	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionId, message: &Message) -> Result<Arc<GenerationSessionImpl>, Error> {
		match *message {
			Message::Generation(GenerationMessage::InitializeSession(ref message)) => {
				// check that there's no finished encryption session with the same id
				if self.key_storage.contains(&id) {
					return Err(Error::DuplicateSessionId);
				}

				// communicating to all other nodes is crucial for generation session
				// => check that we have connections to all cluster nodes
				let cluster = create_cluster_view(&self.data, true)?;
				// if self.nodes.iter().any(|n| !cluster.is_connected(n)) {
				// 	return Err(Error::NodeDisconnected);
				// }

				// check that there's no active encryption session with the same id
				let nonce = self.check_session_nonce(&master, nonce)?;
				Ok(Arc::new(GenerationSessionImpl::new(GenerationSessionParams {
					id: id.clone(),
					self_node_id: self.self_node_id.clone(),
					key_storage: Some(self.key_storage.clone()),
					cluster: cluster,
					nonce: Some(nonce),
				})))
/*TODO:					.map(|session| {
						if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
							session.simulate_faulty_behaviour();
						}
						session
					})*/
			},
			_ => unreachable!("message is checked before create() is called; qed"),
		}
	}
}

fn requires_all_connections(message: &Message) -> bool {
	unimplemented!()
}

fn is_exclusive_session_message(message: &Message) -> bool {
	unimplemented!()
}

fn is_initialization_message(message: &Message) -> bool {
	unimplemented!()
}

pub fn create_cluster_view(data: &Arc<ClusterData>, requires_all_connections: bool) -> Result<Arc<Cluster>, Error> {
	if requires_all_connections {
		if !data.connections.disconnected_nodes().is_empty() {
			return Err(Error::NodeDisconnected);
		}
	}

	let mut connected_nodes = data.connections.connected_nodes();
	connected_nodes.insert(data.self_key_pair.public().clone());

	Ok(Arc::new(ClusterView::new(data.clone(), connected_nodes)))
}

impl<S, SC> ClusterSessionsContainer<S, SC> where S: ClusterSession, SC: ClusterSessionCreator<S> {
	pub fn new(data: Arc<ClusterData>, creator: SC, container_state: Arc<Mutex<ClusterSessionsContainerState>>) -> Self {
		ClusterSessionsContainer {
			data: data,
			creator: creator,
			sessions: RwLock::new(BTreeMap::new()),
			container_state: container_state,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.sessions.read().is_empty()
	}

	pub fn get(&self, session_id: &S::Id, update_last_message_time: bool) -> Option<Arc<S>> {
		let mut sessions = self.sessions.write();
		sessions.get_mut(session_id)
			.map(|s| {
				if update_last_message_time {
					s.last_message_time = time::Instant::now();
				}
				s.session.clone()
			})
	}

	pub fn insert(&self, master: NodeId, session_id: S::Id, message: &Message) -> Result<Arc<S>, Error> {
		let mut sessions = self.sessions.write();
		if sessions.contains_key(&session_id) {
			return Err(Error::DuplicateSessionId);
		}

		// create cluster
		let cluster = create_cluster_view(&self.data, requires_all_connections(&message))?;
		// create session
		let session = self.creator.create(cluster.clone(), master.clone(), Some(message.into_session_nonce()?), session_id.clone(), message)?;
		// check if session can be started
		self.container_state.lock().on_session_starting(is_exclusive_session_message(&message))?;

		// insert session
		let queued_session = QueuedSession {
			master: master,
			cluster_view: cluster,
			last_keep_alive_time: time::Instant::now(),
			last_message_time: time::Instant::now(),
			session: session.clone(),
			queue: VecDeque::new(),
		};
		sessions.insert(session_id, queued_session);
		Ok(session)
	}

	pub fn remove(&self, session_id: &S::Id) {
		if self.sessions.write().remove(session_id).is_some() {
			self.container_state.lock().on_session_completed();
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

impl<S, SC> ClusterSessionsContainer<S, SC> where S: ClusterSession, SC: ClusterSessionCreator<S>, SessionId: From<S::Id> {
	pub fn send_keep_alive(&self, session_id: &S::Id, self_node_id: &NodeId) {
		if let Some(session) = self.sessions.write().get_mut(session_id) {
			let now = time::Instant::now();
			if self_node_id == &session.master && now - session.last_keep_alive_time > time::Duration::from_secs(SESSION_KEEP_ALIVE_INTERVAL) {
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
			let now = time::Instant::now();
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

impl IntoSessionNonce for Message {
	fn into_session_nonce(&self) -> Result<u64, Error> {
		unimplemented!()
	}
}
