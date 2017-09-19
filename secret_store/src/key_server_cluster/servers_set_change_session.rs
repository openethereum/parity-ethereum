// TODO: active_sessions -> key_session, session_id -> key_id, ...
// TODO: when servers set change session is active, pause updating servers set from contract

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
use std::collections::btree_map::Entry;
use parking_lot::{Mutex, Condvar};
use ethkey::Signature;
use tiny_keccak::{keccak256, Keccak};
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, KeyStorage, DummyAclStorage, DocumentKeyShare};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::message::{MessageNodeId, Message, ConsensusMessage, InitializeConsensusSession, ServersSetChangeMessage,
	ServersSetChangeConsensusMessage, ConfirmConsensusInitialization, UnknownSessionsRequest, UnknownSessions,
	ServersSetChangeShareAddMessage, ShareAddMessage, ServersSetChangeError, ServersSetChangeCompleted,
	ShareMoveMessage, ShareRemoveMessage, ServersSetChangeShareMoveMessage, ServersSetChangeShareRemoveMessage,
	ServersSetChangeDelegate, ServersSetChangeDelegateResponse};
use key_server_cluster::share_change_session::{ShareChangeSession, ShareChangeSessionParams};
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
pub struct SessionImpl {
	/// Session core.
	core: SessionCore,
	/// Session data.
	data: Mutex<SessionData>,
}

/// Session state.
enum SessionState {
	GatheringUnknownSessions,
	Finished,
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
type ServersSetChangeConsensusSession = ConsensusSession<ServersSetChangeConsensusTransport, UnknownSessionsJob, UnknownSessionsJobTransport>;

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Consensus-based servers set change session.
	pub consensus_session: ServersSetChangeConsensusSession,
	/// Sessions delegated to other node (valid only on master node).
	pub delegated_sessions: BTreeMap<SessionId, NodeId>,
	/// Active change sessions, where this node is a master.
	pub active_sessions: BTreeMap<SessionId, ShareChangeSession>,
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
struct SessionsQueue {
	/// Known sessions iterator.
	known_sessions: Option<Box<Iterator<Item=(SessionId, DocumentKeyShare)>>>,
	/// Unknown sessions.
	unknown_sessions: Option<BTreeMap<SessionId, BTreeSet<NodeId>>>,
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

impl SessionImpl {
	/// Create new servers set change session.
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
		// TODO: threshold = all_nodes_len() - 1
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
				state: SessionState::GatheringUnknownSessions,
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
				delegated_sessions: BTreeMap::new(),
				active_sessions: BTreeMap::new(),
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
	pub fn process_message(&self, sender: &NodeId, message: &ServersSetChangeMessage) -> Result<(), Error> {
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
			&ServersSetChangeMessage::ServersSetChangeDelegate(ref message) =>
				self.on_sessions_delegation(sender, message),
			&ServersSetChangeMessage::ServersSetChangeDelegateResponse(ref message) =>
				self.on_delegated_session_completed(sender, message),
			&ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref message) =>
				self.on_share_add_message(sender, message),
			&ServersSetChangeMessage::ServersSetChangeShareMoveMessage(ref message) =>
				self.on_share_move_message(sender, message),
			&ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ref message) =>
				self.on_share_remove_message(sender, message),
			&ServersSetChangeMessage::ServersSetChangeError(ref message) =>
				self.on_session_error(sender, message),
			&ServersSetChangeMessage::ServersSetChangeCompleted(ref message) => 
				self.on_session_completed(sender, message),
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

		let unknown_sessions_job = UnknownSessionsJob::new_on_master(self.core.key_storage.clone(), self.core.meta.self_node_id.clone());
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
	pub fn on_unknown_sessions(&self, sender: &NodeId, message: &UnknownSessions) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		data.consensus_session.on_job_response(sender, message.unknown_sessions.iter().cloned().map(Into::into).collect())?;
		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// all nodes have reported their unknown sessions
		// => we are ready to start adding/moving/removing shares

