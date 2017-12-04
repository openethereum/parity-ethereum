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
use bigint::hash::H256;
use ethkey::Public;
use parking_lot::Mutex;
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData};
use key_server_cluster::cluster_sessions::{AdminSession, ClusterSessions, ClusterSession};
use types::all::{Error, NodeId};

/// Servers set change session creator connector.
pub trait ServersSetChangeSessionCreatorConnector: Send + Sync {
	/// Get actual administrator public key. For manual-migration configuration it is the pre-configured
	/// administrator key. For auto-migration configurations it is the key of actual MigrationSession master node.
	fn admin_public(&self, block: Option<H256>, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error>;
	/// Set active servers set change session.
	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>);
}

/// Connection trigger, which executes necessary actions when set of key servers changes.
pub trait ConnectionTrigger: Send + Sync {
	/// When key server set is about to change.
	fn on_servers_set_change(&mut self, connections: &mut ClusterConnectionsData, sessions: &ClusterSessions, block: &H256, change: KeyServerSetChange);
	/// Return connector for the servers set change session creator.
	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector>;
	/// When connection is established.
	fn on_connection_established(&mut self, _node: &NodeId) {}
	/// When connection is closed.
	fn on_connection_closed(&mut self, _node: &NodeId) {}
	/// When connections maintain happens.
	fn maintain(&mut self, _client: &ClusterClient, _data: &mut ClusterConnectionsData) {}
}

#[derive(Debug)]
/// Simple trigger will just disconnect from nodes that are being removed from Secret Store
/// and connect to nodes being added.
pub struct SimpleConnectionTrigger {
	/// This node id.
	self_node_id: NodeId,
	/// Servers set change session creator connector.
	connector: Arc<SimpleServersSetChangeSessionCreatorConnector>,
}

#[derive(Debug)]
/// Simple servers set change session creator connector.
pub struct SimpleServersSetChangeSessionCreatorConnector {
	/// Administator public key.
	pub admin_public: Option<Public>,
}

/// Key servers set change trigger with automated migration procedure.
pub struct ConnectionTriggerWithMigration {
	/// This node id.
	self_node_id: NodeId,
	/// Servers set change session creator connector.
	connector: Arc<ServersSetChangeSessionCreatorConnectorWithMigration>,
	/// Scheduled migration task.
	task: Option<MigrationTask>,
	/// Active meta-session
	session: Option<MigrationSession>,
}

/// Key servers set change session creator connector with migration support.
pub struct ServersSetChangeSessionCreatorConnectorWithMigration {
	/// Active migration task to check when servers set change session is started.
	task: Mutex<Option<MigrationTask>>,
	/// Active servers set change session.
	change_session: Mutex<Option<Arc<AdminSession>>>,
}

#[derive(Debug, Default, Clone)]
/// Key server set change.
pub struct KeyServerSetChange {
	/// Key servers that are being added to the Secret Store.
	pub added_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Key servers that are being removed from the Secret Store.
	pub removed_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Key servers, that have changed their addresses.
	pub changed_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Final key servers set.
	pub nodes: BTreeSet<NodeId>,
}

#[derive(Debug, Default, Clone)]
/// Single migration task
struct MigrationTask {
	/// Block for which this new servers set is actual.
	pub block: H256,
	/// New servers set.
	pub change: KeyServerSetChange,
}

/// Migration session.
struct MigrationSession {
	/// This node id.
	pub self_node_id: NodeId,
	/// Session task.
	pub task: MigrationTask,
	/// Nodes that we need to connected to before starting migration.
	pub nodes_to_connect: BTreeSet<NodeId>,
}

impl SimpleConnectionTrigger {
	/// Create new simple connection trigger.
	pub fn new(self_node_id: NodeId, admin_public: Option<Public>) -> Self {
		SimpleConnectionTrigger {
			self_node_id: self_node_id,
			connector: Arc::new(SimpleServersSetChangeSessionCreatorConnector {
				admin_public: admin_public,
			}),
		}
	}

