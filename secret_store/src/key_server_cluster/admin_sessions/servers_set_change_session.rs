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
use ethereum_types::H256;
use ethkey::{Public, Signature};
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage};
use key_server_cluster::math;
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{Message, ServersSetChangeMessage,
	ConsensusMessageWithServersSet, InitializeConsensusSessionWithServersSet,
	ServersSetChangeConsensusMessage, ConfirmConsensusInitialization, UnknownSessionsRequest, UnknownSessions,
	ServersSetChangeShareAddMessage, ServersSetChangeError, ServersSetChangeCompleted,
	ServersSetChangeDelegate, ServersSetChangeDelegateResponse, InitializeShareChangeSession,
	ConfirmShareChangeSessionInitialization, KeyVersionNegotiationMessage, ShareChangeKeyVersionNegotiation};
use key_server_cluster::share_change_session::{ShareChangeSession, ShareChangeSessionParams, ShareChangeSessionPlan,
	prepare_share_change_session_plan};
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSessionImpl,
	SessionParams as KeyVersionNegotiationSessionParams, LargestSupportResultComputer,
	SessionTransport as KeyVersionNegotiationTransport};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::unknown_sessions_job::{UnknownSessionsJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};
use key_server_cluster::admin_sessions::sessions_queue::SessionsQueue;
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Maximal number of active share change sessions.
const MAX_ACTIVE_KEY_SESSIONS: usize = 64;

/// Servers set change session.
/// Brief overview:
/// 1) consensus establishing
/// 2) master node requests all other nodes for sessions he is not participating (aka unknown sessions)
/// 3) every slave node responds with sessions id => we are able to collect Map<SessionId, Set> of unknown sessions on master
/// 4) for every known session (i.e. session that master participates in):
/// 4.1) share change plan is created = nodes to add shares for, nodes to move shares from/to, nodes to remove shares from
/// 4.2) share change session is started. Share change session = sequential execution of ShareAdd, then ShareMove && then ShareRemove sessions (order matters here) for single key
/// 5) for every unknown session:
/// 5.1) sub_master is selected from sessions participants
/// 5.2) share change session is delegated from master to this sub_master
/// 5.3) share change session is executed by this sub_master
/// 5.4) share change confirm is sent from sub_master to master
/// 6) upon completing all known share change sessions && receiving confirmations for all unknown share change sessions, session completion signal is sent to all slave nodes && session is completed
pub struct SessionImpl {
	/// Session core.
	core: SessionCore,
	/// Session data.
	data: Mutex<SessionData>,
}

/// Session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// Establishing consensus.
	EstablishingConsensus,
	/// Running share change sessions.
	RunningShareChangeSessions,
	/// Session is completed.
	Finished,
}

/// Immutable session data.
struct SessionCore {
	/// Servers set change session meta (id is computed from new_nodes_set).
	pub meta: ShareChangeSessionMeta,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session-level nonce.
	pub nonce: u64,
	/// All known nodes.
	pub all_nodes_set: BTreeSet<NodeId>,
	/// Administrator public key.
	pub admin_public: Public,
	/// Migration id (if this session is a part of auto-migration process).
	pub migration_id: Option<H256>,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
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
	/// Share change sessions key version negotiation.
	pub negotiation_sessions: BTreeMap<SessionId, KeyVersionNegotiationSessionImpl<ServersSetChangeKeyVersionNegotiationTransport>>,
	/// Share change sessions initialization state (valid on master nodes only).
	pub sessions_initialization_state: BTreeMap<SessionId, SessionInitializationData>,
	/// Sessions delegated to other nodes (valid on master node only).
	pub delegated_key_sessions: BTreeMap<SessionId, NodeId>,
	/// Active share change sessions.
	pub active_key_sessions: BTreeMap<SessionId, ShareChangeSession>,
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
	pub meta: ShareChangeSessionMeta,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session nonce.
	pub nonce: u64,
	/// All known nodes.
	pub all_nodes_set: BTreeSet<NodeId>,
	/// Administrator public key.
	pub admin_public: Public,
	/// Migration id (if this session is a part of auto-migration process).
	pub migration_id: Option<H256>,
}

