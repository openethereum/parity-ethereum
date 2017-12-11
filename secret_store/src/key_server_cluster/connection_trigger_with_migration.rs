// TODO: save actions in MigrationState
// TODO: update from fixed nodes list to contract won't work??? Probably woth adding current_set to the KeyServerSet constructor???
// TODO: when current_set != data.nodes.keys().cloned ===> disconnect from old nodes

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
use ethkey::{Public, Signature};
use futures::lazy;
use parking_lot::Mutex;
use key_server_cluster::{KeyServerSet, KeyServerSetState, KeyServerSetStateType, KeyServerSetMigration};
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData, BoxedEmptyFuture};
use key_server_cluster::cluster_sessions::{AdminSession, ClusterSessions, ClusterSession};
use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
use key_server_cluster::connection_trigger::{Maintain, ConnectionsAction, ConnectionTrigger,
	ServersSetChangeSessionCreatorConnector, TriggerConnections};
use types::all::{Error, NodeId};
use {NodeKeyPair};

/// Key servers set change trigger with automated migration procedure.
pub struct ConnectionTriggerWithMigration {
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Key server set.
	key_server_set: Arc<KeyServerSet>,
	/// Last server set state.
	server_set_state: KeyServerSetState,
	/// Required connections action.
	connections_action: Option<ConnectionsAction>,
	/// Required session action.
	session_action: Option<SessionAction>,
	/// Currenty connected nodes.
	connected: BTreeSet<NodeId>,
	/// Trigger migration connections.
	connections: TriggerConnections,
	/// Trigger migration session.
	session: TriggerSession,
}

#[derive(Default)]
/// Key servers set change session creator connector with migration support.
pub struct ServersSetChangeSessionCreatorConnectorWithMigration {
	/// Active migration state to check when servers set change session is started.
	migration: Mutex<Option<KeyServerSetMigration>>,
	/// Active servers set change session.
	session: Mutex<Option<Arc<AdminSession>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Migration session action.
enum SessionAction {
	/// Start migration (confirm migration transaction).
	StartMigration(H256),
	/// Start migration session.
	Start,
	/// Confirm migration and forget migration session.
	ConfirmAndDrop(H256),
	/// Forget migration session.
	Drop,
	/// Forget migration session and retry.
	DropAndRetry,
}

#[derive(Debug, Clone, Copy)]
/// Migration session state.
enum SessionState {
	/// No active session.
	Idle,
	/// Session is running with given migration id.
	Active(Option<H256>),
	/// Session is completed successfully.
	Finished(Option<H256>),
	/// Session is completed with an error.
	Failed(Option<H256>),
}

/// Migration session.
struct TriggerSession {
	/// Servers set change session creator connector.
	connector: Arc<ServersSetChangeSessionCreatorConnectorWithMigration>,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
	/// Key server set.
	key_server_set: Arc<KeyServerSet>,
}

impl ConnectionTriggerWithMigration {
	/// Create new trigge with migration.
	pub fn new(key_server_set: Arc<KeyServerSet>, self_key_pair: Arc<NodeKeyPair>) -> Self {
		let server_set_state = key_server_set.state();
		let migration = server_set_state.migration.clone();
		ConnectionTriggerWithMigration {
			self_key_pair: self_key_pair.clone(),
			key_server_set: key_server_set.clone(),
			server_set_state: server_set_state,
			connected: BTreeSet::new(),
			connections: TriggerConnections {
				self_key_pair: self_key_pair.clone(),
			},
			session: TriggerSession {
				connector: Arc::new(ServersSetChangeSessionCreatorConnectorWithMigration {
					migration: Mutex::new(migration),
					session: Mutex::new(None),
				}),
				self_key_pair: self_key_pair,
				key_server_set: key_server_set,
			},
			connections_action: None,
			session_action: None,
		}
	}
	
