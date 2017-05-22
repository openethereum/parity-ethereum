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

use std::cmp::{Ordering, Ord, PartialOrd};
use std::sync::Arc;
use std::time;
use parking_lot::{Mutex, Condvar};
use ethkey::{self, Public, Secret, Signature};
use util;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::consensus_session::{ConsensusSession, Consensus, SessionParams as ConsensusSessionParams,
	SessionState as ConsensusSessionState, SessionAction as ConsensusSessionAction};
use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, ConsensusMessage};

/// Signing session API.
pub trait Session: Send + Sync + 'static {
	/// Get generation session state.
	fn state(&self) -> SessionState;
	/// Wait until session is completed. Returns signed message.
	fn wait(&self, timeout: Option<time::Duration>) -> Result<util::Bytes, Error>;
}

/// Signing session.
pub struct SessionImpl {
	/// Key generation session id.
	id: SessionId,
	/// Signing session access key.
	access_key: Secret,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Encrypted data.
	encrypted_data: DocumentKeyShare,
	/// ACL storate to check access to the resource.
	acl_storage: Arc<AclStorage>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	cluster: Arc<Cluster>,
	/// SessionImpl completion condvar.
	completed: Condvar,
	/// Mutable session data.
	data: Mutex<SessionData>,
}

/// Signing session Id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningSessionId {
	/// Encryption session id.
	pub id: SessionId,
	/// Signing session access key.
	pub access_key: Secret,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// SessionImpl identifier.
	pub id: SessionId,
	/// SessionImpl access key.
	pub access_key: Secret,
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Encrypted data (result of running encryption_session::SessionImpl).
	pub encrypted_data: DocumentKeyShare,
	/// Key storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
}

/// Mutable data of signing session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,
	/// Public key of requestor.
	requestor: Option<Public>,

	// === Values, filled when consensus is establishing ===
	/// Consensus session.
	consensus_session: Option<ConsensusSession>,
	/// Consensus params.
	consensus: Option<Consensus>,

	/// === Values, filled during final decryption ===
	/// Decrypted secret
	signed_message: Option<Result<(), Error>>,
}

#[derive(Debug, Clone)]
/// Mutable node-specific data.
struct NodeData {
	/// Random unique scalar. Persistent.
	pub id_number: Secret,
}

#[derive(Debug, Clone, PartialEq)]
/// Distributed key generation session state.
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Establishing consensus.
	EstablishingConsensus,

	// === Final states of the session ===
	/// Signing is completed.
	Finished,
	/// Signing is failed.
	Failed,
}

impl SessionImpl {
	/// Create new decryption session.
	pub fn new(params: SessionParams) -> Result<Self, Error> {
		check_encrypted_data(&params.self_node_id, &params.encrypted_data)?;

		Ok(SessionImpl {
			id: params.id,
			access_key: params.access_key,
			self_node_id: params.self_node_id,
			encrypted_data: params.encrypted_data,
			acl_storage: params.acl_storage,
			cluster: params.cluster,
			completed: Condvar::new(),
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				master: None,
				requestor: None,
				consensus_session: None,
				consensus: None,
				signed_message: None,
			})
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Initialize signing session.
	pub fn initialize(&self, requestor_signature: Signature) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		data.master = Some(self.node().clone());
		//data.requestor = Some(requestor_public.clone());
		data.state = SessionState::EstablishingConsensus;

		// create consensus session
		let consensus_session = ConsensusSession::new(ConsensusSessionParams {
			id: self.id.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: self.encrypted_data.clone(),
			acl_storage: self.acl_storage.clone(),
		})?;

		// start consensus session
		let consensus_action = consensus_session.initialize(requestor_signature)?;
		data.consensus_session = Some(consensus_session);
		process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if single node is required to sign message, proceed
		if data.state != SessionState::Failed && data.consensus.is_some() {
					//data.state = SessionState::Finished;
					//SessionImpl::start_waiting_for_partial_signature(self.node().clone(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)?;
					//SessionImpl::make_signature(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
					//self.completed.notify_all();
		}

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: NodeId, message: &SigningConsensusMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// if we are waiting for initialization
		if data.state == SessionState::WaitingForInitialization {
			data.master = Some(sender.clone());
			//data.requestor = Some(requestor_public);
			data.state = SessionState::EstablishingConsensus;
			data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
				id: self.id.clone(),
				self_node_id: self.self_node_id.clone(),
				encrypted_data: self.encrypted_data.clone(),
				acl_storage: self.acl_storage.clone(),
			})?);
		}

		// check state
		if data.state != SessionState::EstablishingConsensus {
			return Err(Error::InvalidStateForRequest);
		}

		let consensus_action = match message.message {
			ConsensusMessage::InitializeConsensusSession(ref message) =>
				data.consensus_session.as_ref().expect("TODO").on_initialize_session(sender, &message)?,
			ConsensusMessage::ConfirmConsensusInitialization(ref message) =>
				data.consensus_session.as_ref().expect("TODO").on_confirm_initialization(sender, &message)?,
		};

		process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)
	}
}

fn process_consensus_session_action(id: &SessionId, access_key: &Secret, cluster: &Arc<Cluster>, completed: &Condvar, data: &mut SessionData, action: ConsensusSessionAction) -> Result<(), Error> {
	match action {
		ConsensusSessionAction::BroadcastMessage(message) => {
			cluster.broadcast(Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
				session: id.clone().into(),
				sub_session: access_key.clone().into(),
				message: message,
			})))
		},
		ConsensusSessionAction::SendMessage(to, message) => {
			cluster.send(&to, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
				session: id.clone().into(),
				sub_session: access_key.clone().into(),
				message: message,
			})))
		},
		ConsensusSessionAction::CheckStatus => match data.consensus_session.as_ref().expect("TODO").consensus() {
			Some(Ok(consensus)) => {
				data.consensus = Some(consensus.clone());
				Ok(())
			},
			Some(Err(err)) => {
				data.state = SessionState::Failed;
				data.signed_message = Some(Err(err));
				completed.notify_all();
				Ok(())
			},
			None => Ok(()),
		},
	}
}

impl ClusterSession for SessionImpl {
	fn is_finished(&self) -> bool {
		unimplemented!()
	}

	fn on_session_timeout(&self) {
		unimplemented!()
	}

	fn on_node_timeout(&self, node_id: &NodeId) {
		unimplemented!()
	}
}

impl Session for SessionImpl {
	fn state(&self) -> SessionState {
		unimplemented!()
	}

	fn wait(&self, timeout: Option<time::Duration>) -> Result<util::Bytes, Error> {
		unimplemented!()
	}
}

impl SigningSessionId {
	/// Create new decryption session Id.
	pub fn new(session_id: SessionId, sub_session_id: Secret) -> Self {
		SigningSessionId {
			id: session_id,
			access_key: sub_session_id,
		}
	}
}

impl PartialOrd for SigningSessionId {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for SigningSessionId {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.id.cmp(&other.id) {
			Ordering::Equal => self.access_key.cmp(&other.access_key),
			r @ _ => r,
		}
	}
}

fn check_encrypted_data(self_node_id: &Public, encrypted_data: &DocumentKeyShare) -> Result<(), Error> {
	use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)
}