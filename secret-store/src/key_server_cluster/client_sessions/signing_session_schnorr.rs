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

use std::collections::BTreeSet;
use std::sync::Arc;
use futures::Oneshot;
use parking_lot::Mutex;
use ethkey::{Public, Secret};
use ethereum_types::H256;
use key_server_cluster::{Error, NodeId, SessionId, Requester, SessionMeta, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::{SessionIdWithSubSession, ClusterSession, CompletionSignal};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	SessionState as GenerationSessionState};
use key_server_cluster::message::{Message, SchnorrSigningMessage, SchnorrSigningConsensusMessage, SchnorrSigningGenerationMessage,
	SchnorrRequestPartialSignature, SchnorrPartialSignature, SchnorrSigningSessionCompleted, GenerationMessage,
	ConsensusMessage, SchnorrSigningSessionError, InitializeConsensusSession, ConfirmConsensusInitialization,
	SchnorrSigningSessionDelegation, SchnorrSigningSessionDelegationCompleted};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::key_access_job::KeyAccessJob;
use key_server_cluster::jobs::signing_job_schnorr::{SchnorrPartialSigningRequest, SchnorrPartialSigningResponse, SchnorrSigningJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Distributed Schnorr-signing session.
/// Based on "Efficient Multi-Party Digital Signature using Adaptive Secret Sharing for Low-Power Devices in Wireless Network" paper.
/// Brief overview:
/// 1) initialization: master node (which has received request for signing the message) requests all other nodes to sign the message
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor has access to the private key
/// 3) partial signing: every node which has succussfully checked access for the requestor do a partial signing
/// 4) signing: master node receives all partial signatures of the secret and computes the signature
pub struct SessionImpl {
	/// Session core.
	core: SessionCore,
	/// Session data.
	data: Mutex<SessionData>,
}

/// Immutable session data.
struct SessionCore {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Signing session access key.
	pub access_key: Secret,
	/// Key share.
	pub key_share: Option<DocumentKeyShare>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	pub cluster: Arc<dyn Cluster>,
	/// Session-level nonce.
	pub nonce: u64,
	/// SessionImpl completion signal.
	pub completed: CompletionSignal<(Secret, Secret)>,
}

/// Signing consensus session type.
type SigningConsensusSession = ConsensusSession<KeyAccessJob, SigningConsensusTransport, SchnorrSigningJob, SigningJobTransport>;

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Message hash.
	pub message_hash: Option<H256>,
	/// Key version to use for decryption.
	pub version: Option<H256>,
	/// Consensus-based signing session.
	pub consensus_session: SigningConsensusSession,
	/// Session key generation session.
	pub generation_session: Option<GenerationSession>,
	/// Delegation status.
	pub delegation_status: Option<DelegationStatus>,
	/// Decryption result.
	pub result: Option<Result<(Secret, Secret), Error>>,
}

/// Signing session state.
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone, Copy))]
pub enum SessionState {
	/// State when consensus is establishing.
	ConsensusEstablishing,
	/// State when session key is generating.
	SessionKeyGeneration,
	/// State when signature is computing.
	SignatureComputing,
}

/// Session creation parameters
pub struct SessionParams {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Session access key.
	pub access_key: Secret,
	/// Key share.
	pub key_share: Option<DocumentKeyShare>,
	/// ACL storage.
	pub acl_storage: Arc<dyn AclStorage>,
	/// Cluster
	pub cluster: Arc<dyn Cluster>,
	/// Session nonce.
	pub nonce: u64,
}

/// Signing consensus transport.
struct SigningConsensusTransport {
	/// Session id.
	id: SessionId,
	/// Session access key.
	access_key: Secret,
	/// Session-level nonce.
	nonce: u64,
	/// Selected key version (on master node).
	version: Option<H256>,
	/// Cluster.
	cluster: Arc<dyn Cluster>,
}

/// Signing key generation transport.
struct SessionKeyGenerationTransport {
	/// Session access key.
	access_key: Secret,
	/// Cluster.
	cluster: Arc<dyn Cluster>,
	/// Session-level nonce.
	nonce: u64,
	/// Other nodes ids.
	other_nodes_ids: BTreeSet<NodeId>,
}

/// Signing job transport
struct SigningJobTransport {
	/// Session id.
	id: SessionId,
	/// Session access key.
	access_key: Secret,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<dyn Cluster>,
}

/// Session delegation status.
enum DelegationStatus {
	/// Delegated to other node.
	DelegatedTo(NodeId),
	/// Delegated from other node.
	DelegatedFrom(NodeId, u64),
}

