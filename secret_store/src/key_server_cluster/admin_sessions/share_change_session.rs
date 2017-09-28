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
use std::collections::{BTreeSet, BTreeMap};
use ethkey::Secret;
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::jobs::servers_set_change_access_job::ServersSetChangeAccessRequest;
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::message::{Message, ServersSetChangeMessage, ServersSetChangeShareAddMessage, ServersSetChangeShareMoveMessage,
	ServersSetChangeShareRemoveMessage};
use key_server_cluster::share_add_session::{SessionTransport as ShareAddSessionTransport,
	SessionImpl as ShareAddSessionImpl, SessionParams as ShareAddSessionParams};
use key_server_cluster::share_move_session::{SessionTransport as ShareMoveSessionTransport,
	SessionImpl as ShareMoveSessionImpl, SessionParams as ShareMoveSessionParams};
use key_server_cluster::share_remove_session::{SessionTransport as ShareRemoveSessionTransport,
	SessionImpl as ShareRemoveSessionImpl, SessionParams as ShareRemoveSessionParams};
use key_server_cluster::message::{ShareAddMessage, ShareMoveMessage, ShareRemoveMessage};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Single session meta-change session. Brief overview:
/// 1) new shares are added to the session
/// 2) shares are moved between nodes
/// 3) shares are removed from nodes 
pub struct ShareChangeSession {
	/// Servers set change session id.
	session_id: SessionId,
	/// Session nonce.
	nonce: u64,
	/// Share change session meta.
	meta: ShareChangeSessionMeta,
	/// Cluster.
	cluster: Arc<Cluster>,
	/// Key storage.
	key_storage: Arc<KeyStorage>,
	/// Old nodes set.
	old_nodes_set: BTreeSet<NodeId>,
	/// Nodes to add shares for.
	nodes_to_add: Option<BTreeMap<NodeId, Secret>>,
	/// Nodes to move shares from/to.
	nodes_to_move: Option<BTreeMap<NodeId, NodeId>>,
	/// Nodes to remove shares from.
	nodes_to_remove: Option<BTreeSet<NodeId>>,
	/// Share add session.
	share_add_session: Option<ShareAddSessionImpl<ShareChangeTransport>>,
	/// Share move session.
	share_move_session: Option<ShareMoveSessionImpl<ShareChangeTransport>>,
	/// Share remove session.
	share_remove_session: Option<ShareRemoveSessionImpl<ShareChangeTransport>>,
	/// Is finished.
	is_finished: bool,
}

/// Share change session plan.
pub struct ShareChangeSessionPlan {
	/// Nodes to add shares for.
	pub nodes_to_add: BTreeMap<NodeId, Secret>,
	/// Nodes to move shares from/to (keys = target nodes, values = source nodes).
	pub nodes_to_move: BTreeMap<NodeId, NodeId>,
	/// Nodes to remove shares from.
	pub nodes_to_remove: BTreeSet<NodeId>,
}

/// Session parameters.
pub struct ShareChangeSessionParams {
	/// Servers set change session id.
	pub session_id: SessionId,
	/// Session nonce.
	pub nonce: u64,
	/// Share change session meta.
	pub meta: ShareChangeSessionMeta,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Old nodes set.
	pub old_nodes_set: BTreeSet<NodeId>,
	/// Session plan.
	pub plan: ShareChangeSessionPlan,
}

/// Share add session transport.
#[derive(Clone)]
pub struct ShareChangeTransport {
	/// Servers set change session id.
	session_id: SessionId,
	/// Session nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl ShareChangeSession {
	/// Create new share change session.
	pub fn new(params: ShareChangeSessionParams) -> Result<Self, Error> {
		// we can't create sessions right now, because key share is read when session is created, but it can change in previous session
		let nodes_to_add = if !params.plan.nodes_to_add.is_empty() { Some(params.plan.nodes_to_add) } else { None };
		let nodes_to_remove = if !params.plan.nodes_to_remove.is_empty() { Some(params.plan.nodes_to_remove) } else { None };
		let nodes_to_move = if !params.plan.nodes_to_move.is_empty() { Some(params.plan.nodes_to_move) } else { None };
		debug_assert!(nodes_to_add.is_some() || nodes_to_move.is_some() || nodes_to_remove.is_some());

		Ok(ShareChangeSession {
			session_id: params.session_id,
			nonce: params.nonce,
			meta: params.meta,
			cluster: params.cluster,
			key_storage: params.key_storage,
			old_nodes_set: params.old_nodes_set,
			nodes_to_add: nodes_to_add,
			nodes_to_remove: nodes_to_remove,
			nodes_to_move: nodes_to_move,
			share_add_session: None,
			share_move_session: None,
			share_remove_session: None,
			is_finished: false,
		})
	}