/// Servers set change consensus transport.
struct ServersSetChangeConsensusTransport {
	/// Session id.
	id: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Migration id (if part of auto-migration process).
	migration_id: Option<H256>,
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

/// Key version negotiation transport.
struct ServersSetChangeKeyVersionNegotiationTransport {
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
		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta,
				cluster: params.cluster,
				key_storage: params.key_storage,
				nonce: params.nonce,
				all_nodes_set: params.all_nodes_set,
				admin_public: params.admin_public,
				migration_id: params.migration_id,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::EstablishingConsensus,
				consensus_session: None,
				new_nodes_set: None,
				sessions_queue: None,
				negotiation_sessions: BTreeMap::new(),
				sessions_initialization_state: BTreeMap::new(),
				delegated_key_sessions: BTreeMap::new(),
				active_key_sessions: BTreeMap::new(),
				result: None,
			}),
		})
	}

	/// Get session id.
	pub fn id(&self) -> &SessionId {
		&self.core.meta.id
	}

	/// Get migration id.
	pub fn migration_id(&self) -> Option<&H256> {
		self.core.migration_id.as_ref()
	}

	/// Wait for session completion.
	pub fn wait(&self) -> Result<(), Error> {
		Self::wait_session(&self.core.completed, &self.data, None, |data| data.result.clone())
	}

	/// Initialize servers set change session on master node.
	pub fn initialize(&self, new_nodes_set: BTreeSet<NodeId>, all_set_signature: Signature, new_set_signature: Signature) -> Result<(), Error> {
		check_nodes_set(&self.core.all_nodes_set, &new_nodes_set)?;

		let mut data = self.data.lock();
		if data.state != SessionState::EstablishingConsensus || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
			meta: self.core.meta.clone().into_consensus_meta(self.core.all_nodes_set.len())?,
			consensus_executor: ServersSetChangeAccessJob::new_on_master(self.core.admin_public.clone(),
				self.core.all_nodes_set.clone(),
				new_nodes_set.clone(),
				all_set_signature,
				new_set_signature),
			consensus_transport: ServersSetChangeConsensusTransport {
				id: self.core.meta.id.clone(),
				nonce: self.core.nonce,
				migration_id: self.core.migration_id.clone(),
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
			&ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(ref message) =>
				self.on_key_version_negotiation(sender, message),
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
			&ServersSetChangeMessage::ServersSetChangeError(ref message) => {
				self.on_session_error(sender, Error::Io(message.error.clone()));
				Ok(())
			},
			&ServersSetChangeMessage::ServersSetChangeCompleted(ref message) => 
				self.on_session_completed(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ServersSetChangeConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::EstablishingConsensus {
			return Err(Error::InvalidStateForRequest);
		}

		// start slave consensus session if needed
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			if data.consensus_session.is_none() {
				match &message.message {
					&ConsensusMessageWithServersSet::InitializeConsensusSession(_) => {
						data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
							meta: self.core.meta.clone().into_consensus_meta(self.core.all_nodes_set.len())?,
							consensus_executor: ServersSetChangeAccessJob::new_on_slave(self.core.admin_public.clone()),
							consensus_transport: ServersSetChangeConsensusTransport {
								id: self.core.meta.id.clone(),
								nonce: self.core.nonce,
								migration_id: self.core.migration_id.clone(),
								cluster: self.core.cluster.clone(),
							},
						})?);
					},
					_ => return Err(Error::InvalidStateForRequest),
				}
			}
		}

		// process consensus message
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
		consensus_session.disseminate_jobs(unknown_sessions_job, self.unknown_sessions_transport(), false)
	}

	/// When unknown sessions are requested.
	pub fn on_unknown_sessions_requested(&self, sender: &NodeId, message: &UnknownSessionsRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		let new_nodes_set = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			let unknown_sessions_job = UnknownSessionsJob::new_on_slave(self.core.key_storage.clone());
			let unknown_sessions_transport = self.unknown_sessions_transport();

			// and respond with unknown sessions
			consensus_session.on_job_request(&sender, sender.clone(), unknown_sessions_job, unknown_sessions_transport)?;

			consensus_session.consensus_job().executor()
				.new_servers_set()
				.expect("consensus session is now completed; new_servers_set is intermediate result of consensus session; qed")
				.clone()
		};

		// update state
		data.state = SessionState::RunningShareChangeSessions;
		data.new_nodes_set = Some(new_nodes_set);

		Ok(())
	}

	/// When unknown sessions are received.
	pub fn on_unknown_sessions(&self, sender: &NodeId, message: &UnknownSessions) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::EstablishingConsensus {
			return Err(Error::InvalidStateForRequest);
		}

		// process message
		let unknown_sessions = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			consensus_session.on_job_response(sender, message.unknown_sessions.iter().cloned().map(Into::into).collect())?;
			if consensus_session.state() != ConsensusSessionState::Finished {
				return Ok(());
			}

			// all nodes have reported their unknown sessions
			// => we are ready to start adding/moving/removing shares
			consensus_session.result()?
		};

		// initialize sessions queue
		data.state = SessionState::RunningShareChangeSessions;
		data.sessions_queue = Some(SessionsQueue::new(&self.core.key_storage, unknown_sessions.keys().cloned().collect()));

		// and disseminate session initialization requests
		Self::disseminate_session_initialization_requests(&self.core, &mut *data)
	}

	/// When key version negotiation message is received.
	pub fn on_key_version_negotiation(&self, sender: &NodeId, message: &ShareChangeKeyVersionNegotiation) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// process message
		match &message.message {
			&KeyVersionNegotiationMessage::RequestKeyVersions(ref message) if sender == &self.core.meta.master_node_id => {
				let key_id = message.session.clone().into();
				let key_share = self.core.key_storage.get(&key_id).map_err(|e| Error::KeyStorage(e.into()))?;
				let negotiation_session = KeyVersionNegotiationSessionImpl::new(KeyVersionNegotiationSessionParams {
					meta: ShareChangeSessionMeta {
						id: key_id.clone(),
						self_node_id: self.core.meta.self_node_id.clone(),
						master_node_id: sender.clone(),
					},
					sub_session: message.sub_session.clone().into(),
					key_share: key_share,
					result_computer: Arc::new(LargestSupportResultComputer {}),
					transport: ServersSetChangeKeyVersionNegotiationTransport {
						id: self.core.meta.id.clone(),
						nonce: self.core.nonce,
						cluster: self.core.cluster.clone(),
					},
					nonce: message.session_nonce,
				});
				negotiation_session.on_key_versions_request(sender, message)?;
				debug_assert!(negotiation_session.is_finished());
				Ok(())
			},
			&KeyVersionNegotiationMessage::KeyVersions(ref message) if self.core.meta.self_node_id == self.core.meta.master_node_id => {
				let key_id = message.session.clone().into();
				{
					let negotiation_session = data.negotiation_sessions.get(&key_id).ok_or(Error::InvalidMessage)?;
					negotiation_session.on_key_versions(sender, message)?;
					if !negotiation_session.is_finished() {
						return Ok(());
					}
				}

				// else prepare plan && start share change session
				if !Self::initialize_share_change_session(&self.core, &mut *data, key_id)? {
					Self::disseminate_session_initialization_requests(&self.core, &mut *data)?;
				}

				Ok(())
			},
			_ => Err(Error::InvalidMessage),
		}
	}

	/// When share change session initialization is requested.
	pub fn on_initialize_share_change_session(&self, sender: &NodeId, message: &InitializeShareChangeSession) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// insert new session
		let key_id = message.key_id.clone().into();
		match data.active_key_sessions.contains_key(&key_id) {
			true => return Err(Error::InvalidMessage),
			false => {
				let master_plan = ShareChangeSessionPlan {
					key_version: message.version.clone().into(),
					consensus_group: message.consensus_group.iter().cloned().map(Into::into).collect(),
					new_nodes_map: message.new_nodes_map.iter().map(|(k, v)| (k.clone().into(), v.clone().map(Into::into))).collect(),
				};

				// if master plan is empty, it is cheating
				if master_plan.is_empty() {
					return Err(Error::InvalidMessage);
				}

				// on nodes, holding selected key share version, we could check if master node plan is correct
				let master_node_id = message.master_node_id.clone().into();
				if let Some(key_share) = self.core.key_storage.get(&key_id).map_err(|e| Error::KeyStorage(e.into()))? {
					let version = message.version.clone().into();
					if let Ok(key_version) = key_share.version(&version) {
						let key_share_owners = key_version.id_numbers.keys().cloned().collect();
						let new_nodes_set = data.new_nodes_set.as_ref()
							.expect("new_nodes_set is filled during consensus establishing; change sessions are running after this; qed");
						let local_plan = prepare_share_change_session_plan(
							&self.core.all_nodes_set,
							key_share.threshold,
							version,
							&master_node_id,
							&key_share_owners,
							new_nodes_set)?;

						if local_plan.new_nodes_map.keys().collect::<BTreeSet<_>>() != master_plan.new_nodes_map.keys().collect::<BTreeSet<_>>() {
							return Err(Error::InvalidMessage);
						}
					}
				}

				let session = Self::create_share_change_session(&self.core, key_id, master_node_id, master_plan)?;
				if !session.is_finished() {
					data.active_key_sessions.insert(key_id.clone(), session);
				}
			},
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

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// add confirmation
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
			data.delegated_key_sessions.insert(key_id, session_master.clone());
			return self.core.cluster.send(&session_master, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegate(ServersSetChangeDelegate {
				session: self.core.meta.id.clone().into(),
				session_nonce: self.core.nonce,
				key_id: key_id.into(),
			})));
		}

		// initialize share change session
		{
			let key_session = data.active_key_sessions.get_mut(&key_id).ok_or(Error::InvalidMessage)?;
			key_session.initialize()?;
			if !key_session.is_finished() {
				return Ok(());
			}
		}

		// complete key session
		Self::complete_key_session(&self.core, &mut *data, true, key_id)
	}

	/// When sessions execution is delegated to this node.
	pub fn on_sessions_delegation(&self, sender: &NodeId, message: &ServersSetChangeDelegate) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests from master node
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// start session
		let key_session = data.active_key_sessions.get_mut(&message.key_id.clone().into()).ok_or(Error::InvalidMessage)?;
		key_session.initialize()
	}

	/// When delegated session execution is completed.
	pub fn on_delegated_session_completed(&self, sender: &NodeId, message: &ServersSetChangeDelegateResponse) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// we only accept delegation requests on master node
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// forget delegated session
		let key_id = message.key_id.clone().into();
		match data.delegated_key_sessions.entry(key_id) {
			Entry::Occupied(entry) => if entry.get() == sender {
				entry.remove()
			} else {
				return Err(Error::InvalidMessage);
			},
			_ => return Err(Error::InvalidMessage),
		};

		// check if we need to complete the whole change session
		Self::disseminate_session_initialization_requests(&self.core, &mut *data)
	}

	/// When share add message is received.
	pub fn on_share_add_message(&self, sender: &NodeId, message: &ServersSetChangeShareAddMessage) -> Result<(), Error> {
		self.on_share_change_message(message.message.session_id().clone().into(), |session|
			session.on_share_add_message(sender, &message.message))
	}

	/// When session completion message is received.
	pub fn on_session_completed(&self, sender: &NodeId, message: &ServersSetChangeCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();
		data.result = Some(Ok(()));
		if data.active_key_sessions.len() != 0 {
			return Err(Error::TooEarlyForRequest);
		}

		// if we are on the set of nodes that are being removed from the cluster, let's clear database
		if !data.new_nodes_set.as_ref()
			.expect("new_nodes_set is filled during initialization; session is completed after initialization; qed")
			.contains(&self.core.meta.self_node_id) {
			self.core.key_storage.clear().map_err(|e| Error::KeyStorage(e.into()))?;
		}

		data.state = SessionState::Finished;
		self.core.completed.notify_all();

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

	/// When share change message is received.
	fn on_share_change_message<F: Fn(&mut ShareChangeSession) -> Result<(), Error>>(&self, session_id: SessionId, message_processor: F) -> Result<(), Error> {
		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::RunningShareChangeSessions {
			return Err(Error::InvalidStateForRequest);
		}

		// process message
		let (is_finished, is_master) = {
			let key_session = data.active_key_sessions.get_mut(&session_id).ok_or(Error::InvalidMessage)?;
			message_processor(key_session)?;
			(key_session.is_finished(), key_session.is_master())
		};

		if is_finished {
			Self::complete_key_session(&self.core, &mut *data, is_master, session_id)?;
		}

		Ok(())
	}

	/// Create share change session.
	fn create_share_change_session(core: &SessionCore, key_id: SessionId, master_node_id: NodeId, session_plan: ShareChangeSessionPlan) -> Result<ShareChangeSession, Error> {
		ShareChangeSession::new(ShareChangeSessionParams {
			session_id: core.meta.id.clone(),
			nonce: core.nonce,
			meta: ShareChangeSessionMeta {
				id: key_id,
				self_node_id: core.meta.self_node_id.clone(),
				master_node_id: master_node_id,
			},
			cluster: core.cluster.clone(),
			key_storage: core.key_storage.clone(),
			plan: session_plan,
		})
	}

	/// Disseminate session initialization requests.
	fn disseminate_session_initialization_requests(core: &SessionCore, data: &mut SessionData) -> Result<(), Error> {
		debug_assert_eq!(core.meta.self_node_id, core.meta.master_node_id);
		if data.sessions_queue.is_some() {
			let number_of_sessions_active = data.active_key_sessions.len()
				+ data.delegated_key_sessions.len()
				+ data.negotiation_sessions.len();
			let mut number_of_sessions_to_start = MAX_ACTIVE_KEY_SESSIONS.saturating_sub(number_of_sessions_active);
			while number_of_sessions_to_start > 0 {
				let key_id = match data.sessions_queue.as_mut().expect("checked before beginning of the loop; qed").next() {
					None => break, // complete session
					Some(Err(e)) => return Err(e),
					Some(Ok(key_id)) => key_id,
				};

				let key_share = core.key_storage.get(&key_id).map_err(|e| Error::KeyStorage(e.into()))?;
				let negotiation_session = KeyVersionNegotiationSessionImpl::new(KeyVersionNegotiationSessionParams {
					meta: ShareChangeSessionMeta {
						id: key_id,
						self_node_id: core.meta.self_node_id.clone(),
						master_node_id: core.meta.self_node_id.clone(),
					},
					sub_session: math::generate_random_scalar()?,
					key_share: key_share,
					result_computer: Arc::new(LargestSupportResultComputer {}), // TODO [Opt]: could use modified Fast version
					transport: ServersSetChangeKeyVersionNegotiationTransport {
						id: core.meta.id.clone(),
						nonce: core.nonce,
						cluster: core.cluster.clone(),
					},
					nonce: 0,
				});
				negotiation_session.initialize(core.cluster.nodes())?;
				if !negotiation_session.is_finished() {
					data.negotiation_sessions.insert(key_id, negotiation_session);
					continue;
				}

				if !Self::initialize_share_change_session(core, data, key_id)? {
					continue;
				}

				number_of_sessions_to_start = number_of_sessions_to_start - 1;
			}

			// if iteration is not yet finished => return
			if number_of_sessions_to_start == 0 {
				return Ok(());
			}
		}

		// iteration is finished => complete session
		if data.state != SessionState::Finished {
			data.sessions_queue = None;
			if data.active_key_sessions.len() == 0 &&
				data.delegated_key_sessions.len() == 0 &&
				data.negotiation_sessions.len() == 0 {
				Self::complete_session(core, data)?;
			}
		}

		Ok(())
	}

	/// Initialize share change session.
	fn initialize_share_change_session(core: &SessionCore, data: &mut SessionData, key_id: SessionId) -> Result<bool, Error> {
		// get selected version && old nodes set from key negotiation session
		let negotiation_session = data.negotiation_sessions.remove(&key_id)
			.expect("share change session is only initialized when negotiation is completed; qed");
		let (selected_version, selected_master) = negotiation_session.wait()?;
		let selected_version_holders = negotiation_session.version_holders(&selected_version)?;
		let selected_version_threshold = negotiation_session.key_threshold()?;

		// prepare session change plan && check if something needs to be changed
		let old_nodes_set = selected_version_holders;
		let new_nodes_set = data.new_nodes_set.as_ref()
			.expect("this method is called after consensus estabished; new_nodes_set is a result of consensus session; qed");
		let session_plan = prepare_share_change_session_plan(&core.all_nodes_set, selected_version_threshold, selected_version.clone(), &selected_master, &old_nodes_set, new_nodes_set)?;
		if session_plan.is_empty() {
			return Ok(false);
		}

		// send key session initialization requests
		let mut confirmations: BTreeSet<_> = session_plan.new_nodes_map.keys().cloned().collect();
		let need_create_session = confirmations.remove(&core.meta.self_node_id);
		let initialization_message = Message::ServersSetChange(ServersSetChangeMessage::InitializeShareChangeSession(InitializeShareChangeSession {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
			key_id: key_id.clone().into(),
			version: selected_version.into(),
			master_node_id: selected_master.clone().into(),
			consensus_group: session_plan.consensus_group.iter().cloned().map(Into::into).collect(),
			new_nodes_map: session_plan.new_nodes_map.iter()
				.map(|(n, nid)| (n.clone().into(), nid.clone().map(Into::into)))
				.collect(),
		}));
		for node in &confirmations {
			core.cluster.send(&node, initialization_message.clone())?;
		}

		// create session on this node if required
		if need_create_session {
			data.active_key_sessions.insert(key_id.clone(), Self::create_share_change_session(core, key_id,
				selected_master.clone(),
				session_plan)?);
		}

		// initialize session if required
		let wait_for_confirmations = !confirmations.is_empty();
		if !wait_for_confirmations {
			data.active_key_sessions.get_mut(&key_id)
				.expect("!wait_for_confirmations is true only if this is the only session participant; if this is session participant, session is created above; qed")
				.initialize()?;
		} else {
			data.sessions_initialization_state.insert(key_id, SessionInitializationData {
				master: selected_master,
				confirmations: confirmations,
			});
		}

		Ok(true)
	}

	/// Return delegated session to master.
	fn return_delegated_session(core: &SessionCore, key_id: &SessionId) -> Result<(), Error> {
		assert!(core.meta.self_node_id != core.meta.master_node_id);
		core.cluster.send(&core.meta.master_node_id, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeDelegateResponse(ServersSetChangeDelegateResponse {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
			key_id: key_id.clone().into(),
		})))
	}

	/// Complete key session.
	fn complete_key_session(core: &SessionCore, data: &mut SessionData, is_master: bool, session_id: SessionId) -> Result<(), Error> {
		data.active_key_sessions.remove(&session_id);
		let is_general_master = core.meta.self_node_id == core.meta.master_node_id;
		if is_master && !is_general_master {
			Self::return_delegated_session(core, &session_id)?;
		}
		if is_general_master {
			Self::disseminate_session_initialization_requests(core, data)?;
		}

		if data.result.is_some() && data.active_key_sessions.len() == 0 {
			data.state = SessionState::Finished;
			core.completed.notify_all();
		}

		Ok(())
	}

	/// Complete servers set change session.
	fn complete_session(core: &SessionCore, data: &mut SessionData) -> Result<(), Error> {
		debug_assert_eq!(core.meta.self_node_id, core.meta.master_node_id);
		
		// send completion notification
		core.cluster.broadcast(Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeCompleted(ServersSetChangeCompleted {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
		})))?;

		// if we are on the set of nodes that are being removed from the cluster, let's clear database
		if !data.new_nodes_set.as_ref()
			.expect("new_nodes_set is filled during initialization; session is completed after initialization; qed")
			.contains(&core.meta.self_node_id) {
			core.key_storage.clear().map_err(|e| Error::KeyStorage(e.into()))?;
		}

		data.state = SessionState::Finished;
		data.result = Some(Ok(()));
		core.completed.notify_all();

		Ok(())
	}
}