impl SessionImpl {
	/// Create new signing session.
	pub fn new(
		params: SessionParams,
		requester: Option<Requester>,
	) -> Result<(Self, Oneshot<Result<(Secret, Secret), Error>>), Error> {
		debug_assert_eq!(params.meta.threshold, params.key_share.as_ref().map(|ks| ks.threshold).unwrap_or_default());

		let consensus_transport = SigningConsensusTransport {
			id: params.meta.id.clone(),
			access_key: params.access_key.clone(),
			nonce: params.nonce,
			version: None,
			cluster: params.cluster.clone(),
		};
		let consensus_session = ConsensusSession::new(ConsensusSessionParams {
			meta: params.meta.clone(),
			consensus_executor: match requester {
				Some(requester) => KeyAccessJob::new_on_master(params.meta.id.clone(), params.acl_storage.clone(), requester),
				None => KeyAccessJob::new_on_slave(params.meta.id.clone(), params.acl_storage.clone()),
			},
			consensus_transport: consensus_transport,
		})?;

		let (completed, oneshot) = CompletionSignal::new();
		Ok((SessionImpl {
			core: SessionCore {
				meta: params.meta,
				access_key: params.access_key,
				key_share: params.key_share,
				cluster: params.cluster,
				nonce: params.nonce,
				completed,
			},
			data: Mutex::new(SessionData {
				state: SessionState::ConsensusEstablishing,
				message_hash: None,
				version: None,
				consensus_session: consensus_session,
				generation_session: None,
				delegation_status: None,
				result: None,
			}),
		}, oneshot))
	}

	/// Wait for session completion.
	#[cfg(test)]
	pub fn wait(&self) -> Result<(Secret, Secret), Error> {
		Self::wait_session(&self.core.completed, &self.data, None, |data| data.result.clone())
			.expect("wait_session returns Some if called without timeout; qed")
	}

	/// Get session state (tests only).
	#[cfg(test)]
	pub fn state(&self) -> SessionState {
		self.data.lock().state
	}