	/// Is finished?.
	pub fn is_finished(&self) -> bool {
		self.is_finished
	}

	/// Is master node?.
	pub fn is_master(&self) -> bool {
		self.meta.self_node_id == self.meta.master_node_id
	}

	/// Initialize session (on master node).
	pub fn initialize(&mut self) -> Result<(), Error> {
		self.proceed_to_next_state()
	}

	/// When share-add message is received.
	pub fn on_share_add_message(&mut self, sender: &NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		if self.share_add_session.is_none() {
			self.create_share_add_session()?;
		}

		let change_state_needed = self.share_add_session.as_ref()
			.map(|share_add_session| {
				let was_finished = share_add_session.is_finished();
				share_add_session.process_message(sender, message)
					.map(|_| share_add_session.is_finished() && !was_finished)
			})
			.unwrap_or(Err(Error::InvalidMessage))?;
		if change_state_needed {
			self.proceed_to_next_state()?;
		}

		Ok(())
	}

	/// When share-move message is received.
	pub fn on_share_move_message(&mut self, sender: &NodeId, message: &ShareMoveMessage) -> Result<(), Error> {
		if self.share_move_session.is_none() {
			self.create_share_move_session()?;
		}

		let change_state_needed = self.share_move_session.as_ref()
			.map(|share_move_session| {
				let was_finished = share_move_session.is_finished();
				share_move_session.process_message(sender, message)
					.map(|_| share_move_session.is_finished() && !was_finished)
			})
			.unwrap_or(Err(Error::InvalidMessage))?;
		if change_state_needed {
			self.proceed_to_next_state()?;
		}

		Ok(())
	}

	/// When share-remove message is received.
	pub fn on_share_remove_message(&mut self, sender: &NodeId, message: &ShareRemoveMessage) -> Result<(), Error> {
		if self.share_remove_session.is_none() {
			self.create_share_remove_session()?;
		}

		let change_state_needed = self.share_remove_session.as_ref()
			.map(|share_remove_session| {
				let was_finished = share_remove_session.is_finished();
				share_remove_session.process_message(sender, message)
					.map(|_| share_remove_session.is_finished() && !was_finished)
			})
			.unwrap_or(Err(Error::InvalidMessage))?;
		if change_state_needed {
			self.proceed_to_next_state()?;
		}

		Ok(())
	}

	/// Create new share add session.
	fn create_share_add_session(&mut self) -> Result<(), Error> {
		let nodes_to_add = self.nodes_to_add.take().ok_or(Error::InvalidStateForRequest)?;
		let new_nodes_set = self.old_nodes_set.iter().map(|n| (n.clone(), None))
			.chain(nodes_to_add.clone().into_iter().map(|(k, v)| (k, Some(v))))
			.collect();
		let share_add_session = ShareAddSessionImpl::new(ShareAddSessionParams {
			meta: self.meta.clone(),
			nonce: self.nonce,
			transport: ShareChangeTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			admin_public: None,
		})?;
		share_add_session.set_consensus_output(self.old_nodes_set.clone(), new_nodes_set)?;
		self.share_add_session = Some(share_add_session);
		Ok(())
	}

	/// Create new share move session.
	fn create_share_move_session(&mut self) -> Result<(), Error> {
		let nodes_to_move = self.nodes_to_move.take().ok_or(Error::InvalidStateForRequest)?;
		let share_move_session = ShareMoveSessionImpl::new(ShareMoveSessionParams {
			meta: self.meta.clone(),
			nonce: self.nonce,
			transport: ShareChangeTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			admin_public: None,
		})?;
		share_move_session.set_consensus_output(nodes_to_move)?;
		self.share_move_session = Some(share_move_session);
		Ok(())
	}

	/// Create new share remove session.
	fn create_share_remove_session(&mut self) -> Result<(), Error> {
		let nodes_to_remove = self.nodes_to_remove.take().ok_or(Error::InvalidStateForRequest)?;
		let share_remove_session = ShareRemoveSessionImpl::new(ShareRemoveSessionParams {
			meta: self.meta.clone(),
			nonce: self.nonce,
			transport: ShareChangeTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			admin_public: None,
		})?;
		share_remove_session.set_consensus_output(nodes_to_remove)?;
		self.share_remove_session = Some(share_remove_session);
		Ok(())
	}

