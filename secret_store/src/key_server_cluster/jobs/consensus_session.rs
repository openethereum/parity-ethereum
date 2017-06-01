use std::marker::PhantomData;
use std::collections::BTreeSet;
use std::sync::Arc;
use ethkey::{Public, Signature, recover};
use key_server_cluster::{Error, NodeId, SessionMeta, AclStorage};
use key_server_cluster::message::ConsensusMessage;
use key_server_cluster::jobs::job_session::{JobSession, JobSessionState, JobTransport, JobExecutor};
use key_server_cluster::jobs::key_access_job::KeyAccessJob;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsensusSessionState {
	WaitingForInitialization,
	EstablishingConsensus,
	ConsensusEstablished,
	WaitingForPartialResults,
	Finished,
	Failed,
}

pub struct ConsensusSession<ConsensusTransport: JobTransport<PartialJobRequest=Signature, PartialJobResponse=bool>, ComputationExecutor: JobExecutor, ComputationTransport: JobTransport<PartialJobRequest=ComputationExecutor::PartialJobRequest, PartialJobResponse=ComputationExecutor::PartialJobResponse>> {
	state: ConsensusSessionState,
	meta: SessionMeta,
	requester: Option<Public>,
	consensus_job: JobSession<KeyAccessJob, ConsensusTransport>,
	computation_job: Option<JobSession<ComputationExecutor, ComputationTransport>>,
	//dummy: PhantomData<(ComputationTransport, ComputationExecutor)>,
}

pub struct ConsensusSessionParams<ConsensusTransport: JobTransport<PartialJobRequest=Signature, PartialJobResponse=bool>> {
	meta: SessionMeta,
	acl_storage: Arc<AclStorage>,
	consensus_transport: ConsensusTransport,
}

impl<ConsensusTransport, ComputationExecutor, ComputationTransport> ConsensusSession<ConsensusTransport, ComputationExecutor, ComputationTransport> where ConsensusTransport: JobTransport<PartialJobRequest=Signature, PartialJobResponse=bool>, ComputationExecutor: JobExecutor, ComputationTransport: JobTransport<PartialJobRequest=ComputationExecutor::PartialJobRequest, PartialJobResponse=ComputationExecutor::PartialJobResponse> {
	pub fn new_on_slave(params: ConsensusSessionParams<ConsensusTransport>) -> Result<Self, Error> {
		debug_assert!(params.meta.self_node_id != params.meta.master_node_id);

		let consensus_job_executor = KeyAccessJob::new_on_slave(params.meta.id.clone(), params.acl_storage);
		let consensus_job = JobSession::new(params.meta, consensus_job_executor, params.consensus_transport);
		debug_assert!(consensus_job.state() == JobSessionState::Inactive);

		Ok(ConsensusSession {
			state: ConsensusSessionState::WaitingForInitialization,
			meta: params.meta,
			requester: None,
			consensus_job: consensus_job,
			computation_job: None,
			//dummy: PhantomData,
		})
	}

	pub fn new_on_master(params: ConsensusSessionParams<ConsensusTransport>, signature: Signature) -> Result<Self, Error> {
		debug_assert!(params.meta.self_node_id == params.meta.master_node_id);

		let requester = recover(&signature, &params.meta.id)?;
		let consensus_job_executor = KeyAccessJob::new_on_master(params.meta.id.clone(), params.acl_storage, signature);
		let consensus_job = JobSession::new(params.meta, consensus_job_executor, params.consensus_transport);

		Ok(ConsensusSession {
			state: ConsensusSessionState::WaitingForInitialization,
			meta: params.meta,
			requester: Some(requester),
			consensus_job: consensus_job,
			computation_job: None,
			dummy: PhantomData,
		})
	}

	pub fn state(&self) -> ConsensusSessionState {
		self.state
	}

	pub fn requester(&self) -> Result<&Public, Error> {
		self.requester.as_ref().ok_or(Error::InvalidStateForRequest)
	}

	pub fn result(&self) -> Result<ComputationExecutor::JobResponse, Error> {
		debug_assert!(self.meta.self_node_id == self.meta.master_node_id);
		if self.state != ConsensusSessionState::Finished {
			return Err(Error::InvalidStateForRequest);
		}

		self.computation_job
			.expect("we are on master node in finished state; computation_job is set on master node during initialization; qed")
			.result()
	}

	pub fn initialize(&mut self, nodes: BTreeSet<NodeId>) -> Result<(), Error> {
		let initialization_result = self.consensus_job.initialize(nodes);
		self.state = ConsensusSessionState::EstablishingConsensus;
		self.process_result(initialization_result)
	}

