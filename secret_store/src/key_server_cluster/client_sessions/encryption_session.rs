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

use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter, Error as FmtError};
use std::time;
use std::sync::Arc;
use parking_lot::{Condvar, Mutex};
use ethkey::{self, Public, Signature};
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage, DocumentKeyShare};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{Message, EncryptionMessage, InitializeEncryptionSession,
	ConfirmEncryptionInitialization, EncryptionSessionError};

/// Encryption session API.
pub trait Session: Send + Sync + 'static {
	/// Get encryption session state.
	fn state(&self) -> SessionState;
	/// Wait until session is completed. Returns distributely generated secret key.
	fn wait(&self, timeout: Option<time::Duration>) -> Result<(), Error>;
}

/// Encryption (distributed key generation) session.
/// Based on "ECDKG: A Distributed Key Generation Protocol Based on Elliptic Curve Discrete Logarithm" paper:
/// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.124.4128&rep=rep1&type=pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for storing the secret) initializes the session on all other nodes
/// 2) master node sends common_point + encrypted_point to all other nodes
/// 3) common_point + encrypted_point are saved on all nodes
/// 4) in case of error, previous values are restored
pub struct SessionImpl {
	/// Unique session id.
	id: SessionId,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Encrypted data.
	encrypted_data: DocumentKeyShare,
	/// Key storage.
	key_storage: Arc<KeyStorage>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	cluster: Arc<Cluster>,
	/// Session nonce.
	nonce: u64,
	/// SessionImpl completion condvar.
	completed: Condvar,
	/// Mutable session data.
	data: Mutex<SessionData>,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// SessionImpl identifier.
	pub id: SessionId,
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Encrypted data (result of running generation_session::SessionImpl).
	pub encrypted_data: DocumentKeyShare,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
	/// Session nonce.
	pub nonce: u64,
}

/// Mutable data of encryption (distributed key generation) session.
#[derive(Debug)]
struct SessionData {
	/// Current state of the session.
	state: SessionState,
	/// Nodes-specific data.
	nodes: BTreeMap<NodeId, NodeData>,
	/// Encryption session result.
	result: Option<Result<(), Error>>,
}

/// Mutable node-specific data.
#[derive(Debug, Clone)]
struct NodeData {
	// === Values, filled during initialization phase ===
	/// Flags marking that node has confirmed session initialization.
	pub initialization_confirmed: bool,
}

/// Encryption (distributed key generation) session state.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Master node waits for every other node to confirm initialization.
	WaitingForInitializationConfirm,

	// === Final states of the session ===
	/// Encryption data is saved.
	Finished,
	/// Failed to save encryption data.
	Failed,
}

impl SessionImpl {
	/// Create new encryption session.
	pub fn new(params: SessionParams) -> Result<Self, Error> {
		check_encrypted_data(&params.self_node_id, &params.encrypted_data)?;

		Ok(SessionImpl {
			id: params.id,
			self_node_id: params.self_node_id,
			encrypted_data: params.encrypted_data,
			key_storage: params.key_storage,
			cluster: params.cluster,
			nonce: params.nonce,
			completed: Condvar::new(),
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				nodes: BTreeMap::new(),
				result: None,
			}),
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Start new session initialization. This must be called on master node.
	pub fn initialize(&self, requestor_signature: Signature, common_point: Public, encrypted_point: Public) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// check that the requester is the author of the encrypted data
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;
		if self.encrypted_data.author != requestor_public {
			return Err(Error::AccessDenied);
		}

		// update state
		data.state = SessionState::WaitingForInitializationConfirm;
		for node_id in self.encrypted_data.id_numbers.keys() {
			data.nodes.insert(node_id.clone(), NodeData {
				initialization_confirmed: node_id == self.node(),
			});
		}

		// TODO: there could be situation when some nodes have failed to store encrypted data
		// => potential problems during restore. some confirmation step is needed?
		// save encryption data
		let mut encrypted_data = self.encrypted_data.clone();
		encrypted_data.common_point = Some(common_point.clone());
		encrypted_data.encrypted_point = Some(encrypted_point.clone());
		self.key_storage.update(self.id.clone(), encrypted_data)
			.map_err(|e| Error::KeyStorage(e.into()))?;

		// start initialization
		if self.encrypted_data.id_numbers.len() > 1 {
			self.cluster.broadcast(Message::Encryption(EncryptionMessage::InitializeEncryptionSession(InitializeEncryptionSession {
				session: self.id.clone().into(),
				session_nonce: self.nonce,
				requestor_signature: requestor_signature.into(),
				common_point: common_point.into(),
				encrypted_point: encrypted_point.into(),
			})))
		} else {
			data.state = SessionState::Finished;
			data.result = Some(Ok(()));
			self.completed.notify_all();

			Ok(())
		}
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&self, sender: NodeId, message: &InitializeEncryptionSession) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(&sender != self.node());

