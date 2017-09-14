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
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::message::{ServersSetChangeMessage, ServersSetChangeConsensusMessage};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::unknown_sessions_job::{UnknownSessionsJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

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
}

/// Immutable session data.
struct SessionCore {
	/// Servers set change session meta (id is computed from new_nodes_set).
	pub meta: SessionMeta,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	pub cluster: Arc<Cluster>,
	/// Session-level nonce.
	pub nonce: u64,
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
	/// Session nonce.
	pub nonce: u64,
}

/// Servers set change consensus transport.
struct ServersSetChangeConsensusTransport {
	/// Session id.
	id: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
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
		debug_assert_eq!(params.meta.id, compute_servers_set_change_session_id(&params.new_nodes_set));

		let consensus_transport = ServersSetChangeConsensusTransport {
			id: params.meta.id.clone(),
			nonce: params.nonce,
			cluster: params.cluster.clone(),
		};

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				cluster: params.cluster,
				nonce: params.nonce,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				consensus_session: match requester_signature {
					Some(requester_signature) => ConsensusSession::new_on_master(ConsensusSessionParams {
						meta: params.meta,
						consensus_transport: consensus_transport,
					}, requester_signature)?,
					None => ConsensusSession::new_on_slave(ConsensusSessionParams {
						meta: params.meta,
						consensus_transport: consensus_transport,
					})?,
				},
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
				unimplemented!(),
			&ServersSetChangeMessage::UnknownSessions(ref message) =>
				unimplemented!(),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ServersSetChangeConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);

		let mut data = self.data.lock();
		let is_establishing_consensus = data.consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
		data.consensus_session.on_consensus_message(&sender, &message.message)?;

		let is_consensus_established = data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished;
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		unimplemented!()
	}
}

impl JobTransport for ServersSetChangeConsensusTransport {
	type PartialJobRequest=Signature;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Signature) -> Result<(), Error> {
		unimplemented!()
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

pub fn compute_servers_set_change_session_id(new_nodes_set: &BTreeSet<NodeId>) -> SessionId {
	unimplemented!()
}