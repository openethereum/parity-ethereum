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
use std::collections::{BTreeSet, BTreeMap};
use parking_lot::{Mutex, Condvar};
use ethkey::Signature;
use tiny_keccak::{keccak256, Keccak};
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, KeyStorage, DummyAclStorage, DocumentKeyShare};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::message::{MessageNodeId, Message, ConsensusMessage, InitializeConsensusSession, ServersSetChangeMessage,
	ServersSetChangeConsensusMessage, UnknownSessionsRequest, UnknownSessions};
use key_server_cluster::share_change_session::ShareChangeSession;
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::unknown_sessions_job::{UnknownSessionsJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Maximal number of active share change sessions.
const MAX_ACTIVE_SESSIONS: usize = 64;

/// Servers set change session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed.
	fn wait(&self) -> Result<(), Error>;
}

/// Servers set change session.
/// Brief overview:
/// 1) initialization: master node (which has received request for change) requests confirmation from ALL other nodes
/// 2) all other nodes send information on session, missing from master node
/// 3) master node splits all nodes from OLD set and from NEW set as follows:
/// 3.1) nodes_to_remove: OLD.except(NEW)
/// 3.2) nodes_to_add: NEW.except(OLD)
/// 3.3) nodes_staying: OLD.intersect(NEW)
/// 4) for each node1 in nodes_to_remove: select node2 from nodes_to_add and move all sessions from node1 to node2
/// 5) for each node1 left in nodes_to_remove (if nodes_to_add.len() < nodes_to_remove.len()): run share removal protocol for each node1 session
/// 6) for each node1 left in nodes_to_add (if nodes_to_add.len() > nodes_to_remove.len()): run share addition protocol for each node1 session
pub struct SessionImpl<'a> {
	/// Session core.
	core: SessionCore,
	/// Session data.
	data: Mutex<SessionData<'a>>,
}

/// Session state.
enum SessionState {
	GetheringUnknownSessions,
}

/// Immutable session data.
struct SessionCore {
	/// Servers set change session meta (id is computed from new_nodes_set).
	pub meta: SessionMeta,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session-level nonce.
	pub nonce: u64,
	/// New nodes set (sent only once to slave nodes).
	pub new_nodes_set: Option<BTreeSet<NodeId>>,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
/*	/// Nodes to remove from the set.
	pub nodes_to_remove: BTreeSet<NodeId>,
	/// Nodes to add to the set.
	pub nodes_to_add: BTreeSet<NodeId>,
	/// Nodes staying in the set.
	pub nodes_to_move: BTreeMap<NodeId, NodeId>,*/
}

/// Servers set change consensus session type.
type ServersSetChangeConsensusSession<'a> = ConsensusSession<ServersSetChangeConsensusTransport, UnknownSessionsJob<'a>, UnknownSessionsJobTransport>;

/// Mutable session data.
struct SessionData<'a> {
	/// Session state.
	pub state: SessionState,
	/// Consensus-based servers set change session.
	pub consensus_session: ServersSetChangeConsensusSession<'a>,
	/// Unknown sessions (actual for master node only).
	pub sessions_queue: Option<SessionsQueue<'a>>,
	/// Active change sessions, where this node is a master.
	pub master_sessions: Option<BTreeMap<SessionId, ShareChangeSession>>,
	/// Servers set change result.
	pub result: Option<Result<(), Error>>,
/*	/// Keys, unknown to master node.
	pub unknown_sessions: BTreeMap<SessionId, BTreeSet<NodeId>>
	/// Active share movement sessions.
	pub movement_sessions: BTreeMap<SessionId, ShareMoveSessions>,
	/// Active share removal sessions.
	pub removal_sessions: BTreeMap<SessionId, ShareRemoveSession>,
	/// Active share addition sessions.
	pub addition_sessions: BTreeMap<SessionId, ShareAddSession>,
	/// Nodes data.
	pub nodes: BTreeMap<NodeId, NodeData>,*/
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// Session meta (artificial).
	pub meta: SessionMeta,
	/// New nodes set.
	pub new_nodes_set: BTreeSet<NodeId>,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session nonce.
	pub nonce: u64,
}

