// TODO: update from fixed nodes list to contract won't work??? Probably woth adding old_set to the KeyServerSet constructor???
// TODO: when old_set != data.nodes.keys().cloned ===> disconnect from old nodes

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
use key_server_cluster::{KeyServerSet, KeyServerSetStateType};
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData, BoxedEmptyFuture};
use key_server_cluster::cluster_sessions::{AdminSession, ClusterSessions, ClusterSession};
use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
use types::all::{Error, NodeId};
use {NodeKeyPair};

/// Servers set change session creator connector.
pub trait ServersSetChangeSessionCreatorConnector: Send + Sync {
	/// Get actual administrator public key. For manual-migration configuration it is the pre-configured
	/// administrator key. For auto-migration configurations it is the key of actual MigrationSession master node.
	fn admin_public(&self, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error>;
	/// Set active servers set change session.
	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>);
}

/// Connection trigger, which executes necessary actions when set of key servers changes.
pub trait ConnectionTrigger: Send + Sync {
	/// When key server set is about to change.
	fn on_servers_set_change(&mut self, client: &Arc<ClusterClient>, connections: &mut ClusterConnectionsData, sessions: &ClusterSessions, key_server_set: &KeyServerSet) -> Option<BoxedEmptyFuture>;
	/// Return connector for the servers set change session creator.
	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector>;
	/// When connection is established.
	fn on_connection_established(&mut self, _node: &NodeId) {}
	/// When connection is closed.
	fn on_connection_closed(&mut self, _node: &NodeId) {}
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
	self_key_pair: Arc<NodeKeyPair>,
	/// Connection trigger state.
	state: TriggerState,
	/// Servers set change session creator connector.
	connector: Arc<ServersSetChangeSessionCreatorConnectorWithMigration>,
	/// Active meta-session.
	session: Option<MigrationSession>,
}

#[derive(Default)]
/// Key servers set change session creator connector with migration support.
pub struct ServersSetChangeSessionCreatorConnectorWithMigration {
	/// Active migration task to check when servers set change session is started.
	migration: Mutex<Option<MigrationScheme>>,
	/// Active servers set change session.
	session: Mutex<Option<Arc<AdminSession>>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
/// Single migration path.
pub struct MigrationScheme {
	/// Old nodes set.
	pub old_set: BTreeMap<NodeId, SocketAddr>,
	/// Key servers that are being added to the Secret Store.
	pub added_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Key servers that are being removed from the Secret Store.
	pub removed_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Key servers, that have changed their addresses.
	pub changed_nodes: BTreeMap<NodeId, SocketAddr>,
	/// Final key servers set.
	pub new_set: BTreeMap<NodeId, SocketAddr>,
}

/// Migration session.
struct MigrationSession {
	/// This node id.
	pub self_key_pair: Arc<NodeKeyPair>,
	/// Session change.
	pub migration: MigrationScheme,
	/// Nodes that we need to connected to before starting migration.
	pub nodes_to_connect: BTreeSet<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Migration trigger state.
enum TriggerState {
	/// Currently doing nothing.
	Idle,
	/// Waiting for connections to be established before starting migration.
	Connecting,
	/// Running KeyServerSet change session.
	Migrating,
}

/// Result of maintain call.
enum MigrateMaintainResult {
	/// Do nothing.
	DoNothing,
	/// Forget successful Servers Set Change session.
	ForgetSuccessfulSession,
	/// Forget failed Servers Set Change session.
	ForgetFailedSession,
	/// Start new Servers Set Change session.
	StartSession(BTreeSet<NodeId>, Signature, Signature),
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
	pub fn disconnect_removed_nodes(self_node_id: &NodeId, data: &mut ClusterConnectionsData, migration: &MigrationScheme) {
		for removed_node in migration.removed_nodes.keys() {
			if let Entry::Occupied(entry) = data.connections.entry(removed_node.clone()) {
				trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
					self_node_id, entry.get().node_id(), entry.get().node_address());
				entry.remove();
			}

			data.nodes.remove(&removed_node);
		}
	}

	/// Remove current connections to nodes that have changed their addresses.
	pub fn reconnect_changed_nodes(self_node_id: &NodeId, data: &mut ClusterConnectionsData, migration: &MigrationScheme) {
		for (changed_node, changed_node_addr) in &migration.changed_nodes {
			if changed_node != self_node_id {
				if let Entry::Occupied(entry) = data.connections.entry(changed_node.clone()) {
					trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
						self_node_id, entry.get().node_id(), entry.get().node_address());
					entry.remove();
				}

				data.nodes.insert(changed_node.clone(), changed_node_addr.clone());
			}
		}
	}