		// delegate unknown sessions to other nodes
		let unknown_sessions = data.consensus_session.result()?;
		let new_nodes_set = self.core.new_nodes_set.as_ref().expect("TODO");
		let mut unknown_sessions_by_master: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
		for (unknown_session_id, unknown_session_nodes) in unknown_sessions {
			let unknown_master_node = unknown_session_nodes.into_iter().nth(0).expect("TODO"); // TODO: check && select randomly
			data.delegated_sessions.insert(unknown_session_id.clone(), unknown_master_node.clone());
			unknown_sessions_by_master.entry(unknown_master_node).or_insert_with(Default::default).insert(unknown_session_id);
		}
		for (master, unknown_sessions) in unknown_sessions_by_master {
			self.core.cluster.send(&master, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegate(ServersSetChangeDelegate {
				session: self.core.meta.id.clone().into(),
				session_nonce: self.core.nonce,
				new_nodes_set: new_nodes_set.iter().cloned().map(Into::into).collect(),
				unknown_sessions: unknown_sessions.into_iter().map(Into::into).collect(),
			})))?
		}

		// start known sessions
		for (known_session_id, key_share) in self.core.key_storage.iter() {
			let session_nodes = key_share.id_numbers.keys().cloned().collect();
			let mut change_session = Self::start_share_change_session(&self.core, &mut *data, known_session_id.clone(), session_nodes)?;
			change_session.initialize()?;
			data.active_sessions.insert(known_session_id, change_session);
		}