	/// Disconnect from nodes, removed from the SS.
	pub fn disconnect_removed_nodes(self_node_id: &NodeId, data: &mut ClusterConnectionsData, change: &KeyServerSetChange) {
		for removed_node in change.removed_nodes.keys() {
			if let Entry::Occupied(entry) = data.connections.entry(removed_node.clone()) {
				trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
					self_node_id, entry.get().node_id(), entry.get().node_address());
				entry.remove();
			}

			data.nodes.remove(&removed_node);
		}
	}

	/// Remove current connections to nodes that have changed their addresses.
	pub fn reconnect_changed_nodes(self_node_id: &NodeId, data: &mut ClusterConnectionsData, change: &KeyServerSetChange) {
		for (changed_node, changed_node_addr) in &change.changed_nodes {
			if let Entry::Occupied(entry) = data.connections.entry(changed_node.clone()) {
				trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
					self_node_id, entry.get().node_id(), entry.get().node_address());
				entry.remove();
			}

			data.nodes.insert(changed_node.clone(), changed_node_addr.clone());
		}
	}

	/// Connect to nodes, added to the SS.
	pub fn connect_added_nodes(data: &mut ClusterConnectionsData, change: &KeyServerSetChange) {
		for (added_node, added_nodes_addr) in &change.added_nodes {
			data.nodes.insert(added_node.clone(), added_nodes_addr.clone());
		}
	}
}

impl ConnectionTrigger for SimpleConnectionTrigger {
	fn on_servers_set_change(&mut self, data: &mut ClusterConnectionsData, sessions: &ClusterSessions, _block: &H256, change: KeyServerSetChange) {
		// do not update nodes set if any admin session is active
		// this could happen, but will possibly lead to admin session error
		// => should be performed later
		if sessions.admin_sessions.is_empty() {
			Self::disconnect_removed_nodes(&self.self_node_id, data, &change);
			Self::reconnect_changed_nodes(&self.self_node_id, data, &change);
			Self::connect_added_nodes(data, &change);
		}
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.connector.clone()
	}
}

impl ServersSetChangeSessionCreatorConnector for SimpleServersSetChangeSessionCreatorConnector {
	fn admin_public(&self, block: Option<H256>, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		self.admin_public.clone().ok_or(Error::AccessDenied)
	}

	fn set_key_servers_set_change_session(&self, _session: Arc<AdminSession>) {
	}
}

impl ConnectionTrigger for ConnectionTriggerWithMigration {
	fn on_servers_set_change(&mut self, data: &mut ClusterConnectionsData, sessions: &ClusterSessions, block: &H256, change: KeyServerSetChange) {
		// no matter what next, we still need to connect to added nodes && reconnect to changed nodes
		SimpleConnectionTrigger::reconnect_changed_nodes(&self.self_node_id, data, &change);
		SimpleConnectionTrigger::connect_added_nodes(data, &change);

		// if there are no nodes to add/remove => no migration is required
		if change.added_nodes.is_empty() && change.removed_nodes.is_empty() {
			SimpleConnectionTrigger::disconnect_removed_nodes(&self.self_node_id, data, &change);
			return;
		}

		// prepare migration task structure
		let task = MigrationTask {
			block: block.clone(),
			change: change,
		};

		// if servers set change session is already running, we must schedule the task
		match self.session.as_mut() {
			// if meta-session has started, but servers set change session is not yet started
			// => update meta-session servers set
			Some(ref mut session) if sessions.admin_sessions.is_empty() => {
				*self.connector.task.lock() = Some(task.clone());
				session.update_task(task);
				return;
			},
			// if meta-session has started and servers set change session has started
			// => schedule task
			Some(ref mut session) => {
				self.task = Some(task);
				return;
			},
			// else => start new meta-session
			None => (),
		}

		// let's start session
		let mut session = MigrationSession::new(self.self_node_id.clone(), task.clone());
		*self.connector.task.lock() = Some(task.clone());
		for current_connection in data.connections.keys() {
			session.connected(current_connection);
		}

		self.session = Some(session);
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.connector.clone()
	}

	fn on_connection_established(&mut self, node: &NodeId) {
		if let Some(session) = self.session.as_mut() {
			session.connected(node);
		}
	}

	fn on_connection_closed(&mut self, node: &NodeId) {
		if let Some(session) = self.session.as_mut() {
			session.disconnected(node);
		}
	}

