use std::sync::Arc;
use std::collections::{BTreeSet, BTreeMap};
use ethkey::{Public, Secret};
use ethcrypto::ecies::encrypt;
use ethcrypto::DEFAULT_MAC;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare, EncryptedDocumentKeyShadow};
use key_server_cluster::math;
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobExecutor};

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
	/// Request id.
	request_id: Option<Secret>,
	/// Is shadow decryption requested.
	is_shadow_decryption: Option<bool>,
	/// Id of nodes, participating in decryption.
	participants: Option<BTreeSet<NodeId>>,
}

/// Decryption job partial request.
pub struct PartialDecryptionRequest {
	/// Request id.
	pub id: Secret,
	/// Is shadow decryption requested.
	pub is_shadow_decryption: bool,
	/// Id of other nodes, participating in decryption.
	pub other_nodes_ids: BTreeSet<NodeId>,
}

/// Decryption job partial response.
pub struct PartialDecryptionResponse {
	/// Request id.
	pub request_id: Secret,
	/// Shadow point.
	pub shadow_point: Public,
	/// Decryption shadow coefficient, if requested.
	pub decrypt_shadow: Option<Vec<u8>>,
}

impl DecryptionJob {
	pub fn new_on_slave(self_node_id: NodeId, access_key: Secret, requester: Public, key_share: DocumentKeyShare) -> Self {
		debug_assert!(key_share.common_point.is_some() && key_share.encrypted_point.is_some());
		DecryptionJob {
			self_node_id: self_node_id,
			access_key: access_key,
			requester: requester,
			key_share: key_share,
			request_id: None,
			is_shadow_decryption: None,
			participants: None,
		}
	}

	pub fn new_on_master(self_node_id: NodeId, access_key: Secret, requester: Public, key_share: DocumentKeyShare, is_shadow_decryption: bool, participants: BTreeSet<NodeId>) -> Self {
		debug_assert!(key_share.common_point.is_some() && key_share.encrypted_point.is_some());
		DecryptionJob {
			self_node_id: self_node_id,
			access_key: access_key,
			requester: requester,
			key_share: key_share,
			is_shadow_decryption: Some(is_shadow_decryption),
			participants: Some(participants),
		}
	}
}

impl JobExecutor for DecryptionJob {
	type PartialJobRequest = PartialDecryptionRequest;
	type PartialJobResponse = PartialDecryptionResponse;
	type JobResponse = EncryptedDocumentKeyShadow;

	fn prepare_partial_request(&self, node: &NodeId) -> Result<PartialDecryptionRequest, Error> {
		let is_shadow_decryption = self.is_shadow_decryption
			.expect("prepare_partial_request is only called on master nodes; is_shadow_decryption is filed in constructor on master nodes; qed");
		let mut other_nodes_ids = self.participants.as_ref()
			.expect("prepare_partial_request is only called on master nodes; participants is filed in constructor on master nodes; qed")
			.clone();
		other_nodes_ids.remove(node);

		Ok(PartialDecryptionRequest {
			is_shadow_decryption: is_shadow_decryption,
			other_nodes_ids: other_nodes_ids,
		})
	}

	fn process_partial_request(&self, partial_request: PartialDecryptionRequest) -> Result<PartialDecryptionResponse, Error> {
		if partial_request.other_nodes_ids.len() != self.key_share.threshold
			|| partial_request.other_nodes_ids.contains(&self.self_node_id)
			|| partial_request.other_nodes_ids.iter().any(|n| !self.key_share.id_numbers.contains_key(n)) {
			return Err(Error::InvalidMessage);
		}

		let self_id_number = &self.key_share.id_numbers[&self.self_node_id];
		let other_id_numbers = partial_request.other_nodes_ids.iter().map(|n| &self.key_share.id_numbers[n]);
		let node_shadow = math::compute_node_shadow(&self.key_share.secret_share, &self_id_number, other_id_numbers)?;
		let decrypt_shadow = if partial_request.is_shadow_decryption { Some(math::generate_random_scalar()?) } else { None };
		let common_point = self.key_share.common_point.as_ref().expect("DecryptionJob is only created when common_point is known; qed");
		let (shadow_point, decrypt_shadow) = math::compute_node_shadow_point(&self.access_key, &common_point, &node_shadow, decrypt_shadow)?;
		Ok(PartialDecryptionResponse {
			shadow_point: shadow_point,
			decrypt_shadow: match decrypt_shadow {
				None => None,
				Some(decrypt_shadow) => Some(encrypt(&self.requester, &DEFAULT_MAC, &**decrypt_shadow)?),
			},
		})
	}

	fn check_partial_response(&self, partial_response: &PartialDecryptionResponse) -> Result<JobPartialResponseAction, Error> {
		if Some(partial_response.request_id) != self.request_id {
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