		Ok(())
	}

	/// When sessions execution is delegated to this node.
	pub fn on_sessions_delegation(&self, sender: &NodeId, message: &ServersSetChangeDelegate) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if sender != &self.core.meta.master_node_id {
println!("=== 1");
			return Err(Error::InvalidMessage);
		}

		// start sessions
		let mut data = self.data.lock();
		for unknown_session_id in &message.unknown_sessions {
			let unknown_session_id = unknown_session_id.clone().into();
			let key_share = self.core.key_storage.get(&unknown_session_id).map_err(|e| Error::KeyStorage(e.into()))?;
			let session_nodes = key_share.id_numbers.keys().cloned().collect();
			let mut change_session = Self::start_share_change_session(&self.core, &mut *data, unknown_session_id.clone(), session_nodes)?;
			change_session.initialize()?;
			data.active_sessions.insert(unknown_session_id, change_session);
		}

		Ok(())
	}

	/// When delegated session execution is completed.
	pub fn on_delegated_session_completed(&self, sender: &NodeId, message: &ServersSetChangeDelegateResponse) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests on master node
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
println!("=== 2");
			return Err(Error::InvalidMessage);
		}

		// forget delegated session
		let delegated_session_id = message.unknown_session.clone().into();
		let mut data = self.data.lock();
		match data.delegated_sessions.entry(delegated_session_id) {
			Entry::Occupied(entry) => if entry.get() == sender {
				entry.remove()
			} else {
println!("=== 3");
				return Err(Error::InvalidMessage);
			},
			_ => {
println!("=== 4");
				return Err(Error::InvalidMessage)
			},
		};

		// check if we need to complete the whole change session
		if data.delegated_sessions.is_empty() && data.active_sessions.is_empty() {
			Self::complete_session(&self.core, &mut *data)?;
		}

		Ok(())
	}

	/// When share add message is received.
	pub fn on_share_add_message(&self, sender: &NodeId, message: &ServersSetChangeShareAddMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// start session if not started yet
		if let &ShareAddMessage::InitializeShareAddSession(ref message) = &message.message {
			match data.active_sessions.entry(message.session.clone().into()) {
				Entry::Occupied(_) => {
println!("=== 5");
					return Err(Error::InvalidMessage)
				},
				Entry::Vacant(entry) => entry.insert(Self::join_share_change_session(&self.core, sender, message.session.clone().into())?),
			};
		}

		let session_id = message.message.session().clone().into();
		let (is_finished, is_master) = {
			let mut change_session = data.active_sessions.get_mut(&session_id).ok_or(Error::InvalidMessage).map_err(|e| { println!("=== 6"); e })?;
			change_session.on_share_add_message(sender, &message.message)?;
			(change_session.is_finished(), change_session.is_master())
		};
		if is_finished {
			data.active_sessions.remove(&session_id);
			if is_master && self.core.meta.self_node_id != self.core.meta.master_node_id {
println!("=== 2");
				Self::return_delegated_session(&self.core, &session_id)?;
			}
		}

		Ok(())
	}

	/// When share move message is received.
	pub fn on_share_move_message(&self, sender: &NodeId, message: &ServersSetChangeShareMoveMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// start session if not started yet
		if let &ShareMoveMessage::InitializeShareMoveSession(ref message) = &message.message {
			match data.active_sessions.entry(message.session.clone().into()) {
				Entry::Occupied(_) => {
println!("=== 7");
					return Err(Error::InvalidMessage)
				},
				Entry::Vacant(entry) => entry.insert(Self::join_share_change_session(&self.core, sender, message.session.clone().into())?),
			};
		}

		let mut change_session = data.active_sessions.get_mut(&message.message.session().clone().into()).ok_or(Error::InvalidMessage).map_err(|e| { println!("=== 8"); e })?;
		change_session.on_share_move_message(sender, &message.message)
	}

	/// When share remove message is received.
	pub fn on_share_remove_message(&self, sender: &NodeId, message: &ServersSetChangeShareRemoveMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// start session if not started yet
		if let &ShareRemoveMessage::InitializeShareRemoveSession(ref message) = &message.message {
			match data.active_sessions.entry(message.session.clone().into()) {
				Entry::Occupied(_) => {
println!("=== 9");
					return Err(Error::InvalidMessage)
				},
				Entry::Vacant(entry) => entry.insert(Self::join_share_change_session(&self.core, sender, message.session.clone().into())?),
			};
		}

		let mut change_session = data.active_sessions.get_mut(&message.message.session().clone().into()).ok_or(Error::InvalidMessage).map_err(|e| { println!("=== 10"); e })?;
		change_session.on_share_remove_message(sender, &message.message)
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, _sender: &NodeId, _message: &ServersSetChangeError) -> Result<(), Error> {
		unimplemented!()
	}

	/// When session completion message is received.
	pub fn on_session_completed(&self, sender: &NodeId, message: &ServersSetChangeCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		if sender != &self.core.meta.master_node_id {
println!("=== 11");
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		data.state = SessionState::Finished;

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

	/// Start share change session (called on master node).
	fn start_share_change_session(core: &SessionCore, data: &mut SessionData, key_id: SessionId, old_nodes_set: BTreeSet<NodeId>) -> Result<ShareChangeSession, Error> {
		let new_nodes_set = core.new_nodes_set.as_ref().expect("TODO");
		let mut nodes_to_add: BTreeSet<_> = new_nodes_set.difference(&old_nodes_set).cloned().collect();
		let mut nodes_to_move = BTreeMap::new();
		let mut nodes_to_remove: BTreeSet<_> = old_nodes_set.difference(&new_nodes_set).cloned().collect();
		while !nodes_to_remove.is_empty() && !nodes_to_add.is_empty() {
			let source_node = nodes_to_remove.iter().cloned().nth(0).expect("nodes_to_remove.is_empty is checked in while condition; qed");
			let target_node = nodes_to_add.iter().cloned().nth(0).expect("nodes_to_add.is_empty is checked in while condition; qed");
			nodes_to_remove.remove(&source_node);
			nodes_to_add.remove(&target_node);
			nodes_to_move.insert(source_node, target_node);
		}

		ShareChangeSession::new(ShareChangeSessionParams {
			session_id: core.meta.id,
			nonce: core.nonce,
			key_id: key_id,
			self_node_id: core.meta.self_node_id,
			master_node_id: core.meta.self_node_id,
			cluster: core.cluster.clone(),
			key_storage: core.key_storage.clone(),
			nodes_to_add: Some(nodes_to_add),
			nodes_to_move: Some(nodes_to_move),
			nodes_to_remove: Some(nodes_to_remove),
		})
	}

	/// Join share change session (called on slave node).
	fn join_share_change_session(core: &SessionCore, sender: &NodeId, key_id: SessionId) -> Result<ShareChangeSession, Error> {
		ShareChangeSession::new(ShareChangeSessionParams {
			session_id: core.meta.id,
			nonce: core.nonce,
			key_id: key_id,
			self_node_id: core.meta.self_node_id,
			master_node_id: sender.clone(),
			cluster: core.cluster.clone(),
			key_storage: core.key_storage.clone(),
			nodes_to_add: None,
			nodes_to_move: None,
			nodes_to_remove: None,
		})
	}

	/// Return delegated session to master.
	fn return_delegated_session(core: &SessionCore, key_id: &SessionId) -> Result<(), Error> {
		assert!(core.meta.self_node_id != core.meta.master_node_id);
		core.cluster.send(&core.meta.master_node_id, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegateResponse(ServersSetChangeDelegateResponse {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
			unknown_session: key_id.clone().into(),
		})))
	}

	/// Complete servers set change session.
	fn complete_session(core: &SessionCore, data: &mut SessionData) -> Result<(), Error> {
		debug_assert_eq!(core.meta.self_node_id, core.meta.master_node_id);
		core.cluster.broadcast(Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeCompleted(ServersSetChangeCompleted {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
		})))?;

		data.result = Some(Ok(()));
		core.completed.notify_all();

		Ok(())
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
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ServersSetChangeConsensusMessage {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			new_nodes_set: self.new_nodes_set.clone(),
			message: ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			}),
		})))
	}
}

