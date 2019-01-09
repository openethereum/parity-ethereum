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
use ethkey::{Public, Secret, Signature};
use ethereum_types::H256;
use key_server_cluster::{Error, NodeId, DocumentKeyShare};
use key_server_cluster::math;
use key_server_cluster::jobs::job_session::{JobPartialRequestAction, JobPartialResponseAction, JobExecutor};

/// Signing job.
pub struct EcdsaSigningJob {
	/// Key share.
	key_share: DocumentKeyShare,
	/// Key version.
	key_version: H256,
	/// Share of inv(nonce).
	inv_nonce_share: Secret,
	/// Nonce public.
	nonce_public: Public,
	/// Request id.
	request_id: Option<Secret>,
	/// 
	inversed_nonce_coeff: Option<Secret>,
	/// Message hash.
	message_hash: Option<H256>,
}

/// Signing job partial request.
pub struct EcdsaPartialSigningRequest {
	/// Request id.
	pub id: Secret,
	/// 
	pub inversed_nonce_coeff: Secret,
	/// Message hash to sign.
	pub message_hash: H256,
}

/// Signing job partial response.
#[derive(Clone)]
pub struct EcdsaPartialSigningResponse {
	/// Request id.
	pub request_id: Secret,
	/// Partial signature' s share.
	pub partial_signature_s: Secret,
}

impl EcdsaSigningJob {
	pub fn new_on_slave(key_share: DocumentKeyShare, key_version: H256, nonce_public: Public, inv_nonce_share: Secret) -> Result<Self, Error> {
		Ok(EcdsaSigningJob {
			key_share: key_share,
			key_version: key_version,
			nonce_public: nonce_public,
			inv_nonce_share: inv_nonce_share,
			request_id: None,
			inversed_nonce_coeff: None,
			message_hash: None,
		})
	}

	pub fn new_on_master(key_share: DocumentKeyShare, key_version: H256, nonce_public: Public, inv_nonce_share: Secret, inversed_nonce_coeff: Secret, message_hash: H256) -> Result<Self, Error> {
		Ok(EcdsaSigningJob {
			key_share: key_share,
			key_version: key_version,
			nonce_public: nonce_public,
			inv_nonce_share: inv_nonce_share,
			request_id: Some(math::generate_random_scalar()?),
			inversed_nonce_coeff: Some(inversed_nonce_coeff),
			message_hash: Some(message_hash),
		})
	}
}

impl JobExecutor for EcdsaSigningJob {
	type PartialJobRequest = EcdsaPartialSigningRequest;
	type PartialJobResponse = EcdsaPartialSigningResponse;
	type JobResponse = Signature;

	fn prepare_partial_request(&self, _node: &NodeId, nodes: &BTreeSet<NodeId>) -> Result<EcdsaPartialSigningRequest, Error> {
		debug_assert!(nodes.len() == self.key_share.threshold * 2 + 1);

		let request_id = self.request_id.as_ref()
			.expect("prepare_partial_request is only called on master nodes; request_id is filed in constructor on master nodes; qed");
		let inversed_nonce_coeff = self.inversed_nonce_coeff.as_ref()
			.expect("prepare_partial_request is only called on master nodes; inversed_nonce_coeff is filed in constructor on master nodes; qed");
		let message_hash = self.message_hash.as_ref()
			.expect("compute_response is only called on master nodes; message_hash is filed in constructor on master nodes; qed");

		Ok(EcdsaPartialSigningRequest {
			id: request_id.clone(),
			inversed_nonce_coeff: inversed_nonce_coeff.clone(),
			message_hash: message_hash.clone(),
		})
	}

	fn process_partial_request(&mut self, partial_request: EcdsaPartialSigningRequest) -> Result<JobPartialRequestAction<EcdsaPartialSigningResponse>, Error> {
		let inversed_nonce_coeff_mul_nonce = math::compute_secret_mul(&partial_request.inversed_nonce_coeff, &self.inv_nonce_share)?;
		let key_version = self.key_share.version(&self.key_version)?;
		let signature_r = math::compute_ecdsa_r(&self.nonce_public)?;
		let inv_nonce_mul_secret = math::compute_secret_mul(&inversed_nonce_coeff_mul_nonce, &key_version.secret_share)?;
		let partial_signature_s = math::compute_ecdsa_s_share(
			&inversed_nonce_coeff_mul_nonce,
			&inv_nonce_mul_secret,
			&signature_r,
			&math::to_scalar(partial_request.message_hash)?,
		)?;

		Ok(JobPartialRequestAction::Respond(EcdsaPartialSigningResponse {
			request_id: partial_request.id,
			partial_signature_s: partial_signature_s,
		}))
	}

	fn check_partial_response(&mut self, _sender: &NodeId, partial_response: &EcdsaPartialSigningResponse) -> Result<JobPartialResponseAction, Error> {
		if Some(&partial_response.request_id) != self.request_id.as_ref() {
			return Ok(JobPartialResponseAction::Ignore);
		}
		// TODO [Trust]: check_ecdsa_signature_share()

		Ok(JobPartialResponseAction::Accept)
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, EcdsaPartialSigningResponse>) -> Result<Signature, Error> {
		let key_version = self.key_share.version(&self.key_version)?;
		if partial_responses.keys().any(|n| !key_version.id_numbers.contains_key(n)) {
			return Err(Error::InvalidMessage);
		}

		let id_numbers: Vec<_> = partial_responses.keys().map(|n| key_version.id_numbers[n].clone()).collect();
		let signature_s_shares: Vec<_> = partial_responses.values().map(|r| r.partial_signature_s.clone()).collect();
		let signature_s = math::compute_ecdsa_s(self.key_share.threshold, &signature_s_shares, &id_numbers)?;
		let signature_r = math::compute_ecdsa_r(&self.nonce_public)?;

		let signature = math::serialize_ecdsa_signature(&self.nonce_public, signature_r, signature_s);

		Ok(signature)
	}
}
