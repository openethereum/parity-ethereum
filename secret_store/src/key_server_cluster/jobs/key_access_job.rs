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
use ethkey::{Public, Signature, recover};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage};
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobPartialRequestAction, JobExecutor};

/// Purpose of this job is to construct set of nodes, which have agreed to provide access to the given key for the given requestor.
pub struct KeyAccessJob {
	/// Key id.
	id: SessionId,
	/// ACL storage.
	acl_storage: Arc<AclStorage>,
	/// Requester signature.
	signature: Option<Signature>,
}

impl KeyAccessJob {
	pub fn new_on_slave(id: SessionId, acl_storage: Arc<AclStorage>) -> Self {
		KeyAccessJob {
			id: id,
			acl_storage: acl_storage,
			signature: None,
		}
	}

	pub fn new_on_master(id: SessionId, acl_storage: Arc<AclStorage>, signature: Signature) -> Self {
		KeyAccessJob {
			id: id,
			acl_storage: acl_storage,
			signature: Some(signature),
		}
	}

	pub fn requester(&self) -> Result<Option<Public>, Error> {
		match self.signature.as_ref() {
			Some(signature) => Ok(Some(recover(signature, &self.id)?)),
			None => Ok(None),
		}
	}
}

impl JobExecutor for KeyAccessJob {
	type PartialJobRequest = Signature;
	type PartialJobResponse = bool;
	type JobResponse = BTreeSet<NodeId>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<Signature, Error> {
		Ok(self.signature.as_ref().expect("prepare_partial_request is only called on master nodes; new_on_master fills the signature; qed").clone())
	}

	fn process_partial_request(&mut self, partial_request: Signature) -> Result<JobPartialRequestAction<bool>, Error> {
		self.signature = Some(partial_request.clone());
		self.acl_storage.check(&recover(&partial_request, &self.id)?, &self.id)
			.map_err(|_| Error::AccessDenied)
			.map(|is_confirmed| if is_confirmed { JobPartialRequestAction::Respond(true) } else { JobPartialRequestAction::Reject(false) })
	}

	fn check_partial_response(&self, partial_response: &bool) -> Result<JobPartialResponseAction, Error> {
		Ok(if *partial_response { JobPartialResponseAction::Accept } else { JobPartialResponseAction::Reject })
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, bool>) -> Result<BTreeSet<NodeId>, Error> {
		Ok(partial_responses.keys().cloned().collect())
	}
}
