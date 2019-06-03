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
use ethereum_types::H256;
use ethkey::{Public, Secret};
use crypto::DEFAULT_MAC;
use ethkey::crypto::ecies::encrypt;
use key_server_cluster::{Error, NodeId, DocumentKeyShare, EncryptedDocumentKeyShadow};
use key_server_cluster::math;
use key_server_cluster::jobs::job_session::{JobPartialRequestAction, JobPartialResponseAction, JobExecutor};

/// Decryption job.
pub struct DecryptionJob {
	/// This node id.
	self_node_id: NodeId,
	/// Access key.
	access_key: Secret,
	/// Requester public key.
	requester: Public,
	/// Key share.
	key_share: DocumentKeyShare,
	/// Key version.
	key_version: H256,
	/// Request id.
	request_id: Option<Secret>,
	/// Is shadow decryption requested.
	is_shadow_decryption: Option<bool>,
	/// Is broadcast decryption requested.
	is_broadcast_session: Option<bool>,
}

/// Decryption job partial request.
#[derive(Debug)]
pub struct PartialDecryptionRequest {
	/// Request id.
	pub id: Secret,
	/// Is shadow decryption requested.
	pub is_shadow_decryption: bool,
	/// Is broadcast decryption requested.
	pub is_broadcast_session: bool,
	/// Id of other nodes, participating in decryption.
	pub other_nodes_ids: BTreeSet<NodeId>,
}

/// Decryption job partial response.
#[derive(Clone)]
pub struct PartialDecryptionResponse {
	/// Request id.
	pub request_id: Secret,
	/// Shadow point.
	pub shadow_point: Public,
	/// Decryption shadow coefficient, if requested.
	pub decrypt_shadow: Option<Vec<u8>>,
}

impl DecryptionJob {
	pub fn new_on_slave(self_node_id: NodeId, access_key: Secret, requester: Public, key_share: DocumentKeyShare, key_version: H256) -> Result<Self, Error> {
		debug_assert!(key_share.common_point.is_some() && key_share.encrypted_point.is_some());
		Ok(DecryptionJob {
			self_node_id: self_node_id,
			access_key: access_key,
			requester: requester,
			key_share: key_share,
			key_version: key_version,
			request_id: None,
			is_shadow_decryption: None,
			is_broadcast_session: None,
		})
	}

	pub fn new_on_master(self_node_id: NodeId, access_key: Secret, requester: Public, key_share: DocumentKeyShare, key_version: H256, is_shadow_decryption: bool, is_broadcast_session: bool) -> Result<Self, Error> {
		debug_assert!(key_share.common_point.is_some() && key_share.encrypted_point.is_some());
		Ok(DecryptionJob {
			self_node_id: self_node_id,
			access_key: access_key,
			requester: requester,
			key_share: key_share,
			key_version: key_version,
			request_id: Some(math::generate_random_scalar()?),
			is_shadow_decryption: Some(is_shadow_decryption),
			is_broadcast_session: Some(is_broadcast_session),
		})
	}

	pub fn request_id(&self) -> &Option<Secret> {
		&self.request_id
	}

	pub fn set_request_id(&mut self, request_id: Secret) {
		self.request_id = Some(request_id);
	}
}

impl JobExecutor for DecryptionJob {
	type PartialJobRequest = PartialDecryptionRequest;
	type PartialJobResponse = PartialDecryptionResponse;
	type JobResponse = EncryptedDocumentKeyShadow;

	fn prepare_partial_request(&self, node: &NodeId, nodes: &BTreeSet<NodeId>) -> Result<PartialDecryptionRequest, Error> {
		debug_assert!(nodes.len() == self.key_share.threshold + 1);

		let request_id = self.request_id.as_ref()
			.expect("prepare_partial_request is only called on master nodes; request_id is filed in constructor on master nodes; qed");
		let is_shadow_decryption = self.is_shadow_decryption
			.expect("prepare_partial_request is only called on master nodes; is_shadow_decryption is filed in constructor on master nodes; qed");
		let is_broadcast_session = self.is_broadcast_session
			.expect("prepare_partial_request is only called on master nodes; is_broadcast_session is filed in constructor on master nodes; qed");
		let mut other_nodes_ids = nodes.clone();
		other_nodes_ids.remove(node);

		Ok(PartialDecryptionRequest {
			id: request_id.clone(),
			is_shadow_decryption: is_shadow_decryption,
			is_broadcast_session: is_broadcast_session,
			other_nodes_ids: other_nodes_ids,
		})
	}