impl ClusterSession for SessionImpl {
	type Id = SessionId;

	fn type_name() -> &'static str {
		"servers set change"
	}

	fn id(&self) -> SessionId {
		self.core.meta.id.clone()
	}

	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		self.on_session_error(&self.core.meta.self_node_id, Error::NodeDisconnected);
	}

	fn on_node_timeout(&self, node: &NodeId) {
		self.on_session_error(node, Error::NodeDisconnected);
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		// error in generation session is considered fatal
		// => broadcast error if error occured on this node
		if *node == self.core.meta.self_node_id {
			// do not bother processing send error, as we already processing error
			let _ = self.core.cluster.broadcast(Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeError(ServersSetChangeError {
				session: self.core.meta.id.clone().into(),
				session_nonce: self.core.nonce,
				error: error.clone().into(),
			})));
		}

		let mut data = self.data.lock();

		warn!("{}: servers set change session failed: {} on {}", self.core.meta.self_node_id, error, node);

		data.state = SessionState::Finished;
		data.result = Some(Err(error));
		self.core.completed.notify_all();
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		match *message {
			Message::ServersSetChange(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
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
				migration_id: self.migration_id.clone().map(Into::into),
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

	fn send_partial_request(&self, node: &NodeId, _request: NodeId) -> Result<(), Error> {
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

impl KeyVersionNegotiationTransport for ServersSetChangeKeyVersionNegotiationTransport {
	fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ShareChangeKeyVersionNegotiation(ShareChangeKeyVersionNegotiation {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
			message: message,
		})))
	}
}

