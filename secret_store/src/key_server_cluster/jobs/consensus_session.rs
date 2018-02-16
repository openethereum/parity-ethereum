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

use std::collections::BTreeSet;
use ethkey::Signature;
use key_server_cluster::{Error, NodeId, SessionMeta};
use key_server_cluster::message::ConsensusMessage;
use key_server_cluster::jobs::job_session::{JobSession, JobSessionState, JobTransport, JobExecutor};

/// Consensus session state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsensusSessionState {
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Consensus group is establishing.
	EstablishingConsensus,
	/// Consensus group is established.
	/// Master node can start jobs dissemination.
	/// Slave node waits for partial job requests.
	ConsensusEstablished,
	/// Master node waits for partial jobs responses.
	WaitingForPartialResults,
	/// Consensus session is completed successfully.
	/// Master node can call result() to get computation result.
	Finished,
	/// Consensus session has failed with error.
	Failed,
}

/// Consensus session consists of following states:
/// 1) consensus group is established
/// 2) master node sends partial job requests to every member of consensus group
/// 3) slave nodes are computing partial responses
/// 4) master node computes result from partial responses
pub struct ConsensusSession<ConsensusExecutor: JobExecutor<PartialJobResponse=bool>,
	ConsensusTransport: JobTransport<PartialJobRequest=ConsensusExecutor::PartialJobRequest, PartialJobResponse=ConsensusExecutor::PartialJobResponse>,
	ComputationExecutor: JobExecutor,
	ComputationTransport: JobTransport<PartialJobRequest=ComputationExecutor::PartialJobRequest, PartialJobResponse=ComputationExecutor::PartialJobResponse>
> {
	/// Current session state.
	state: ConsensusSessionState,
	/// Session metadata.
	meta: SessionMeta,
	/// Consensus establish job.
	consensus_job: JobSession<ConsensusExecutor, ConsensusTransport>,
	/// Consensus group.
	consensus_group: BTreeSet<NodeId>,
	/// Computation job.
	computation_job: Option<JobSession<ComputationExecutor, ComputationTransport>>,
}

/// Consensus session creation parameters.
pub struct ConsensusSessionParams<ConsensusExecutor: JobExecutor<PartialJobResponse=bool>,
	ConsensusTransport: JobTransport<PartialJobRequest=ConsensusExecutor::PartialJobRequest, PartialJobResponse=ConsensusExecutor::PartialJobResponse>
> {
	/// Session metadata.
	pub meta: SessionMeta,
	/// ACL storage for access check.
	pub consensus_executor: ConsensusExecutor,
	/// Transport for consensus establish job.
	pub consensus_transport: ConsensusTransport,
}