	/// Actually do mainteinance.
	fn do_maintain(&mut self) -> Option<Maintain> {
		loop {
			let session_state = session_state(self.session.connector.session.lock().clone());
			let session_action = maintain_session(&*self.self_key_pair, &self.connected, &self.server_set_state, session_state);
			let session_maintain_required = session_action.map(|session_action|
				self.session.process(session_action)).unwrap_or_default();
			self.session_action = session_action;

			let connections_action = maintain_connections(&self.server_set_state, session_state);
			let connections_maintain_required = connections_action.map(|_| true).unwrap_or_default();
			self.connections_action = connections_action;

			if session_action != Some(SessionAction::DropAndRetry) {
				return match (session_maintain_required, connections_maintain_required) {
					(true, true) => Some(Maintain::SessionAndConnections),
					(true, false) => Some(Maintain::Session),
					(false, true) => Some(Maintain::Connections),
					(false, false) => None,
				};
			}
		}
	}
}

impl ConnectionTrigger for ConnectionTriggerWithMigration {
	fn on_maintain(&mut self) -> Option<Maintain> {
		self.server_set_state = self.key_server_set.state();
		self.do_maintain()
	}

	fn on_connection_established(&mut self, node: &NodeId) -> Option<Maintain> {
		self.connected.insert(node.clone());
		self.do_maintain()
	}

	fn on_connection_closed(&mut self, node: &NodeId) -> Option<Maintain> {
		self.connected.remove(node);
		self.do_maintain()
	}

	fn maintain_session(&mut self, sessions: &ClusterClient) {
		if let Some(action) = self.session_action {
			self.session.maintain(action, sessions, &self.server_set_state);
		}
	}

	fn maintain_connections(&mut self, connections: &mut ClusterConnectionsData) {
		if let Some(action) = self.connections_action {
			self.connections.maintain(action, connections, &self.server_set_state);
		}
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.session.connector.clone()
	}
}

impl ServersSetChangeSessionCreatorConnector for ServersSetChangeSessionCreatorConnectorWithMigration {
	fn admin_public(&self, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		// the idea is that all nodes are agreed upon a block number and a new set of nodes in this block
		// then master node is selected of all nodes set && this master signs the old set && new set
		// (signatures are inputs to ServerSetChangeSession)
		self.migration.lock().as_ref()
			.map(|migration| {
				let is_same_set = new_server_set == migration.set.keys().cloned().collect();
				if is_same_set {
					Ok(migration.master.clone())
				} else {
					Err(Error::AccessDenied)
				}
			})
			.unwrap_or(Err(Error::AccessDenied))
	}

	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>) {
		// TODO: is it possible that session is overwritten?
		*self.session.lock() = Some(session);
	}
}

impl TriggerSession {
	/// Process session action.
	pub fn process(&mut self, action: SessionAction) -> bool {
		match action {
			SessionAction::ConfirmAndDrop(migration_id) => {
				*self.connector.session.lock() = None;
				self.key_server_set.confirm_migration(migration_id);
				false
			},
			SessionAction::Drop | SessionAction::DropAndRetry => {
				*self.connector.session.lock() = None;
				false
			},
			SessionAction::StartMigration(migration_id) => {
				self.key_server_set.start_migration(migration_id);
				false
			},
			SessionAction::Start => true,
		}
	}

	/// Maintain session.
	pub fn maintain(&mut self, action: SessionAction, sessions: &ClusterClient, server_set: &KeyServerSetState) {
		if action == SessionAction::Start {
			let migration = server_set.migration.as_ref().expect("TODO");

			let current_set: BTreeSet<_> = server_set.current_set.keys().cloned().collect();
			let migration_set: BTreeSet<_> = migration.set.keys().cloned().collect();
			let signatures = self.self_key_pair.sign(&ordered_nodes_hash(&current_set))
				.and_then(|current_set_signature| self.self_key_pair.sign(&ordered_nodes_hash(&migration_set))
					.map(|migration_set_signature| (current_set_signature, migration_set_signature)))
				.map_err(Into::into);
			let session = signatures.and_then(|(current_set_signature, migration_set_signature)|
				sessions.new_servers_set_change_session(None, migration_set, current_set_signature, migration_set_signature));

			match session {
				Ok(_) => trace!(target: "secretstore_net", "{}: started auto-migrate session",
					self.self_key_pair.public()),
				Err(err) => trace!(target: "secretstore_net", "{}: failed to start auto-migrate session with: {}",
					self.self_key_pair.public(), err),
			}
		}
	}
}