	pub fn on_consensus_message(&mut self, sender: &NodeId, message: &ConsensusMessage) -> Result<(), Error> {
		let consensus_result = match message {
			&ConsensusMessage::InitializeConsensusSession(ref message) => {
				let signature = message.requestor_signature.clone().into();
				self.requester = Some(recover(&signature, &self.meta.id)?);
				self.consensus_job.on_partial_request(sender, signature)
			},
			&ConsensusMessage::ConfirmConsensusInitialization(ref message) =>
				self.consensus_job.on_partial_response(sender, message.is_confirmed),
		};
		self.process_result(consensus_result)
	}

	pub fn disseminate_jobs(&mut self, executor: ComputationExecutor, transport: ComputationTransport) -> Result<(), Error> {
		if self.meta.self_node_id != self.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}
		if self.state != ConsensusSessionState::ConsensusEstablished {
			return Err(Error::InvalidStateForRequest);
		}

		let consensus_nodes = self.consensus_job.result().expect("disseminate_jobs is only called on master node when consensus is established; qed");
		let is_self_in_consensus = consensus_nodes.contains(&self.meta.self_node_id);
		let mut consensus_nodes: BTreeSet<_> = consensus_nodes.into_iter().take(self.meta.threshold + 1).collect();
		if is_self_in_consensus {
			consensus_nodes.remove(&self.meta.master_node_id);
			consensus_nodes.insert(self.meta.master_node_id.clone());
		}

		let mut computation_job = JobSession::new(self.meta, executor, transport);
		let computation_result = computation_job.initialize(consensus_nodes);
		self.computation_job = Some(computation_job);
		self.state = ConsensusSessionState::WaitingForPartialResults;
		self.process_result(computation_result)
	}

	pub fn on_job_request(&mut self, node: &NodeId, request: ComputationExecutor::PartialJobRequest, executor: ComputationExecutor, transport: ComputationTransport) -> Result<(), Error> {
		if &self.meta.master_node_id != node {
			return Err(Error::InvalidMessage);
		}
		if self.state != ConsensusSessionState::ConsensusEstablished {
			return Err(Error::InvalidStateForRequest);
		}

		let mut computation_job = JobSession::new(self.meta, executor, transport);
		let computation_result = computation_job.on_partial_request(node, request);
		self.process_result(computation_result)
	}

	pub fn on_job_response(&mut self, node: &NodeId, response: ComputationExecutor::PartialJobResponse) -> Result<(), Error> {
		if self.state != ConsensusSessionState::WaitingForPartialResults {
			return Err(Error::InvalidStateForRequest);
		}

		let computation_result = self.computation_job.as_mut()
			.expect("WaitingForPartialResults is only set when computation_job is created; qed")
			.on_partial_response(node, response);
		self.process_result(computation_result)
	}

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

	pub fn on_node_error(&mut self, node: &NodeId) -> Result<bool, Error> {
		let timeout_result = match self.state {
			ConsensusSessionState::WaitingForInitialization | ConsensusSessionState::Finished | ConsensusSessionState::Failed =>
				Err(Error::ConsensusUnreachable),
			ConsensusSessionState::EstablishingConsensus | ConsensusSessionState::ConsensusEstablished =>
				self.consensus_job.on_node_timeout(node),
			ConsensusSessionState::WaitingForPartialResults => {
				let is_computation_node = self.computation_job.as_mut()
					.expect("WaitingForPartialResults state is only set when computation_job is created; qed")
					.on_node_timeout(node)
					.is_err();
				if !is_computation_node {
					Ok(())
				} else {
					self.consensus_job.on_node_timeout(node)
				}
			},
		};
		self.process_result(timeout_result)
	}

	pub fn on_session_timeout(&mut self) -> Result<bool, Error> {
		match self.state {
			ConsensusSessionState::WaitingForInitialization | ConsensusSessionState::Finished | ConsensusSessionState::Failed |
				ConsensusSessionState::EstablishingConsensus | ConsensusSessionState::ConsensusEstablished =>
				return self.process_result(Err(Error::ConsensusUnreachable)),
			ConsensusSessionState::WaitingForPartialResults => ()
		};

		let timeouted_nodes = self.computation_job.as_ref()
			.expect("WaitingForPartialResults state is only set when computation_job is created; qed")
			.requests()
			.clone();
		for timeouted_node in timeouted_nodes {
			let timeout_result = self.consensus_job.on_node_timeout(&timeouted_node);
			self.process_result(timeout_result);
		}

		self.state = ConsensusSessionState::ConsensusEstablished;
		Ok(())
	}

	fn process_result(&mut self, result: Result<(), Error>) -> Result<(), Error> {
		match self.state {
			ConsensusSessionState::EstablishingConsensus => match self.consensus_job.state() {
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
