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
use key_server_cluster::{Error, NodeId, SessionId, Requester, AclStorage};
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobPartialRequestAction, JobExecutor};

/// Purpose of this job is to construct set of nodes, which have agreed to provide access to the given key for the given requestor.
pub struct KeyAccessJob {
	/// Key id.
	id: SessionId,
	/// Has key share?
	has_key_share: bool,
	/// ACL storage.
	acl_storage: Arc<dyn AclStorage>,
	/// Requester data.
	requester: Option<Requester>,
}

impl KeyAccessJob {
	pub fn new_on_slave(id: SessionId, acl_storage: Arc<dyn AclStorage>) -> Self {
		KeyAccessJob {
			id: id,
			has_key_share: true,
			acl_storage: acl_storage,
			requester: None,
		}
	}

	pub fn new_on_master(id: SessionId, acl_storage: Arc<dyn AclStorage>, requester: Requester) -> Self {
		KeyAccessJob {
			id: id,
			has_key_share: true,
			acl_storage: acl_storage,
			requester: Some(requester),
		}
	}

	pub fn set_has_key_share(&mut self, has_key_share: bool) {
		self.has_key_share = has_key_share;
	}

	pub fn set_requester(&mut self, requester: Requester) {
		self.requester = Some(requester);
	}

	pub fn requester(&self) -> Option<&Requester> {
		self.requester.as_ref()
	}
}

impl JobExecutor for KeyAccessJob {
	type PartialJobRequest = Requester;
	type PartialJobResponse = bool;
	type JobResponse = BTreeSet<NodeId>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<Requester, Error> {
		Ok(self.requester.as_ref().expect("prepare_partial_request is only called on master nodes; new_on_master fills the signature; qed").clone())
	}

	fn process_partial_request(&mut self, partial_request: Requester) -> Result<JobPartialRequestAction<bool>, Error> {
		if !self.has_key_share {
			return Ok(JobPartialRequestAction::Reject(false));
		}

		self.requester = Some(partial_request.clone());
		self.acl_storage.check(partial_request.address(&self.id).map_err(Error::InsufficientRequesterData)?, &self.id)
			.map(|is_confirmed| if is_confirmed { JobPartialRequestAction::Respond(true) } else { JobPartialRequestAction::Reject(false) })
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &bool) -> Result<JobPartialResponseAction, Error> {
		Ok(if *partial_response { JobPartialResponseAction::Accept } else { JobPartialResponseAction::Reject })
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, bool>) -> Result<BTreeSet<NodeId>, Error> {
		Ok(partial_responses.keys().cloned().collect())
	}
}