fn session_state(session: Option<Arc<AdminSession>>) -> SessionState {
	session
		.and_then(|s| match s.as_servers_set_change() {
			Some(s) if !s.is_finished() => Some(SessionState::Active(s.migration_id())),
			Some(s) => match s.wait() {
				Ok(_) => Some(SessionState::Finished(s.migration_id())),
				Err(_) => Some(SessionState::Failed(s.migration_id())),
			},
			None => None,
		})
		.unwrap_or(SessionState::Idle)
}

fn maintain_session(self_key_pair: &NodeKeyPair, connected: &BTreeSet<NodeId>, server_set: &KeyServerSetState, session_state: SessionState) -> Option<SessionAction> {
	let server_set_state = server_set.state();
	match (server_set_state, session_state) {
		// === NORMAL combinations ===

		// having no session when it is not required => ok
		(KeyServerSetStateType::Idle, SessionState::Idle) => None,
		// migration is required && no active session => start migration
		(KeyServerSetStateType::MigrationRequired, SessionState::Idle) => {
			match select_master_node(server_set) == Some(self_key_pair.public()) {
				true => Some(SessionAction::StartMigration(
					ordered_nodes_hash(&server_set.new_set.keys().cloned().collect())
				)),
				// we are not on master node
				false => None,
			}
		},
		// migration is active && there's no active session => start it
		(KeyServerSetStateType::MigrationStarted, SessionState::Idle) => {
			match is_connected_to_all_nodes(&server_set.current_set, connected) &&
				is_connected_to_all_nodes(&server_set.migration.as_ref().expect("TODO").set, connected) {
				true => Some(SessionAction::Start),
				// we are not connected to all required nodes yet => wait for it
				false => None,
			}
		},
		// migration is active && session is not yet started/finished => ok
		(KeyServerSetStateType::MigrationStarted, SessionState::Active(_)) => None,
		// migration has finished => confirm migration
		(KeyServerSetStateType::MigrationStarted, SessionState::Finished(session_migration_id)) => {
			match server_set.migration.as_ref().map(|m| &m.id) == session_migration_id.as_ref() {
				true if server_set.migration.as_ref().map(|m| m.set.contains_key(self_key_pair.public())).unwrap_or_default()
					=> Some(SessionAction::ConfirmAndDrop(
						session_migration_id.expect("TODO")
					)),
				// migration ids are not the same => probably obsolete session
				true | false => {
					warn!(target: "secretstore_net", "{}: suspicious auto-migration state: {:?} with different migration id",
						self_key_pair.public(), (server_set_state, session_state));
					Some(SessionAction::Drop)
				},
			}
		},
		// migration has failed => it should be dropped && restarted later
		(KeyServerSetStateType::MigrationStarted, SessionState::Failed(_)) => Some(SessionAction::Drop),

		// ABNORMAL combinations, which are still possible when contract misbehaves ===

		// having active session when it is not required => drop it && wait for other tasks
		(KeyServerSetStateType::Idle, SessionState::Active(_)) |
		// no migration required && there's finished session => drop it && wait for other tasks
		(KeyServerSetStateType::Idle, SessionState::Finished(_)) |
		// no migration required && there's failed session => drop it && wait for other tasks
		(KeyServerSetStateType::Idle, SessionState::Failed(_)) |
		(KeyServerSetStateType::MigrationRequired, SessionState::Active(_)) |
		// migration is required && session has failed => we need to forget this obolete session and retry
		(KeyServerSetStateType::MigrationRequired, SessionState::Finished(_)) |
		// migration is required && session has failed => we need to forget this obolete session and retry
		(KeyServerSetStateType::MigrationRequired, SessionState::Failed(_)) => {
			warn!(target: "secretstore_net", "{}: suspicious auto-migration state: {:?}",
				self_key_pair.public(), (server_set_state, session_state));
			Some(SessionAction::DropAndRetry)
		},
	}
}

fn maintain_connections(server_set: &KeyServerSetState, session_state: SessionState) -> Option<ConnectionsAction> {
	let server_set_state = server_set.state();
	match (server_set_state, session_state) {
		// session is active => we do not alter connections when session is active
		(KeyServerSetStateType::Idle, SessionState::Active(_)) => None,
		// when no migration required => we just keep us connected to old nodes set
		(KeyServerSetStateType::Idle, _) => Some(ConnectionsAction::ConnectToCurrentSet),
		// when migration is either scheduled, or in progress => connect to both old and migration set
		(KeyServerSetStateType::MigrationRequired, _) |
		(KeyServerSetStateType::MigrationStarted, _) => Some(ConnectionsAction::ConnectToCurrentAndMigrationSet),
	}
}