	/// Delegate session to other node.
	pub fn delegate(&self, master: NodeId, version: H256, message_hash: H256) -> Result<(), Error> {
		if self.core.meta.master_node_id != self.core.meta.self_node_id {
			return Err(Error::InvalidStateForRequest);
		}

		let mut data = self.data.lock();
		if data.consensus_session.state() != ConsensusSessionState::WaitingForInitialization || data.delegation_status.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		data.consensus_session.consensus_job_mut().executor_mut().set_has_key_share(false);
		self.core.cluster.send(&master, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionDelegation(SchnorrSigningSessionDelegation {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.access_key.clone().into(),
			session_nonce: self.core.nonce,
			requester: data.consensus_session.consensus_job().executor().requester()
				.expect("requester is passed to master node on creation; session can be delegated from master node only; qed")
				.clone().into(),
			version: version.into(),
			message_hash: message_hash.into(),
		})))?;
		data.delegation_status = Some(DelegationStatus::DelegatedTo(master));
		Ok(())

	}

	/// Initialize signing session on master node.
	pub fn initialize(&self, version: H256, message_hash: H256) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		// check if version exists
		let key_version = match self.core.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share.version(&version)?,
		};

		let mut data = self.data.lock();
		let non_isolated_nodes = self.core.cluster.nodes();
		let mut consensus_nodes: BTreeSet<_> = key_version.id_numbers.keys()
			.filter(|n| non_isolated_nodes.contains(*n))
			.cloned()
			.chain(::std::iter::once(self.core.meta.self_node_id.clone()))
			.collect();
		if let Some(&DelegationStatus::DelegatedFrom(delegation_master, _)) = data.delegation_status.as_ref() {
			consensus_nodes.remove(&delegation_master);
		}

		data.consensus_session.consensus_job_mut().transport_mut().version = Some(version.clone());
		data.version = Some(version.clone());
		data.message_hash = Some(message_hash);
		data.consensus_session.initialize(consensus_nodes)?;

		if data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished {
			let generation_session = GenerationSession::new(GenerationSessionParams {
				id: self.core.meta.id.clone(),
				self_node_id: self.core.meta.self_node_id.clone(),
				key_storage: None,
				cluster: Arc::new(SessionKeyGenerationTransport {
					access_key: self.core.access_key.clone(),
					cluster: self.core.cluster.clone(),
					nonce: self.core.nonce,
					other_nodes_ids: BTreeSet::new()
				}),
				nonce: None,
			}).0;
			generation_session.initialize(Default::default(), Default::default(), false, 0, vec![self.core.meta.self_node_id.clone()].into_iter().collect::<BTreeSet<_>>().into())?;

			debug_assert_eq!(generation_session.state(), GenerationSessionState::Finished);
			let joint_public_and_secret = generation_session
				.joint_public_and_secret()
				.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")?;
			data.generation_session = Some(generation_session);
			data.state = SessionState::SignatureComputing;

			self.core.disseminate_jobs(&mut data.consensus_session, &version, joint_public_and_secret.0, joint_public_and_secret.1, message_hash)?;

			debug_assert!(data.consensus_session.state() == ConsensusSessionState::Finished);
			let result = data.consensus_session.result()?;
			Self::set_signing_result(&self.core, &mut *data, Ok(result));
		}

		Ok(())
	}

	/// Process signing message.
	pub fn process_message(&self, sender: &NodeId, message: &SchnorrSigningMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&SchnorrSigningMessage::SchnorrSigningGenerationMessage(ref message) =>
				self.on_generation_message(sender, message),
			&SchnorrSigningMessage::SchnorrRequestPartialSignature(ref message) =>
				self.on_partial_signature_requested(sender, message),
			&SchnorrSigningMessage::SchnorrPartialSignature(ref message) =>
				self.on_partial_signature(sender, message),
			&SchnorrSigningMessage::SchnorrSigningSessionError(ref message) =>
				self.process_node_error(Some(&sender), message.error.clone()),
			&SchnorrSigningMessage::SchnorrSigningSessionCompleted(ref message) =>
				self.on_session_completed(sender, message),
			&SchnorrSigningMessage::SchnorrSigningSessionDelegation(ref message) =>
				self.on_session_delegated(sender, message),
			&SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(ref message) =>
				self.on_session_delegation_completed(sender, message),
		}
	}

	/// When session is delegated to this node.
	pub fn on_session_delegated(&self, sender: &NodeId, message: &SchnorrSigningSessionDelegation) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);

		{
			let mut data = self.data.lock();
			if data.consensus_session.state() != ConsensusSessionState::WaitingForInitialization || data.delegation_status.is_some() {
				return Err(Error::InvalidStateForRequest);
			}

			data.consensus_session.consensus_job_mut().executor_mut().set_requester(message.requester.clone().into());
			data.delegation_status = Some(DelegationStatus::DelegatedFrom(sender.clone(), message.session_nonce));
		}

		self.initialize(message.version.clone().into(), message.message_hash.clone().into())
	}

	/// When delegated session is completed on other node.
	pub fn on_session_delegation_completed(&self, sender: &NodeId, message: &SchnorrSigningSessionDelegationCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);

		if self.core.meta.master_node_id != self.core.meta.self_node_id {
			return Err(Error::InvalidStateForRequest);
		}

		let mut data = self.data.lock();
		match data.delegation_status.as_ref() {
			Some(&DelegationStatus::DelegatedTo(ref node)) if node == sender => (),
			_ => return Err(Error::InvalidMessage),
		}

		Self::set_signing_result(&self.core, &mut *data, Ok((message.signature_c.clone().into(), message.signature_s.clone().into())));

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &SchnorrSigningConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		let is_establishing_consensus = data.consensus_session.state() == ConsensusSessionState::EstablishingConsensus;

		if let &ConsensusMessage::InitializeConsensusSession(ref msg) = &message.message {
			let version = msg.version.clone().into();
			let has_key_share = self.core.key_share.as_ref()
				.map(|ks| ks.version(&version).is_ok())
				.unwrap_or(false);
			data.consensus_session.consensus_job_mut().executor_mut().set_has_key_share(has_key_share);
			data.version = Some(version);
		}
		data.consensus_session.on_consensus_message(&sender, &message.message)?;

		let is_consensus_established = data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished;
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		let consensus_group = data.consensus_session.select_consensus_group()?.clone();
		let mut other_consensus_group_nodes = consensus_group.clone();
		other_consensus_group_nodes.remove(&self.core.meta.self_node_id);

		let key_share = match self.core.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let generation_session = GenerationSession::new(GenerationSessionParams {
			id: self.core.meta.id.clone(),
			self_node_id: self.core.meta.self_node_id.clone(),
			key_storage: None,
			cluster: Arc::new(SessionKeyGenerationTransport {
				access_key: self.core.access_key.clone(),
				cluster: self.core.cluster.clone(),
				nonce: self.core.nonce,
				other_nodes_ids: other_consensus_group_nodes,
			}),
			nonce: None,
		}).0;

		generation_session.initialize(Default::default(), Default::default(), false, key_share.threshold, consensus_group.into())?;
		data.generation_session = Some(generation_session);
		data.state = SessionState::SessionKeyGeneration;

		Ok(())
	}

	/// When session key related message is received.
	pub fn on_generation_message(&self, sender: &NodeId, message: &SchnorrSigningGenerationMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		if let &GenerationMessage::InitializeSession(ref message) = &message.message {
			if &self.core.meta.master_node_id != sender {
				match data.delegation_status.as_ref() {
					Some(&DelegationStatus::DelegatedTo(s)) if s == *sender => (),
					_ => return Err(Error::InvalidMessage),
				}
			}

			let consensus_group: BTreeSet<NodeId> = message.nodes.keys().cloned().map(Into::into).collect();
			let mut other_consensus_group_nodes = consensus_group.clone();
			other_consensus_group_nodes.remove(&self.core.meta.self_node_id);

			let generation_session = GenerationSession::new(GenerationSessionParams {
				id: self.core.meta.id.clone(),
				self_node_id: self.core.meta.self_node_id.clone(),
				key_storage: None,
				cluster: Arc::new(SessionKeyGenerationTransport {
					access_key: self.core.access_key.clone(),
					cluster: self.core.cluster.clone(),
					nonce: self.core.nonce,
					other_nodes_ids: other_consensus_group_nodes
				}),
				nonce: None,
			}).0;
			data.generation_session = Some(generation_session);
			data.state = SessionState::SessionKeyGeneration;
		}

		{
			let generation_session = data.generation_session.as_ref().ok_or(Error::InvalidStateForRequest)?;
			let is_key_generating = generation_session.state() != GenerationSessionState::Finished;
			generation_session.process_message(sender, &message.message)?;

			let is_key_generated = generation_session.state() == GenerationSessionState::Finished;
			if !is_key_generating || !is_key_generated {
				return Ok(());
			}
		}

		data.state = SessionState::SignatureComputing;
		if self.core.meta.master_node_id != self.core.meta.self_node_id {
			return Ok(());
		}

		let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
		let message_hash = data.message_hash
			.expect("we are on master node; on master node message_hash is filled in initialize(); on_generation_message follows initialize; qed");
		let joint_public_and_secret = data.generation_session.as_ref()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")
			.joint_public_and_secret()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")?;
		self.core.disseminate_jobs(&mut data.consensus_session, &version, joint_public_and_secret.0, joint_public_and_secret.1, message_hash)
	}

	/// When partial signature is requested.
	pub fn on_partial_signature_requested(&self, sender: &NodeId, message: &SchnorrRequestPartialSignature) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let key_share = match self.core.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let mut data = self.data.lock();

		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}
		if data.state != SessionState::SignatureComputing {
			return Err(Error::InvalidStateForRequest);
		}

		let joint_public_and_secret = data.generation_session.as_ref()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")
			.joint_public_and_secret()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")?;
		let key_version = key_share.version(data.version.as_ref().ok_or(Error::InvalidMessage)?)?.hash.clone();
		let signing_job = SchnorrSigningJob::new_on_slave(self.core.meta.self_node_id.clone(), key_share.clone(), key_version, joint_public_and_secret.0, joint_public_and_secret.1)?;
		let signing_transport = self.core.signing_transport();

		data.consensus_session.on_job_request(sender, SchnorrPartialSigningRequest {
			id: message.request_id.clone().into(),
			message_hash: message.message_hash.clone().into(),
			other_nodes_ids: message.nodes.iter().cloned().map(Into::into).collect(),
		}, signing_job, signing_transport).map(|_| ())
	}

	/// When partial signature is received.
	pub fn on_partial_signature(&self, sender: &NodeId, message: &SchnorrPartialSignature) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		data.consensus_session.on_job_response(sender, SchnorrPartialSigningResponse {
			request_id: message.request_id.clone().into(),
			partial_signature: message.partial_signature.clone().into(),
		})?;

		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// send compeltion signal to all nodes, except for rejected nodes
		for node in data.consensus_session.consensus_non_rejected_nodes() {
			self.core.cluster.send(&node, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionCompleted(SchnorrSigningSessionCompleted {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.access_key.clone().into(),
				session_nonce: self.core.nonce,
			})))?;
		}

		let result = data.consensus_session.result()?;
		Self::set_signing_result(&self.core, &mut *data, Ok(result));

		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&self, sender: &NodeId, message: &SchnorrSigningSessionCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		self.data.lock().consensus_session.on_session_completed(sender)
	}

	/// Process error from the other node.
	fn process_node_error(&self, node: Option<&NodeId>, error: Error) -> Result<(), Error> {
		let mut data = self.data.lock();
		let is_self_node_error = node.map(|n| n == &self.core.meta.self_node_id).unwrap_or(false);
		// error is always fatal if coming from this node
		if is_self_node_error {
			Self::set_signing_result(&self.core, &mut *data, Err(error.clone()));
			return Err(error);
		}

		match {
			match node {
				Some(node) => data.consensus_session.on_node_error(node, error.clone()),
				None => data.consensus_session.on_session_timeout(),
			}
		} {
			Ok(false) => {
				Ok(())
			},
			Ok(true) => {
				let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
				let message_hash = data.message_hash.as_ref().cloned()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed");
				let joint_public_and_secret = data.generation_session.as_ref()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed")
					.joint_public_and_secret()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed")?;
				let disseminate_result = self.core.disseminate_jobs(&mut data.consensus_session, &version, joint_public_and_secret.0, joint_public_and_secret.1, message_hash);
				match disseminate_result {
					Ok(()) => Ok(()),
					Err(err) => {
						warn!("{}: signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);
						Self::set_signing_result(&self.core, &mut *data, Err(err.clone()));
						Err(err)
					}
				}
			},
			Err(err) => {
				warn!("{}: signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);
				Self::set_signing_result(&self.core, &mut *data, Err(err.clone()));
				Err(err)
			},
		}
	}

	/// Set signing session result.
	fn set_signing_result(core: &SessionCore, data: &mut SessionData, result: Result<(Secret, Secret), Error>) {
		if let Some(DelegationStatus::DelegatedFrom(master, nonce)) = data.delegation_status.take() {
			// error means can't communicate => ignore it
			let _ = match result.as_ref() {
				Ok(signature) => core.cluster.send(&master, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionDelegationCompleted(SchnorrSigningSessionDelegationCompleted {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					signature_c: signature.0.clone().into(),
					signature_s: signature.1.clone().into(),
				}))),
				Err(error) => core.cluster.send(&master, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionError(SchnorrSigningSessionError {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					error: error.clone().into(),
				}))),
			};
		}

		data.result = Some(result.clone());
		core.completed.send(result);
	}
}

