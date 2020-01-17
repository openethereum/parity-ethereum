// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter, Error as FmtError};
use std::sync::Arc;
use futures::Oneshot;
use parking_lot::Mutex;
use ethereum_types::Address;
use crypto::publickey::Public;
use key_server_cluster::{Error, NodeId, SessionId, Requester, KeyStorage,
	DocumentKeyShare, ServerKeyId};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::{ClusterSession, CompletionSignal};
use key_server_cluster::message::{Message, EncryptionMessage, InitializeEncryptionSession,
	ConfirmEncryptionInitialization, EncryptionSessionError};

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
	encrypted_data: Option<DocumentKeyShare>,
	/// Key storage.
	key_storage: Arc<dyn KeyStorage>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	cluster: Arc<dyn Cluster>,
	/// Session nonce.
	nonce: u64,
	/// Session completion signal.
	completed: CompletionSignal<()>,
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
	pub encrypted_data: Option<DocumentKeyShare>,
	/// Key storage.
	pub key_storage: Arc<dyn KeyStorage>,
	/// Cluster
	pub cluster: Arc<dyn Cluster>,
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
	pub fn new(params: SessionParams) -> Result<(Self, Oneshot<Result<(), Error>>), Error> {
		check_encrypted_data(params.encrypted_data.as_ref())?;

		let (completed, oneshot) = CompletionSignal::new();
		Ok((SessionImpl {
			id: params.id,
			self_node_id: params.self_node_id,
			encrypted_data: params.encrypted_data,
			key_storage: params.key_storage,
			cluster: params.cluster,
			nonce: params.nonce,
			completed,
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				nodes: BTreeMap::new(),
				result: None,
			}),
		}, oneshot))
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Start new session initialization. This must be called on master node.
	pub fn initialize(&self, requester: Requester, common_point: Public, encrypted_point: Public) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		data.state = SessionState::WaitingForInitializationConfirm;
		data.nodes.extend(self.cluster.nodes().into_iter().map(|n| (n, NodeData {
			initialization_confirmed: &n == self.node(),
		})));

		// TODO [Sec]: id signature is not enough here, as it was already used in key generation
		// TODO [Reliability]: there could be situation when some nodes have failed to store encrypted data
		// => potential problems during restore. some confirmation step is needed (2pc)?
		// save encryption data
		if let Some(encrypted_data) = self.encrypted_data.clone() {
			let requester_address = requester.address(&self.id).map_err(Error::InsufficientRequesterData)?;
			update_encrypted_data(&self.key_storage, self.id.clone(),
				encrypted_data, requester_address, common_point.clone(), encrypted_point.clone())?;
		}

		// start initialization
		if data.nodes.len() > 1 {
			self.cluster.broadcast(Message::Encryption(EncryptionMessage::InitializeEncryptionSession(InitializeEncryptionSession {
				session: self.id.clone().into(),
				session_nonce: self.nonce,
				requester: requester.into(),
				common_point: common_point.into(),
				encrypted_point: encrypted_point.into(),
			})))
		} else {
			data.state = SessionState::Finished;
			data.result = Some(Ok(()));
			self.completed.send(Ok(()));

			Ok(())
		}
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&self, sender: NodeId, message: &InitializeEncryptionSession) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// check that the requester is the author of the encrypted data
		if let Some(encrypted_data) = self.encrypted_data.clone() {
			let requester: Requester = message.requester.clone().into();
			let requester_address = requester.address(&self.id).map_err(Error::InsufficientRequesterData)?;
			update_encrypted_data(&self.key_storage, self.id.clone(),
				encrypted_data, requester_address, message.common_point.clone().into(), message.encrypted_point.clone().into())?;
		}

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
		self.completed.send(Ok(()));

		Ok(())
	}
}

impl ClusterSession for SessionImpl {
	type Id = SessionId;
	type CreationData = ();
	type SuccessfulResult = ();

	fn type_name() -> &'static str {
		"encryption"
	}

	fn id(&self) -> SessionId {
		self.id.clone()
	}

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
		self.completed.send(Err(Error::NodeDisconnected));
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		warn!("{}: encryption session failed with timeout", self.node());

		data.state = SessionState::Failed;
		data.result = Some(Err(Error::NodeDisconnected));
		self.completed.send(Err(Error::NodeDisconnected));
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		// error in encryption session is considered fatal
		// => broadcast error if error occured on this node
		if *node == self.self_node_id {
			// do not bother processing send error, as we already processing error
			let _ = self.cluster.broadcast(Message::Encryption(EncryptionMessage::EncryptionSessionError(EncryptionSessionError {
				session: self.id.clone().into(),
				session_nonce: self.nonce,
				error: error.clone().into(),
			})));
		}

		let mut data = self.data.lock();

		warn!("{}: encryption session failed with error: {} from {}", self.node(), error, node);

		data.state = SessionState::Failed;
		data.result = Some(Err(error.clone()));
		self.completed.send(Err(error));
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		if Some(self.nonce) != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&Message::Encryption(ref message) => match message {
				&EncryptionMessage::InitializeEncryptionSession(ref message) =>
					self.on_initialize_session(sender.clone(), message),
				&EncryptionMessage::ConfirmEncryptionInitialization(ref message) =>
					self.on_confirm_initialization(sender.clone(), message),
				&EncryptionMessage::EncryptionSessionError(ref message) => {
					self.on_session_error(sender, message.error.clone());
					Ok(())
				},
			},
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl Debug for SessionImpl {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		write!(f, "Encryption session {} on {}", self.id, self.self_node_id)
	}
}

/// Check that common_point and encrypted point are not yet set in key share.
pub fn check_encrypted_data(key_share: Option<&DocumentKeyShare>) -> Result<(), Error> {
	if let Some(key_share) = key_share {
		// check that common_point and encrypted_point are still not set yet
		if key_share.common_point.is_some() || key_share.encrypted_point.is_some() {
			return Err(Error::DocumentKeyAlreadyStored);
		}
	}

	Ok(())
}

/// Update key share with encrypted document key.
pub fn update_encrypted_data(key_storage: &Arc<dyn KeyStorage>, key_id: ServerKeyId, mut key_share: DocumentKeyShare, author: Address, common_point: Public, encrypted_point: Public) -> Result<(), Error> {
	// author must be the same
	if key_share.author != author {
		return Err(Error::AccessDenied);
	}

	// save encryption data
	key_share.common_point = Some(common_point);
	key_share.encrypted_point = Some(encrypted_point);
	key_storage.update(key_id, key_share)
}