	fn process_partial_request(&mut self, partial_request: PartialDecryptionRequest) -> Result<JobPartialRequestAction<PartialDecryptionResponse>, Error> {
		let key_version = self.key_share.version(&self.key_version)?;
		if partial_request.other_nodes_ids.len() != self.key_share.threshold
			|| partial_request.other_nodes_ids.contains(&self.self_node_id)
			|| partial_request.other_nodes_ids.iter().any(|n| !key_version.id_numbers.contains_key(n)) {
			return Err(Error::InvalidMessage);
		}

		let self_id_number = &key_version.id_numbers[&self.self_node_id];
		let other_id_numbers = partial_request.other_nodes_ids.iter().map(|n| &key_version.id_numbers[n]);
		let node_shadow = math::compute_node_shadow(&key_version.secret_share, &self_id_number, other_id_numbers)?;
		let decrypt_shadow = if partial_request.is_shadow_decryption { Some(math::generate_random_scalar()?) } else { None };
		let common_point = self.key_share.common_point.as_ref().expect("DecryptionJob is only created when common_point is known; qed");
		let (shadow_point, decrypt_shadow) = math::compute_node_shadow_point(&self.access_key, &common_point, &node_shadow, decrypt_shadow)?;

		Ok(JobPartialRequestAction::Respond(PartialDecryptionResponse {
			request_id: partial_request.id,
			shadow_point: shadow_point,
			decrypt_shadow: match decrypt_shadow.clone() {
				None => None,
				Some(decrypt_shadow) => Some(encrypt(&self.requester, &DEFAULT_MAC, decrypt_shadow.as_bytes())?),
			},
		}))
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &PartialDecryptionResponse) -> Result<JobPartialResponseAction, Error> {
		if Some(&partial_response.request_id) != self.request_id.as_ref() {
			return Ok(JobPartialResponseAction::Ignore);
		}
		if self.is_shadow_decryption != Some(partial_response.decrypt_shadow.is_some()) {
			return Ok(JobPartialResponseAction::Reject);
		}
		Ok(JobPartialResponseAction::Accept)
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, PartialDecryptionResponse>) -> Result<EncryptedDocumentKeyShadow, Error> {
		let is_shadow_decryption = self.is_shadow_decryption
			.expect("compute_response is only called on master nodes; is_shadow_decryption is filed in constructor on master nodes; qed");
		let common_point = self.key_share.common_point.as_ref().expect("DecryptionJob is only created when common_point is known; qed");
		let encrypted_point = self.key_share.encrypted_point.as_ref().expect("DecryptionJob is only created when encrypted_point is known; qed");
		let joint_shadow_point = math::compute_joint_shadow_point(partial_responses.values().map(|s| &s.shadow_point))?;
		let decrypted_secret = math::decrypt_with_joint_shadow(self.key_share.threshold, &self.access_key, encrypted_point, &joint_shadow_point)?;
		Ok(EncryptedDocumentKeyShadow {
			decrypted_secret: decrypted_secret,
			common_point: if is_shadow_decryption {
				Some(math::make_common_shadow_point(self.key_share.threshold, common_point.clone())?)
			} else { None },
			decrypt_shadows: if is_shadow_decryption {
				Some(partial_responses.values().map(|r| r.decrypt_shadow.as_ref()
					.expect("is_shadow_decryption == true; decrypt_shadow.is_some() is checked in check_partial_response; qed")
					.clone())
					.collect())
			} else { None },
		})
	}
}