fn is_connected_to_all_nodes(nodes: &BTreeMap<NodeId, SocketAddr>, connected: &BTreeSet<NodeId>) -> bool {
	nodes.keys().all(|n| connected.contains(n))
}

fn select_master_node(server_set_state: &KeyServerSetState) -> Option<&NodeId> {
	// we want to minimize a number of UnknownSession messages =>
	// try to select a node which was in SS && will be in SS
	server_set_state.migration.as_ref()
		.map(|m| Some(&m.master))
		.unwrap_or_else(|| server_set_state.current_set.keys()
			.filter(|n| server_set_state.new_set.contains_key(n))
			.nth(0)
			.or_else(|| server_set_state.new_set.keys().nth(0)))
}

/*
		trace!(target: "secretstore_net", "{}: servers set has changed: added {}, removed {}, changed {}, new {}",
			self.self_key_pair.public(), change.added_nodes.len(), change.removed_nodes.len(),
			change.changed_nodes.len(), change.nodes.len());

		// no matter what next, we still need to connect to added nodes && reconnect to changed nodes
		SimpleConnectionTrigger::reconnect_changed_nodes(self.self_key_pair.public(), data, &change);
		SimpleConnectionTrigger::connect_added_nodes(self.self_key_pair.public(), data, &change);

		// if there are no new nodes at all => no migration is required (TODO: is this correct?)
		// if there are no nodes to add/remove => no migration is required
		if change.nodes.is_empty() || (change.added_nodes.is_empty() && change.removed_nodes.is_empty()) {
println!("=== {}: 1", self.self_key_pair.public());
			SimpleConnectionTrigger::disconnect_removed_nodes(self.self_key_pair.public(), data, &change);
			return;
		}

		// prepare migration task structure
		let task = MigrationTask {
			change: change,
		};

		// if servers set change session is already running, we must schedule the task
		match self.session.as_mut() {
			// if meta-session has started, but servers set change session is not yet started
			// => update meta-session servers set
			Some(ref mut session) if sessions.admin_sessions.is_empty() => {
println!("=== {}: 2 ===> ADMIN = {}", self.self_key_pair.public(), task.select_master_node());
				*self.connector.task.lock() = Some(task.clone());
				session.update_task(task);
				return;
			},
			// if meta-session has started and servers set change session has started
			// => schedule task
			Some(_) => {
println!("=== {}: 3", self.self_key_pair.public());
				self.task = Some(task);
				return;
			},
			// else => start new meta-session
			None => (),
		}
println!("=== {}: 4 ===> ADMIN = {}", self.self_key_pair.public(), task.select_master_node());
		// let's start session
		let mut session = MigrationSession::new(self.self_key_pair.clone(), task.clone());
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

	fn maintain(&mut self, client: &Arc<ClusterClient>, data: &mut ClusterConnectionsData) -> Option<BoxedEmptyFuture> {
		// to make things easier, this is the only place when servers set change session is started
		// but actually it can be started earlier - as soon as we connect to the last required node (possible TODO)
		if let Some(session) = self.session.as_mut() {
			match session.maintain(data, self.connector.change_session.lock().clone()) {
				MigrateMaintainResult::DoNothing => (),
				MigrateMaintainResult::ForgetSession => *self.connector.change_session.lock() = None,
				MigrateMaintainResult::StartSession(nodes, old_signature, new_signature) => {
					let client = client.clone();
					let self_node_id = self.self_key_pair.public().clone();
					return Some(Box::new(lazy(move || {
						let session_result = client.new_servers_set_change_session(None,
							nodes, old_signature, new_signature);
						match session_result {
							Ok(_) => trace!(target: "secretstore_net", "{}: started auto-migrate session", self_node_id),
							Err(err) => trace!(target: "secretstore_net", "{}: failed to start auto-migrate session with: {}", self_node_id, err),
						}

						Ok(())
					})));
				},
			}
		}

		None
	}
}

impl MigrationSession {
	/// Create new migration session for given task.
	pub fn new(self_key_pair: Arc<NodeKeyPair>, task: MigrationTask) -> Self {
		let mut session = MigrationSession {
			self_key_pair: self_key_pair,
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
	fn maintain(&mut self, data: &mut ClusterConnectionsData, change_session: Option<Arc<AdminSession>>) -> MigrateMaintainResult {
println!("=== {}: 5", self.self_key_pair.public());
		// if ServerSetChange session is active, check if it is completed
		if let Some(change_session) = change_session {
			if change_session.is_finished() {
println!("=== {}: 6", self.self_key_pair.public());
				// servers set change session is completed => disconnect from all removed nodes
				SimpleConnectionTrigger::disconnect_removed_nodes(&self.self_key_pair.public(), data, &self.task.change);
				return MigrateMaintainResult::ForgetSession;
			}
println!("=== {}: 7", self.self_key_pair.public());
			return MigrateMaintainResult::DoNothing;
		}

		// if we have connected to all required nodes => start session
		if !self.nodes_to_connect.is_empty() || self.task.select_master_node() != self.self_key_pair.public() {
println!("=== {}: 8. to_connect: {:?}. master: {:?}", self.self_key_pair.public(), self.nodes_to_connect, self.task.select_master_node());
			return MigrateMaintainResult::DoNothing;
		}

		let old_nodes = self.task.change.nodes.iter()
			.filter(|n| !self.task.change.added_nodes.contains_key(n))
			.chain(self.task.change.removed_nodes.keys())
			.cloned()
			.collect();
		let signatures = self.self_key_pair.sign(&ordered_nodes_hash(&old_nodes))
			.and_then(|current_set_signature| self.self_key_pair.sign(&ordered_nodes_hash(&self.task.change.nodes))
				.map(|new_set_signature| (current_set_signature, new_set_signature)));
		match signatures {
			Ok((current_set_signature, new_set_signature)) =>
				MigrateMaintainResult::StartSession(self.task.change.nodes.clone(), current_set_signature, new_set_signature),
			Err(err) => {
				trace!(target: "secretstore_net", "{}: failed to auto-sign servers set: {}",
					self.self_key_pair.public(), err);
				MigrateMaintainResult::DoNothing
			},
		}
/*			.and_then(|(current_set_signature, new_set_signature)| client.new_servers_set_change_session(None,
				self.task.change.nodes.clone(), current_set_signature, new_set_signature));
		match session_result {
			Ok(_) => trace!(target: "secretstore_net", "{}: started auto-migrate session",
				self.self_key_pair.public()),
			Err(err) => trace!(target: "secretstore_net", "{}: failed to start auto-migrate session with: {}",
				self.self_key_pair.public(), err),
		}

		false*/
	}
}

