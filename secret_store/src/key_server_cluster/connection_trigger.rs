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

use std::collections::{BTreeSet, BTreeMap};
use std::collections::btree_map::Entry;
use std::net::SocketAddr;
use std::sync::Arc;
use ethereum_types::H256;
use ethkey::Public;
use key_server_cluster::{KeyServerSet, KeyServerSetSnapshot};
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData};
use key_server_cluster::cluster_sessions::AdminSession;
use types::{Error, NodeId};
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
	fn admin_public(&self, migration_id: Option<&H256>, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error>;
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

#[derive(Debug, Clone, Copy, PartialEq)]
/// Action with trigger connections.
pub enum ConnectionsAction {
	/// Connect to nodes from old set only.
	ConnectToCurrentSet,
	/// Connect to nodes from migration set.
	ConnectToMigrationSet,
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

	fn on_connection_closed(&mut self, _node: &NodeId) -> Option<Maintain> {
		// we do not want to reconnect after every connection close
		// because it could be a part of something bigger
		None
	}

	fn maintain_session(&mut self, _sessions: &ClusterClient) {
	}

	fn maintain_connections(&mut self, connections: &mut ClusterConnectionsData) {
		self.connections.maintain(ConnectionsAction::ConnectToCurrentSet, connections, &self.key_server_set.snapshot())
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.connector.clone()
	}
}

impl ServersSetChangeSessionCreatorConnector for SimpleServersSetChangeSessionCreatorConnector {
	fn admin_public(&self, _migration_id: Option<&H256>, _new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		self.admin_public.clone().ok_or(Error::AccessDenied)
	}

	fn set_key_servers_set_change_session(&self, _session: Arc<AdminSession>) {
	}
}

impl TriggerConnections {
	pub fn maintain(&self, action: ConnectionsAction, data: &mut ClusterConnectionsData, server_set: &KeyServerSetSnapshot) {
		match action {
			ConnectionsAction::ConnectToCurrentSet => {
				adjust_connections(self.self_key_pair.public(), data, &server_set.current_set);
			},
			ConnectionsAction::ConnectToMigrationSet => {
				let migration_set = server_set.migration.as_ref().map(|s| s.set.clone()).unwrap_or_default();
				adjust_connections(self.self_key_pair.public(), data, &migration_set);
			},
		}
	}
}

fn adjust_connections(self_node_id: &NodeId, data: &mut ClusterConnectionsData, required_set: &BTreeMap<NodeId, SocketAddr>) {
	if !required_set.contains_key(self_node_id) {
		trace!(target: "secretstore_net", "{}: isolated from cluser", self_node_id);
		data.connections.clear();
		data.nodes.clear();
		return;
	}

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

#[cfg(test)]
mod tests {
	use std::collections::BTreeSet;
	use std::sync::Arc;
	use ethkey::{Random, Generator};
	use key_server_cluster::cluster::ClusterConnectionsData;
	use key_server_cluster::{MapKeyServerSet, PlainNodeKeyPair, KeyServerSetSnapshot, KeyServerSetMigration};
	use super::{Maintain, TriggerConnections, ConnectionsAction, ConnectionTrigger, SimpleConnectionTrigger,
		select_nodes_to_disconnect, adjust_connections};

	fn create_connections() -> TriggerConnections {
		TriggerConnections {
			self_key_pair: Arc::new(PlainNodeKeyPair::new(Random.generate().unwrap())),
		}
	}

	#[test]
	fn do_not_disconnect_if_set_is_not_changed() {
		let node_id = Random.generate().unwrap().public().clone();
		assert_eq!(select_nodes_to_disconnect(
			&vec![(node_id, "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			&vec![(node_id, "127.0.0.1:8081".parse().unwrap())].into_iter().collect()),
			vec![]);
	}

	#[test]
	fn disconnect_if_address_has_changed() {
		let node_id = Random.generate().unwrap().public().clone();
		assert_eq!(select_nodes_to_disconnect(
			&vec![(node_id.clone(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			&vec![(node_id.clone(), "127.0.0.1:8082".parse().unwrap())].into_iter().collect()),
			vec![node_id.clone()]);
	}

	#[test]
	fn disconnect_if_node_has_removed() {
		let node_id = Random.generate().unwrap().public().clone();
		assert_eq!(select_nodes_to_disconnect(
			&vec![(node_id.clone(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			&vec![].into_iter().collect()),
			vec![node_id.clone()]);
	}

	#[test]
	fn does_not_disconnect_if_node_has_added() {
		let node_id = Random.generate().unwrap().public().clone();
		assert_eq!(select_nodes_to_disconnect(
			&vec![(node_id.clone(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			&vec![(node_id.clone(), "127.0.0.1:8081".parse().unwrap()),
				(Random.generate().unwrap().public().clone(), "127.0.0.1:8082".parse().unwrap())]
				.into_iter().collect()),
			vec![]);
	}

	#[test]
	fn adjust_connections_disconnects_from_all_nodes_if_not_a_part_of_key_server() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let other_node_id = Random.generate().unwrap().public().clone();
		let mut connection_data: ClusterConnectionsData = Default::default();
		connection_data.nodes.insert(other_node_id.clone(), "127.0.0.1:8081".parse().unwrap());

		let required_set = connection_data.nodes.clone();
		adjust_connections(&self_node_id, &mut connection_data, &required_set);
		assert!(connection_data.nodes.is_empty());
	}

	#[test]
	fn adjust_connections_connects_to_new_nodes() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let other_node_id = Random.generate().unwrap().public().clone();
		let mut connection_data: ClusterConnectionsData = Default::default();

		let required_set = vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap()),
			(other_node_id.clone(), "127.0.0.1:8082".parse().unwrap())].into_iter().collect();
		adjust_connections(&self_node_id, &mut connection_data, &required_set);
		assert!(connection_data.nodes.contains_key(&other_node_id));
	}

	#[test]
	fn adjust_connections_reconnects_from_changed_nodes() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let other_node_id = Random.generate().unwrap().public().clone();
		let mut connection_data: ClusterConnectionsData = Default::default();
		connection_data.nodes.insert(other_node_id.clone(), "127.0.0.1:8082".parse().unwrap());

		let required_set = vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap()),
			(other_node_id.clone(), "127.0.0.1:8083".parse().unwrap())].into_iter().collect();
		adjust_connections(&self_node_id, &mut connection_data, &required_set);
		assert_eq!(connection_data.nodes.get(&other_node_id), Some(&"127.0.0.1:8083".parse().unwrap()));
	}

	#[test]
	fn adjust_connections_disconnects_from_removed_nodes() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let other_node_id = Random.generate().unwrap().public().clone();
		let mut connection_data: ClusterConnectionsData = Default::default();
		connection_data.nodes.insert(other_node_id.clone(), "127.0.0.1:8082".parse().unwrap());

		let required_set = vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect();
		adjust_connections(&self_node_id, &mut connection_data, &required_set);
		assert!(connection_data.nodes.is_empty());
	}

	#[test]
	fn adjust_connections_does_not_connects_to_self() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let mut connection_data: ClusterConnectionsData = Default::default();

		let required_set = vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect();
		adjust_connections(&self_node_id, &mut connection_data, &required_set);
		assert!(connection_data.nodes.is_empty());
	}

	#[test]
	fn maintain_connects_to_current_set_works() {
		let connections = create_connections();
		let self_node_id = connections.self_key_pair.public().clone();
		let current_node_id = Random.generate().unwrap().public().clone();
		let migration_node_id = Random.generate().unwrap().public().clone();
		let new_node_id = Random.generate().unwrap().public().clone();

		let mut connections_data: ClusterConnectionsData = Default::default();
		connections.maintain(ConnectionsAction::ConnectToCurrentSet, &mut connections_data, &KeyServerSetSnapshot {
			current_set: vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap()),
				(current_node_id.clone(), "127.0.0.1:8082".parse().unwrap())].into_iter().collect(),
			new_set: vec![(new_node_id.clone(), "127.0.0.1:8083".parse().unwrap())].into_iter().collect(),
			migration: Some(KeyServerSetMigration {
				set: vec![(migration_node_id.clone(), "127.0.0.1:8084".parse().unwrap())].into_iter().collect(),
				..Default::default()
			}),
		});

		assert_eq!(vec![current_node_id], connections_data.nodes.keys().cloned().collect::<Vec<_>>());
	}

	#[test]
	fn maintain_connects_to_migration_set_works() {
		let connections = create_connections();
		let self_node_id = connections.self_key_pair.public().clone();
		let current_node_id = Random.generate().unwrap().public().clone();
		let migration_node_id = Random.generate().unwrap().public().clone();
		let new_node_id = Random.generate().unwrap().public().clone();

		let mut connections_data: ClusterConnectionsData = Default::default();
		connections.maintain(ConnectionsAction::ConnectToMigrationSet, &mut connections_data, &KeyServerSetSnapshot {
			current_set: vec![(current_node_id.clone(), "127.0.0.1:8082".parse().unwrap())].into_iter().collect(),
			new_set: vec![(new_node_id.clone(), "127.0.0.1:8083".parse().unwrap())].into_iter().collect(),
			migration: Some(KeyServerSetMigration {
				set: vec![(self_node_id.clone(), "127.0.0.1:8081".parse().unwrap()),
					(migration_node_id.clone(), "127.0.0.1:8084".parse().unwrap())].into_iter().collect(),
				..Default::default()
			}),
		});

		assert_eq!(vec![migration_node_id].into_iter().collect::<BTreeSet<_>>(),
			connections_data.nodes.keys().cloned().collect::<BTreeSet<_>>());
	}

	#[test]
	fn simple_connections_trigger_only_maintains_connections() {
		let key_server_set = Arc::new(MapKeyServerSet::new(Default::default()));
		let self_key_pair = Arc::new(PlainNodeKeyPair::new(Random.generate().unwrap()));
		let mut trigger = SimpleConnectionTrigger::new(key_server_set, self_key_pair, None);
		assert_eq!(trigger.on_maintain(), Some(Maintain::Connections));
	}
}
