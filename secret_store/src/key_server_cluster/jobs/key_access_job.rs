use std::sync::Arc;
use std::collections::{BTreeSet, BTreeMap};
use ethkey::{Signature, recover};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage};
use key_server_cluster::jobs::job_session::{JobPartialResponseAction, JobExecutor};

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
}

impl JobExecutor for KeyAccessJob {
	type PartialJobRequest = Signature;
	type PartialJobResponse = bool;
	type JobResponse = BTreeSet<NodeId>;

	fn prepare_partial_request(&self, _node: &NodeId, _nodes: &BTreeSet<NodeId>) -> Result<Signature, Error> {
		Ok(self.signature.as_ref().expect("prepare_partial_request is only called on master nodes; new_on_master fills the signature; qed").clone())
	}

	fn process_partial_request(&self, partial_request: Signature) -> Result<bool, Error> {
		self.acl_storage.check(&recover(&partial_request, &self.id)?, &self.id).map_err(|_| Error::AccessDenied)
	}

	fn check_partial_response(&self, partial_response: &bool) -> Result<JobPartialResponseAction, Error> {
		Ok(if *partial_response { JobPartialResponseAction::Accept } else { JobPartialResponseAction::Reject })
	}

	fn compute_response(&self, partial_responses: &BTreeMap<NodeId, bool>) -> Result<BTreeSet<NodeId>, Error> {
		Ok(partial_responses.keys().cloned().collect())
	}
}
