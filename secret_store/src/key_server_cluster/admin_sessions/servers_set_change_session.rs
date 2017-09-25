// TODO: active_sessions -> key_session, session_id -> key_id, ...
// TODO: when servers set change session is active, pause updating servers set from contract
// TODO: do not need SessionError messages in nested sessions + do not need nonces + sub sessions
// TODO: even if node was lost, it is still required for share removal, ...

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
use ethkey::{Public, Signature};
use tiny_keccak::{keccak256, Keccak};
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, KeyStorage, DummyAclStorage, DocumentKeyShare};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::message::{MessageNodeId, Message, ServersSetChangeMessage,
	ConsensusMessageWithServersSet, InitializeConsensusSessionWithServersSet,
	ServersSetChangeConsensusMessage, ConfirmConsensusInitialization, UnknownSessionsRequest, UnknownSessions,
	ServersSetChangeShareAddMessage, ShareAddMessage, ServersSetChangeError, ServersSetChangeCompleted,
	ShareMoveMessage, ShareRemoveMessage, ServersSetChangeShareMoveMessage, ServersSetChangeShareRemoveMessage,
	ServersSetChangeDelegate, ServersSetChangeDelegateResponse, InitializeShareChangeSession,
	ConfirmShareChangeSessionInitialization};
use key_server_cluster::share_change_session::{ShareChangeSession, ShareChangeSessionParams, ShareChangeSessionPlan,
	prepare_share_change_session_plan};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::unknown_sessions_job::{UnknownSessionsJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};
use key_server_cluster::admin_sessions::sessions_queue::{SessionsQueue, QueuedSession};

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
	/// All known nodes.
	pub all_nodes_set: BTreeSet<NodeId>,
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
type ServersSetChangeConsensusSession = ConsensusSession<ServersSetChangeAccessJob, ServersSetChangeConsensusTransport, UnknownSessionsJob, UnknownSessionsJobTransport>;

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Consensus-based servers set change session.
	pub consensus_session: Option<ServersSetChangeConsensusSession>,
	/// New nodes set.
	pub new_nodes_set: Option<BTreeSet<NodeId>>,
	/// Share change sessions queue (valid on master nodes only).
	pub sessions_queue: Option<SessionsQueue>,
	/// Share change sessions initialization state (valid on master nodes only).
	pub sessions_initialization_state: BTreeMap<SessionId, SessionInitializationData>,
	/// Sessions delegated to other nodes (valid on master node only).
	pub delegated_sessions: BTreeMap<SessionId, NodeId>,
	/// Active share change sessions.
	pub active_sessions: BTreeMap<SessionId, ShareChangeSession>,
	/// Servers set change result.
	pub result: Option<Result<(), Error>>,
}

