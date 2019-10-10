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
use crypto::publickey::{Public, Secret};
use ethereum_types::H256;
use key_server_cluster::{Error, NodeId, DocumentKeyShare};
use key_server_cluster::math;
use key_server_cluster::jobs::job_session::{JobPartialRequestAction, JobPartialResponseAction, JobExecutor};

/// Signing job.
pub struct SchnorrSigningJob {
	/// This node id.
	self_node_id: NodeId,
	/// Key share.
	key_share: DocumentKeyShare,
	/// Key version.
	key_version: H256,
	/// Session public key.
	session_public: Public,
	/// Session secret coefficient.
	session_secret_coeff: Secret,
	/// Request id.
	request_id: Option<Secret>,
	/// Message hash.
	message_hash: Option<H256>,
}

/// Signing job partial request.
pub struct SchnorrPartialSigningRequest {
	/// Request id.
	pub id: Secret,
	/// Message hash.
	pub message_hash: H256,
	/// Id of other nodes, participating in signing.
	pub other_nodes_ids: BTreeSet<NodeId>,
}

/// Signing job partial response.
#[derive(Clone)]
pub struct SchnorrPartialSigningResponse {
	/// Request id.
	pub request_id: Secret,
	/// Partial signature.
	pub partial_signature: Secret,
}

impl SchnorrSigningJob {
	pub fn new_on_slave(self_node_id: NodeId, key_share: DocumentKeyShare, key_version: H256, session_public: Public, session_secret_coeff: Secret) -> Result<Self, Error> {
		Ok(SchnorrSigningJob {
			self_node_id: self_node_id,
			key_share: key_share,
			key_version: key_version,
			session_public: session_public,
			session_secret_coeff: session_secret_coeff,
			request_id: None,
			message_hash: None,
		})
	}

	pub fn new_on_master(self_node_id: NodeId, key_share: DocumentKeyShare, key_version: H256, session_public: Public, session_secret_coeff: Secret, message_hash: H256) -> Result<Self, Error> {
		Ok(SchnorrSigningJob {
			self_node_id: self_node_id,
			key_share: key_share,
			key_version: key_version,
			session_public: session_public,
			session_secret_coeff: session_secret_coeff,
			request_id: Some(math::generate_random_scalar()?),
			message_hash: Some(message_hash),
		})
	}
}

impl JobExecutor for SchnorrSigningJob {
	type PartialJobRequest = SchnorrPartialSigningRequest;
	type PartialJobResponse = SchnorrPartialSigningResponse;
	type JobResponse = (Secret, Secret);

	fn prepare_partial_request(&self, node: &NodeId, nodes: &BTreeSet<NodeId>) -> Result<SchnorrPartialSigningRequest, Error> {
		debug_assert!(nodes.len() == self.key_share.threshold + 1);

		let request_id = self.request_id.as_ref()
			.expect("prepare_partial_request is only called on master nodes; request_id is filed in constructor on master nodes; qed");
		let message_hash = self.message_hash.as_ref()
			.expect("compute_response is only called on master nodes; message_hash is filed in constructor on master nodes; qed");
		let mut other_nodes_ids = nodes.clone();
		other_nodes_ids.remove(node);

		Ok(SchnorrPartialSigningRequest {
			id: request_id.clone(),
			message_hash: message_hash.clone(),
			other_nodes_ids: other_nodes_ids,
		})
	}

	fn process_partial_request(&mut self, partial_request: SchnorrPartialSigningRequest) -> Result<JobPartialRequestAction<SchnorrPartialSigningResponse>, Error> {
		let key_version = self.key_share.version(&self.key_version)?;
		if partial_request.other_nodes_ids.len() != self.key_share.threshold
			|| partial_request.other_nodes_ids.contains(&self.self_node_id)
			|| partial_request.other_nodes_ids.iter().any(|n| !key_version.id_numbers.contains_key(n)) {
			return Err(Error::InvalidMessage);
		}

		let self_id_number = &key_version.id_numbers[&self.self_node_id];
		let other_id_numbers = partial_request.other_nodes_ids.iter().map(|n| &key_version.id_numbers[n]);
		let combined_hash = math::combine_message_hash_with_public(&partial_request.message_hash, &self.session_public)?;
		Ok(JobPartialRequestAction::Respond(SchnorrPartialSigningResponse {
			request_id: partial_request.id,
			partial_signature: math::compute_schnorr_signature_share(
				self.key_share.threshold,
				&combined_hash,
				&self.session_secret_coeff,
				&key_version.secret_share,
				self_id_number,
				other_id_numbers
			)?,
		}))
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &SchnorrPartialSigningResponse) -> Result<JobPartialResponseAction, Error> {
		if Some(&partial_response.request_id) != self.request_id.as_ref() {
			return Ok(JobPartialResponseAction::Ignore);
		}
		// TODO [Trust]: check_schnorr_signature_share()

		Ok(JobPartialResponseAction::Accept)
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, SchnorrPartialSigningResponse>) -> Result<(Secret, Secret), Error> {
		let message_hash = self.message_hash.as_ref()
			.expect("compute_response is only called on master nodes; message_hash is filed in constructor on master nodes; qed");

		let signature_c = math::combine_message_hash_with_public(message_hash, &self.session_public)?;
		let signature_s = math::compute_schnorr_signature(partial_responses.values().map(|r| &r.partial_signature))?;

		Ok((signature_c, signature_s))
	}
}
