// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

pub mod key_version_negotiation_session;
pub mod servers_set_change_session;
pub mod share_add_session;
pub mod share_change_session;

mod sessions_queue;

use key_server_cluster::{SessionId, NodeId, SessionMeta, Error};

/// Share change session metadata.
#[derive(Debug, Clone)]
pub struct ShareChangeSessionMeta {
	/// Key id.
	pub id: SessionId,
	/// Id of node, which has started this session.
	pub master_node_id: NodeId,
	/// Id of node, on which this session is running.
	pub self_node_id: NodeId,
	/// Count of all configured key server nodes.
	pub configured_nodes_count: usize,
	/// Count of all connected key server nodes.
	pub connected_nodes_count: usize,
}

impl ShareChangeSessionMeta {
	/// Convert to consensus session meta. `all_nodes_set` is the union of `old_nodes_set` && `new_nodes_set`.
	pub fn into_consensus_meta(self, all_nodes_set_len: usize) -> Result<SessionMeta, Error> {
		Ok(SessionMeta {
			id: self.id,
			master_node_id: self.master_node_id,
			self_node_id: self.self_node_id,
			threshold: all_nodes_set_len.checked_sub(1).ok_or(Error::ConsensusUnreachable)?,
			configured_nodes_count: self.configured_nodes_count,
			connected_nodes_count: self.connected_nodes_count,
		})
	}
}