impl JobTransport for UnknownSessionsJobTransport {
	type PartialJobRequest=NodeId;
	type PartialJobResponse=BTreeSet<SessionId>;

	fn send_partial_request(&self, node: &NodeId, request: NodeId) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::UnknownSessionsRequest(UnknownSessionsRequest {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: BTreeSet<SessionId>) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::UnknownSessions(UnknownSessions {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			unknown_sessions: response.into_iter().map(Into::into).collect(),
		})))
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

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, KeyPair, sign};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage, SessionMeta};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::MessageLoop as GenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::Message;
	use super::{SessionImpl, SessionParams, nodes_hash};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	impl MessageLoop {
		pub fn new(gml: GenerationMessageLoop, master_node_id: NodeId, new_nodes_ids: BTreeSet<NodeId>) -> Self {
			let mut new_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().collect();
			new_nodes_set.extend(new_nodes_ids.iter().cloned());

			let session_id = nodes_hash(&new_nodes_set);
			let requester_signature = sign(Random.generate().unwrap().secret(), &session_id).unwrap();
			let mut nodes = BTreeMap::new();
			let meta = SessionMeta {
				self_node_id: master_node_id.clone(),
				master_node_id: master_node_id.clone(),
				id: session_id.clone(),
				threshold: new_nodes_set.len() - 1,
			};
			for (n, nd) in &gml.nodes {
				let cluster = nd.cluster.clone();
				let key_storage = nd.key_storage.clone();
				let mut meta = meta.clone();
				meta.self_node_id = n.clone();
				let requester_signature = if meta.self_node_id == meta.master_node_id { Some(requester_signature.clone()) } else { None };
				let session = SessionImpl::new(SessionParams {
					meta: meta,
					new_nodes_set: new_nodes_set.clone(),
					cluster: nd.cluster.clone(),
					key_storage: nd.key_storage.clone(),
					nonce: 1,
				}, requester_signature).unwrap();
				nodes.insert(n.clone(), Node {
					cluster: cluster,
					key_storage: key_storage,
					session: session,
				});
			}
			for new_node_id in new_nodes_ids {
				let cluster = Arc::new(DummyCluster::new(new_node_id.clone()));
				let key_storage = Arc::new(DummyKeyStorage::default());
				let mut meta = meta.clone();
				meta.self_node_id = new_node_id;
				let requester_signature = if meta.self_node_id == meta.master_node_id { Some(requester_signature.clone()) } else { None };
				let session = SessionImpl::new(SessionParams {
					meta: meta,
					new_nodes_set: new_nodes_set.clone(),
					cluster: cluster.clone(),
					key_storage: key_storage.clone(),
					nonce: 1,
				}, requester_signature).unwrap();
				nodes.insert(new_node_id, Node {
					cluster: cluster,
					key_storage: key_storage,
					session: session,
				});
			}

			MessageLoop {
				session_id: session_id,
				nodes: nodes,
				queue: Default::default(),
			}
		}

		pub fn run(&mut self) {
			while let Some((from, to, message)) = self.take_message() {
println!("=== {} -> {}: {}", from, to, message);
				self.process_message((from, to, message)).unwrap();
			}
		}

		pub fn take_message(&mut self) -> Option<(NodeId, NodeId, Message)> {
			self.nodes.values()
				.filter_map(|n| n.cluster.take_message().map(|m| (n.session.core.meta.self_node_id.clone(), m.0, m.1)))
				.nth(0)
				.or_else(|| self.queue.pop_front())
		}

		pub fn process_message(&mut self, msg: (NodeId, NodeId, Message)) -> Result<(), Error> {
			match {
				match msg.2 {
					Message::ServersSetChange(ref message) => self.nodes[&msg.1].session.process_message(&msg.0, message),
					_ => unreachable!("only servers set change messages are expected"),
				}
			} {
				Ok(_) => Ok(()),
				Err(Error::TooEarlyForRequest) => {
					self.queue.push_back(msg);
					Ok(())
				},
				Err(err) => Err(err),
			}
		}
	}

	pub fn generate_key(threshold: usize, num_nodes: usize) -> GenerationMessageLoop {
		let mut gml = GenerationMessageLoop::new(num_nodes);
		gml.master().initialize(Public::default(), threshold, gml.nodes.keys().cloned().collect()).unwrap();
		while let Some((from, to, message)) = gml.take_message() {
			gml.process_message((from, to, message)).unwrap();
		}
		gml
	}

	#[test]
	fn node_added_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, 3);
		let key_id = gml.session_id.clone();
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();
		let joint_secret = math::compute_joint_secret(gml.nodes.values()
			.map(|nd| nd.key_storage.get(&key_id).unwrap().polynom1[0].clone())
			.collect::<Vec<_>>()
			.iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// insert 1 node so that it becames 2-of-4 session
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add);
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect());
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..4 {
			for n2 in n1+1..4 {
				let share1 = ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).unwrap();
				let share2 = ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).unwrap();
				let id_number1 = share1.id_numbers[ml.nodes.keys().nth(n1).unwrap()].clone();
				let id_number2 = share1.id_numbers[ml.nodes.keys().nth(n2).unwrap()].clone();

				// now encrypt and decrypt data
				let (document_secret_decrypted, document_secret_decrypted_test) =
					math::tests::do_encryption_and_decryption(1,
						joint_key_pair.public(),
						&[id_number1, id_number2],
						&[share1.secret_share, share2.secret_share],
						Some(&joint_secret),
						document_secret_plain.clone());

				assert_eq!(document_secret_plain, document_secret_decrypted_test);
				assert_eq!(document_secret_plain, document_secret_decrypted);
			}
		}
	}

	#[test]
	fn node_added_using_server_set_change_from_this_node() {
		// initial 2-of-3 session
		let gml = generate_key(1, 3);
		let key_id = gml.session_id.clone();

		// insert 1 node so that it becames 2-of-4 session
		// master node is the node we are adding =>
		// 1) add session is delegated to one of old nodes
		// 2) key share is pushed to new node
		// 3) delegated session is returned back to added node
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let master_node_id = nodes_to_add.iter().cloned().nth(0).unwrap();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add);
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect());
		ml.run();
	}
}
