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
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::message::{Message, ServersSetChangeMessage, ServersSetChangeShareAddMessage};
use key_server_cluster::share_add_session::{SessionTransport as ShareAddSessionTransport,
	SessionImpl as ShareAddSessionImpl, SessionParams as ShareAddSessionParams};
use key_server_cluster::message::{ShareAddMessage, ShareMoveMessage, ShareRemoveMessage};

/// Single session meta-change session. Brief overview:
/// 1) new shares are added to the session
/// 2) shares are moved between nodes
/// 3) shares are removed from nodes 
pub struct ShareChangeSession {
	/// Servers set change session id.
	session_id: SessionId,
	/// Session nonce.
	nonce: u64,
	/// Session (key) id.
	key_id: SessionId,
	/// This node id.
	self_node_id: NodeId,
	/// Master node id.
	master_node_id: NodeId,
	/// Document key share.
	document_key_share: Option<DocumentKeyShare>,
	/// Cluster.
	cluster: Arc<Cluster>,
	/// Key storage.
	key_storage: Arc<KeyStorage>,
	/// Nodes to add to session.
	nodes_to_add: Option<BTreeSet<NodeId>>,
	/// Nodes to remove from session.
	nodes_to_remove: Option<BTreeSet<NodeId>>,
	/// Nodes to move.
	nodes_to_move: Option<BTreeMap<NodeId, NodeId>>,
	/// Share add session.
	share_add_session: Option<ShareAddSessionImpl<ShareAddTransport>>,
}

/// Session parameters.
pub struct ShareChangeSessionParams {
	/// Servers set change session id.
	pub session_id: SessionId,
	/// Session nonce.
	pub nonce: u64,
	/// Session (key) id.
	pub key_id: SessionId,
	/// This node id.
	pub self_node_id: NodeId,
	/// Master node id.
	pub master_node_id: NodeId,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Keys storage.
	pub key_storage: Arc<KeyStorage>,
	/// Nodes to add shares for.
	pub nodes_to_add: Option<BTreeSet<NodeId>>,
	/// Nodes to move shares from/to.
	pub nodes_to_move: Option<BTreeMap<NodeId, NodeId>>,
	/// Nodes to remove shares from.
	pub nodes_to_remove: Option<BTreeSet<NodeId>>,
}

/// Share add session transport.
struct ShareAddTransport {
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
		Ok(ShareChangeSession {
			session_id: params.session_id,
			nonce: params.nonce,
			key_id: params.key_id,
			self_node_id: params.self_node_id,
			master_node_id: params.master_node_id,
			document_key_share: params.key_storage
				.get(&params.key_id)
				.ok(),
			cluster: params.cluster,
			key_storage: params.key_storage,
			nodes_to_add: params.nodes_to_add,
			nodes_to_remove: params.nodes_to_remove,
			nodes_to_move: params.nodes_to_move,
			share_add_session: None,
		})
	}

	/// Initialize session (on master node).
	pub fn initialize(&mut self) -> Result<(), Error> {
		self.proceed_to_next_state(&math::generate_random_scalar()?)
	}

	/// When share-add message is received.
	pub fn on_share_add_message(&mut self, sender: &NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		if let &ShareAddMessage::InitializeShareAddSession(ref message) = message {
			if self.share_add_session.is_some() {
				return Err(Error::InvalidMessage);
			}
			if sender != &self.master_node_id {
				return Err(Error::InvalidMessage);
			}

			self.create_share_add_session(&message.sub_session.clone().into())?;
		}

		let change_state_needed = self.share_add_session.as_ref()
			.map(|share_add_session| {
				let was_finished = share_add_session.is_finished();
				share_add_session.process_message(sender, message)
					.map(|_| share_add_session.is_finished() && !was_finished)
			})
			.unwrap_or(Err(Error::InvalidMessage))?;
		if change_state_needed && self.self_node_id == self.master_node_id {
			self.proceed_to_next_state(&math::generate_random_scalar()?)?;
		}

		Ok(())
	}

	/// When share-move message is received.
	pub fn on_share_move_message(&mut self, _sender: &NodeId, _message: &ShareMoveMessage) -> Result<(), Error> {
		unimplemented!()
	}

	/// When share-remove message is received.
	pub fn on_share_remove_message(&mut self, _sender: &NodeId, _message: &ShareRemoveMessage) -> Result<(), Error> {
		unimplemented!()
	}

	/// Create new share add session.
	fn create_share_add_session(&mut self, sub_session: &Secret) -> Result<(), Error> {
		self.share_add_session = Some(ShareAddSessionImpl::new_nested(ShareAddSessionParams {
			meta: SessionMeta {
				id: self.key_id.clone(),
				threshold: 0,
				self_node_id: self.self_node_id.clone(),
				master_node_id: self.master_node_id.clone(),
			},
			nonce: 0,
			sub_session: sub_session.clone(),
			transport: ShareAddTransport::new(self.session_id, self.nonce, self.cluster.clone()),
			key_storage: self.key_storage.clone(),
			key_share: self.document_key_share.clone(),
		})?);
		Ok(())
	}

	/// Proceed to the next state (on master node).
	fn proceed_to_next_state(&mut self, sub_session: &Secret) -> Result<(), Error> {
		if self.self_node_id != self.master_node_id {
			return Ok(());
		}

		if let Some(nodes_to_add) = self.nodes_to_add.take() {
			self.create_share_add_session(sub_session);
			return self.share_add_session.as_ref().expect("TODO").initialize(nodes_to_add);
		}

		/*if self.state == State::AddShares {
			self.state = State::MoveShares;
			if !self.nodes_to_move.is_empty() {
				self.share_move_session = Some(ShareMoveSessionImpl::new(self.key_share.clone(), self.nodes_to_move.clone()));
				return;
			}
		}

		if self.state == State::MoveShares {
			self.state = State::RemoveShares;
			if !self.nodes_to_remove.is_empty() {
				self.share_remove_session = Some(ShareRemoveSessionImpl::new(self.key_share.clone(), self.nodes_to_remove.clone()));
				return;
			}
		}*/

		Ok(())
	}
}

impl ShareAddTransport {
	pub fn new(session_id: SessionId, nonce: u64, cluster: Arc<Cluster>) -> Self {
		ShareAddTransport {
			session_id: session_id,
			nonce: nonce,
			cluster: cluster,
		}
	}
}

impl ShareAddSessionTransport for ShareAddTransport {
	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareAddMessage(ServersSetChangeShareAddMessage {
			session: self.session_id.clone().into(),
			session_nonce: self.nonce,
			message: message,
		})))
	}
}
