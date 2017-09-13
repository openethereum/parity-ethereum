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

use std::collections::{BTreeSet, BTreeMap};
use key_server_cluster::{Error, NodeId, SessionId};

/// Servers set change session API.
pub trait Session: Send + Sync + 'static {
}

/// Servers set change session.
/// Brief overview:
/// 1) initialization: master node (which has received request for change) requests ALL other nodes to make change
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor can make changes
/// 3) master node splits all nodes from OLD set and from NEW set as follows:
/// 3.1) nodes_to_remove: OLD.except(NEW)
/// 3.2) nodes_to_add: NEW.except(OLD)
/// 3.3) nodes_staying: OLD.intersect(NEW)
/// 4) for each node1 in nodes_to_remove: select node2 from nodes_to_add and move all sessions from node1 to node2
/// 5) for each node1 left in nodes_to_remove (if nodes_to_add.len() < nodes_to_remove.len()): run share removal protocol for each node1 session
/// 6) for each node1 left in nodes_to_add (if nodes_to_add.len() > nodes_to_remove.len()): run share addition protocol for each node1 session
pub struct SessionImpl {

}

/// Immutable session data.
struct SessionCore {
	/// Nodes to remove from the set.
	pub nodes_to_remove: BTreeSet<NodeId>,
	/// Nodes to add to the set.
	pub nodes_to_add: BTreeSet<NodeId>,
	/// Nodes staying in the set.
	pub nodes_to_move: BTreeMap<NodeId, NodeId>,
}

/// Mutable session data.
struct SessionData {
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

/// Mutable node-specific data.
struct NodeData {
}

/// Single meta-session: (1) shares are added (2) shares are moved (3) shares are removed.
enum MetaSession {
	/// Add shares.
	AddShares(BTreeSet<NodeId>, BTreeMap<NodeId, NodeId>, BTreeSet<NodeId>),
	/// Move shares.
	MoveShares(BTreeMap<NodeId, NodeId>, BTreeSet<NodeId>),
	/// Remove shares.
	RemoveShares(BTreeSet<NodeId>),
}

impl SessionImpl {
	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId) -> Result<(), Error> {
		unimplemented!()
		// at this moment we only know sessions, involving master node
		// => start reading sessions from db and act accordingly
		/*for (key_id, key_data) in &self.key_storage {
			let mut nodes_to_add = BTreeSet::new();
			let mut nodes_to_remove = BTreeSet::new();
			let mut nodes_to_move = BTreeMap::new();

			for key_node in key_data.id_numbers.keys() {
				if self.core.nodes_to_remove.contains(key_node) {
					nodes_to_remove.insert(key_node.clone());
				} else if self.core.nodes_to_add.contains(key_node) {
					nodes_to_add.insert(key_node.clone());
				} else if self.core.nodes_to_move.contains(key_node) {
					nodes_to_move.insert(key_node.clone());
				}
			}

			data.movement_ss
		}*/
	}
}