	/// Connect to nodes, added to the SS.
	pub fn connect_added_nodes(self_node_id: &NodeId, data: &mut ClusterConnectionsData, migration: &MigrationScheme) {
		for (added_node, added_nodes_addr) in &migration.added_nodes {
			if added_node != self_node_id {
				data.nodes.insert(added_node.clone(), added_nodes_addr.clone());
			}
		}
	}

	/// Adjust connections.
	pub fn adjust_connections(self_node_id: &NodeId, data: &mut ClusterConnectionsData, migration: &MigrationScheme) {
		trace!(target: "secretstore_net", "{}: servers set has changed: added {}, removed {}, changed {}, new {}",
			self_node_id, migration.added_nodes.len(), migration.removed_nodes.len(),
			migration.changed_nodes.len(), migration.new_set.len());

		Self::disconnect_removed_nodes(self_node_id, data, &migration);
		Self::reconnect_changed_nodes(self_node_id, data, &migration);
		Self::connect_added_nodes(self_node_id, data, &migration);
	}
}

impl ConnectionTrigger for SimpleConnectionTrigger {
	fn on_servers_set_change(&mut self, client: &Arc<ClusterClient>, data: &mut ClusterConnectionsData, sessions: &ClusterSessions, key_server_set: &KeyServerSet) -> Option<BoxedEmptyFuture> {
		// do not update nodes set if any admin session is active
		// this could happen, but will possibly lead to admin session error
		// => should be performed later
		if !sessions.admin_sessions.is_empty() {
			return None;
		}

		// compute migration path
		let new_set = key_server_set.state().new_set;
		let migration = match compute_migration_scheme(&data.nodes, &new_set) {
			None => return None,
			Some(migration) => migration,
		};

		// apply change
		Self::adjust_connections(&self.self_node_id, data, &migration);

		None
	}

	fn servers_set_change_creator_connector(&self) -> Arc<ServersSetChangeSessionCreatorConnector> {
		self.connector.clone()
	}
}

impl ServersSetChangeSessionCreatorConnector for SimpleServersSetChangeSessionCreatorConnector {
	fn admin_public(&self, _new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		self.admin_public.clone().ok_or(Error::AccessDenied)
	}

	fn set_key_servers_set_change_session(&self, _session: Arc<AdminSession>) {
	}
}

impl ConnectionTriggerWithMigration {
	/// Create new connection trigger with migration support.
	pub fn new(self_key_pair: Arc<NodeKeyPair>) -> Self {
		ConnectionTriggerWithMigration {
			self_key_pair: self_key_pair,
			connector: Arc::new(Default::default()),
			state: TriggerState::Idle,
			session: None,
		}
	}