	fn maintain(&mut self, client: &ClusterClient, data: &mut ClusterConnectionsData) {
		// to make things easier, this is the only place when servers set change session is started
		// but actually it can be started earlier - as soon as we connect to the last required node (possible TODO)
		if let Some(session) = self.session.as_mut() {
			if !session.maintain(client, data, self.connector.change_session.lock().clone()) {
				*self.connector.change_session.lock() = None;
			}
		}
	}
}

impl ServersSetChangeSessionCreatorConnector for ServersSetChangeSessionCreatorConnectorWithMigration {
	fn admin_public(&self, block: Option<H256>, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		// the idea is that all nodes are agreed upon a block number and a new set of nodes in this block
		// then master node is selected of all nodes set && this master signs the old set && new set
		// (signatures are inputs to ServerSetChangeSession)
		self.task.lock().as_ref()
			.map(|task| if Some(&task.block) == block.as_ref() && task.change.nodes == new_server_set {
				Ok(task.select_master_node().clone())
			} else {
				Err(Error::AccessDenied)
			})
			.unwrap_or(Err(Error::AccessDenied))
	}

	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>) {
		// TODO: is it possible that session is overwritten?
		*self.change_session.lock() = Some(session);
	}
}

impl MigrationTask {
	/// Select master node.
	pub fn select_master_node(&self) -> &NodeId {
		debug_assert!(!self.change.nodes.is_empty());

		// we want to minimize a number of UnknownSession messages =>
		// try to select a node which was in SS && will be in SS
		self.change.nodes.iter()
			.filter(|n| !self.change.added_nodes.contains_key(n))
			.nth(0)
			// else let's select a master from new nodes set
			.unwrap_or_else(|| self.change.nodes.iter().nth(0).unwrap())
	}
}

impl MigrationSession {
	/// Create new migration session for given task.
	pub fn new(self_node_id: NodeId, task: MigrationTask) -> Self {
		let mut session = MigrationSession {
			self_node_id: self_node_id,
			task: Default::default(),
			nodes_to_connect: Default::default(),
		};
		session.update_task(task);
		session
	}

	/// Update session task.
	pub fn update_task(&mut self, task: MigrationTask) {
		self.task = task;
		self.nodes_to_connect = self.task.change.added_nodes.keys()
			.chain(self.task.change.changed_nodes.keys())
			.chain(self.task.change.removed_nodes.keys())
			.cloned().collect();
	}

	/// When node is connected.
	pub fn connected(&mut self, node: &NodeId) {
		self.nodes_to_connect.remove(node);
	}

	/// When node is disconnected.
	pub fn disconnected(&mut self, node: &NodeId) {
		if self.task.change.added_nodes.contains_key(node)
			|| self.task.change.changed_nodes.contains_key(node)
			|| self.task.change.removed_nodes.contains_key(node) {
			self.nodes_to_connect.insert(node.clone());
		}
	}

	/// Maintain session.
	fn maintain(&mut self, client: &ClusterClient, data: &mut ClusterConnectionsData, change_session: Option<Arc<AdminSession>>) -> bool {
		// if ServerSetChange session is active, check if it is completed
		if let Some(change_session) = change_session {
			if change_session.is_finished() {
				// servers set change session is completed => disconnect from all removed nodes
				SimpleConnectionTrigger::disconnect_removed_nodes(&self.self_node_id, data, &self.task.change);
				return true;
			}

			return false;
		}

		// if we have connected to all required nodes => start session
		if !self.nodes_to_connect.is_empty() || self.task.select_master_node() != &self.self_node_id {
			return false;
		}
		
		false
	}
}

/// Prepare key servers set change structure.
pub fn compute_servers_set_change(old: &BTreeMap<NodeId, SocketAddr>, new: &BTreeMap<NodeId, SocketAddr>) -> Option<KeyServerSetChange> {
	// check if nothing has changed
	if old == new {
		return None;
	}

	// prepare a set of changes
	let mut change = KeyServerSetChange::default();

	for (new_node_id, new_node_addr) in new {
		match old.get(new_node_id) {
			Some(old_node_addr) if old_node_addr != new_node_addr => {
				change.changed_nodes.insert(new_node_id.clone(), new_node_addr.clone());
			},
			None => {
				change.added_nodes.insert(new_node_id.clone(), new_node_addr.clone());
			},
			_ => (),
		}
	}

	for (old_node_id, old_node_addr) in old {
		if !new.contains_key(old_node_id) {
			change.removed_nodes.insert(old_node_id.clone(), old_node_addr.clone());
		}
	}

	Some(change)
}

