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
use hash::keccak_buffer;
use tiny_keccak::{keccak256, Keccak};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage};
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobPartialRequestAction, JobExecutor};

/// Purpose of this job is to check if requestor is administrator of SecretStore (i.e. it have access to change key servers set).
pub struct ServersSetChangeAccessJob {
	/// Servers set administrator public key (this could be changed to ACL-based check later).
	administrator: Public,
	/// Current servers set (in session/cluster).
	current_servers_set: BTreeSet<NodeId>,
	/// Old servers set.
	old_servers_set: Option<BTreeSet<NodeId>>
	/// New servers set.
	servers_set: Option<BTreeSet<NodeId>>,
	/// Requester signature.
	signature: Option<Signature>,
}

/// Servers set change job partial request.
pub struct ServersSetChangeAccessRequest {
	/// Session id.
	session_id: SessionId,
	/// Old servers set.
	old_servers_set: BTreeSet<NodeId>,
	/// New servers set.
	new_servers_set: BTreeSet<NodeId>,
	/// Hash(old_servers_set), signed by requester.
	old_set_signature: Signature,
	/// Hash(new_servers_set), signed by requester.
	new_set_signature: Signature,
}

impl ServersSetChangeAccessJob {
	pub fn new_on_slave(administrator: Public, current_servers_set: BTreeSet<NodeId>) -> Self {
		ServersSetChangeAccessJob {
			administrator: administrator,
			current_servers_set: current_servers_set,
			old_servers_set: None,
			new_servers_set: None,
			old_set_signature: None,
			new_set_signature: None,
		}
	}

	pub fn new_on_master(administrator: Public, current_servers_set: BTreeSet<NodeId>, old_servers_set: BTreeSet<NodeId>, new_servers_set: BTreeSet<NodeId>, old_set_signature: Signature, new_set_signature: Signature) -> Self {
		ServersSetChangeAccessJob {
			administrator: administrator,
			current_servers_set: current_servers_set,
			old_servers_set: Some(old_servers_set),
			new_servers_set: Some(new_servers_set),
			old_set_signature: Some(old_set_signature),
			new_set_signature: Some(new_set_signature),
		}
	}
}

impl JobExecutor for ServersSetChangeAccessJob {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;
	type JobResponse = BTreeSet<NodeId>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<ServersSetChangeAccessRequest, Error> {
		let explanation = "prepare_partial_request is only called on master nodes; this field is filled on master nodes in constructor; qed";
		Ok(ServersSetChangeAccessRequest {
			old_servers_set: self.old_servers_set.clone().expect(explanation),
			new_servers_set: self.new_servers_set.clone().expect(explanation),
			old_set_signature: self.old_set_signature.clone().expect(explanation),
			new_set_signature: self.new_set_signature.clone().expect(explanation),
		})
	}

	fn process_partial_request(&self, partial_request: ServersSetChangeAccessRequest) -> Result<JobPartialRequestAction<bool>, Error> {
		let ServersSetChangeAccessRequest {
			old_servers_set: old_servers_set,
			new_servers_set: new_servers_set,
			old_set_signature: old_set_signature
			new_set_signature: new_set_signature,
		} = partial_request;

		// check that current set is exactly the same set as old set
		if self.current_servers_set.symmetric_difference(&old_servers_set).next().is_some() {
			return Ok(JobPartialResponseAction::Reject(false));
		}

		let mut new_servers_set_keccak = Keccak::new_keccak256();
		for new_server in new_servers_set {
			new_servers_set_keccak.update(&*new_server);
		}

		let mut new_servers_set_keccak_value = [0u8; 32];
		new_servers_set_keccak.finalize(&mut new_servers_set_keccak_value);

		let actual_public = recover(&signature, &new_servers_set_keccak_value.into())?;
		let is_administrator_signature = actual_public == self.administrator;
		Ok(if is_administrator_signature { JobPartialRequestAction::Respond(true) } else { JobPartialRequestAction::Reject(false) })
	}

	fn check_partial_response(&self, partial_response: &bool) -> Result<JobPartialResponseAction, Error> {
		Ok(if *partial_response { JobPartialResponseAction::Accept } else { JobPartialResponseAction::Reject })
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, bool>) -> Result<BTreeSet<NodeId>, Error> {
		Ok(partial_responses.keys().cloned().collect())
	}
}