*/
/*#[cfg(test)]
mod tests {
	use ethkey::KeyPair;
	use super::{MigrationScheme, compute_migration_scheme};

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

		let change = compute_migration_scheme(&old_nodes, &new_nodes).unwrap();

		assert_eq!(change.added_nodes.into_iter().collect::<Vec<_>>(), vec![(pub4.clone(), "0.0.0.0:4".parse().unwrap())]);
		assert_eq!(change.removed_nodes.into_iter().collect::<Vec<_>>(), vec![(pub1.clone(), "0.0.0.0:1".parse().unwrap())]);
		assert_eq!(change.changed_nodes.into_iter().collect::<Vec<_>>(), vec![(pub2.clone(), "0.0.0.0:1".parse().unwrap())]);
	}

	#[test]
	fn select_master_node_works() {
		let nodes = vec![
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
		let task = MigrationScheme {
			current_set: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
			changed_nodes: vec![nodes[2].clone()].into_iter().collect(),
			removed_nodes: vec![nodes[3].clone()].into_iter().collect(),
			new_set: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[2].0);

		// 3 is changed + 0+1+2 are removed => 3
		let task = MigrationScheme {
			current_set: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![].into_iter().collect(),
			changed_nodes: vec![nodes[3].clone()].into_iter().collect(),
			removed_nodes: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()].into_iter().collect(),
			new_set: vec![nodes[3].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[3].0);

		// 0+1 are added + 2+3 are removed => 0
		let task = MigrationScheme {
			current_set: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
			changed_nodes: vec![].into_iter().collect(),
			removed_nodes: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			new_set: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[0].0);
	}
}*/
/*

What to do when servers set changes when nodes are offline

*/