impl<ConsensusExecutor, ConsensusTransport, ComputationExecutor, ComputationTransport> ConsensusSession<ConsensusExecutor, ConsensusTransport, ComputationExecutor, ComputationTransport>
	where ConsensusExecutor: JobExecutor<PartialJobResponse=bool, JobResponse=BTreeSet<NodeId>>,
		ConsensusTransport: JobTransport<PartialJobRequest=ConsensusExecutor::PartialJobRequest, PartialJobResponse=ConsensusExecutor::PartialJobResponse>,
		ComputationExecutor: JobExecutor,
		ComputationTransport: JobTransport<PartialJobRequest=ComputationExecutor::PartialJobRequest, PartialJobResponse=ComputationExecutor::PartialJobResponse> {
	/// Create new consensus session.
	pub fn new(params: ConsensusSessionParams<ConsensusExecutor, ConsensusTransport>) -> Result<Self, Error> {
		let consensus_job = JobSession::new(params.meta.clone(), params.consensus_executor, params.consensus_transport);
		debug_assert!(consensus_job.state() == JobSessionState::Inactive);

		Ok(ConsensusSession {
			state: ConsensusSessionState::WaitingForInitialization,
			meta: params.meta,
			consensus_job: consensus_job,
			consensus_group: BTreeSet::new(),
			computation_job: None,
		})
	}

	/// Get consensus job reference.
	pub fn consensus_job(&self) -> &JobSession<ConsensusExecutor, ConsensusTransport> {
		&self.consensus_job
	}

	/// Get mutable consensus job reference.
	pub fn consensus_job_mut(&mut self) -> &mut JobSession<ConsensusExecutor, ConsensusTransport> {
		&mut self.consensus_job
	}

	/// Get all nodes, which has not rejected consensus request.
	pub fn consensus_non_rejected_nodes(&self) -> BTreeSet<NodeId> {
		self.consensus_job.responses().iter()
			.filter(|r| *r.1)
			.map(|r| r.0)
			.chain(self.consensus_job.requests())
			.filter(|n| **n != self.meta.self_node_id)
			.cloned()
			.collect()
	}

	/// Get computation job reference.
	#[cfg(test)]
	pub fn computation_job(&self) -> &JobSession<ComputationExecutor, ComputationTransport> {
		self.computation_job.as_ref()
			.expect("computation_job must only be called on master nodes")
	}

	/// Get consensus session state.
	pub fn state(&self) -> ConsensusSessionState {
		self.state
	}

	/// Get computation result.
	pub fn result(&self) -> Result<ComputationExecutor::JobResponse, Error> {
		debug_assert!(self.meta.self_node_id == self.meta.master_node_id);
		if self.state != ConsensusSessionState::Finished {
			return Err(Error::InvalidStateForRequest);
		}

		self.computation_job.as_ref()
			.expect("we are on master node in finished state; computation_job is set on master node during initialization; qed")
			.result()
	}

	/// Initialize session on master node.
	pub fn initialize(&mut self, nodes: BTreeSet<NodeId>) -> Result<(), Error> {
		debug_assert!(self.meta.self_node_id == self.meta.master_node_id);
		let initialization_result = self.consensus_job.initialize(nodes, false);
		self.state = ConsensusSessionState::EstablishingConsensus;
		self.process_result(initialization_result)
	}

	/// Process consensus request message.
	pub fn on_consensus_partial_request(&mut self, sender: &NodeId, request: ConsensusExecutor::PartialJobRequest) -> Result<(), Error> {
		let consensus_result = self.consensus_job.on_partial_request(sender, request);
		self.process_result(consensus_result)
	}

	/// Process consensus message response.
	pub fn on_consensus_partial_response(&mut self, sender: &NodeId, response: bool) -> Result<(), Error> {
		let consensus_result = self.consensus_job.on_partial_response(sender, response);
		self.process_result(consensus_result)
	}

	/// Select nodes for processing partial requests.
	pub fn select_consensus_group(&mut self) -> Result<&BTreeSet<NodeId>, Error> {
		debug_assert!(self.meta.self_node_id == self.meta.master_node_id);
		if self.state != ConsensusSessionState::ConsensusEstablished {
			return Err(Error::InvalidStateForRequest);
		}

		if self.consensus_group.is_empty() {
			let consensus_group = self.consensus_job.result()?;
			let is_self_in_consensus = consensus_group.contains(&self.meta.self_node_id);
			self.consensus_group = consensus_group.into_iter().take(self.meta.threshold + 1).collect();

			if is_self_in_consensus {
				self.consensus_group.remove(&self.meta.master_node_id);
				self.consensus_group.insert(self.meta.master_node_id.clone());
			}
		}

		Ok(&self.consensus_group)
	}

	/// Disseminate jobs from master node.
	pub fn disseminate_jobs(&mut self, executor: ComputationExecutor, transport: ComputationTransport, broadcast_self_response: bool) -> Result<(), Error> {
		let consensus_group = self.select_consensus_group()?.clone();
		self.consensus_group.clear();

		let mut computation_job = JobSession::new(self.meta.clone(), executor, transport);
		let computation_result = computation_job.initialize(consensus_group, broadcast_self_response);
		self.computation_job = Some(computation_job);
		self.state = ConsensusSessionState::WaitingForPartialResults;
		self.process_result(computation_result)
	}

	/// Process job request on slave node.
	pub fn on_job_request(&mut self, node: &NodeId, request: ComputationExecutor::PartialJobRequest, executor: ComputationExecutor, transport: ComputationTransport) -> Result<(), Error> {
		if &self.meta.master_node_id != node {
			return Err(Error::InvalidMessage);
		}
		if self.state != ConsensusSessionState::ConsensusEstablished {
			return Err(Error::InvalidStateForRequest);
		}

		JobSession::new(self.meta.clone(), executor, transport).on_partial_request(node, request)
	}

	/// Process job response on slave node.
	pub fn on_job_response(&mut self, node: &NodeId, response: ComputationExecutor::PartialJobResponse) -> Result<(), Error> {
		if self.state != ConsensusSessionState::WaitingForPartialResults {
			return Err(Error::InvalidStateForRequest);
		}

		let computation_result = self.computation_job.as_mut()
			.expect("WaitingForPartialResults is only set when computation_job is created; qed")
			.on_partial_response(node, response);

		self.process_result(computation_result)
	}

	/// When session is completed on slave node.
	pub fn on_session_completed(&mut self, node: &NodeId) -> Result<(), Error> {
		if node != &self.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}
		if self.state != ConsensusSessionState::ConsensusEstablished {
			return Err(Error::InvalidStateForRequest);
		}

		self.state = ConsensusSessionState::Finished;

		Ok(())
	}

	/// When error is received from node.
	pub fn on_node_error(&mut self, node: &NodeId) -> Result<bool, Error> {
		let is_self_master = self.meta.master_node_id == self.meta.self_node_id;
		let is_node_master = self.meta.master_node_id == *node;
		let (is_restart_needed, timeout_result) = match self.state {
			ConsensusSessionState::WaitingForInitialization if is_self_master => {
				// it is strange to receive error before session is initialized && slave doesn't know access_key
				// => fatal error
				self.state = ConsensusSessionState::Failed;
				(false, Err(Error::ConsensusUnreachable))
			}
			ConsensusSessionState::WaitingForInitialization if is_node_master => {
				// can not establish consensus
				// => fatal error
				self.state = ConsensusSessionState::Failed;
				(false, Err(Error::ConsensusUnreachable))
			},
			ConsensusSessionState::EstablishingConsensus => {
				debug_assert!(is_self_master);

				// consensus still can be established
				// => try to live without this node
				(false, self.consensus_job.on_node_error(node))
			},
			ConsensusSessionState::ConsensusEstablished => {
				// we could try to continue without this node, if enough nodes left
				(false, self.consensus_job.on_node_error(node))
			},
			ConsensusSessionState::WaitingForPartialResults => {
				// check if *current* computation job can continue without this node
				let is_computation_node = self.computation_job.as_mut()
					.expect("WaitingForPartialResults state is only set when computation_job is created; qed")
					.on_node_error(node)
					.is_err();
				if !is_computation_node {
					// it is not used by current computation job
					// => no restart required
					(false, Ok(()))
				} else {
					// it is used by current computation job
					// => restart is required if there are still enough nodes
					self.consensus_group.clear();
					self.state = ConsensusSessionState::EstablishingConsensus;

					let consensus_result = self.consensus_job.on_node_error(node);
					let is_consensus_established = self.consensus_job.state() == JobSessionState::Finished;
					(is_consensus_established, consensus_result)
				}
			},
			// in all other cases - just ignore error
			ConsensusSessionState::WaitingForInitialization | ConsensusSessionState::Failed | ConsensusSessionState::Finished => (false, Ok(())),
		};
		self.process_result(timeout_result)?;
		Ok(is_restart_needed)
	}

	/// When session is timeouted.
	pub fn on_session_timeout(&mut self) -> Result<bool, Error> {
		match self.state {
			// if we are waiting for results from slaves, there is a chance to send request to other nodes subset => fall through
			ConsensusSessionState::WaitingForPartialResults => (),
			// in some states this error is fatal
			ConsensusSessionState::WaitingForInitialization | ConsensusSessionState::EstablishingConsensus | ConsensusSessionState::ConsensusEstablished => {
				let _ = self.consensus_job.on_session_timeout();

				self.consensus_group.clear();
				self.state = ConsensusSessionState::EstablishingConsensus;
				return self.process_result(Err(Error::ConsensusUnreachable)).map(|_| unreachable!());
			},
			// in all other cases - just ignore error
			ConsensusSessionState::Finished | ConsensusSessionState::Failed => return Ok(false),
		};

		let timeouted_nodes = self.computation_job.as_ref()
			.expect("WaitingForPartialResults state is only set when computation_job is created; qed")
			.requests()
			.clone();
		assert!(!timeouted_nodes.is_empty()); // timeout should not ever happen if no requests are active && we are waiting for responses

		self.consensus_group.clear();
		for timeouted_node in timeouted_nodes {
			let timeout_result = self.consensus_job.on_node_error(&timeouted_node);
			self.state = ConsensusSessionState::EstablishingConsensus;
			self.process_result(timeout_result)?;
		}

		Ok(self.state == ConsensusSessionState::ConsensusEstablished)
	}

	/// Process result of job.
	fn process_result(&mut self, result: Result<(), Error>) -> Result<(), Error> {
		match self.state {
			ConsensusSessionState::WaitingForInitialization | ConsensusSessionState::EstablishingConsensus | ConsensusSessionState::ConsensusEstablished => match self.consensus_job.state() {
				JobSessionState::Finished => self.state = ConsensusSessionState::ConsensusEstablished,
				JobSessionState::Failed => self.state = ConsensusSessionState::Failed,
				_ => (),
			},
			ConsensusSessionState::WaitingForPartialResults => match self.computation_job.as_ref()
				.expect("WaitingForPartialResults state is only set when computation_job is created; qed")
				.state() {
				JobSessionState::Finished => self.state = ConsensusSessionState::Finished,
				JobSessionState::Failed => self.state = ConsensusSessionState::Failed,
				_ => (),
			},
			_ => (),
		}

		result
	}
}