/// Share change sessions queue.
struct SessionsQueue<'a> {
	/// Known sessions iterator.
	known_sessions: Box<Iterator<Item=(SessionId, DocumentKeyShare)> + 'a>,
	/// Unknown sessions.
	unknown_sessions: BTreeMap<SessionId, BTreeSet<NodeId>>,
}

/// Servers set change consensus transport.
struct ServersSetChangeConsensusTransport {
	/// Session id.
	id: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
	/// New nodes set.
	new_nodes_set: Option<BTreeSet<MessageNodeId>>,
}

/// Unknown sessions job transport.
struct UnknownSessionsJobTransport {
	/// Session id.
	id: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl<'a> SessionImpl<'a> {
	/// Create new servers set change session.
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
		// session id must be the hash of sorted nodes set
		let new_nodes_set_hash = nodes_hash(&params.new_nodes_set);
		if new_nodes_set_hash != params.meta.id {
			return Err(Error::InvalidNodesConfiguration);
		}

		let consensus_transport = ServersSetChangeConsensusTransport {
			id: params.meta.id.clone(),
			nonce: params.nonce,
			cluster: params.cluster.clone(),
			new_nodes_set: if params.meta.self_node_id == params.meta.self_node_id {
				Some(params.new_nodes_set.iter().cloned().map(Into::into).collect())
			} else {
				None
			},
		};

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				cluster: params.cluster,
				key_storage: params.key_storage,
				nonce: params.nonce,
				new_nodes_set: Some(params.new_nodes_set),
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::GetheringUnknownSessions,
				consensus_session: match requester_signature {
					Some(requester_signature) => ConsensusSession::new_on_master(ConsensusSessionParams {
						meta: params.meta,
						acl_storage: Arc::new(DummyAclStorage::default()), // TODO: change for something real
						consensus_transport: consensus_transport,
					}, requester_signature)?,
					None => ConsensusSession::new_on_slave(ConsensusSessionParams {
						meta: params.meta,
						acl_storage: Arc::new(DummyAclStorage::default()), // TODO: change for something real
						consensus_transport: consensus_transport,
					})?,
				},
				sessions_queue: None,
				master_sessions: None,
				result: None,
			}),
		})
	}

	/// Initialize servers set change session on master node.
	pub fn initialize(&self, all_nodes_set: BTreeSet<NodeId>) -> Result<(), Error> {
		// TODO: check that all_nodes_set.contains(new_nodes_set)
		// TODO: check that threshold + 1 == all_nodes_set.len()

		let mut data = self.data.lock();
		data.consensus_session.initialize(all_nodes_set)?;

		if data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished {
			// TODO: this means that there's single node in all_nodes_set => can't happen???
		}

		Ok(())
	}

	/// Process servers set change message.
	pub fn process_message(&'a self, sender: &NodeId, message: &ServersSetChangeMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&ServersSetChangeMessage::UnknownSessionsRequest(ref message) =>
				self.on_unknown_sessions_requested(sender, message),
			&ServersSetChangeMessage::UnknownSessions(ref message) =>
				self.on_unknown_sessions(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ServersSetChangeConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);

		let mut data = self.data.lock();
		let is_establishing_consensus = data.consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
		data.consensus_session.on_consensus_message(&sender, &message.message)?;

		// whn consensus is established => request unknown sessions
		let is_consensus_established = data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished;
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		let unknown_sessions_job = UnknownSessionsJob::new_on_master(self.core.meta.self_node_id.clone());
		data.consensus_session.disseminate_jobs(unknown_sessions_job, self.unknown_sessions_transport())
	}

	/// When unknown sessions are requested.
	pub fn on_unknown_sessions_requested(&self, sender: &NodeId, message: &UnknownSessionsRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		let requester = data.consensus_session.requester()?.clone();
		let unknown_sessions_job = UnknownSessionsJob::new_on_slave(self.core.key_storage.clone());
		let unknown_sessions_transport = self.unknown_sessions_transport();

		data.consensus_session.on_job_request(&sender, sender.clone(), unknown_sessions_job, unknown_sessions_transport)
	}

	/// When unknown sessions are received.
	pub fn on_unknown_sessions(&'a self, sender: &NodeId, message: &UnknownSessions) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		data.consensus_session.on_job_response(sender, message.unknown_sessions.iter().cloned().map(Into::into).collect())?;

		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// all nodes has reported their unknown sessions
		// => we are ready to start adding/moving/removing shares
		// => take some sessions and start
		let mut sessions_queue = SessionsQueue::new(&self.core, data.consensus_session.result()?);
		for (session_id, session_nodes) in sessions_queue.dequeue(MAX_ACTIVE_SESSIONS) {
			if session_nodes.contains(&self.core.meta.self_node_id) {
				let change_session = Self::start_share_change_session(&self.core, &mut *data, session_id, session_nodes);
			} else {
				unimplemented!()
			}
		}
		data.sessions_queue = Some(sessions_queue);

		Ok(())
	}

	/// Create unknown sessions transport.
	fn unknown_sessions_transport(&self) -> UnknownSessionsJobTransport {
		UnknownSessionsJobTransport {
			id: self.core.meta.id.clone(),
			nonce: self.core.nonce,
			cluster: self.core.cluster.clone(),
		}
	}

	/// Start share change session.
	fn start_share_change_session(core: &SessionCore, data: &mut SessionData<'a>, session_id: SessionId, session_nodes: BTreeSet<NodeId>) -> Result<ShareChangeSession, Error> {
		unimplemented!()
	}
}

