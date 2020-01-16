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
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage};
use key_server_cluster::jobs::job_session::{JobPartialRequestAction, JobPartialResponseAction, JobExecutor};

/// Unknown sessions report job.
pub struct UnknownSessionsJob {
	/// Target node id.
	target_node_id: Option<NodeId>,
	/// Keys storage.
	key_storage: Arc<dyn KeyStorage>,
}

impl UnknownSessionsJob {
	pub fn new_on_slave(key_storage: Arc<dyn KeyStorage>) -> Self {
		UnknownSessionsJob {
			target_node_id: None,
			key_storage: key_storage,
		}
	}

	pub fn new_on_master(key_storage: Arc<dyn KeyStorage>, self_node_id: NodeId) -> Self {
		UnknownSessionsJob {
			target_node_id: Some(self_node_id),
			key_storage: key_storage,
		}
	}
}

impl JobExecutor for UnknownSessionsJob {
	type PartialJobRequest = NodeId;
	type PartialJobResponse = BTreeSet<SessionId>;
	type JobResponse = BTreeMap<SessionId, BTreeSet<NodeId>>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<NodeId, Error> {
		Ok(self.target_node_id.clone().expect("prepare_partial_request is only called on master nodes; this field is filled on master nodes in constructor; qed"))
	}

	fn process_partial_request(&mut self, partial_request: NodeId) -> Result<JobPartialRequestAction<BTreeSet<SessionId>>, Error> {
		Ok(JobPartialRequestAction::Respond(self.key_storage.iter()
			.filter(|&(_, ref key_share)| !key_share.versions.last().map(|v| v.id_numbers.contains_key(&partial_request)).unwrap_or(true))
			.map(|(id, _)| id.clone())
			.collect()))
	}

	fn check_partial_response(&mut self, _sender: &NodeId, _partial_response: &BTreeSet<SessionId>) -> Result<JobPartialResponseAction, Error> {
		Ok(JobPartialResponseAction::Accept)
	}

	// TODO [Opt]:
	// currently ALL unknown sessions are sent at once - it is better to limit messages by size/len => add partial-partial responses
	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, BTreeSet<SessionId>>) -> Result<BTreeMap<SessionId, BTreeSet<NodeId>>, Error> {
		let mut result: BTreeMap<SessionId, BTreeSet<NodeId>> = BTreeMap::new();
		for (node_id, node_sessions) in partial_responses {
			for node_session in node_sessions {
				result.entry(node_session.clone())
					.or_insert_with(Default::default)
					.insert(node_id.clone());
			}
		}

		Ok(result)
	}
}