impl<ConsensusExecutor, ConsensusTransport, ComputationExecutor, ComputationTransport> ConsensusSession<ConsensusExecutor, ConsensusTransport, ComputationExecutor, ComputationTransport>
	where ConsensusExecutor: JobExecutor<PartialJobRequest=Signature, PartialJobResponse=bool, JobResponse=BTreeSet<NodeId>>,
		ConsensusTransport: JobTransport<PartialJobRequest=ConsensusExecutor::PartialJobRequest, PartialJobResponse=ConsensusExecutor::PartialJobResponse>,
		ComputationExecutor: JobExecutor,
		ComputationTransport: JobTransport<PartialJobRequest=ComputationExecutor::PartialJobRequest, PartialJobResponse=ComputationExecutor::PartialJobResponse> {
	/// Process basic consensus message.
	pub fn on_consensus_message(&mut self, sender: &NodeId, message: &ConsensusMessage) -> Result<(), Error> {
		let consensus_result = match message {
			
			&ConsensusMessage::InitializeConsensusSession(ref message) =>
				self.consensus_job.on_partial_request(sender, message.requestor_signature.clone().into()),
			&ConsensusMessage::ConfirmConsensusInitialization(ref message) =>
				self.consensus_job.on_partial_response(sender, message.is_confirmed),
		};
		self.process_result(consensus_result)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use ethkey::{Signature, KeyPair, Random, Generator, sign};
	use key_server_cluster::{Error, NodeId, SessionId, DummyAclStorage};
	use key_server_cluster::message::{ConsensusMessage, InitializeConsensusSession, ConfirmConsensusInitialization};
	use key_server_cluster::jobs::job_session::tests::{make_master_session_meta, make_slave_session_meta, SquaredSumJobExecutor, DummyJobTransport};
	use key_server_cluster::jobs::key_access_job::KeyAccessJob;
	use super::{ConsensusSession, ConsensusSessionParams, ConsensusSessionState};

	type SquaredSumConsensusSession = ConsensusSession<KeyAccessJob, DummyJobTransport<Signature, bool>, SquaredSumJobExecutor, DummyJobTransport<u32, u32>>;

	fn make_master_consensus_session(threshold: usize, requester: Option<KeyPair>, acl_storage: Option<DummyAclStorage>) -> SquaredSumConsensusSession {
		let secret = requester.map(|kp| kp.secret().clone()).unwrap_or(Random.generate().unwrap().secret().clone());
		SquaredSumConsensusSession::new(ConsensusSessionParams {
			meta: make_master_session_meta(threshold),
			consensus_executor: KeyAccessJob::new_on_master(SessionId::default(), Arc::new(acl_storage.unwrap_or(DummyAclStorage::default())), sign(&secret, &SessionId::default()).unwrap()),
			consensus_transport: DummyJobTransport::default(),
		}).unwrap()
	}

	fn make_slave_consensus_session(threshold: usize, acl_storage: Option<DummyAclStorage>) -> SquaredSumConsensusSession {
		SquaredSumConsensusSession::new(ConsensusSessionParams {
			meta: make_slave_session_meta(threshold),
			consensus_executor: KeyAccessJob::new_on_slave(SessionId::default(), Arc::new(acl_storage.unwrap_or(DummyAclStorage::default()))),
			consensus_transport: DummyJobTransport::default(),
		}).unwrap()
	}

	#[test]
	fn consensus_session_consensus_is_not_reached_when_initializes_with_non_zero_threshold() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_consensus_is_reached_when_initializes_with_zero_threshold() {
		let mut session = make_master_consensus_session(0, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_consensus_is_not_reached_when_initializes_with_zero_threshold_and_master_rejects() {
		let requester = Random.generate().unwrap();
		let acl_storage = DummyAclStorage::default();
		acl_storage.prohibit(requester.public().clone(), SessionId::default());

		let mut session = make_master_consensus_session(0, Some(requester), Some(acl_storage));
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_consensus_is_failed_by_master_node() {
		let requester = Random.generate().unwrap();
		let acl_storage = DummyAclStorage::default();
		acl_storage.prohibit(requester.public().clone(), SessionId::default());

		let mut session = make_master_consensus_session(1, Some(requester), Some(acl_storage));
		assert_eq!(session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap_err(), Error::ConsensusUnreachable);
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_consensus_is_failed_by_slave_node() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		assert_eq!(session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: false,
		})).unwrap_err(), Error::ConsensusUnreachable);
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_job_dissemination_fails_if_consensus_is_not_reached() {
		let mut session = make_master_consensus_session(1, None, None);
		assert_eq!(session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn consensus_session_job_dissemination_selects_master_node_if_agreed() {
		let mut session = make_master_consensus_session(0, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::Finished);
		assert!(session.computation_job().responses().contains_key(&NodeId::from(1)));
	}

	#[test]
	fn consensus_session_job_dissemination_does_not_select_master_node_if_rejected() {
		let requester = Random.generate().unwrap();
		let acl_storage = DummyAclStorage::default();
		acl_storage.prohibit(requester.public().clone(), SessionId::default());

		let mut session = make_master_consensus_session(0, Some(requester), Some(acl_storage));
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);
		assert!(!session.computation_job().responses().contains_key(&NodeId::from(1)));
	}

	#[test]
	fn consensus_session_computation_request_is_rejected_when_received_by_master_node() {
		let mut session = make_master_consensus_session(0, None, None);
		assert_eq!(session.on_job_request(&NodeId::from(2), 2, SquaredSumJobExecutor, DummyJobTransport::default()).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn consensus_session_computation_request_is_rejected_when_received_before_consensus_is_established() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.on_job_request(&NodeId::from(1), 2, SquaredSumJobExecutor, DummyJobTransport::default()).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn consensus_session_computation_request_is_ignored_when_wrong() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.state(), ConsensusSessionState::WaitingForInitialization);
		session.on_consensus_message(&NodeId::from(1), &ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
			requestor_signature: sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
			version: Default::default(),
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		assert_eq!(session.on_job_request(&NodeId::from(1), 20, SquaredSumJobExecutor, DummyJobTransport::default()).unwrap_err(), Error::InvalidMessage);
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_computation_request_is_processed_when_correct() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.state(), ConsensusSessionState::WaitingForInitialization);
		session.on_consensus_message(&NodeId::from(1), &ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
			requestor_signature: sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
			version: Default::default(),
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.on_job_request(&NodeId::from(1), 2, SquaredSumJobExecutor, DummyJobTransport::default()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_computation_response_is_ignored_when_consensus_is_not_reached() {
		let mut session = make_master_consensus_session(1, None, None);
		assert_eq!(session.on_job_response(&NodeId::from(2), 4).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn consessus_session_completion_is_ignored_when_received_from_non_master_node() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.on_session_completed(&NodeId::from(3)).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn consessus_session_completion_is_ignored_when_consensus_is_not_established() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.on_session_completed(&NodeId::from(1)).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn consessus_session_completion_is_accepted() {
		let mut session = make_slave_consensus_session(0, None);
		session.on_consensus_message(&NodeId::from(1), &ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
			requestor_signature: sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
			version: Default::default(),
		})).unwrap();
		session.on_session_completed(&NodeId::from(1)).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::Finished);
	}

	#[test]
	fn consensus_session_fails_if_node_error_received_by_uninitialized_master() {
		let mut session = make_master_consensus_session(0, None, None);
		assert_eq!(session.on_node_error(&NodeId::from(2)), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_fails_if_node_error_received_by_uninitialized_slave_from_master() {
		let mut session = make_slave_consensus_session(0, None);
		assert_eq!(session.on_node_error(&NodeId::from(1)), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_continues_if_node_error_received_by_master_during_establish_and_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3)].into_iter().collect()).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Ok(false));
	}

	#[test]
	fn consensus_session_fails_if_node_error_received_by_master_during_establish_and_not_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_continues_if_node2_error_received_by_master_after_consensus_established_and_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Ok(false));
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_continues_if_node3_error_received_by_master_after_consensus_established_and_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(3)), Ok(false));
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn consensus_session_fails_if_node_error_received_by_master_after_consensus_established_and_not_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_continues_if_node_error_received_from_slave_not_participating_in_computation() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3), NodeId::from(4)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.on_consensus_message(&NodeId::from(3), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(3)), Ok(false));
		assert_eq!(session.on_node_error(&NodeId::from(4)), Ok(false));
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);
	}

	#[test]
	fn consensus_session_restarts_if_node_error_received_from_slave_participating_in_computation_and_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3), NodeId::from(4)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		session.on_consensus_message(&NodeId::from(3), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Ok(true));
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		assert_eq!(session.on_node_error(&NodeId::from(3)), Ok(false));
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
	}

	#[test]
	fn consensus_session_fails_if_node_error_received_from_slave_participating_in_computation_and_not_enough_nodes_left() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.on_node_error(&NodeId::from(2)), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn consensus_session_fails_if_uninitialized_session_timeouts() {
		let mut session = make_master_consensus_session(1, None, None);
		assert_eq!(session.on_session_timeout(), Err(Error::ConsensusUnreachable));
	}

	#[test]
	fn consensus_session_continues_if_session_timeouts_and_enough_nodes_left_for_computation() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3), NodeId::from(4)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();

		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		session.on_consensus_message(&NodeId::from(3), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.on_session_timeout(), Ok(true));
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);

		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		assert_eq!(session.on_session_timeout(), Ok(false));
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
	}

	#[test]
	fn consensus_session_continues_if_session_timeouts_and_not_enough_nodes_left_for_computation() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		assert_eq!(session.on_session_timeout(), Err(Error::ConsensusUnreachable));
		assert_eq!(session.state(), ConsensusSessionState::Failed);
	}

	#[test]
	fn same_consensus_group_returned_after_second_selection() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3)].into_iter().collect()).unwrap();
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		session.on_consensus_message(&NodeId::from(3), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();

		let consensus_group1 = session.select_consensus_group().unwrap().clone();
		let consensus_group2 = session.select_consensus_group().unwrap().clone();
		assert_eq!(consensus_group1, consensus_group2);
	}

	#[test]
	fn consensus_session_complete_2_of_4() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3), NodeId::from(3)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);
		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);
		session.on_job_response(&NodeId::from(2), 16).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::Finished);
		assert_eq!(session.result(), Ok(20));
	}

	#[test]
	fn consensus_session_complete_2_of_4_after_restart() {
		let mut session = make_master_consensus_session(1, None, None);
		session.initialize(vec![NodeId::from(1), NodeId::from(2), NodeId::from(3), NodeId::from(4)].into_iter().collect()).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);
		session.on_consensus_message(&NodeId::from(2), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);

		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		session.on_consensus_message(&NodeId::from(3), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();

		assert_eq!(session.on_node_error(&NodeId::from(2)).unwrap(), true);
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);

		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		assert_eq!(session.on_node_error(&NodeId::from(3)).unwrap(), false);
		assert_eq!(session.state(), ConsensusSessionState::EstablishingConsensus);

		session.on_consensus_message(&NodeId::from(4), &ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
			is_confirmed: true,
		})).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::ConsensusEstablished);

		session.disseminate_jobs(SquaredSumJobExecutor, DummyJobTransport::default(), false).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::WaitingForPartialResults);

		session.on_job_response(&NodeId::from(4), 16).unwrap();
		assert_eq!(session.state(), ConsensusSessionState::Finished);
		assert_eq!(session.result(), Ok(20));
	}
}