impl<'a> SessionsQueue<'a> {
	/// Create new sessions queue.
	pub fn new(core: &'a SessionCore, unknown_sessions: BTreeMap<SessionId, BTreeSet<NodeId>>) -> Self {
		SessionsQueue {
			known_sessions: core.key_storage.iter(),
			unknown_sessions: unknown_sessions,
		}
	}

	/// Dequeue at most N sessions.
	pub fn dequeue(&mut self, mut n: usize) -> Vec<(SessionId, BTreeSet<NodeId>)> {
		let mut result = Vec::with_capacity(n);
		while n > 0 {
			if let Some(s) = self.next() {
				result.push(s);
				n = n - 1;
			} else {
				break;
			}
		}
		result
	}
}

impl<'a> Iterator for SessionsQueue<'a> {
	type Item = (SessionId, BTreeSet<NodeId>);

	fn next(&mut self) -> Option<Self::Item> {
		unimplemented!()
	}
}

impl JobTransport for ServersSetChangeConsensusTransport {
	type PartialJobRequest=Signature;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Signature) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ServersSetChangeConsensusMessage {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			new_nodes_set: self.new_nodes_set.clone(),
			message: ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
				requestor_signature: request.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		unimplemented!()
	}
}

impl JobTransport for UnknownSessionsJobTransport {
	type PartialJobRequest=NodeId;
	type PartialJobResponse=BTreeSet<SessionId>;

	fn send_partial_request(&self, node: &NodeId, request: NodeId) -> Result<(), Error> {
		unimplemented!()
	}

	fn send_partial_response(&self, node: &NodeId, response: BTreeSet<SessionId>) -> Result<(), Error> {
		unimplemented!()
	}
}

pub fn nodes_hash(nodes: &BTreeSet<NodeId>) -> SessionId {
	let mut nodes_keccak = Keccak::new_keccak256();
	for node in nodes {
		nodes_keccak.update(&*node);
	}

	let mut nodes_keccak_value = [0u8; 32];
	nodes_keccak.finalize(&mut nodes_keccak_value);

	nodes_keccak_value.into()
}