	fn maintain(&mut self, client: &Arc<ClusterClient>, data: &mut ClusterConnectionsData, key_server_set: &KeyServerSet) -> Option<BoxedEmptyFuture> {
		// to make things easier, this is the only place when servers set change session is started
		// but actually it can be started earlier - as soon as we connect to the last required node (possible TODO)
		let mut drop_session = false;
		if let Some(session) = self.session.as_mut() {
			let admin_session = {
				(*self.connector.session.lock()).clone()
			};
			match session.maintain(data, admin_session) {
				MigrateMaintainResult::DoNothing => (),
				MigrateMaintainResult::ForgetSuccessfulSession => {
					*self.connector.session.lock() = None;
					if session.migration.new_set.contains_key(self.self_key_pair.public()) {
						key_server_set.confirm_migration();
					}
println!("=== {}: SWICHED TO Idle", self.self_key_pair.public());
					self.state = TriggerState::Idle;
				},
				MigrateMaintainResult::ForgetFailedSession => {
					*self.connector.session.lock() = None;
println!("=== {}: SWICHED TO Idle", self.self_key_pair.public());
				},
				MigrateMaintainResult::StartSession(nodes, old_signature, new_signature) => {
println!("=== {}: SWICHED TO Migrating", self.self_key_pair.public());
					self.state = TriggerState::Migrating;

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
		if self.state == TriggerState::Idle {
			self.session = None;
		}

		None
	}
}

impl ConnectionTrigger for ConnectionTriggerWithMigration {
	fn on_servers_set_change(&mut self, client: &Arc<ClusterClient>, data: &mut ClusterConnectionsData, sessions: &ClusterSessions, key_server_set: &KeyServerSet) -> Option<BoxedEmptyFuture> {
		let server_set_state = key_server_set.state();
		let server_set_state_type = server_set_state.state();
println!("=== {}: STATE-BEFORE = {:?}", self.self_key_pair.public(), (self.state, server_set_state_type));
		let is_migrating = self.state == TriggerState::Migrating;
		let r = self.maintain(client, data, key_server_set);
		let is_idle = self.state == TriggerState::Idle;

		// TODO: since confirmation transaction can be mined several blocks after => need to make a pause before restarting session
		if is_migrating && is_idle {
println!("=== {}: MigratingAndIdleReturn", self.self_key_pair.public());
			return r;
		}

println!("=== {}: STATE-AFTER = {:?}", self.self_key_pair.public(), (self.state, server_set_state_type));
		match (self.state, server_set_state_type) {
			// both are idle => no actions required
			(TriggerState::Idle, KeyServerSetStateType::Idle) => {
				// TODO: it could be that old_set == new_set, but we are currently connected to different set of nodes (not the old_set)
				// => adjust connections
			},
			// migration is required => start migration if we are master node
			(TriggerState::Idle, KeyServerSetStateType::MigrationRequired) => {
				let migration = compute_migration_scheme(&server_set_state.old_set, &server_set_state.new_set).expect("TODO");
				if !migration.requires_migration() {
					SimpleConnectionTrigger::adjust_connections(self.self_key_pair.public(), data, &migration);
					return r;
				}
				
				let master_node = migration.select_master_node();
				if self.self_key_pair.public() == master_node {
					key_server_set.start_migration();
				}
			},
			// migration is started => start connecting to other nodes
			(TriggerState::Idle, KeyServerSetStateType::MigrationStarted) => {
				// create new migration session
				let migration = compute_migration_scheme(&server_set_state.old_set, &server_set_state.new_set).expect("TODO");
				let mut session = MigrationSession::new(self.self_key_pair.clone(), migration.clone());

				// notify about current connections
				for current_connection in data.connections.keys() {
					session.connected(current_connection);
				}

				// remember session
				self.session = Some(session);
				self.state = TriggerState::Connecting;
				*self.connector.migration.lock() = Some(migration.clone());
println!("=== {}: SWICHED TO Connecting", self.self_key_pair.public());
				// start connecting to other nodes (if required)
				SimpleConnectionTrigger::reconnect_changed_nodes(self.self_key_pair.public(), data, &migration);
				SimpleConnectionTrigger::connect_added_nodes(self.self_key_pair.public(), data, &migration);

				// if we are connected to all nodes, start server set change session
				// if self.session.as_ref().map(|s| s.is_connected()).unwrap_or_default() {
				// 	self.start_servers_set_change_session();
				// }
			},
			(TriggerState::Connecting, KeyServerSetStateType::Idle) => {
				// ERROR: stop everything && connect to new_set only + ToIdle
				// unimplemented!()
				// TODO: not an error if maintain() && on_Server_set_change are called asynchronously - maintain should change state to Idle, but on_Server_set_change happens before
				self.state = TriggerState::Idle;
			},
			(TriggerState::Connecting, KeyServerSetStateType::MigrationRequired) => {
				// ERROR: stop everything && connect to new_set only + ToIdle
				// unimplemented!()
				self.state = TriggerState::Idle;
			},
			(TriggerState::Connecting, KeyServerSetStateType::MigrationStarted) => {
				// if migration scheme has changed, it is not yet too late to change it
				let migration = compute_migration_scheme(&server_set_state.old_set, &server_set_state.new_set).expect("TODO");
				let mut session = self.session.as_mut().expect("TODO");
				if session.migration != migration {
					// remember new migration scheme
					session.migration = migration.clone();
					*self.connector.migration.lock() = Some(migration.clone());

					// start connecting to other nodes (if required)
					SimpleConnectionTrigger::reconnect_changed_nodes(self.self_key_pair.public(), data, &migration);
					SimpleConnectionTrigger::connect_added_nodes(self.self_key_pair.public(), data, &migration);

					// if we are connected to all nodes, start server set change session
					// if self.session.as_ref().map(|s| s.is_connected()).unwrap_or_default() {
					// 	self.start_servers_set_change_session();
					// }
				}
			},
			(TriggerState::Migrating, KeyServerSetStateType::Idle) => {
				// ERROR: stop everything && connect to new_set only + ToIdle
				// TODO: see above unimplemented!()
				self.state = TriggerState::Idle;
			},
			(TriggerState::Migrating, KeyServerSetStateType::MigrationRequired) => {
				// ERROR: stop everything && connect to new_set only + ToIdle
				// TODO: see above unimplemented!()
				self.state = TriggerState::Idle;
			},
			(TriggerState::Migrating, KeyServerSetStateType::MigrationStarted) => {
				// TODO: schedule???
				self.state = TriggerState::Idle;
			},
		}

		r
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
}

impl ServersSetChangeSessionCreatorConnector for ServersSetChangeSessionCreatorConnectorWithMigration {
	fn admin_public(&self, new_server_set: BTreeSet<NodeId>) -> Result<Public, Error> {
		// the idea is that all nodes are agreed upon a block number and a new set of nodes in this block
		// then master node is selected of all nodes set && this master signs the old set && new set
		// (signatures are inputs to ServerSetChangeSession)
		self.migration.lock().as_ref()
			.map(|migration| if new_server_set == migration.new_set.keys().cloned().collect() { // TODO: old set?
				Ok(migration.select_master_node().clone())
			} else {
				Err(Error::AccessDenied)
			})
			.unwrap_or(Err(Error::AccessDenied))
	}

	fn set_key_servers_set_change_session(&self, session: Arc<AdminSession>) {
		// TODO: is it possible that session is overwritten?
		*self.session.lock() = Some(session);
	}
}

impl MigrationScheme {
	/// Returns true if ServersSetChange session is required.
	pub fn requires_migration(&self) -> bool {
		!self.added_nodes.is_empty()
			|| !self.removed_nodes.is_empty()
	}

	/// Select master node.
	pub fn select_master_node(&self) -> &NodeId {
		// at least 1 node is required for migration
		debug_assert!(!self.new_set.is_empty());

		// we want to minimize a number of UnknownSession messages =>
		// try to select a node which was in SS && will be in SS
		self.new_set.keys()
			.filter(|n| !self.added_nodes.contains_key(n))
			.nth(0)
			// else let's select a master from new nodes set
			.unwrap_or_else(|| self.new_set.keys().nth(0).unwrap())
	}
}

impl MigrationSession {
	/// Create new migration session for given task.
	pub fn new(self_key_pair: Arc<NodeKeyPair>, migration: MigrationScheme) -> Self {
		let mut session = MigrationSession {
			self_key_pair: self_key_pair,
			migration: Default::default(),
			nodes_to_connect: Default::default(),
		};
		session.update_scheme(migration);
		session
	}

	/// Update session task.
	pub fn update_scheme(&mut self, migration: MigrationScheme) {
		self.migration = migration;
		self.nodes_to_connect = self.migration.added_nodes.keys()
			.chain(self.migration.changed_nodes.keys())
			.chain(self.migration.removed_nodes.keys())
			.filter(|n| *n != self.self_key_pair.public())
			.cloned().collect();
	}

	/// When node is connected.
	pub fn connected(&mut self, node: &NodeId) {
		self.nodes_to_connect.remove(node);
	}

	/// When node is disconnected.
	pub fn disconnected(&mut self, node: &NodeId) {
		if self.migration.added_nodes.contains_key(node)
			|| self.migration.changed_nodes.contains_key(node)
			|| self.migration.removed_nodes.contains_key(node) {
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
				SimpleConnectionTrigger::disconnect_removed_nodes(&self.self_key_pair.public(), data, &self.migration); // TODO: adjust connections instead!!! because of fixed -> empty contract -> non-empty contract => still connected to fixed list
				if change_session.as_servers_set_change().expect("TODO").wait().is_ok() {
println!("=== {}: 6.success", self.self_key_pair.public());
					return MigrateMaintainResult::ForgetSuccessfulSession;
				} else {
println!("=== {}: 6.failed", self.self_key_pair.public());
					return MigrateMaintainResult::ForgetFailedSession;
				}
			}
println!("=== {}: 7", self.self_key_pair.public());
			return MigrateMaintainResult::DoNothing;
		}

		// if we have connected to all required nodes => start session
		if !self.nodes_to_connect.is_empty() || self.migration.select_master_node() != self.self_key_pair.public() {
println!("=== {}: 8. to_connect: {:?}. master: {:?}", self.self_key_pair.public(), self.nodes_to_connect, self.migration.select_master_node());
			return MigrateMaintainResult::DoNothing;
		}

		let mut old_set: BTreeSet<_> = self.migration.old_set.keys().cloned().collect();
		if old_set.is_empty() {
			// assume that all currently configured nodes are the previous version of SecretStore
			old_set.extend(data.nodes.keys().cloned());
			old_set.insert(self.self_key_pair.public().clone());
		}

		let signatures = self.self_key_pair.sign(&ordered_nodes_hash(&old_set))
			.and_then(|old_set_signature| self.self_key_pair.sign(&ordered_nodes_hash(&self.migration.new_set.keys().cloned().collect()))
				.map(|new_set_signature| (old_set_signature, new_set_signature)));
		match signatures {
			Ok((old_set_signature, new_set_signature)) =>
				MigrateMaintainResult::StartSession(self.migration.new_set.keys().cloned().collect(), old_set_signature, new_set_signature),
			Err(err) => {
				trace!(target: "secretstore_net", "{}: failed to auto-sign servers set: {}",
					self.self_key_pair.public(), err);
				MigrateMaintainResult::DoNothing
			},
		}
/*			.and_then(|(old_set_signature, new_set_signature)| client.new_servers_set_change_session(None,
				self.task.change.nodes.clone(), old_set_signature, new_set_signature));
		match session_result {
			Ok(_) => trace!(target: "secretstore_net", "{}: started auto-migrate session",
				self.self_key_pair.public()),
			Err(err) => trace!(target: "secretstore_net", "{}: failed to start auto-migrate session with: {}",
				self.self_key_pair.public(), err),
		}

		false*/
	}
}

/// Prepare key servers set change structure.
pub fn compute_migration_scheme(old: &BTreeMap<NodeId, SocketAddr>, new: &BTreeMap<NodeId, SocketAddr>) -> Option<MigrationScheme> {
	// check if nothing has changed
	if old == new {
		return None;
	}

	// prepare a set of changes
	let mut scheme = MigrationScheme::default();
	scheme.old_set = old.clone();
	scheme.new_set = new.clone();

	for (new_node_id, new_node_addr) in new {
		match old.get(new_node_id) {
			Some(old_node_addr) if old_node_addr != new_node_addr => {
				scheme.changed_nodes.insert(new_node_id.clone(), new_node_addr.clone());
			},
			None => {
				scheme.added_nodes.insert(new_node_id.clone(), new_node_addr.clone());
			},
			_ => (),
		}
	}

	for (old_node_id, old_node_addr) in old {
		if !new.contains_key(old_node_id) {
			scheme.removed_nodes.insert(old_node_id.clone(), old_node_addr.clone());
		}
	}

	Some(scheme)
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
			.and_then(|old_set_signature| self.self_key_pair.sign(&ordered_nodes_hash(&self.task.change.nodes))
				.map(|new_set_signature| (old_set_signature, new_set_signature)));
		match signatures {
			Ok((old_set_signature, new_set_signature)) =>
				MigrateMaintainResult::StartSession(self.task.change.nodes.clone(), old_set_signature, new_set_signature),
			Err(err) => {
				trace!(target: "secretstore_net", "{}: failed to auto-sign servers set: {}",
					self.self_key_pair.public(), err);
				MigrateMaintainResult::DoNothing
			},
		}
/*			.and_then(|(old_set_signature, new_set_signature)| client.new_servers_set_change_session(None,
				self.task.change.nodes.clone(), old_set_signature, new_set_signature));
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
#[cfg(test)]
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
			old_set: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
			changed_nodes: vec![nodes[2].clone()].into_iter().collect(),
			removed_nodes: vec![nodes[3].clone()].into_iter().collect(),
			new_set: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[2].0);

		// 3 is changed + 0+1+2 are removed => 3
		let task = MigrationScheme {
			old_set: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![].into_iter().collect(),
			changed_nodes: vec![nodes[3].clone()].into_iter().collect(),
			removed_nodes: vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()].into_iter().collect(),
			new_set: vec![nodes[3].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[3].0);

		// 0+1 are added + 2+3 are removed => 0
		let task = MigrationScheme {
			old_set: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			added_nodes: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
			changed_nodes: vec![].into_iter().collect(),
			removed_nodes: vec![nodes[2].clone(), nodes[3].clone()].into_iter().collect(),
			new_set: vec![nodes[0].clone(), nodes[1].clone()].into_iter().collect(),
		};
		assert_eq!(task.select_master_node(), &nodes[0].0);
	}
}
/*

What to do when servers set changes when nodes are offline

*/