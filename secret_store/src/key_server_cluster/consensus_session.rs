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
use std::collections::BTreeSet;
use parking_lot::Mutex;
use ethkey::{self, Public, Secret, Signature};
use util;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{ConsensusMessage, InitializeConsensusSession, ConfirmConsensusInitialization};

#[derive(Default, Debug, Clone)]
/// Consensus data. TODO: also move restart logic here (it is in signing + decryption)
pub struct Consensus {
	/// Nodes, which have been requested for signing initialization.
	pub requested_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with reject to initialization request.
	pub rejected_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with confirm to initialization request.
	pub confirmed_nodes: BTreeSet<NodeId>,
}

/// Signing session.
pub struct ConsensusSession {
	/// Key generation session id.
	id: SessionId,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Key generation data.
	encrypted_data: DocumentKeyShare,
	/// ACL storate to check access to the resource.
	acl_storage: Arc<AclStorage>,
	/// Mutable session data.
	data: Mutex<SessionData>,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// Key generation session id.
	pub id: SessionId,
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Encrypted data (result of running encryption_session::SessionImpl).
	pub encrypted_data: DocumentKeyShare,
	/// Key storage.
	pub acl_storage: Arc<AclStorage>,
}

#[derive(Debug)]
/// Mutable data of signing session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,
	/// Consensus data.
	consensus: Consensus,
	/// Consensus result.
	result: Option<Result<(), Error>>,
}

#[derive(Debug, Clone, PartialEq)]
/// Distributed key generation session state.
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Master node waits for other nodes to confirm session initialization.
	WaitingForInitializationConfirm,

	// === Final states of the session ===
	/// Signing is completed.
	Finished,
	/// Signing is failed.
	Failed,
}

#[derive(Debug, Clone)]
/// Session action.
pub enum SessionAction {
	/// Check session status.
	CheckStatus,
	/// Broadcast consensus message.
	BroadcastMessage(ConsensusMessage),
	/// Send consensus message.
	SendMessage(NodeId, ConsensusMessage),
}

impl ConsensusSession {
	/// Create new signing session.
	pub fn new(params: SessionParams) -> Result<Self, Error> {
		Ok(ConsensusSession {
			id: params.id,
			self_node_id: params.self_node_id,
			encrypted_data: params.encrypted_data,
			acl_storage: params.acl_storage,
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				consensus: Consensus::default(),
				result: None,
			})
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Get current session state.
	pub fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	/// Get result of consensus.
	pub fn consensus(&self) -> Option<Result<Consensus, Error>> {
		let data = self.data.lock();
		data.result.clone().map(|r| r.map(|_| data.consensus.clone()))
	}

	/// Initialize consensus session.
	pub fn initialize(&self, requestor_signature: Signature) -> Result<SessionAction, Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		data.state = SessionState::WaitingForInitializationConfirm;
		data.consensus.requested_nodes.extend(self.encrypted_data.id_numbers.keys().cloned());

		// ..and finally check access on our's own
		let is_permitted = self.acl_storage.check(&requestor_public, &self.id).unwrap_or(false);
		process_initialization_response(&self.encrypted_data, &mut *data, self.node(), is_permitted)?;

		// check if we have enough nodes to sign message
		match data.state {
			// not enough nodes => pass initialization message to all other nodes
			SessionState::WaitingForInitializationConfirm =>
				Ok(SessionAction::BroadcastMessage(ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
					requestor_signature: requestor_signature.clone().into(),
				}))),
			// else state must be checked
			_ => Ok(SessionAction::CheckStatus),
		}
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&self, sender: NodeId, message: &InitializeConsensusSession) -> Result<SessionAction, Error> {
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&message.requestor_signature, &self.id)?;

		// check access
		let is_permitted = self.acl_storage.check(&requestor_public, &self.id).unwrap_or(false);
		data.state = if is_permitted { SessionState::Finished } else { SessionState::Failed };

		// respond to sender
		Ok(SessionAction::SendMessage(sender, ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: is_permitted,
		})))
	}

	/// When session initialization confirmation message is reeived.
	pub fn on_confirm_initialization(&self, sender: NodeId, message: &ConfirmConsensusInitialization) -> Result<SessionAction, Error> {
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitializationConfirm {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		process_initialization_response(&self.encrypted_data, &mut *data, &sender, message.is_confirmed)?;

		// check if we have enough nodes for consensus
		match data.state {
			// we do not yet have enough nodes for consensus
			SessionState::WaitingForInitializationConfirm => Ok(SessionAction::CheckStatus),
			// else state must be checked
			_ => Ok(SessionAction::CheckStatus),
		}
	}
}

fn process_initialization_response(encrypted_data: &DocumentKeyShare, data: &mut SessionData, node: &NodeId, is_permitted: bool) -> Result<(), Error> {
	if !data.consensus.requested_nodes.remove(node) {
		return Err(Error::InvalidMessage);
	}

	match is_permitted {
		true => {
			data.consensus.confirmed_nodes.insert(node.clone());

			// check if we have enough nodes for consensus?
			if data.consensus.confirmed_nodes.len() == encrypted_data.threshold + 1 {
				data.result = Some(Ok(()));
				data.state = SessionState::Finished;
			}
		},
		false => {
			data.consensus.rejected_nodes.insert(node.clone());

			// check if we still can receive enough confirmations for consensus?
			if encrypted_data.id_numbers.len() - data.consensus.rejected_nodes.len() < encrypted_data.threshold + 1 {
				data.result = Some(Err(Error::AccessDenied));
				data.state = SessionState::Failed;
			}
		},
	}

	Ok(())
}