	/// Proceed to the next state.
	fn proceed_to_next_state(&mut self) -> Result<(), Error> {
		if self.meta.self_node_id != self.meta.master_node_id {
			if self.nodes_to_add.is_none() && self.nodes_to_move.is_none() && self.nodes_to_remove.is_none() {
				self.is_finished = true;
			}
			return Ok(());
		}

		if self.nodes_to_add.is_some() {
			self.create_share_add_session()?;
			return self.share_add_session.as_ref()
				.expect("either create_share_add_session fails, or session is created; qed")
				.initialize(None, None, None);
		}

		if self.nodes_to_move.is_some() {
			self.create_share_move_session()?;
			return self.share_move_session.as_ref()
				.expect("either create_share_move_session fails, or session is created; qed")
				.initialize(None, None, None);
		}

		if self.nodes_to_remove.is_some() {
			self.create_share_remove_session()?;
			return self.share_remove_session.as_ref()
				.expect("either create_share_remove_session fails, or session is created; qed")
				.initialize(None, None, None);
		}

		self.is_finished = true;

		Ok(())
	}
}

impl ShareChangeTransport {
	pub fn new(session_id: SessionId, nonce: u64, cluster: Arc<Cluster>) -> Self {
		ShareChangeTransport {
			session_id: session_id,
			nonce: nonce,
			cluster: cluster,
		}
	}
}

impl JobTransport for ShareChangeTransport {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;

	fn send_partial_request(&self, _node: &NodeId, _request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		unreachable!("only called when establishing consensus; this transport is never used for establishing consensus; qed")
	}

	fn send_partial_response(&self, _node: &NodeId, _response: bool) -> Result<(), Error> {
		unreachable!("only called when establishing consensus; this transport is never used for establishing consensus; qed")
	}
}

impl ShareAddSessionTransport for ShareChangeTransport {
	fn set_id_numbers(&mut self, _id_numbers: BTreeMap<NodeId, Secret>) {
		unreachable!("only called when establishing consensus; this transport is never used for establishing consensus; qed")
	}

	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareAddMessage(ServersSetChangeShareAddMessage {
			session: self.session_id.clone().into(),
			session_nonce: self.nonce,
			message: message,
		})))
	}
}

impl ShareMoveSessionTransport for ShareChangeTransport {
	fn set_shares_to_move_reversed(&mut self, _shares_to_move: BTreeMap<NodeId, NodeId>) {
		unreachable!("only called when establishing consensus; this transport is never used for establishing consensus; qed")
	}

	fn send(&self, node: &NodeId, message: ShareMoveMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareMoveMessage(ServersSetChangeShareMoveMessage {
			session: self.session_id.clone().into(),
			session_nonce: self.nonce,
			message: message,
		})))
	}
}

impl ShareRemoveSessionTransport for ShareChangeTransport {
	fn send(&self, node: &NodeId, message: ShareRemoveMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ServersSetChangeShareRemoveMessage {
			session: self.session_id.clone().into(),
			session_nonce: self.nonce,
			message: message,
		})))
	}
}

/// Prepare share change plan for moving from old `session_nodes` to `new_nodes_set`.
pub fn prepare_share_change_session_plan(session_nodes: &BTreeSet<NodeId>, new_nodes_set: &BTreeSet<NodeId>) -> Result<ShareChangeSessionPlan, Error> {
	let mut nodes_to_add: BTreeSet<_> = new_nodes_set.difference(&session_nodes).cloned().collect();
	let mut nodes_to_move = BTreeMap::new();
	let mut nodes_to_remove: BTreeSet<_> = session_nodes.difference(&new_nodes_set).cloned().collect();
	while !nodes_to_remove.is_empty() && !nodes_to_add.is_empty() {
		let source_node = nodes_to_remove.iter().cloned().nth(0).expect("nodes_to_remove.is_empty is checked in while condition; qed");
		let target_node = nodes_to_add.iter().cloned().nth(0).expect("nodes_to_add.is_empty is checked in while condition; qed");
		nodes_to_remove.remove(&source_node);
		nodes_to_add.remove(&target_node);
		nodes_to_move.insert(target_node, source_node);
	}

	Ok(ShareChangeSessionPlan {
		nodes_to_add: nodes_to_add.into_iter()
			.map(|n| math::generate_random_scalar().map(|s| (n, s)))
			.collect::<Result<BTreeMap<_, _>, _>>()?,
		nodes_to_move: nodes_to_move,
		nodes_to_remove: nodes_to_remove,
	})
}

impl ShareChangeSessionPlan {
	/// Is empty (nothing-to-do) plan?
	pub fn is_empty(&self) -> bool {
		self.nodes_to_add.is_empty()
			&& self.nodes_to_move.is_empty()
			&& self.nodes_to_remove.is_empty()
	}
}
