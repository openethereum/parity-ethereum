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
use std::net::SocketAddr;
use std::sync::Arc;
use bigint::hash::H256;
use ethkey::Public;
use parking_lot::Mutex;
use key_server_cluster::{KeyServerSet, KeyServerSetSnapshot, KeyServerSetMigration};
use key_server_cluster::cluster::{ClusterClient, ClusterConnectionsData};
use key_server_cluster::cluster_sessions::{AdminSession, ClusterSession};
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
	snapshot: KeyServerSetSnapshot,
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

#[derive(Debug, Clone, Copy, PartialEq)]
/// Migration state.
pub enum MigrationState {
	/// No migration required.
	Idle,
	/// Migration is required.
	Required,
	/// Migration has started.
	Started,
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
		let snapshot = key_server_set.snapshot();
		let migration = snapshot.migration.clone();
		ConnectionTriggerWithMigration {
			self_key_pair: self_key_pair.clone(),
			key_server_set: key_server_set.clone(),
			snapshot: snapshot,
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
			let migration_state = migration_state(self.self_key_pair.public(), &self.snapshot);
			
			let session_action = maintain_session(self.self_key_pair.public(), &self.connected, &self.snapshot, migration_state, session_state);
			let session_maintain_required = session_action.map(|session_action|
				self.session.process(session_action)).unwrap_or_default();
			self.session_action = session_action;

			let connections_action = maintain_connections(migration_state, session_state);
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
		self.snapshot = self.key_server_set.snapshot();
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
			self.session.maintain(action, sessions, &self.snapshot);
		}
	}

	fn maintain_connections(&mut self, connections: &mut ClusterConnectionsData) {
		if let Some(action) = self.connections_action {
			self.connections.maintain(action, connections, &self.snapshot);
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
	pub fn maintain(&mut self, action: SessionAction, sessions: &ClusterClient, server_set: &KeyServerSetSnapshot) {
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

fn migration_state(self_node_id: &NodeId, snapshot: &KeyServerSetSnapshot) -> MigrationState {
	// if this node is not on current && old set => we do not participate in migration
	if !snapshot.current_set.contains_key(self_node_id) &&
		!snapshot.migration.as_ref().map(|s| s.set.contains_key(self_node_id)).unwrap_or_default() {
		return MigrationState::Idle;
	}

	// if migration has already started no other states possible
	if snapshot.migration.is_some() {
		return MigrationState::Started;
	}

	// we only require migration if set actually changes
	// when only address changes, we could simply adjust connections
	let no_nodes_removed = snapshot.current_set.keys().all(|n| snapshot.new_set.contains_key(n));
	let no_nodes_added = snapshot.new_set.keys().all(|n| snapshot.current_set.contains_key(n));
	if no_nodes_removed && no_nodes_added {
		return MigrationState::Idle;
	}

	return MigrationState::Required;
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

fn maintain_session(self_node_id: &NodeId, connected: &BTreeSet<NodeId>, snapshot: &KeyServerSetSnapshot, migration_state: MigrationState, session_state: SessionState) -> Option<SessionAction> {
	match (migration_state, session_state) {
		// === NORMAL combinations ===

		// having no session when it is not required => ok
		(MigrationState::Idle, SessionState::Idle) => None,
		// migration is required && no active session => start migration
		(MigrationState::Required, SessionState::Idle) => {
			match select_master_node(snapshot) == Some(self_node_id) {
				true => Some(SessionAction::StartMigration(
					ordered_nodes_hash(&snapshot.new_set.keys().cloned().collect())
				)),
				// we are not on master node
				false => None,
			}
		},
		// migration is active && there's no active session => start it
		(MigrationState::Started, SessionState::Idle) => {
			match is_connected_to_all_nodes(&snapshot.current_set, connected) &&
				is_connected_to_all_nodes(&snapshot.migration.as_ref().expect("TODO").set, connected) {
				true => Some(SessionAction::Start),
				// we are not connected to all required nodes yet => wait for it
				false => None,
			}
		},
		// migration is active && session is not yet started/finished => ok
		(MigrationState::Started, SessionState::Active(_)) => None,
		// migration has finished => confirm migration
		(MigrationState::Started, SessionState::Finished(session_migration_id)) => {
			match snapshot.migration.as_ref().map(|m| &m.id) == session_migration_id.as_ref() {
				true if snapshot.migration.as_ref().map(|m| m.set.contains_key(self_node_id)).unwrap_or_default()
					=> Some(SessionAction::ConfirmAndDrop(
						session_migration_id.expect("TODO")
					)),
				// migration ids are not the same => probably obsolete session
				true | false => {
					warn!(target: "secretstore_net", "{}: suspicious auto-migration state: {:?} with different migration id",
						self_node_id, (migration_state, session_state));
					Some(SessionAction::Drop)
				},
			}
		},
		// migration has failed => it should be dropped && restarted later
		(MigrationState::Started, SessionState::Failed(_)) => Some(SessionAction::Drop),

		// ABNORMAL combinations, which are still possible when contract misbehaves ===

		// having active session when it is not required => drop it && wait for other tasks
		(MigrationState::Idle, SessionState::Active(_)) |
		// no migration required && there's finished session => drop it && wait for other tasks
		(MigrationState::Idle, SessionState::Finished(_)) |
		// no migration required && there's failed session => drop it && wait for other tasks
		(MigrationState::Idle, SessionState::Failed(_)) |
		(MigrationState::Required, SessionState::Active(_)) |
		// migration is required && session has failed => we need to forget this obolete session and retry
		(MigrationState::Required, SessionState::Finished(_)) |
		// migration is required && session has failed => we need to forget this obolete session and retry
		(MigrationState::Required, SessionState::Failed(_)) => {
			warn!(target: "secretstore_net", "{}: suspicious auto-migration state: {:?}",
				self_node_id, (migration_state, session_state));
			Some(SessionAction::DropAndRetry)
		},
	}
}

fn maintain_connections(migration_state: MigrationState, session_state: SessionState) -> Option<ConnectionsAction> {
	match (migration_state, session_state) {
		// session is active => we do not alter connections when session is active
		(_, SessionState::Active(_)) => None,
		// when no migration required => we just keep us connected to old nodes set
		(MigrationState::Idle, _) => Some(ConnectionsAction::ConnectToCurrentSet),
		// when migration is either scheduled, or in progress => connect to both old and migration set.
		// this could lead to situation when node is not 'officially' a part of KeyServer (i.e. it is not in current_set)
		// but it participates in new key generation session
		// it is ok, since 'officialy' here means that this node is a owner of all old shares
		(MigrationState::Required, _) |
		(MigrationState::Started, _) => Some(ConnectionsAction::ConnectToCurrentAndMigrationSet),
	}
}

fn is_connected_to_all_nodes(nodes: &BTreeMap<NodeId, SocketAddr>, connected: &BTreeSet<NodeId>) -> bool {
	nodes.keys().all(|n| connected.contains(n))
}

// TODO: is it possible to return None here???
fn select_master_node(server_set_state: &KeyServerSetSnapshot) -> Option<&NodeId> {
	// we want to minimize a number of UnknownSession messages =>
	// try to select a node which was in SS && will be in SS
	server_set_state.migration.as_ref()
		.map(|m| Some(&m.master))
		.unwrap_or_else(|| server_set_state.current_set.keys()
			.filter(|n| server_set_state.new_set.contains_key(n))
			.nth(0)
			.or_else(|| server_set_state.new_set.keys().nth(0)))
}

#[cfg(test)]
mod tests {
	use key_server_cluster::{KeyServerSetSnapshot, KeyServerSetMigration};
	use key_server_cluster::connection_trigger::ConnectionsAction;
	use super::{MigrationState, SessionState, migration_state, maintain_session, maintain_connections, select_master_node};

	#[test]
	fn migration_state_is_idle_when_required_but_this_node_is_not_on_the_list() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(2.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			new_set: vec![(3.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), MigrationState::Idle);
	}

	#[test]
	fn migration_state_is_idle_when_sets_are_equal() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			new_set: vec![(1.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), MigrationState::Idle);
	}

	#[test]
	fn migration_state_is_idle_when_only_address_changes() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap())].into_iter().collect(),
			new_set: vec![(1.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), MigrationState::Idle);
	}

	#[test]
	fn migration_state_is_required_when_node_is_added() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap())].into_iter().collect(),
			new_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap()),
				(2.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), MigrationState::Required);
	}

	#[test]
	fn migration_state_is_required_when_node_is_removed() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap()),
				(2.into(), "127.0.0.1:8081".parse().unwrap())].into_iter().collect(),
			new_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), MigrationState::Required);
	}

	#[test]
	fn migration_state_is_started_when_migration_is_some() {
		assert_eq!(migration_state(&1.into(), &KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8080".parse().unwrap())].into_iter().collect(),
			new_set: Default::default(),
			migration: Some(KeyServerSetMigration {
				id: Default::default(),
				set: Default::default(),
				master: Default::default(),
				is_confirmed: Default::default(),
			}),
		}), MigrationState::Started);
	}

	#[test]
	fn existing_master_is_selected_when_migration_has_started() {
		assert_eq!(select_master_node(&KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8180".parse().unwrap())].into_iter().collect(),
			new_set: vec![(2.into(), "127.0.0.1:8181".parse().unwrap())].into_iter().collect(),
			migration: Some(KeyServerSetMigration {
				master: 3.into(),
				..Default::default()
			}),
		}), Some(&3.into()));
	}

	#[test]
	fn persistent_master_is_selected_when_migration_has_not_started_yet() {
		assert_eq!(select_master_node(&KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8180".parse().unwrap()),
				(2.into(), "127.0.0.1:8180".parse().unwrap())].into_iter().collect(),
			new_set: vec![(2.into(), "127.0.0.1:8181".parse().unwrap()),
				(4.into(), "127.0.0.1:8181".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), Some(&2.into()));
	}

	#[test]
	fn new_master_is_selected_in_worst_case() {
		assert_eq!(select_master_node(&KeyServerSetSnapshot {
			current_set: vec![(1.into(), "127.0.0.1:8180".parse().unwrap()),
				(2.into(), "127.0.0.1:8180".parse().unwrap())].into_iter().collect(),
			new_set: vec![(3.into(), "127.0.0.1:8181".parse().unwrap()),
				(4.into(), "127.0.0.1:8181".parse().unwrap())].into_iter().collect(),
			migration: None,
		}), Some(&3.into()));
	}

	#[test]
	fn maintain_connections_returns_none_when_session_is_active() {
		assert_eq!(maintain_connections(MigrationState::Required,
			SessionState::Active(Default::default())), None);
	}

	#[test]
	fn maintain_connections_connects_to_current_set_when_no_migration() {
		assert_eq!(maintain_connections(MigrationState::Idle,
			SessionState::Idle), Some(ConnectionsAction::ConnectToCurrentSet));
	}

	#[test]
	fn maintain_connections_connects_to_current_and_old_set_when_migration_is_required() {
		assert_eq!(maintain_connections(MigrationState::Required,
			SessionState::Idle), Some(ConnectionsAction::ConnectToCurrentAndMigrationSet));
	}

	#[test]
	fn maintain_connections_connects_to_current_and_old_set_when_migration_is_started() {
		assert_eq!(maintain_connections(MigrationState::Started,
			SessionState::Idle), Some(ConnectionsAction::ConnectToCurrentAndMigrationSet));
	}

	#[test]
	fn maintain_sessions_does_nothing_if_no_session_and_no_migration() {
		assert_eq!(maintain_session(&1.into(), &Default::default(), &Default::default(),
			MigrationState::Idle, SessionState::Idle), None);
	}

	#[test]
	fn maintain_session_starts_migration_when_required_on_main_node_and_no_session() {
/*		assert_eq!(maintain_session(&1.into(), &PlainNodeKeyPair::new(Random.generate().unwrap()),
			&Default::default(), &KeyServerSetSnapshot {
				current_set: vec![(1.into(), "127.0.0.1:8181".parse().unwrap())].into_iter().collect(),
				new_set: vec![(2.into(), "127.0.0.1:8181".parse().unwrap())].into_iter().collect(),
				migration: None,
			}, SessionState::Idle), None);*/
	}
}
