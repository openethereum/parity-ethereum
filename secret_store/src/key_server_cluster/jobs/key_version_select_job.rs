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
use key_server_cluster::{Error, NodeId, SessionId, KeyStorage, DocumentKeyShare};
use key_server_cluster::jobs::job_session::{JobPartialRequestAction, JobPartialResponseAction, JobExecutor};

/// Key version select job.
pub struct KeyVersionSelectJob {
	/// Target node id.
	target_node_id: Option<NodeId>,
	/// Key share.
	key_share: Option<DocumentKeyShare>,
	/// Received partial responses.
	versions: Option<BTreeMap<H256, BTreeSet<NodeId>>>,
}

/// Selected key version.
pub enum SelectedKeyVersion {
	/// Key version on this node.
	LocalKeyVersion(H256),
	/// Key version on external node.
	ExternalKeyVersion(NodeId, H256),
}

impl KeyVersionSelectJob {
	pub fn new_on_slave(key_share: Option<DocumentKeyShare>) -> Self {
		KeyVersionSelectJob {
			target_node_id: None,
			key_share: key_share,
			versions: None,
		}
	}

	pub fn new_on_master(key_share: Option<DocumentKeyShare>, self_node_id: NodeId) -> Self {
		KeyVersionSelectJob {
			target_node_id: Some(self_node_id),
			versions: Some(key_share.as_ref()
				.map(|ks| ks.versions.iter().map(|v| (v.hash.clone(), BTreeSet::new())).collect())
				.unwrap_or_default()),
			key_share: key_share,
		}
	}
}

impl JobExecutor for KeyVersionSelectJob {
	type PartialJobRequest = NodeId;
	type PartialJobResponse = Vec<H256>;
	type JobResponse = BTreeMap<SessionId, BTreeSet<NodeId>>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<NodeId, Error> {
		Ok(self.target_node_id.clone().expect("prepare_partial_request is only called on master nodes; this field is filled on master nodes in constructor; qed"))
	}

	fn process_partial_request(&mut self, partial_request: NodeId) -> Result<JobPartialRequestAction<Vec<H256>>, Error> {
		// versions sort order:
		// 1) versions with requester included
		// 2) versions without requester
		let key_share = self.key_share.as_ref().expect("TODO");
		let key_versions: Vec<_> = key_share.versions.iter().rev()
			.filter(|v| v.id_numbers.contains_key(&partial_request))
			.chain(key_share.versions.iter().rev().filter(|v| !v.id_numbers.contains_key(&partial_request)))
			.map(|v| v.hash.clone())
			.take(10)
			.collect();

		Ok(JobPartialRequestAction::Respond(key_versions))
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &Vec<H256>) -> Result<JobPartialResponseAction, Error> {
		Ok(JobPartialResponseAction::Accept)
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, Vec<H256>>) -> Result<BTreeMap<SessionId, BTreeSet<NodeId>>, Error> {
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
