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
use bigint::hash::H256;
use ethkey::Secret;
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::jobs::servers_set_change_access_job::ServersSetChangeAccessRequest;
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::message::{Message, ServersSetChangeMessage, ServersSetChangeShareAddMessage};
use key_server_cluster::share_add_session::{SessionTransport as ShareAddSessionTransport,
	SessionImpl as ShareAddSessionImpl, SessionParams as ShareAddSessionParams};
use key_server_cluster::message::ShareAddMessage;
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Single session meta-change session. Brief overview:
/// 1) nodes that have been already removed from cluster (isolated nodes) are removed from session
/// 2) new shares are added to the session
/// 3) shares are moved between nodes
/// 4) shares are removed from nodes
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
	/// Key version.
	key_version: H256,
	/// Old nodes set.
	old_nodes_set: BTreeSet<NodeId>,
	/// All cluster nodes set.
	cluster_nodes_set: BTreeSet<NodeId>,
	/// Consensus group to use in ShareAdd session.
	consensus_group: Option<BTreeSet<NodeId>>,
	/// Nodes to add shares for.
	new_nodes_map: Option<BTreeMap<NodeId, Option<Secret>>>,
	/// Nodes to remove shares from.
	nodes_to_remove: Option<BTreeSet<NodeId>>,
	/// Share add session.
	share_add_session: Option<ShareAddSessionImpl<ShareChangeTransport>>,
	/// Is finished.
	is_finished: bool,
}

/// Share change session plan.
#[derive(Debug)]
pub struct ShareChangeSessionPlan {
	/// Key version that plan is valid for.
	pub key_version: H256,
	/// Consensus group to use in ShareAdd session.
	pub consensus_group: BTreeSet<NodeId>,
	/// Nodes to add shares for.
	pub new_nodes_map: BTreeMap<NodeId, Option<Secret>>,
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
	/// All cluster nodes set.
	pub cluster_nodes_set: BTreeSet<NodeId>,
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
		let key_version = params.plan.key_version;
		let consensus_group = if !params.plan.consensus_group.is_empty() { Some(params.plan.consensus_group) } else { None };
		let new_nodes_map = if !params.plan.new_nodes_map.is_empty() { Some(params.plan.new_nodes_map) } else { None };
		let nodes_to_remove = if !params.plan.nodes_to_remove.is_empty() { Some(params.plan.nodes_to_remove) } else { None };
		debug_assert!(new_nodes_map.is_some() || nodes_to_remove.is_some());

		let is_finished = new_nodes_map.is_none() && nodes_to_remove.is_none();
		Ok(ShareChangeSession {
			session_id: params.session_id,
			nonce: params.nonce,
			meta: params.meta,
			cluster: params.cluster,
			key_storage: params.key_storage,
			key_version: key_version,
			old_nodes_set: params.old_nodes_set,
			cluster_nodes_set: params.cluster_nodes_set,
			consensus_group: consensus_group,
			new_nodes_map: new_nodes_map,
			nodes_to_remove: nodes_to_remove,
			share_add_session: None,
			is_finished: is_finished,
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

	/// Create new share add session.
	fn create_share_add_session(&mut self) -> Result<(), Error> {
		let consensus_group = self.consensus_group.take().ok_or(Error::InvalidStateForRequest)?;
		let new_nodes_map = self.new_nodes_map.take().ok_or(Error::InvalidStateForRequest)?;
		let share_add_session = ShareAddSessionImpl::new(ShareAddSessionParams {
			meta: self.meta.clone(),
			nonce: self.nonce,
			transport: ShareChangeTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			admin_public: None,
		})?;
		share_add_session.set_consensus_output(&self.key_version, consensus_group, new_nodes_map)?;
		self.share_add_session = Some(share_add_session);
		Ok(())
	}

	/// Proceed to the next state.
	fn proceed_to_next_state(&mut self) -> Result<(), Error> {
		if self.meta.self_node_id != self.meta.master_node_id {
			if self.new_nodes_map.is_none() && self.nodes_to_remove.is_none() {
				self.is_finished = true;
			}
			return Ok(());
		}

		if self.new_nodes_map.is_some() {
			self.create_share_add_session()?;
			return self.share_add_session.as_ref()
				.expect("either create_share_add_session fails, or session is created; qed")
				.initialize(None, None, None, None);
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
	fn nodes(&self) -> BTreeSet<NodeId> {
		self.cluster.nodes()
	}

	fn set_master_data(&mut self, _consensus_group: BTreeSet<NodeId>, _id_numbers: BTreeMap<NodeId, Option<Secret>>) {
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

/// Prepare share change plan for moving from old `session_nodes` to `new_nodes_set`.
pub fn prepare_share_change_session_plan(threshold: usize, key_version: H256, master: &NodeId, non_isolated_nodes: &BTreeSet<NodeId>, old_nodes_set: &BTreeSet<NodeId>, new_nodes_set: &BTreeSet<NodeId>) -> Result<ShareChangeSessionPlan, Error> {
	let mut consensus_group: BTreeSet<_> = ::std::iter::once(master.clone())
		.chain(old_nodes_set.iter()
			.filter(|n| *n != master && non_isolated_nodes.contains(*n))
			.take(threshold)
			.cloned())
		.collect();

	let nodes_to_add = new_nodes_set.difference(&old_nodes_set).cloned();
	let new_nodes_map = old_nodes_set.iter()
		.filter(|n| non_isolated_nodes.contains(n))
		.map(|n| Ok((n.clone(), None)))
		.chain(nodes_to_add.map(|n| math::generate_random_scalar().map(|id| (n, Some(id)))))
		.collect::<Result<BTreeMap<_, _>, _>>()?;

	let mut nodes_to_remove: BTreeSet<_> = old_nodes_set.difference(&new_nodes_set)
		.filter(|n| non_isolated_nodes.contains(n))
		.cloned()
		.collect();

	Ok(ShareChangeSessionPlan {
		key_version: key_version,
		consensus_group: consensus_group,
		new_nodes_map: new_nodes_map,
		nodes_to_remove: nodes_to_remove,
	})
}

impl ShareChangeSessionPlan {
	/// Is empty (nothing-to-do) plan?
	pub fn is_empty(&self) -> bool {
		self.new_nodes_map.is_empty()
			&& self.nodes_to_remove.is_empty()
	}
}