impl ClusterSession for SessionImpl {
	type Id = SessionIdWithSubSession;
	type CreationData = Requester;
	type SuccessfulResult = (Secret, Secret);

	fn type_name() -> &'static str {
		"signing"
	}

	fn id(&self) -> SessionIdWithSubSession {
		SessionIdWithSubSession::new(self.core.meta.id.clone(), self.core.access_key.clone())
	}

	fn is_finished(&self) -> bool {
		let data = self.data.lock();
		data.consensus_session.state() == ConsensusSessionState::Failed
			|| data.consensus_session.state() == ConsensusSessionState::Finished
			|| data.result.is_some()
	}

	fn on_node_timeout(&self, node: &NodeId) {
		// ignore error, only state matters
		let _ = self.process_node_error(Some(node), Error::NodeDisconnected);
	}

	fn on_session_timeout(&self) {
		// ignore error, only state matters
		let _ = self.process_node_error(None, Error::NodeDisconnected);
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		let is_fatal = self.process_node_error(Some(node), error.clone()).is_err();
		let is_this_node_error = *node == self.core.meta.self_node_id;
		if is_fatal || is_this_node_error {
			// error in signing session is non-fatal, if occurs on slave node
			// => either respond with error
			// => or broadcast error
			let message = Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionError(SchnorrSigningSessionError {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.access_key.clone().into(),
				session_nonce: self.core.nonce,
				error: error.clone().into(),
			}));

			// do not bother processing send error, as we already processing error
			let _ = if self.core.meta.master_node_id == self.core.meta.self_node_id {
				self.core.cluster.broadcast(message)
			} else {
				self.core.cluster.send(&self.core.meta.master_node_id, message)
			};
		}
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		match *message {
			Message::SchnorrSigning(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl SessionKeyGenerationTransport {
	fn map_message(&self, message: Message) -> Result<Message, Error> {
		match message {
			Message::Generation(message) => Ok(Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningGenerationMessage(SchnorrSigningGenerationMessage {
				session: message.session_id().clone().into(),
				sub_session: self.access_key.clone().into(),
				session_nonce: self.nonce,
				message: message,
			}))),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl Cluster for SessionKeyGenerationTransport {
	fn broadcast(&self, message: Message) -> Result<(), Error> {
		let message = self.map_message(message)?;
		for to in &self.other_nodes_ids {
			self.cluster.send(to, message.clone())?;
		}
		Ok(())
	}

	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error> {
		debug_assert!(self.other_nodes_ids.contains(to));
		self.cluster.send(to, self.map_message(message)?)
	}

	fn is_connected(&self, node: &NodeId) -> bool {
		self.cluster.is_connected(node)
	}

	fn nodes(&self) -> BTreeSet<NodeId> {
		self.cluster.nodes()
	}

	fn configured_nodes_count(&self) -> usize {
		self.cluster.configured_nodes_count()
	}

	fn connected_nodes_count(&self) -> usize {
		self.cluster.connected_nodes_count()
	}
}

impl SessionCore {
	pub fn signing_transport(&self) -> SigningJobTransport {
		SigningJobTransport {
			id: self.meta.id.clone(),
			access_key: self.access_key.clone(),
			nonce: self.nonce,
			cluster: self.cluster.clone()
		}
	}

	pub fn disseminate_jobs(&self, consensus_session: &mut SigningConsensusSession, version: &H256, session_public: Public, session_secret_share: Secret, message_hash: H256) -> Result<(), Error> {
		let key_share = match self.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let key_version = key_share.version(version)?.hash.clone();
		let signing_job = SchnorrSigningJob::new_on_master(self.meta.self_node_id.clone(), key_share.clone(), key_version,
			session_public, session_secret_share, message_hash)?;
		consensus_session.disseminate_jobs(signing_job, self.signing_transport(), false).map(|_| ())
	}
}

impl JobTransport for SigningConsensusTransport {
	type PartialJobRequest=Requester;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Requester) -> Result<(), Error> {
		let version = self.version.as_ref()
			.expect("send_partial_request is called on initialized master node only; version is filled in before initialization starts on master node; qed");
		self.cluster.send(node, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningConsensusMessage(SchnorrSigningConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
				requester: request.into(),
				version: version.clone().into(),
			})
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningConsensusMessage(SchnorrSigningConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			})
		})))
	}
}