mod tests {
	use ethkey::{KeyPair};
	use super::{MigrationTask, KeyServerSetChange, compute_servers_set_change};

	#[test]
	fn change_computed_works() {
		let pub1 = KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap()).unwrap().public().clone();
		let pub2 = KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap()).unwrap().public().clone();
		let pub3 = KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap()).unwrap().public().clone();
		let pub4 = KeyPair::from_secret("0000000000000000000000000000000000000000000000000000000000000004".parse().unwrap()).unwrap().public().clone();

		let old_nodes = vec![(pub1.clone(), "0.0.0.0:1".parse().unwrap()),
			(pub2.clone(), "0.0.0.0:2".parse().unwrap()),
			(pub3.clone(), "0.0.0.0:3".parse().unwrap())].into_iter().collect();
		let new_nodes = vec![(pub2.clone(), "0.0.0.0:1".parse().unwrap()),
			(pub3.clone(), "0.0.0.0:3".parse().unwrap()),
			(pub4.clone(), "0.0.0.0:4".parse().unwrap())].into_iter().collect();

		let change = compute_servers_set_change(&old_nodes, &new_nodes).unwrap();

		assert_eq!(change.added_nodes.into_iter().collect::<Vec<_>>(), vec![(pub4.clone(), "0.0.0.0:4".parse().unwrap())]);
		assert_eq!(change.removed_nodes.into_iter().collect::<Vec<_>>(), vec![(pub1.clone(), "0.0.0.0:1".parse().unwrap())]);
		assert_eq!(change.changed_nodes.into_iter().collect::<Vec<_>>(), vec![(pub2.clone(), "0.0.0.0:1".parse().unwrap())]);
	}

	#[test]
	fn select_master_node_works() {
		let mut nodes = vec![
			// secret: 0000000000000000000000000000000000000000000000000000000000000001
			("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000002
			("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee51ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000004
			("e493dbf1c10d80f3581e4904930b1404cc6c13900ee0758474fa94abe8c4cd1351ed993ea0d455b75642e2098ea51448d967ae33bfbdfe40cfe97bdc47739922".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
			// secret: 0000000000000000000000000000000000000000000000000000000000000003
			("f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672".parse().unwrap(),
				"127.0.0.1:8080".parse().unwrap()),
		];

		// 0+1 are added + 2 is changed + 3 is removed => 2
		let task = MigrationTask {
			block: Default::default(),
			change: KeyServerSetChange {
				added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
				changed_nodes: vec![nodes[2].clone()].into_iter().collect(),
				removed_nodes: vec![nodes[3].clone()].into_iter().collect(),
				nodes: vec![nodes[0].0.clone(), nodes[1].0.clone(), nodes[2].0.clone()].into_iter().collect(),
			},
		};
		assert_eq!(task.select_master_node(), &nodes[2].0);

		// 3 is changed + 0+1+2 are removed => 3
		let task = MigrationTask {
			block: Default::default(),
			change: KeyServerSetChange {
				added_nodes: vec![].into_iter().collect(),
				changed_nodes: vec![nodes[3].clone()].into_iter().collect(),
				removed_nodes: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()].into_iter().collect(),
				nodes: vec![nodes[3].0.clone()].into_iter().collect(),
			},
		};
		assert_eq!(task.select_master_node(), &nodes[3].0);

		// 0+1 are added + 2+3 are removed => 0
		let task = MigrationTask {
			block: Default::default(),
			change: KeyServerSetChange {
				added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
				changed_nodes: vec![].into_iter().collect(),
				removed_nodes: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
				nodes: vec![nodes[0].0.clone(), nodes[1].0.clone()].into_iter().collect(),
			},
		};
		assert_eq!(task.select_master_node(), &nodes[0].0);
	}
}
