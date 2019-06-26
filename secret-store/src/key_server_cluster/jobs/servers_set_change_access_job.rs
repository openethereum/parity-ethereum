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

use std::collections::{BTreeSet, BTreeMap};
use ethkey::{Public, Signature, recover};
use tiny_keccak::Keccak;
use key_server_cluster::{Error, NodeId, SessionId};
use key_server_cluster::message::{InitializeConsensusSessionWithServersSet, InitializeConsensusSessionOfShareAdd};
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobPartialRequestAction, JobExecutor};

/// Purpose of this job is to check if requestor is administrator of SecretStore (i.e. it have access to change key servers set).
pub struct ServersSetChangeAccessJob {
	/// Servers set administrator public key (this could be changed to ACL-based check later).
	administrator: Public,
	/// Old servers set.
	old_servers_set: Option<BTreeSet<NodeId>>,
	/// New servers set.
	new_servers_set: Option<BTreeSet<NodeId>>,
	/// Old servers set, signed by requester.
	old_set_signature: Option<Signature>,
	/// New servers set, signed by requester.
	new_set_signature: Option<Signature>,
}

/// Servers set change job partial request.
pub struct ServersSetChangeAccessRequest {
	/// Old servers set.
	pub old_servers_set: BTreeSet<NodeId>,
	/// New servers set.
	pub new_servers_set: BTreeSet<NodeId>,
	/// Hash(old_servers_set), signed by requester.
	pub old_set_signature: Signature,
	/// Hash(new_servers_set), signed by requester.
	pub new_set_signature: Signature,
}

impl<'a> From<&'a InitializeConsensusSessionWithServersSet> for ServersSetChangeAccessRequest {
	fn from(message: &InitializeConsensusSessionWithServersSet) -> Self {
		ServersSetChangeAccessRequest {
			old_servers_set: message.old_nodes_set.iter().cloned().map(Into::into).collect(),
			new_servers_set: message.new_nodes_set.iter().cloned().map(Into::into).collect(),
			old_set_signature: message.old_set_signature.clone().into(),
			new_set_signature: message.new_set_signature.clone().into(),
		}
	}
}

impl<'a> From<&'a InitializeConsensusSessionOfShareAdd> for ServersSetChangeAccessRequest {
	fn from(message: &InitializeConsensusSessionOfShareAdd) -> Self {
		ServersSetChangeAccessRequest {
			old_servers_set: message.old_nodes_set.iter().cloned().map(Into::into).collect(),
			new_servers_set: message.new_nodes_map.keys().cloned().map(Into::into).collect(),
			old_set_signature: message.old_set_signature.clone().into(),
			new_set_signature: message.new_set_signature.clone().into(),
		}
	}
}

impl ServersSetChangeAccessJob {
	pub fn new_on_slave(administrator: Public) -> Self {
		ServersSetChangeAccessJob {
			administrator: administrator,
			old_servers_set: None,
			new_servers_set: None,
			old_set_signature: None,
			new_set_signature: None,
		}
	}

	pub fn new_on_master(administrator: Public, old_servers_set: BTreeSet<NodeId>, new_servers_set: BTreeSet<NodeId>, old_set_signature: Signature, new_set_signature: Signature) -> Self {
		ServersSetChangeAccessJob {
			administrator: administrator,
			old_servers_set: Some(old_servers_set),
			new_servers_set: Some(new_servers_set),
			old_set_signature: Some(old_set_signature),
			new_set_signature: Some(new_set_signature),
		}
	}

	pub fn new_servers_set(&self) -> Option<&BTreeSet<NodeId>> {
		self.new_servers_set.as_ref()
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

	fn process_partial_request(&mut self, partial_request: ServersSetChangeAccessRequest) -> Result<JobPartialRequestAction<bool>, Error> {
		let ServersSetChangeAccessRequest {
			old_servers_set,
			new_servers_set,
			old_set_signature,
			new_set_signature,
		} = partial_request;

		// check old servers set signature
		let old_actual_public = recover(&old_set_signature, &ordered_nodes_hash(&old_servers_set).into())?;
		let new_actual_public = recover(&new_set_signature, &ordered_nodes_hash(&new_servers_set).into())?;
		let is_administrator = old_actual_public == self.administrator && new_actual_public == self.administrator;
		self.new_servers_set = Some(new_servers_set);

		Ok(if is_administrator { JobPartialRequestAction::Respond(true) } else { JobPartialRequestAction::Reject(false) })
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &bool) -> Result<JobPartialResponseAction, Error> {
		Ok(if *partial_response { JobPartialResponseAction::Accept } else { JobPartialResponseAction::Reject })
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, bool>) -> Result<BTreeSet<NodeId>, Error> {
		Ok(partial_responses.keys().cloned().collect())
	}
}

pub fn ordered_nodes_hash(nodes: &BTreeSet<NodeId>) -> SessionId {
	let mut nodes_keccak = Keccak::new_keccak256();
	for node in nodes {
		nodes_keccak.update(node.as_bytes());
	}

	let mut nodes_keccak_value = [0u8; 32];
	nodes_keccak.finalize(&mut nodes_keccak_value);

	nodes_keccak_value.into()
}