		self.check_nonce(message.session_nonce)?;

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// check that the requester is the author of the encrypted data
		let requestor_public = ethkey::recover(&message.requestor_signature.clone().into(), &self.id)?;
		if self.encrypted_data.author != requestor_public {
			return Err(Error::AccessDenied);
		}

		// save encryption data
		let mut encrypted_data = self.encrypted_data.clone();
		encrypted_data.common_point = Some(message.common_point.clone().into());
		encrypted_data.encrypted_point = Some(message.encrypted_point.clone().into());
		self.key_storage.update(self.id.clone(), encrypted_data)
			.map_err(|e| Error::KeyStorage(e.into()))?;

		// update state
		data.state = SessionState::Finished;

		// send confirmation back to master node
		self.cluster.send(&sender, Message::Encryption(EncryptionMessage::ConfirmEncryptionInitialization(ConfirmEncryptionInitialization {
			session: self.id.clone().into(),
			session_nonce: self.nonce,
		})))
	}

	/// When session initialization confirmation message is reeived.
	pub fn on_confirm_initialization(&self, sender: NodeId, message: &ConfirmEncryptionInitialization) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(&sender != self.node());

		self.check_nonce(message.session_nonce)?;

		let mut data = self.data.lock();
		debug_assert!(data.nodes.contains_key(&sender));

		// check if all nodes have confirmed initialization
		data.nodes.get_mut(&sender)
			.expect("message is received from cluster; nodes contains all cluster nodes; qed")
			.initialization_confirmed = true;
		if !data.nodes.values().all(|n| n.initialization_confirmed) {
			return Ok(());
		}

		// update state
		data.state = SessionState::Finished;
		data.result = Some(Ok(()));
		self.completed.notify_all();

		Ok(())
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &EncryptionSessionError) -> Result<(), Error> {
		self.check_nonce(message.session_nonce)?;

		let mut data = self.data.lock();

		warn!("{}: encryption session failed with error: {} from {}", self.node(), message.error, sender);

		data.state = SessionState::Failed;
		data.result = Some(Err(Error::Io(message.error.clone())));
		self.completed.notify_all();

		Ok(())
	}

	/// Check session nonce.
	fn check_nonce(&self, message_session_nonce: u64) -> Result<(), Error> {
		match self.nonce == message_session_nonce {
			true => Ok(()),
			false => Err(Error::ReplayProtection),
		}
	}
}

impl ClusterSession for SessionImpl {
	fn is_finished(&self) -> bool {
		let data = self.data.lock();
		data.state == SessionState::Failed
			|| data.state == SessionState::Finished
	}

	fn on_node_timeout(&self, node: &NodeId) {
		let mut data = self.data.lock();

		warn!("{}: encryption session failed because {} connection has timeouted", self.node(), node);

		data.state = SessionState::Failed;
		data.result = Some(Err(Error::NodeDisconnected));
		self.completed.notify_all();
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		warn!("{}: encryption session failed with timeout", self.node());

		data.state = SessionState::Failed;
		data.result = Some(Err(Error::NodeDisconnected));
		self.completed.notify_all();
	}
}

impl Session for SessionImpl {
	fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	fn wait(&self, timeout: Option<time::Duration>) -> Result<(), Error> {
		let mut data = self.data.lock();
		if !data.result.is_some() {
			match timeout {
				None => self.completed.wait(&mut data),
				Some(timeout) => { self.completed.wait_for(&mut data, timeout); },
			}
		}

		data.result.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when result.is_some(); qed")
			.clone()
	}
}

impl Debug for SessionImpl {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		write!(f, "Encryption session {} on {}", self.id, self.self_node_id)
	}
}

fn check_encrypted_data(self_node_id: &Public, encrypted_data: &DocumentKeyShare) -> Result<(), Error> {
	use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

	// check that common_point and encrypted_point are still not set yet
	if encrypted_data.common_point.is_some() || encrypted_data.encrypted_point.is_some() {
		return Err(Error::CompletedSessionId);
	}

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)
}
