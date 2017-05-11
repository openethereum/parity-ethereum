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
use ethkey::{Public, Secret};
use util;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;

/// Signing session API.
pub trait Session: Send + Sync + 'static {
	/// Get generation session state.
	fn state(&self) -> SessionState;
	/// Wait until session is completed. Returns signed message.
	fn wait(&self, timeout: Option<time::Duration>) -> Result<util::Bytes, Error>;
}

/// Signing session.
pub struct SessionImpl {
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
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Key storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
}

#[derive(Debug)]
/// Mutable data of signing session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,
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

	// === Final states of the session ===
	/// Signing is completed.
	Finished,
	/// Signing is failed.
	Failed,
}

impl SessionImpl {
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
