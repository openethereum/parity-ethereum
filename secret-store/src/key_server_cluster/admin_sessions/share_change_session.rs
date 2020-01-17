// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::collections::{BTreeSet, BTreeMap};
use ethereum_types::H256;
use crypto::publickey::Secret;
use key_server_cluster::{Error, NodeId, SessionId, ServerKeyId, KeyStorage};
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
	cluster: Arc<dyn Cluster>,
	/// Key storage.
	key_storage: Arc<dyn KeyStorage>,
	/// Key version.
	key_version: H256,
	/// Nodes that have reported version ownership.
	version_holders: Option<BTreeSet<NodeId>>,
	/// Consensus group to use in ShareAdd session.
	consensus_group: Option<BTreeSet<NodeId>>,
	/// Nodes to add shares for.
	new_nodes_map: Option<BTreeMap<NodeId, Option<Secret>>>,
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
	/// Nodes that have reported version ownership.
	pub version_holders: BTreeSet<NodeId>,
	/// Consensus group to use in ShareAdd session.
	pub consensus_group: BTreeSet<NodeId>,
	/// Nodes to add shares for.
	pub new_nodes_map: BTreeMap<NodeId, Option<Secret>>,
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
	pub cluster: Arc<dyn Cluster>,
	/// Keys storage.
	pub key_storage: Arc<dyn KeyStorage>,
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
	cluster: Arc<dyn Cluster>,
}

impl ShareChangeSession {
	/// Create new share change session.
	pub fn new(params: ShareChangeSessionParams) -> Result<Self, Error> {
		// we can't create sessions right now, because key share is read when session is created, but it can change in previous session
		let key_version = params.plan.key_version;
		let consensus_group = if !params.plan.consensus_group.is_empty() { Some(params.plan.consensus_group) } else { None };
		let version_holders = if !params.plan.version_holders.is_empty() { Some(params.plan.version_holders) } else { None };
		let new_nodes_map = if !params.plan.new_nodes_map.is_empty() { Some(params.plan.new_nodes_map) } else { None };
		debug_assert!(new_nodes_map.is_some());

		let is_finished = new_nodes_map.is_none();
		Ok(ShareChangeSession {
			session_id: params.session_id,
			nonce: params.nonce,
			meta: params.meta,
			cluster: params.cluster,
			key_storage: params.key_storage,
			key_version: key_version,
			version_holders: version_holders,
			consensus_group: consensus_group,
			new_nodes_map: new_nodes_map,
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
		let version_holders = self.version_holders.take().ok_or(Error::InvalidStateForRequest)?;
		let new_nodes_map = self.new_nodes_map.take().ok_or(Error::InvalidStateForRequest)?;
		let (share_add_session, _) = ShareAddSessionImpl::new(ShareAddSessionParams {
			meta: self.meta.clone(),
			nonce: self.nonce,
			transport: ShareChangeTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			admin_public: None,
		})?;
		share_add_session.set_consensus_output(&self.key_version, consensus_group, version_holders, new_nodes_map)?;
		self.share_add_session = Some(share_add_session);
		Ok(())
	}

	/// Proceed to the next state.
	fn proceed_to_next_state(&mut self) -> Result<(), Error> {
		if self.meta.self_node_id != self.meta.master_node_id {
			if self.new_nodes_map.is_none() {
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
	pub fn new(session_id: SessionId, nonce: u64, cluster: Arc<dyn Cluster>) -> Self {
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

	fn set_master_data(&mut self, _consensus_group: BTreeSet<NodeId>, _version_holders: BTreeSet<NodeId>, _id_numbers: BTreeMap<NodeId, Option<Secret>>) {
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

/// Prepare share change plan for moving from old `old_key_version_owners` to `new_nodes_set`.
pub fn prepare_share_change_session_plan(cluster_nodes: &BTreeSet<NodeId>, threshold: usize, key_id: &ServerKeyId, key_version: H256, master: &NodeId, old_key_version_owners: &BTreeSet<NodeId>, new_nodes_set: &BTreeSet<NodeId>) -> Result<ShareChangeSessionPlan, Error> {
	// we can't do anything if there are no enought shares
	if old_key_version_owners.len() < threshold + 1 {
		warn!("cannot add shares to key {} with threshold {}: only {} shares owners are available",
			key_id, threshold, old_key_version_owners.len());
		return Ok(ShareChangeSessionPlan {
			key_version: key_version,
			version_holders: Default::default(),
			consensus_group: Default::default(),
			new_nodes_map: Default::default(),
		});
	}

	// warn if we're loosing the key
	if new_nodes_set.len() < threshold + 1 {
		warn!("losing key {} with threshold {}: only {} nodes left after servers set change session",
			key_id, threshold, new_nodes_set.len());
	}

	// make new nodes map, so that:
	// all non-isolated old nodes will have their id number preserved
	// all new nodes will have new id number
	let mut new_nodes_map = new_nodes_set.difference(&old_key_version_owners)
		.map(|n| math::generate_random_scalar().map(|id| (n.clone(), Some(id))))
		.collect::<Result<BTreeMap<_, _>, _>>()?;
	if !new_nodes_map.is_empty() {
		for old_node in old_key_version_owners.iter().filter(|n| cluster_nodes.contains(n)) {
			new_nodes_map.insert(old_node.clone(), None);
		}
	}

	// select consensus group if there are some nodes to add
	let consensus_group = if !new_nodes_map.is_empty() {
			::std::iter::once(master.clone())
				.chain(old_key_version_owners.iter()
					.filter(|n| *n != master && cluster_nodes.contains(*n))
					.take(threshold)
					.cloned())
				.collect()
		} else {
			BTreeSet::new()
		};

	Ok(ShareChangeSessionPlan {
		key_version: key_version,
		version_holders: old_key_version_owners.clone(),
		consensus_group: consensus_group,
		new_nodes_map: new_nodes_map,
	})
}

impl ShareChangeSessionPlan {
	/// Is empty (nothing-to-do) plan?
	pub fn is_empty(&self) -> bool {
		self.new_nodes_map.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use key_server_cluster::math;
	use super::prepare_share_change_session_plan;

	#[test]
	fn share_change_plan_creates_empty_plan() {
		let cluster_nodes: Vec<_> = (0..3).map(|_| math::generate_random_point().unwrap()).collect();
		let master = cluster_nodes[0].clone();
		let old_key_version_owners = cluster_nodes.iter().cloned().collect();
		let new_nodes_set = cluster_nodes.iter().cloned().collect();
		let plan = prepare_share_change_session_plan(&cluster_nodes.iter().cloned().collect(),
			1, &Default::default(), Default::default(), &master, &old_key_version_owners, &new_nodes_set).unwrap();

		assert!(plan.is_empty());
	}

	#[test]
	fn share_change_plan_adds_new_nodes() {
		let cluster_nodes: Vec<_> = (0..3).map(|_| math::generate_random_point().unwrap()).collect();
		let master = cluster_nodes[0].clone();
		let old_key_version_owners = cluster_nodes[0..2].iter().cloned().collect();
		let new_nodes_set = cluster_nodes.iter().cloned().collect();
		let plan = prepare_share_change_session_plan(&cluster_nodes.iter().cloned().collect(),
			1, &Default::default(), Default::default(), &master, &old_key_version_owners, &new_nodes_set).unwrap();

		assert!(!plan.is_empty());
		assert_eq!(old_key_version_owners, plan.consensus_group);
		assert_eq!(new_nodes_set, plan.new_nodes_map.keys().cloned().collect());
	}
}