impl JobTransport for SigningJobTransport {
	type PartialJobRequest=SchnorrPartialSigningRequest;
	type PartialJobResponse=SchnorrPartialSigningResponse;

	fn send_partial_request(&self, node: &NodeId, request: SchnorrPartialSigningRequest) -> Result<(), Error> {
		self.cluster.send(node, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrRequestPartialSignature(SchnorrRequestPartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			request_id: request.id.into(),
			message_hash: request.message_hash.into(),
			nodes: request.other_nodes_ids.into_iter().map(Into::into).collect(),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: SchnorrPartialSigningResponse) -> Result<(), Error> {
		self.cluster.send(node, Message::SchnorrSigning(SchnorrSigningMessage::SchnorrPartialSignature(SchnorrPartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			request_id: response.request_id.into(),
			partial_signature: response.partial_signature.into(),
		})))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use std::collections::BTreeMap;
	use ethereum_types::{Address, H256};
	use ethkey::{self, Random, Generator, Public, Secret, public_to_address};
	use acl_storage::DummyAclStorage;
	use key_server_cluster::{SessionId, Requester, SessionMeta, Error, KeyStorage};
	use key_server_cluster::cluster::tests::MessageLoop as ClusterMessageLoop;
	use key_server_cluster::generation_session::tests::MessageLoop as GenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::{SchnorrSigningMessage, SchnorrSigningConsensusMessage,
		ConsensusMessage, ConfirmConsensusInitialization, SchnorrSigningGenerationMessage, GenerationMessage,
		ConfirmInitialization, InitializeSession, SchnorrRequestPartialSignature};
	use key_server_cluster::signing_session_schnorr::{SessionImpl, SessionState, SessionParams};

	#[derive(Debug)]
	pub struct MessageLoop(pub ClusterMessageLoop);

	impl MessageLoop {
		pub fn new(num_nodes: usize, threshold: usize) -> Result<Self, Error> {
			let ml = GenerationMessageLoop::new(num_nodes).init(threshold)?;
			ml.0.loop_until(|| ml.0.is_empty()); // complete generation session

			Ok(MessageLoop(ml.0))
		}

		pub fn into_session(&self, at_node: usize) -> SessionImpl {
			let requester = Some(Requester::Signature(ethkey::sign(Random.generate().unwrap().secret(),
				&SessionId::default()).unwrap()));
			SessionImpl::new(SessionParams {
				meta: SessionMeta {
					id: SessionId::default(),
					self_node_id: self.0.node(at_node),
					master_node_id: self.0.node(0),
					threshold: self.0.key_storage(at_node).get(&Default::default()).unwrap().unwrap().threshold,
					configured_nodes_count: self.0.nodes().len(),
					connected_nodes_count: self.0.nodes().len(),
				},
				access_key: Random.generate().unwrap().secret().clone(),
				key_share: self.0.key_storage(at_node).get(&Default::default()).unwrap(),
				acl_storage: Arc::new(DummyAclStorage::default()),
				cluster: self.0.cluster(0).view().unwrap(),
				nonce: 0,
			}, requester).unwrap().0
		}

		pub fn init_with_version(self, key_version: Option<H256>) -> Result<(Self, Public, H256), Error> {
			let message_hash = H256::random();
			let requester = Random.generate().unwrap();
			let signature = ethkey::sign(requester.secret(), &SessionId::default()).unwrap();
			self.0.cluster(0).client().new_schnorr_signing_session(
				Default::default(),
				signature.into(),
				key_version,
				message_hash).map(|_| (self, *requester.public(), message_hash))
		}

		pub fn init(self) -> Result<(Self, Public, H256), Error> {
			let key_version = self.key_version();
			self.init_with_version(Some(key_version))
		}

		pub fn init_delegated(self) -> Result<(Self, Public, H256), Error> {
			self.0.key_storage(0).remove(&Default::default()).unwrap();
			self.init_with_version(None)
		}

		pub fn init_with_isolated(self) -> Result<(Self, Public, H256), Error> {
			self.0.isolate(1);
			self.init()
		}

		pub fn init_without_share(self) -> Result<(Self, Public, H256), Error> {
			let key_version = self.key_version();
			self.0.key_storage(0).remove(&Default::default()).unwrap();
			self.init_with_version(Some(key_version))
		}

		pub fn session_at(&self, idx: usize) -> Arc<SessionImpl> {
			self.0.sessions(idx).schnorr_signing_sessions.first().unwrap()
		}

		pub fn ensure_completed(&self) {
			self.0.loop_until(|| self.0.is_empty());
			assert!(self.session_at(0).wait().is_ok());
		}

		pub fn key_version(&self) -> H256 {
			self.0.key_storage(0).get(&Default::default())
				.unwrap().unwrap().versions.iter().last().unwrap().hash
		}
	}

	#[test]
	fn schnorr_complete_gen_sign_session() {
		let test_cases = [(0, 1), (0, 5), (2, 5), (3, 5)];
		for &(threshold, num_nodes) in &test_cases {
			let (ml, _, message) = MessageLoop::new(num_nodes, threshold).unwrap().init().unwrap();
			ml.0.loop_until(|| ml.0.is_empty());

			let signer_public = ml.0.key_storage(0).get(&Default::default()).unwrap().unwrap().public;
			let signature = ml.session_at(0).wait().unwrap();
			assert!(math::verify_schnorr_signature(&signer_public, &signature, &message).unwrap());
		}
	}

	#[test]
	fn schnorr_constructs_in_cluster_of_single_node() {
		MessageLoop::new(1, 0).unwrap().init().unwrap();
	}

	#[test]
	fn schnorr_fails_to_initialize_if_does_not_have_a_share() {
		assert!(MessageLoop::new(2, 1).unwrap().init_without_share().is_err());
	}

	#[test]
	fn schnorr_fails_to_initialize_if_threshold_is_wrong() {
		let mut ml = MessageLoop::new(3, 2).unwrap();
		ml.0.exclude(2);
		assert_eq!(ml.init().unwrap_err(), Error::ConsensusUnreachable);
	}

	#[test]
	fn schnorr_fails_to_initialize_when_already_initialized() {
		let (ml, _, _) = MessageLoop::new(1, 0).unwrap().init().unwrap();
		assert_eq!(ml.session_at(0).initialize(ml.key_version(), H256::from_low_u64_be(777)),
			Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn schnorr_does_not_fail_when_consensus_message_received_after_consensus_established() {
		let (ml, _, _) = MessageLoop::new(3, 1).unwrap().init().unwrap();

		// consensus is established
		let session = ml.session_at(0);
		ml.0.loop_until(|| session.state() == SessionState::SessionKeyGeneration);

		// but 3rd node continues to send its messages
		// this should not fail session
		let consensus_group = session.data.lock().consensus_session.select_consensus_group().unwrap().clone();
		let mut had_3rd_message = false;
		while let Some((from, to, message)) = ml.0.take_message() {
			if !consensus_group.contains(&from) {
				had_3rd_message = true;
				ml.0.process_message(from, to, message);
			}
		}
		assert!(had_3rd_message);
	}

	#[test]
	fn schnorr_fails_when_consensus_message_is_received_when_not_initialized() {
		let ml = MessageLoop::new(3, 1).unwrap();
		let session = ml.into_session(0);
		assert_eq!(session.on_consensus_message(&ml.0.node(1), &SchnorrSigningConsensusMessage {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 0,
			message: ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: true,
			}),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn schnorr_fails_when_generation_message_is_received_when_not_initialized() {
		let ml = MessageLoop::new(3, 1).unwrap();
		let session = ml.into_session(0);
		assert_eq!(session.on_generation_message(&ml.0.node(1), &SchnorrSigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 0,
			message: GenerationMessage::ConfirmInitialization(ConfirmInitialization {
				session: SessionId::default().into(),
				session_nonce: 0,
				derived_point: Public::default().into(),
			}),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn schnorr_fails_when_generation_sesson_is_initialized_by_slave_node() {
		let (ml, _, _) = MessageLoop::new(3, 1).unwrap().init().unwrap();
		let session = ml.session_at(0);
		ml.0.loop_until(|| session.state() == SessionState::SessionKeyGeneration);

		let slave2_id = ml.0.node(2);
		let slave1_session = ml.session_at(1);

		assert_eq!(slave1_session.on_generation_message(&slave2_id, &SchnorrSigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 0,
			message: GenerationMessage::InitializeSession(InitializeSession {
				session: SessionId::default().into(),
				session_nonce: 0,
				origin: None,
				author: Address::zero().into(),
				nodes: BTreeMap::new(),
				is_zero: false,
				threshold: 1,
				derived_point: Public::default().into(),
			})
		}), Err(Error::InvalidMessage));
	}

	#[test]
	fn schnorr_fails_when_signature_requested_when_not_initialized() {
		let ml = MessageLoop::new(3, 1).unwrap();
		let session = ml.into_session(1);
		assert_eq!(session.on_partial_signature_requested(&ml.0.node(0), &SchnorrRequestPartialSignature {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 0,
			request_id: Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
			message_hash: H256::zero().into(),
			nodes: Default::default(),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn schnorr_fails_when_signature_requested_by_slave_node() {
		let ml = MessageLoop::new(3, 1).unwrap();
		let session = ml.into_session(0);
		assert_eq!(session.on_partial_signature_requested(&ml.0.node(1), &SchnorrRequestPartialSignature {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 0,
			request_id: Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
			message_hash: H256::zero().into(),
			nodes: Default::default(),
		}), Err(Error::InvalidMessage));
	}

	#[test]
	fn schnorr_failed_signing_session() {
		let (ml, requester, _) = MessageLoop::new(3, 1).unwrap().init().unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 2 of 3 nodes disagee
		ml.0.acl_storage(1).prohibit(public_to_address(&requester), SessionId::default());
		ml.0.acl_storage(2).prohibit(public_to_address(&requester), SessionId::default());

		// then consensus is unreachable
		ml.0.loop_until(|| ml.0.is_empty());
		assert_eq!(ml.session_at(0).wait().unwrap_err(), Error::ConsensusUnreachable);
	}

	#[test]
	fn schnorr_complete_signing_session_with_single_node_failing() {
		let (ml, requester, _) = MessageLoop::new(3, 1).unwrap().init().unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 1 of 3 nodes disagee
		ml.0.acl_storage(1).prohibit(public_to_address(&requester), SessionId::default());

		// then consensus reachable, but single node will disagree
		ml.ensure_completed();
	}

	#[test]
	fn schnorr_complete_signing_session_with_acl_check_failed_on_master() {
		let (ml, requester, _) = MessageLoop::new(3, 1).unwrap().init().unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 1 of 3 nodes disagee
		ml.0.acl_storage(0).prohibit(public_to_address(&requester), SessionId::default());

		// then consensus reachable, but single node will disagree
		ml.ensure_completed();
	}

	#[test]
	fn schnorr_signing_message_fails_when_nonce_is_wrong() {
		let ml = MessageLoop::new(3, 1).unwrap();
		let session = ml.into_session(1);
		let msg = SchnorrSigningMessage::SchnorrSigningGenerationMessage(SchnorrSigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: session.core.access_key.clone().into(),
			session_nonce: 10,
			message: GenerationMessage::ConfirmInitialization(ConfirmInitialization {
				session: SessionId::default().into(),
				session_nonce: 0,
				derived_point: Public::default().into(),
			}),
		});
		assert_eq!(session.process_message(&ml.0.node(1), &msg), Err(Error::ReplayProtection));
	}

	#[test]
	fn schnorr_signing_works_when_delegated_to_other_node() {
		let (ml, _, _) = MessageLoop::new(3, 1).unwrap().init_delegated().unwrap();
		ml.ensure_completed();
	}

	#[test]
	fn schnorr_signing_works_when_share_owners_are_isolated() {
		let (ml, _, _) = MessageLoop::new(3, 1).unwrap().init_with_isolated().unwrap();
		ml.ensure_completed();
	}
}
