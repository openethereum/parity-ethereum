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
use std::collections::btree_map::Entry;
use std::net::SocketAddr;
use std::sync::Arc;
use ethkey::{Public, Signature};
use futures::lazy;
use parking_lot::Mutex;
use key_server_cluster::{KeyServerSet, KeyServerSetState, KeyServerSetStateType};
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData, BoxedEmptyFuture};
use key_server_cluster::cluster_sessions::{AdminSession, ClusterSessions, ClusterSession};
use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
use types::all::{Error, NodeId};
use {NodeKeyPair};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Describes which maintain() call is required.
pub enum Maintain {
	/// We need to maintain() both connections && session.
	SessionAndConnections,
	/// Only call maintain_session.
	Session,
	/// Only call maintain_connections.
	Connections,
}

/// Connection trigger, which executes necessary actions when set of key servers changes.
pub trait ConnectionTrigger: Send + Sync {
	/// On maintain interval.
	fn on_maintain(&mut self) -> Option<Maintain>;
	/// When connection is established.
	fn on_connection_established(&mut self, node: &NodeId) -> Option<Maintain>;
	/// When connection is closed.
	fn on_connection_closed(&mut self, node: &NodeId) -> Option<Maintain>;
	/// Maintain active sessions.
	fn maintain_session(&mut self, sessions: &ClusterClient);
	/// Maintain active connections.
	fn maintain_connections(&mut self, connections: &mut ClusterConnectionsData);
	/// Return connector for the servers set change session creator.
	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector>;
}

/// Servers set change session creator connector.
pub trait ServersSetChangeSessionCreatorConnector: Send + Sync {
	/// Get actual administrator public key. For manual-migration configuration it is the pre-configured
	/// administrator key. For auto-migration configurations it is the key of actual MigrationSession master node.
	fn admin_public(&self, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error>;
	/// Set active servers set change session.
	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>);
}

/// Simple connection trigger, which only keeps connections to current_set.
pub struct SimpleConnectionTrigger {
	/// Key server set cluster.
	key_server_set: Arc<KeyServerSet>,
	/// Trigger connections.
	connections: TriggerConnections,
	/// Servers set change session creator connector.
	connector: Arc<ServersSetChangeSessionCreatorConnector>,
}

/// Simple Servers set change session creator connector, which will just return
/// pre-configured administartor public when asked.
pub struct SimpleServersSetChangeSessionCreatorConnector {
	/// Secret store administrator public key.
	pub admin_public: Option<Public>,
}

#[derive(Debug, Clone, Copy)]
/// Action with trigger connections.
pub enum ConnectionsAction {
	/// Connect to nodes from old set only.
	ConnectToCurrentSet,
	/// Connect to nodes from both old and migration sets.
	ConnectToCurrentAndMigrationSet,
}

/// Trigger connections.
pub struct TriggerConnections {
	/// This node key pair.
	pub self_key_pair: Arc<NodeKeyPair>,
}

impl SimpleConnectionTrigger {
	/// Create new simple connection trigger.
	pub fn new(key_server_set: Arc<KeyServerSet>, self_key_pair: Arc<NodeKeyPair>, admin_public: Option<Public>) -> Self {
		SimpleConnectionTrigger {
			key_server_set: key_server_set,
			connections: TriggerConnections {
				self_key_pair: self_key_pair,
			},
			connector: Arc::new(SimpleServersSetChangeSessionCreatorConnector {
				admin_public: admin_public,
			}),
		}
	}
}

impl ConnectionTrigger for SimpleConnectionTrigger {
	fn on_maintain(&mut self) -> Option<Maintain> {
		Some(Maintain::Connections)
	}

	fn on_connection_established(&mut self, _node: &NodeId) -> Option<Maintain> {
		None
	}

	fn on_connection_closed(&mut self, node: &NodeId) -> Option<Maintain> {
		// we do not want to reconnect after every connection close
		// because it could be a part of something bigger
		None
	}

	fn maintain_session(&mut self, _sessions: &ClusterClient) {
	}

	fn maintain_connections(&mut self, connections: &mut ClusterConnectionsData) {
		self.connections.maintain(ConnectionsAction::ConnectToCurrentSet, connections, &self.key_server_set.state())
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.connector.clone()
	}
}

impl ServersSetChangeSessionCreatorConnector for SimpleServersSetChangeSessionCreatorConnector {
	fn admin_public(&self, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		self.admin_public.clone().ok_or(Error::AccessDenied)
	}

	fn set_key_servers_set_change_session(&self, _session: Arc<AdminSession>) {
	}
}

impl TriggerConnections {
	pub fn maintain(&mut self, action: ConnectionsAction, data: &mut ClusterConnectionsData, server_set: &KeyServerSetState) {
		match action {
			ConnectionsAction::ConnectToCurrentSet => {
				adjust_connections(self.self_key_pair.public(), data, &server_set.current_set);
			},
			ConnectionsAction::ConnectToCurrentAndMigrationSet => {
				let mut old_and_migration_set = BTreeMap::new();
				if let Some(migration) = server_set.migration.as_ref() {
					old_and_migration_set.extend(migration.set.iter().map(|(node_id, node_addr)| (node_id.clone(), node_addr.clone())));
				}
				old_and_migration_set.extend(server_set.current_set.iter().map(|(node_id, node_addr)| (node_id.clone(), node_addr.clone())));

				adjust_connections(self.self_key_pair.public(), data, &old_and_migration_set);
			},
		}
	}
}

fn adjust_connections(self_node_id: &NodeId, data: &mut ClusterConnectionsData, required_set: &BTreeMap<NodeId, SocketAddr>) {
	for node_to_disconnect in select_nodes_to_disconnect(&data.nodes, required_set) {
		if let Entry::Occupied(entry) = data.connections.entry(node_to_disconnect.clone()) {
			trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
				self_node_id, entry.get().node_id(), entry.get().node_address());
			entry.remove();
		}

		data.nodes.remove(&node_to_disconnect);
	}

	for (node_to_connect, node_addr) in required_set {
		if node_to_connect != self_node_id {
			data.nodes.insert(node_to_connect.clone(), node_addr.clone());
		}
	}
}

fn select_nodes_to_disconnect(current_set: &BTreeMap<NodeId, SocketAddr>, new_set: &BTreeMap<NodeId, SocketAddr>) -> Vec<NodeId> {
	current_set.iter()
		.filter(|&(node_id, node_addr)| match new_set.get(node_id) {
			Some(new_node_addr) => node_addr != new_node_addr,
			None => true,
		})
		.map(|(node_id, _)| node_id.clone())
		.collect()
}

/*fn select_nodes_to_connect(current_set: &BTreeMap<NodeId, SocketAddr>, new_set: &BTreeMap<NodeId, SocketAddr>) -> Vec<(NodeId, SocketAddr)> {
	new_set.iter()
		//.filter(|&(node_id, _)| !current_set.contains_key(node_id))
		.map(|(node_id, node_addr)| (node_id.clone(), node_addr.clone()))
		.collect()
}*/
