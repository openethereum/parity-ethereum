// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::collections::BTreeSet;
use std::sync::Arc;
use key_server_cluster::{Error, NodeId};
use key_server_cluster::message::Message;

/// Connection to the single node. Provides basic information about connected node and
/// allows sending messages to this node.
pub trait Connection: Send + Sync {
	/// Is this inbound connection? This only matters when both nodes are simultaneously establishing
	/// two connections to each other. The agreement is that the inbound connection from the node with
	/// lower NodeId is used and the other connection is closed.
	fn is_inbound(&self) -> bool;
	/// Returns id of the connected node.
	fn node_id(&self) -> &NodeId;
	/// Returns 'address' of the node to use in traces.
	fn node_address(&self) -> String;
	/// Send message to the connected node.
	fn send_message(&self, message: Message);
}

/// Connections manager. Responsible for keeping us connected to all required nodes.
pub trait ConnectionManager: 'static + Send + Sync {
	/// Returns shared reference to connections provider.
	fn provider(&self) -> Arc<ConnectionProvider>;
	/// Try to reach all disconnected nodes immediately. This method is exposed mostly for
	/// tests, where all 'nodes' are starting listening for incoming connections first and
	/// only after this, they're actually start connecting to each other.
	fn connect(&self);
}

/// Connections provider. Holds all active connections and the set of nodes that we need to
/// connect to. At any moment connection could be lost and the set of connected/disconnected
/// nodes could change (at behalf of the connection manager).
/// Clone operation should be cheap (Arc).
pub trait ConnectionProvider: Send + Sync {
	/// Returns the set of currently connected nodes. Error is returned when our node is
	/// not a part of the cluster ('isolated' node).
	fn connected_nodes(&self) -> Result<BTreeSet<NodeId>, Error>;
	/// Returns the set of currently disconnected nodes.
	fn disconnected_nodes(&self) -> BTreeSet<NodeId>;
	/// Returns the reference to the active node connection or None if the node is not connected.
	fn connection(&self, node: &NodeId) -> Option<Arc<Connection>>;
}

#[cfg(test)]
pub mod tests {
	use std::collections::{BTreeSet, VecDeque};
	use std::sync::Arc;
	use std::sync::atomic::{AtomicBool, Ordering};
	use parking_lot::Mutex;
	use key_server_cluster::{Error, NodeId};
	use key_server_cluster::message::Message;
	use super::{ConnectionManager, Connection, ConnectionProvider};

	/// Shared messages queue.
	pub type MessagesQueue = Arc<Mutex<VecDeque<(NodeId, NodeId, Message)>>>;

	/// Single node connections.
	pub struct TestConnections {
		node: NodeId,
		is_isolated: AtomicBool,
		connected_nodes: Mutex<BTreeSet<NodeId>>,
		disconnected_nodes: Mutex<BTreeSet<NodeId>>,
		messages: MessagesQueue,
	}

	/// Single connection.
	pub struct TestConnection {
		from: NodeId,
		to: NodeId,
		messages: MessagesQueue,
	}

	impl TestConnections {
		pub fn isolate(&self) {
			let connected_nodes = ::std::mem::replace(&mut *self.connected_nodes.lock(), Default::default());
			self.is_isolated.store(true, Ordering::Relaxed);
			self.disconnected_nodes.lock().extend(connected_nodes)
		}

		pub fn disconnect(&self, node: NodeId) {
			self.connected_nodes.lock().remove(&node);
			self.disconnected_nodes.lock().insert(node);
		}

		pub fn exclude(&self, node: NodeId) {
			self.connected_nodes.lock().remove(&node);
			self.disconnected_nodes.lock().remove(&node);
		}

		pub fn include(&self, node: NodeId) {
			self.connected_nodes.lock().insert(node);
		}
	}

	impl ConnectionManager for Arc<TestConnections> {
		fn provider(&self) -> Arc<ConnectionProvider> {
			self.clone()
		}

		fn connect(&self) {}
	}

	impl ConnectionProvider for TestConnections {
		fn connected_nodes(&self) -> Result<BTreeSet<NodeId>, Error> {
			match self.is_isolated.load(Ordering::Relaxed) {
				false => Ok(self.connected_nodes.lock().clone()),
				true => Err(Error::NodeDisconnected),
			}
		}

		fn disconnected_nodes(&self) -> BTreeSet<NodeId> {
			self.disconnected_nodes.lock().clone()
		}

		fn connection(&self, node: &NodeId) -> Option<Arc<Connection>> {
			match self.connected_nodes.lock().contains(node) {
				true => Some(Arc::new(TestConnection {
					from: self.node,
					to: *node,
					messages: self.messages.clone(),
				})),
				false => None,
			}
		}
	}

	impl Connection for TestConnection {
		fn is_inbound(&self) -> bool {
			false
		}

		fn node_id(&self) -> &NodeId {
			&self.to
		}

		fn node_address(&self) -> String {
			format!("{}", self.to)
		}

		fn send_message(&self, message: Message) {
			self.messages.lock().push_back((self.from, self.to, message))
		}
	}

	pub fn new_test_connections(
		messages: MessagesQueue,
		node: NodeId,
		mut nodes: BTreeSet<NodeId>
	) -> Arc<TestConnections> {
		let is_isolated = !nodes.remove(&node);
		Arc::new(TestConnections {
			node,
			is_isolated: AtomicBool::new(is_isolated),
			connected_nodes: Mutex::new(nodes),
			disconnected_nodes: Default::default(),
			messages,
		})
	}
}
