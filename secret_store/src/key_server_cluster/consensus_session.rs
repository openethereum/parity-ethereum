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
use std::fmt::Debug;
use std::collections::BTreeSet;
use parking_lot::Mutex;
use ethkey::{self, Public, Secret, Signature};
use util;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::consensus::Consensus;
use key_server_cluster::message::{ConsensusMessage, InitializeConsensusSession, ConfirmConsensusInitialization};

/// Consenus checker.
pub trait ConsensusChecker {
	/// Check if we want to accept offer to join consensus group. Consensus is about revealing key `key` to requestor `requestor`.
	fn check_offer(&self, key: &SessionId, requestor: &Public) -> bool;
}

/// Consensus establishing session.
pub struct ConsensusSession<C: ConsensusChecker> {
	/// Key generation session id.
	id: SessionId,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Master node id.
	master_node_id: Public,
	/// Consensus checker.
	consensus_checker: C,
	/// Mutable session data.
	data: SessionData,
}

/// ACL checker for consensus establishing session.
pub struct AclConsensusChecker {
	/// ACL storate to check access to the resource.
	acl_storage: Arc<AclStorage>,
}

/// Always accept checker for consensus establishing session.
pub struct TrueConsensusChecker;

/// SessionImpl creation parameters
pub struct SessionParams<C: ConsensusChecker> {
	/// Key generation session id.
	pub id: SessionId,
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Master node id.
	pub master_node_id: Public,
	/// Consensus checker.
	pub consensus_checker: C,
}

#[derive(Debug)]
/// Mutable data of signing session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,
	/// Consensus establishing result.
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
	/// Consensus group is established.
	Finished,
	/// Consensus establish has failed.
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

impl<C> ConsensusSession<C> where C: ConsensusChecker {
	/// Create new signing session.
	pub fn new(params: SessionParams<C>) -> Result<Self, Error> {
		Ok(ConsensusSession {
			id: params.id,
			self_node_id: params.self_node_id,
			master_node_id: params.master_node_id,
			consensus_checker: params.consensus_checker,
			data: SessionData {
				state: SessionState::WaitingForInitialization,
				result: None,
			}
		})
	}

	/// Get current session state.
	pub fn state(&self) -> SessionState {
		self.data.state.clone()
	}

	/// Initialize consensus session.
	pub fn initialize<T: Debug>(&mut self, requestor_signature: Signature, consensus: &mut Consensus<T>) -> Result<SessionAction, Error> {
		debug_assert_eq!(self.self_node_id, self.master_node_id);

		// check state
		if self.data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor public
		let requestor = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		self.data.state = SessionState::WaitingForInitializationConfirm;

		// ..and finally check access on our's own
		let self_node_id = self.self_node_id.clone();
		let is_confirmed = self.consensus_checker.check_offer(&self.id, &requestor);
		self.process_initialization_response(&self_node_id, is_confirmed, consensus)?;
		if self.data.state != SessionState::Finished {
			Ok(SessionAction::BroadcastMessage(ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
				requestor_signature: requestor_signature.into(),
			})))
		} else {
			Ok(SessionAction::CheckStatus)
		}
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&mut self, sender: NodeId, requestor: &Public) -> Result<SessionAction, Error> {
		debug_assert!(sender != self.self_node_id);

		// check message
		if self.master_node_id != sender {
			return Err(Error::InvalidMessage);
		}
		// check state
		if self.data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// check access
		let is_confirmed = self.consensus_checker.check_offer(&self.id, &requestor);

		// update state
		self.data.state = if is_confirmed { SessionState::Finished } else { SessionState::Failed };

		// respond to sender
		Ok(SessionAction::SendMessage(sender, ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: is_confirmed,
		})))
	}

	/// When session initialization confirmation message is reeived.
	pub fn on_confirm_initialization<T: Debug>(&mut self, sender: NodeId, is_confirmed: bool, consensus: &mut Consensus<T>) -> Result<SessionAction, Error> {
		debug_assert!(sender != self.self_node_id);

		// check state
		if self.self_node_id != self.master_node_id {
			return Err(Error::InvalidMessage);
		}
		if self.data.state != SessionState::WaitingForInitializationConfirm && self.data.state != SessionState::Finished {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		self.process_initialization_response(&sender, is_confirmed, consensus)
	}

	/// Process initialization response from given node.
	fn process_initialization_response<T: Debug>(&mut self, node: &NodeId, is_confirmed: bool, consensus: &mut Consensus<T>) -> Result<SessionAction, Error> {
		match consensus.offer_response(node, is_confirmed) {
			Ok(_) if consensus.is_established() => {
				self.data.result = Some(Ok(()));
				self.data.state = SessionState::Finished;
				Ok(SessionAction::CheckStatus)
			}
			Ok(_) => Ok(SessionAction::CheckStatus),
			Err(err) => {
				self.data.result = Some(Err(err.clone()));
				self.data.state = SessionState::Failed;
				Err(err)
			},
		}
	}
}

impl AclConsensusChecker {
	/// Create new ACL-consensus checker.
	pub fn new(acl_storage: Arc<AclStorage>) -> Self {
		AclConsensusChecker {
			acl_storage: acl_storage,
		}
	}
}

impl ConsensusChecker for AclConsensusChecker {
	fn check_offer(&self, key: &SessionId, requestor: &Public) -> bool {
		self.acl_storage.check(requestor, key).unwrap_or(false)
	}
}

impl ConsensusChecker for TrueConsensusChecker {
	fn check_offer(&self, _key: &SessionId, _requestor: &Public) -> bool {
		true
	}
}