/// Session initialization data.
struct SessionInitializationData {
	/// Master node id.
	pub master: NodeId,
	/// Nodes that have confirmed session initialization request.
	pub confirmations: BTreeSet<NodeId>,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// Session meta (artificial).
	pub meta: SessionMeta,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session nonce.
	pub nonce: u64,
	/// All known nodes.
	pub all_nodes_set: BTreeSet<NodeId>,
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
	pub fn new(params: SessionParams) -> Result<Self, Error> {
		// TODO: threshold = all_nodes_len() - 1
		// session id must be the hash of sorted nodes set
		/*let new_nodes_set_hash = nodes_hash(&params.new_nodes_set);
		if new_nodes_set_hash != params.meta.id {
			return Err(Error::InvalidNodesConfiguration);
		}*/

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				cluster: params.cluster,
				key_storage: params.key_storage,
				nonce: params.nonce,
				all_nodes_set: params.all_nodes_set,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::GatheringUnknownSessions,
				consensus_session: None,
				new_nodes_set: None,
				sessions_queue: None,
				sessions_initialization_state: BTreeMap::new(),
				delegated_sessions: BTreeMap::new(),
				active_sessions: BTreeMap::new(),
				result: None,
			}),
		})
	}

	/// Initialize servers set change session on master node.
	pub fn initialize(&self, new_nodes_set: BTreeSet<NodeId>, all_set_signature: Signature, new_set_signature: Signature) -> Result<(), Error> {
		// TODO: check that all_nodes_set.contains(new_nodes_set)
		// TODO: check that threshold + 1 == all_nodes_set.len()

		let mut data = self.data.lock();
		let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
			meta: self.core.meta.clone(),
			consensus_executor: ServersSetChangeAccessJob::new_on_master(Public::default(), // TODO: admin key instead of default
				self.core.all_nodes_set.clone(),
				self.core.all_nodes_set.clone(),
				new_nodes_set.clone(),
				all_set_signature,
				new_set_signature),
			consensus_transport: ServersSetChangeConsensusTransport {
				id: self.core.meta.id.clone(),
				nonce: self.core.nonce,
				cluster: self.core.cluster.clone(),
			},
		})?;
		consensus_session.initialize(self.core.all_nodes_set.clone())?;
		debug_assert!(consensus_session.state() != ConsensusSessionState::ConsensusEstablished);
		data.consensus_session = Some(consensus_session);
		data.new_nodes_set = Some(new_nodes_set);

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
			&ServersSetChangeMessage::InitializeShareChangeSession(ref message) =>
				self.on_initialize_share_change_session(sender, message),
			&ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ref message) =>
				self.on_share_change_session_confirmation(sender, message),
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

		// start slave consensus session if needed
		let mut data = self.data.lock();
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			if data.consensus_session.is_none() {
				match &message.message {
					&ConsensusMessageWithServersSet::InitializeConsensusSession(ref message) => {
						data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
							meta: self.core.meta.clone(),
							consensus_executor: ServersSetChangeAccessJob::new_on_slave(Public::default(), // TODO: administrator public
								self.core.all_nodes_set.clone(),
							),
							consensus_transport: ServersSetChangeConsensusTransport {
								id: self.core.meta.id.clone(),
								nonce: self.core.nonce,
								cluster: self.core.cluster.clone(),
							},
						})?);
					},
					_ => return Err(Error::InvalidStateForRequest),
				}
			}
		}

		let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
		let is_establishing_consensus = consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
		match &message.message {
			&ConsensusMessageWithServersSet::InitializeConsensusSession(ref message) =>
				consensus_session.on_consensus_partial_request(sender, ServersSetChangeAccessRequest::from(message))?,
			&ConsensusMessageWithServersSet::ConfirmConsensusInitialization(ref message) =>
				consensus_session.on_consensus_partial_response(sender, message.is_confirmed)?,
		}

		// when consensus is established => request unknown sessions
		let is_consensus_established = consensus_session.state() == ConsensusSessionState::ConsensusEstablished;
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		let unknown_sessions_job = UnknownSessionsJob::new_on_master(self.core.key_storage.clone(), self.core.meta.self_node_id.clone());
		consensus_session.disseminate_jobs(unknown_sessions_job, self.unknown_sessions_transport())
	}

	/// When unknown sessions are requested.
	pub fn on_unknown_sessions_requested(&self, sender: &NodeId, message: &UnknownSessionsRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
		let unknown_sessions_job = UnknownSessionsJob::new_on_slave(self.core.key_storage.clone());
		let unknown_sessions_transport = self.unknown_sessions_transport();

		consensus_session.on_job_request(&sender, sender.clone(), unknown_sessions_job, unknown_sessions_transport)
	}

	/// When unknown sessions are received.
	pub fn on_unknown_sessions(&self, sender: &NodeId, message: &UnknownSessions) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		let (unknown_sessions, new_nodes_set) = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			consensus_session.on_job_response(sender, message.unknown_sessions.iter().cloned().map(Into::into).collect())?;
			if consensus_session.state() != ConsensusSessionState::Finished {
				return Ok(());
			}

			// all nodes have reported their unknown sessions
			// => we are ready to start adding/moving/removing shares
			(
				consensus_session.result()?,
				consensus_session.consensus_job().executor().new_servers_set().expect("TODO").clone(),
			)
		};

		// initialize sessions queue
		data.new_nodes_set = Some(new_nodes_set);
		data.sessions_queue = Some(SessionsQueue::new(self.core.key_storage.clone(), unknown_sessions));

		// and disseminate session initialization requests
		Self::disseminate_session_initialization_requests(&self.core, &mut *data)
	}

	/// When share change session initialization is requested.
	pub fn on_initialize_share_change_session(&self, sender: &NodeId, message: &InitializeShareChangeSession) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// insert new session
		let mut data = self.data.lock();
		match data.active_sessions.entry(message.key_id.clone().into()) {
			Entry::Occupied(_) => return Err(Error::InvalidMessage),
			Entry::Vacant(entry) => entry.insert(ShareChangeSession::new(ShareChangeSessionParams {
				session_id: message.key_id.clone().into(),
				nonce: 0,
				key_id: message.key_id.clone().into(),
				self_node_id: self.core.meta.self_node_id.clone(),
				master_node_id: message.master_node_id.clone().into(),
				cluster: self.core.cluster.clone(),
				key_storage: self.core.key_storage.clone(),
				old_nodes_set: message.old_shares_set.iter().cloned().map(Into::into).collect(),
				plan: ShareChangeSessionPlan {
					nodes_to_add: message.shares_to_add.iter().map(|(k, v)| (k.clone().into(), v.clone().into())).collect(),
					nodes_to_move: message.shares_to_move.iter().map(|(k, v)| (k.clone().into(), v.clone().into())).collect(),
					nodes_to_remove: message.shares_to_remove.iter().cloned().map(Into::into).collect(),
				},
			})?),
		};

		// send confirmation
		self.core.cluster.send(sender, Message::ServersSetChange(ServersSetChangeMessage::ConfirmShareChangeSessionInitialization(ConfirmShareChangeSessionInitialization {
			session: message.session.clone(),
			session_nonce: message.session_nonce.clone(),
			key_id: message.key_id.clone(),
		})))
	}

	/// When share change session initialization is confirmed.
	pub fn on_share_change_session_confirmation(&self, sender: &NodeId, message: &ConfirmShareChangeSessionInitialization) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// add confirmation
		let mut data = self.data.lock();
		let key_id = message.key_id.clone().into();
		let session_master = {
			let session_init_data = data.sessions_initialization_state.get_mut(&key_id).ok_or(Error::InvalidMessage)?;
			if !session_init_data.confirmations.remove(sender) {
				return Err(Error::InvalidMessage);
			}

			if !session_init_data.confirmations.is_empty() {
				return Ok(());
			}

			session_init_data.master.clone()
		};

		// and start/delegate session if required
		data.sessions_initialization_state.remove(&key_id);
		if self.core.meta.self_node_id != session_master {
			data.delegated_sessions.insert(key_id, session_master.clone());
			return self.core.cluster.send(&session_master, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegate(ServersSetChangeDelegate {
				session: self.core.meta.id.clone().into(),
				session_nonce: self.core.nonce,
				key_id: key_id.into(),
			})));
		}

		let session = data.active_sessions.get_mut(&key_id).ok_or(Error::InvalidMessage)?;
		session.initialize()
	}

	/// When sessions execution is delegated to this node.
	pub fn on_sessions_delegation(&self, sender: &NodeId, message: &ServersSetChangeDelegate) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// start session
		let mut data = self.data.lock();
		let session = data.active_sessions.get_mut(&message.key_id.clone().into()).ok_or(Error::InvalidMessage)?;
		session.initialize()
	}

	/// When delegated session execution is completed.
	pub fn on_delegated_session_completed(&self, sender: &NodeId, message: &ServersSetChangeDelegateResponse) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests on master node
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// forget delegated session
		let key_id = message.key_id.clone().into();
		let mut data = self.data.lock();
		match data.delegated_sessions.entry(key_id) {
			Entry::Occupied(entry) => if entry.get() == sender {
				entry.remove()
			} else {
				return Err(Error::InvalidMessage);
			},
			_ => return Err(Error::InvalidMessage),
		};

		// check if we need to complete the whole change session
		if data.delegated_sessions.is_empty() && data.active_sessions.is_empty() {
			Self::disseminate_session_initialization_requests(&self.core, &mut *data)?;
		}

		Ok(())
	}

	/// When share add message is received.
	pub fn on_share_add_message(&self, sender: &NodeId, message: &ServersSetChangeShareAddMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		let session_id = message.message.session().clone().into();
		let (is_finished, is_master) = {
			let mut change_session = data.active_sessions.get_mut(&session_id).ok_or(Error::InvalidMessage)?;
			change_session.on_share_add_message(sender, &message.message)?;
			(change_session.is_finished(), change_session.is_master())
		};
		if is_finished {
			data.active_sessions.remove(&session_id);
			if is_master && self.core.meta.self_node_id != self.core.meta.master_node_id {
				Self::return_delegated_session(&self.core, &session_id)?;
			}
		}

		Ok(())
	}

	/// When share move message is received.
	pub fn on_share_move_message(&self, sender: &NodeId, message: &ServersSetChangeShareMoveMessage) -> Result<(), Error> {
		/*let mut data = self.data.lock();

		// start session if not started yet
		if let &ShareMoveMessage::InitializeShareMoveSession(ref message) = &message.message {
			match data.active_sessions.entry(message.session.clone().into()) {
				Entry::Occupied(_) => return Err(Error::InvalidMessage),
				Entry::Vacant(entry) => entry.insert(Self::join_share_change_session(&self.core, sender, message.session.clone().into())?),
			};
		}

		let mut change_session = data.active_sessions.get_mut(&message.message.session().clone().into()).ok_or(Error::InvalidMessage)?;
		change_session.on_share_move_message(sender, &message.message)*/
		unimplemented!()
	}

	/// When share remove message is received.
	pub fn on_share_remove_message(&self, sender: &NodeId, message: &ServersSetChangeShareRemoveMessage) -> Result<(), Error> {
/*		let mut data = self.data.lock();

		// start session if not started yet
		if let &ShareRemoveMessage::InitializeShareRemoveSession(ref message) = &message.message {
			match data.active_sessions.entry(message.session.clone().into()) {
				Entry::Occupied(_) => return Err(Error::InvalidMessage),
				Entry::Vacant(entry) => entry.insert(Self::join_share_change_session(&self.core, sender, message.session.clone().into())?),
			};
		}

		let mut change_session = data.active_sessions.get_mut(&message.message.session().clone().into()).ok_or(Error::InvalidMessage)?;
		change_session.on_share_remove_message(sender, &message.message)*/
		unimplemented!()
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

	fn disseminate_session_initialization_requests(core: &SessionCore, data: &mut SessionData) -> Result<(), Error> {
		if let Some(sessions_queue) = data.sessions_queue.as_mut() {
			let mut number_of_sessions_to_start = MAX_ACTIVE_SESSIONS.saturating_sub(data.active_sessions.len() + data.delegated_sessions.len());
			let new_nodes_set = data.new_nodes_set.as_ref().expect("TODO");
			while number_of_sessions_to_start > 0 {
				let session = match sessions_queue.next() {
					None => break, // complete session
					Some(Err(e)) => return Err(e),
					Some(Ok(session)) => session,
				};
				number_of_sessions_to_start = number_of_sessions_to_start - 1;

				// select master for this session
				let session_master = match &session {
					&QueuedSession::Known(_, _) => core.meta.self_node_id.clone(),
					&QueuedSession::Unknown(_, ref nodes) => nodes.iter().cloned().nth(0).expect("TODO"),
				};

				// send confirmations requests
				let old_nodes_set = session.nodes();
				let session_id = session.id().clone();
				let session_plan = prepare_share_change_session_plan(&old_nodes_set, new_nodes_set)?;
				let mut confirmations: BTreeSet<_> = old_nodes_set.iter().cloned()
					.chain(session_plan.nodes_to_add.keys().cloned())
					.chain(session_plan.nodes_to_move.values().cloned())
					.collect();
				confirmations.remove(&core.meta.self_node_id);
				for node in &confirmations {
					core.cluster.send(&node, Message::ServersSetChange(ServersSetChangeMessage::InitializeShareChangeSession(InitializeShareChangeSession {
						session: core.meta.id.clone().into(),
						session_nonce: core.nonce,
						key_id: session_id.clone().into(),
						master_node_id: session_master.clone().into(),
						old_shares_set: old_nodes_set.iter().cloned().map(Into::into).collect(),
						shares_to_add: session_plan.nodes_to_add.iter()
							.map(|(n, nid)| (n.clone().into(), nid.clone().into()))
							.collect(),
						shares_to_move: session_plan.nodes_to_move.iter()
							.map(|(source, target)| (source.clone().into(), target.clone().into()))
							.collect(),
						shares_to_remove: session_plan.nodes_to_remove.iter().cloned().map(Into::into).collect(),
					})))?;
				}

				// create session if required
				if let &QueuedSession::Known(_, ref key_share) = &session {
					data.active_sessions.insert(session_id.clone(), ShareChangeSession::new(ShareChangeSessionParams {
						session_id: session_id.clone(),
						nonce: 0,
						key_id: session_id.clone(),
						self_node_id: core.meta.self_node_id.clone(),
						master_node_id: core.meta.self_node_id.clone(),
						cluster: core.cluster.clone(),
						key_storage: core.key_storage.clone(),
						old_nodes_set: key_share.id_numbers.keys().cloned().collect(),
						plan: session_plan,
					})?);
				}

				// initialize session if required
				let wait_for_confirmations = confirmations.is_empty();
				data.sessions_initialization_state.insert(session_id.clone(), SessionInitializationData {
					master: session_master,
					confirmations: confirmations,
				});
				if !wait_for_confirmations {
					data.active_sessions.get_mut(&session_id).expect("TODO").initialize()?;
				}
			}
		}

// TODO: complete when last session is completed!!!
//		data.sessions_queue = None;
//		return Self::complete_session(core, data);
		Ok(())
	}

/*

		// delegate unknown sessions to other nodes
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
*/

	/// Return delegated session to master.
	fn return_delegated_session(core: &SessionCore, key_id: &SessionId) -> Result<(), Error> {
		assert!(core.meta.self_node_id != core.meta.master_node_id);
		core.cluster.send(&core.meta.master_node_id, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegateResponse(ServersSetChangeDelegateResponse {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
			key_id: key_id.clone().into(),
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
	type PartialJobRequest=ServersSetChangeAccessRequest;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ServersSetChangeConsensusMessage {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersSet::InitializeConsensusSession(InitializeConsensusSessionWithServersSet {
				old_nodes_set: request.old_servers_set.into_iter().map(Into::into).collect(),
				new_nodes_set: request.new_servers_set.into_iter().map(Into::into).collect(),
				old_set_signature: request.old_set_signature.into(),
				new_set_signature: request.new_set_signature.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ServersSetChangeConsensusMessage {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersSet::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
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

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, Signature, KeyPair, sign};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage, SessionMeta};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::MessageLoop as GenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::Message;
	use super::{SessionImpl, SessionParams};

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
		pub fn new(gml: GenerationMessageLoop, master_node_id: NodeId, new_nodes_ids: BTreeSet<NodeId>, removed_nodes_ids: BTreeSet<NodeId>) -> Self {
			let mut all_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().collect();
			all_nodes_set.extend(new_nodes_ids.iter().cloned());
			//let mut new_nodes_set = all_nodes_set.iter().cloned().filter(|n| !removed_nodes_ids.contains(n)).collect();

			let session_id = *math::generate_random_scalar().unwrap();
			let requester_signature = sign(Random.generate().unwrap().secret(), &session_id).unwrap();
			let mut nodes = BTreeMap::new();
			let meta = SessionMeta {
				self_node_id: master_node_id.clone(),
				master_node_id: master_node_id.clone(),
				id: session_id.clone(),
				threshold: all_nodes_set.len() - 1,
			};
			for (n, nd) in &gml.nodes {
				let cluster = nd.cluster.clone();
				let key_storage = nd.key_storage.clone();
				let mut meta = meta.clone();
				meta.self_node_id = n.clone();
				let requester_signature = if meta.self_node_id == meta.master_node_id { Some(requester_signature.clone()) } else { None };
				let session = SessionImpl::new(SessionParams {
					meta: meta,
					all_nodes_set: all_nodes_set.clone(),
					cluster: nd.cluster.clone(),
					key_storage: nd.key_storage.clone(),
					nonce: 1,
				}).unwrap();
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
					all_nodes_set: all_nodes_set.clone(),
					cluster: cluster.clone(),
					key_storage: key_storage.clone(),
					nonce: 1,
				}).unwrap();
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
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add, BTreeSet::new());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), Signature::default(), Signature::default());
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
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add, BTreeSet::new());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), Signature::default(), Signature::default());
		ml.run();
	}

	#[test]
	fn node_moved_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, 3);
		let key_id = gml.session_id.clone();
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();
		let joint_secret = math::compute_joint_secret(gml.nodes.values()
			.map(|nd| nd.key_storage.get(&key_id).unwrap().polynom1[0].clone())
			.collect::<Vec<_>>()
			.iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// remove 1 node && insert 1 node so that one share is moved
		let nodes_to_remove: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add.clone(), nodes_to_remove.clone());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), Signature::default(), Signature::default());
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..4 {
			for n2 in n1+1..4 {
				let node1 = ml.nodes.keys().nth(n1).unwrap();
				let node2 = ml.nodes.keys().nth(n2).unwrap();
				if nodes_to_remove.contains(node1) {
					assert!(ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}
				if nodes_to_remove.contains(node2) {
					assert!(ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}

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
	fn node_removed_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, 3);
		let key_id = gml.session_id.clone();
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();
		let joint_secret = math::compute_joint_secret(gml.nodes.values()
			.map(|nd| nd.key_storage.get(&key_id).unwrap().polynom1[0].clone())
			.collect::<Vec<_>>()
			.iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// remove 1 node so that session becames 2-of-2
		let nodes_to_remove: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, BTreeSet::new(), nodes_to_remove.clone());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), Signature::default(), Signature::default());
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..3 {
			for n2 in n1+1..3 {
				let node1 = ml.nodes.keys().nth(n1).unwrap();
				let node2 = ml.nodes.keys().nth(n2).unwrap();
				if nodes_to_remove.contains(node1) {
					assert!(ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}
				if nodes_to_remove.contains(node2) {
					assert!(ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}

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
}
