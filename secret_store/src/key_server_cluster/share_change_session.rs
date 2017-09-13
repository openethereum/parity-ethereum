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

use key_server_cluster::{Error, NodeId};
use key_server_cluster::message::{ShareAddMessage, ShareMoveMessage, ShareRemoveMessage};

/// Single session meta-change session. Brief overview:
/// 1) new shares are added to the session
/// 2) shares are moved between nodes
/// 3) shares are removed from nodes 
pub struct ShareChangeSession {
	/// Nodes to add to session.
	nodes_to_add: BTreeSet<NodeId>,
	/// Nodes to remove from session.
	nodes_to_remove: BTreeSet<NodeId>,
	/// Nodes to move.
	nodes_to_move: BTreeMap<NodeId, NodeId>,
}

/// Session state.
enum State {
	/// Initialization state.
	Initialization,
	/// New nodes are adding to the session.
	AddShares,
	/// Shares from nodes are moving to other nodes.
	MoveShares,
	/// Existing nodes are removed from the session.
	RemoveShares,
	/// Finished.
	Finished,
}

impl ShareChangeSession {
	/// When share-add message is received.
	pub fn on_share_add_message(&mut self, sender: NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		self.proceed_to_next_state(self.share_add_session.as_ref()
			.map(|share_add_session| {
				let was_finished = share_add_session.is_finished();
				share_add_session.process_message(sender, message)
					.map(|_| share_add_session.is_finished() && !was_finished))
			.unwrap_or(Err(Error::InvalidMessage))?)
	}

	/// When share-move message is received.
	pub fn on_share_move_message(&mut self, sender: NodeId, message: &ShareMoveMessage) -> Result<(), Error> {
		self.proceed_to_next_state(self.share_move_session.as_ref()
			.map(|share_move_session| {
				let was_finished = share_move_session.is_finished();
				share_move_session.process_message(sender, message)
					.map(|_| share_move_session.is_finished() && !was_finished))
			.unwrap_or(Err(Error::InvalidMessage))?)
	}

	/// When share-remove message is received.
	pub fn on_share_remove_message(&mut self, sender: NodeId, message: &ShareRemoveMessage) -> Result<(), Error> {
		self.proceed_to_next_state(self.share_remove_session.as_ref()
			.map(|share_remove_session| {
				let was_finished = share_remove_session.is_finished();
				share_remove_session.process_message(sender, message)
					.map(|_| share_remove_session.is_finished() && !was_finished))
			.unwrap_or(Err(Error::InvalidMessage))?)
	}

	/// Proceed to the next state.
	fn proceed_to_next_state(&mut self, need_proceed: bool) -> Result<(), Error> {
		if !need_proceed {
			return Ok(());
		}

		if self.state == State::Initialization {
			self.state = State::AddShares;
			if !self.nodes_to_add.is_empty() {
				self.share_add_session = Some(ShareAddSessionImpl::new(self.key_share.clone(), self.nodes_to_add.clone()));
				return;
			}
		}

		if self.state == State::AddShares {
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
		}

		self.state = State::Finished;

		Ok(())
	}
}