fn check_nodes_set(all_nodes_set: &BTreeSet<NodeId>, new_nodes_set: &BTreeSet<NodeId>) -> Result<(), Error> {
	// all new nodes must be a part of all nodes set
	match new_nodes_set.iter().any(|n| !all_nodes_set.contains(n)) {
		true => Err(Error::InvalidNodesConfiguration),
		false => Ok(())
	}
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, Signature, KeyPair, sign};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage};
	use key_server_cluster::cluster::Cluster;
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::{MessageLoop as GenerationMessageLoop, Node as GenerationNode, generate_nodes_ids};
	use key_server_cluster::message::Message;
	use key_server_cluster::admin_sessions::ShareChangeSessionMeta;
	use key_server_cluster::admin_sessions::share_add_session::tests::check_secret_is_preserved;
	use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
	use super::{SessionImpl, SessionParams};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl,
	}

	struct MessageLoop {
		pub admin_key_pair: KeyPair,
		pub original_key_pair: KeyPair,
		pub all_nodes_set: BTreeSet<NodeId>,
		pub new_nodes_set: BTreeSet<NodeId>,
		pub all_set_signature: Signature,
		pub new_set_signature: Signature,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	fn create_session(mut meta: ShareChangeSessionMeta, self_node_id: NodeId, admin_public: Public, all_nodes_set: BTreeSet<NodeId>, cluster: Arc<Cluster>, key_storage: Arc<KeyStorage>) -> SessionImpl {
		meta.self_node_id = self_node_id;
		SessionImpl::new(SessionParams {
			meta: meta,
			all_nodes_set: all_nodes_set,
			cluster: cluster,
			key_storage: key_storage,
			nonce: 1,
			admin_public: admin_public,
			migration_id: None,
		}).unwrap()
	}

	fn create_node(meta: ShareChangeSessionMeta, admin_public: Public, all_nodes_set: BTreeSet<NodeId>, node: GenerationNode) -> Node {
		for n in &all_nodes_set {
			node.cluster.add_node(n.clone());
		}

		Node {
			cluster: node.cluster.clone(),
			key_storage: node.key_storage.clone(),
			session: create_session(meta, node.session.node().clone(), admin_public, all_nodes_set, node.cluster, node.key_storage),
		}
	}

	impl MessageLoop {
		pub fn new(gml: GenerationMessageLoop, master_node_id: NodeId, new_nodes_ids: BTreeSet<NodeId>, removed_nodes_ids: BTreeSet<NodeId>, isolated_nodes_ids: BTreeSet<NodeId>) -> Self {
			// generate admin key pair
			let admin_key_pair = Random.generate().unwrap();
			let admin_public = admin_key_pair.public().clone();

			// compute original secret key
			let original_key_pair = gml.compute_key_pair(1);

			// all active nodes set
			let mut all_nodes_set: BTreeSet<_> = gml.nodes.keys()
				.filter(|n| !isolated_nodes_ids.contains(n))
				.cloned()
				.collect();
			// new nodes set includes all old nodes, except nodes being removed + all nodes being added
			let new_nodes_set: BTreeSet<NodeId> = all_nodes_set.iter().cloned()
				.chain(new_nodes_ids.iter().cloned())
				.filter(|n| !removed_nodes_ids.contains(n))
				.collect();
			all_nodes_set.extend(new_nodes_ids.iter().cloned());
			for isolated_node_id in &isolated_nodes_ids {
				all_nodes_set.remove(isolated_node_id);
			}

			let meta = ShareChangeSessionMeta {
				self_node_id: master_node_id.clone(),
				master_node_id: master_node_id.clone(),
				id: SessionId::default(),
			};

			let old_nodes = gml.nodes.into_iter().map(|n| create_node(meta.clone(), admin_public.clone(), all_nodes_set.clone(), n.1));
			let new_nodes = new_nodes_ids.into_iter().map(|new_node_id| {
				let new_node_cluster = Arc::new(DummyCluster::new(new_node_id.clone()));
				for node in &all_nodes_set {
					new_node_cluster.add_node(node.clone());
				}

				let new_node_key_storage = Arc::new(DummyKeyStorage::default());
				let new_node_session = create_session(meta.clone(), new_node_id, admin_public.clone(), all_nodes_set.clone(), new_node_cluster.clone(), new_node_key_storage.clone());
				Node {
					cluster: new_node_cluster,
					key_storage: new_node_key_storage,
					session: new_node_session,
				}
			});
			let nodes: BTreeMap<_, _> = old_nodes.chain(new_nodes).map(|n| (n.session.core.meta.self_node_id.clone(), n)).collect();

			for node in nodes.values() {
				for isolated_node_id in &isolated_nodes_ids {
					node.cluster.remove_node(isolated_node_id);
				}
			}

			let all_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&all_nodes_set)).unwrap();
			let new_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&new_nodes_set)).unwrap();

			MessageLoop {
				admin_key_pair: admin_key_pair,
				original_key_pair: original_key_pair,
				all_nodes_set: all_nodes_set.clone(),
				new_nodes_set: new_nodes_set,
				all_set_signature: all_set_signature,
				new_set_signature: new_set_signature,
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
			match { match msg.2 {
				Message::ServersSetChange(ref message) => self.nodes[&msg.1].session.process_message(&msg.0, message),
				_ => unreachable!("only servers set change messages are expected"),
			} } {
				Ok(_) => Ok(()),
				Err(Error::TooEarlyForRequest) => {
					self.queue.push_back(msg);
					Ok(())
				},
				Err(err) => Err(err),
			}
		}
	}

	pub fn generate_key(threshold: usize, nodes_ids: BTreeSet<NodeId>) -> GenerationMessageLoop {
		let mut gml = GenerationMessageLoop::with_nodes_ids(nodes_ids);
		gml.master().initialize(Default::default(), false, threshold, gml.nodes.keys().cloned().collect::<BTreeSet<_>>().into()).unwrap();
		while let Some((from, to, message)) = gml.take_message() {
			gml.process_message((from, to, message)).unwrap();
		}
		gml
	}

	#[test]
	fn node_added_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, generate_nodes_ids(3));
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();

		// insert 1 node so that it becames 2-of-4 session
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add, BTreeSet::new(), BTreeSet::new());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), ml.all_set_signature.clone(), ml.new_set_signature.clone()).unwrap();
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter().map(|(k, v)| (k.clone(), v.key_storage.clone())).collect());

		// check that all sessions have finished
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));
	}

	#[test]
	fn node_added_using_server_set_change_from_this_node() {
		// initial 2-of-3 session
		let gml = generate_key(1, generate_nodes_ids(3));

		// insert 1 node so that it becames 2-of-4 session
		// master node is the node we are adding =>
		// 1) add session is delegated to one of old nodes
		// 2) key share is pushed to new node
		// 3) delegated session is returned back to added node
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let master_node_id = nodes_to_add.iter().cloned().nth(0).unwrap();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add, BTreeSet::new(), BTreeSet::new());
		ml.nodes[&master_node_id].session.initialize(ml.nodes.keys().cloned().collect(), ml.all_set_signature.clone(), ml.new_set_signature.clone()).unwrap();
		ml.run();

		// check that all sessions have finished
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));
	}

	#[test]
	fn node_moved_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, generate_nodes_ids(3));
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();

		// remove 1 node && insert 1 node so that one share is moved
		let nodes_to_remove: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let nodes_to_add: BTreeSet<_> = (0..1).map(|_| Random.generate().unwrap().public().clone()).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, nodes_to_add.clone(), nodes_to_remove.clone(), BTreeSet::new());
		let new_nodes_set = ml.nodes.keys().cloned().filter(|n| !nodes_to_remove.contains(n)).collect();
		ml.nodes[&master_node_id].session.initialize(new_nodes_set, ml.all_set_signature.clone(), ml.new_set_signature.clone()).unwrap();
		ml.run();

		// check that secret is still the same as before moving the share
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
			.filter(|&(k, _)| !nodes_to_remove.contains(k))
			.map(|(k, v)| (k.clone(), v.key_storage.clone()))
			.collect());

		// check that all removed nodes do not own key share
		assert!(ml.nodes.iter().filter(|&(k, _)| nodes_to_remove.contains(k)).all(|(_, v)| v.key_storage.get(&SessionId::default()).unwrap().is_none()));

		// check that all sessions have finished
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));
	}

	#[test]
	fn node_removed_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, generate_nodes_ids(3));
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();

		// remove 1 node so that session becames 2-of-2
		let nodes_to_remove: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let new_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().filter(|n| !nodes_to_remove.contains(&n)).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, BTreeSet::new(), nodes_to_remove.clone(), BTreeSet::new());
		ml.nodes[&master_node_id].session.initialize(new_nodes_set, ml.all_set_signature.clone(), ml.new_set_signature.clone()).unwrap();
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
			.filter(|&(k, _)| !nodes_to_remove.contains(k))
			.map(|(k, v)| (k.clone(), v.key_storage.clone()))
			.collect());

		// check that all removed nodes do not own key share
		assert!(ml.nodes.iter().filter(|&(k, _)| nodes_to_remove.contains(k)).all(|(_, v)| v.key_storage.get(&SessionId::default()).unwrap().is_none()));

		// check that all sessions have finished
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));
	}

	#[test]
	fn isolated_node_removed_using_servers_set_change() {
		// initial 2-of-3 session
		let gml = generate_key(1, generate_nodes_ids(3));
		let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();

		// remove 1 node so that session becames 2-of-2
		let nodes_to_isolate: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let new_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().filter(|n| !nodes_to_isolate.contains(&n)).collect();
		let mut ml = MessageLoop::new(gml, master_node_id, BTreeSet::new(), BTreeSet::new(), nodes_to_isolate.clone());
		ml.nodes[&master_node_id].session.initialize(new_nodes_set, ml.all_set_signature.clone(), ml.new_set_signature.clone()).unwrap();
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
			.filter(|&(k, _)| !nodes_to_isolate.contains(k))
			.map(|(k, v)| (k.clone(), v.key_storage.clone()))
			.collect());

		// check that all isolated nodes still OWN key share
		assert!(ml.nodes.iter().filter(|&(k, _)| nodes_to_isolate.contains(k)).all(|(_, v)| v.key_storage.get(&SessionId::default()).unwrap().is_some()));

		// check that all sessions have finished
		assert!(ml.nodes.iter().filter(|&(k, _)| !nodes_to_isolate.contains(k)).all(|(_, v)| v.session.is_finished()));
	}
}
