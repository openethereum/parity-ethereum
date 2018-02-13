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
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use ethkey::{Public, Secret, Signature};
use ethereum_types::H256;
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::{SessionIdWithSubSession, ClusterSession};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	SessionState as GenerationSessionState};
use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, SigningGenerationMessage,
	RequestPartialSignature, PartialSignature, SigningSessionCompleted, GenerationMessage, ConsensusMessage, SigningSessionError,
	InitializeConsensusSession, ConfirmConsensusInitialization, SigningSessionDelegation, SigningSessionDelegationCompleted};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::key_access_job::KeyAccessJob;
use key_server_cluster::jobs::signing_job::{PartialSigningRequest, PartialSigningResponse, SigningJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Distributed signing session.
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
	pub cluster: Arc<Cluster>,
	/// Session-level nonce.
	pub nonce: u64,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
}

/// Signing consensus session type.
type SigningConsensusSession = ConsensusSession<KeyAccessJob, SigningConsensusTransport, SigningJob, SigningJobTransport>;

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
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
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
	cluster: Arc<Cluster>,
}

/// Signing key generation transport.
struct SessionKeyGenerationTransport {
	/// Session access key.
	access_key: Secret,
	/// Cluster.
	cluster: Arc<Cluster>,
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
	cluster: Arc<Cluster>,
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
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
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
			consensus_executor: match requester_signature {
				Some(requester_signature) => KeyAccessJob::new_on_master(params.meta.id.clone(), params.acl_storage.clone(), requester_signature),
				None => KeyAccessJob::new_on_slave(params.meta.id.clone(), params.acl_storage.clone()),
			},
			consensus_transport: consensus_transport,
		})?;

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta,
				access_key: params.access_key,
				key_share: params.key_share,
				cluster: params.cluster,
				nonce: params.nonce,
				completed: Condvar::new(),
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
		})
	}

	/// Get session state.
	#[cfg(test)]
	pub fn state(&self) -> SessionState {
		self.data.lock().state
	}

	/// Wait for session completion.
	pub fn wait(&self) -> Result<(Secret, Secret), Error> {
		Self::wait_session(&self.core.completed, &self.data, None, |data| data.result.clone())
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
		self.core.cluster.send(&master, Message::Signing(SigningMessage::SigningSessionDelegation(SigningSessionDelegation {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.access_key.clone().into(),
			session_nonce: self.core.nonce,
			requestor_signature: data.consensus_session.consensus_job().executor().requester_signature()
				.expect("signature is passed to master node on creation; session can be delegated from master node only; qed")
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
			Some(key_share) => key_share.version(&version).map_err(|e| Error::KeyStorage(e.into()))?,
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
			});
			generation_session.initialize(Public::default(), 0, vec![self.core.meta.self_node_id.clone()].into_iter().collect())?;

			debug_assert_eq!(generation_session.state(), GenerationSessionState::WaitingForGenerationConfirmation);
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
	pub fn process_message(&self, sender: &NodeId, message: &SigningMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&SigningMessage::SigningConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&SigningMessage::SigningGenerationMessage(ref message) =>
				self.on_generation_message(sender, message),
			&SigningMessage::RequestPartialSignature(ref message) =>
				self.on_partial_signature_requested(sender, message),
			&SigningMessage::PartialSignature(ref message) =>
				self.on_partial_signature(sender, message),
			&SigningMessage::SigningSessionError(ref message) =>
				self.process_node_error(Some(&sender), Error::Io(message.error.clone())),
			&SigningMessage::SigningSessionCompleted(ref message) =>
				self.on_session_completed(sender, message),
			&SigningMessage::SigningSessionDelegation(ref message) =>
				self.on_session_delegated(sender, message),
			&SigningMessage::SigningSessionDelegationCompleted(ref message) =>
				self.on_session_delegation_completed(sender, message),
		}
	}

	/// When session is delegated to this node.
	pub fn on_session_delegated(&self, sender: &NodeId, message: &SigningSessionDelegation) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);

		{
			let mut data = self.data.lock();
			if data.consensus_session.state() != ConsensusSessionState::WaitingForInitialization || data.delegation_status.is_some() {
				return Err(Error::InvalidStateForRequest);
			}

			data.consensus_session.consensus_job_mut().executor_mut().set_requester_signature(message.requestor_signature.clone().into());
			data.delegation_status = Some(DelegationStatus::DelegatedFrom(sender.clone(), message.session_nonce));
		}

		self.initialize(message.version.clone().into(), message.message_hash.clone().into())
	}

	/// When delegated session is completed on other node.
	pub fn on_session_delegation_completed(&self, sender: &NodeId, message: &SigningSessionDelegationCompleted) -> Result<(), Error> {
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
	pub fn on_consensus_message(&self, sender: &NodeId, message: &SigningConsensusMessage) -> Result<(), Error> {
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
		});
		generation_session.initialize(Public::default(), key_share.threshold, consensus_group)?;
		data.generation_session = Some(generation_session);
		data.state = SessionState::SessionKeyGeneration;

		Ok(())
	}

	/// When session key related message is received.
	pub fn on_generation_message(&self, sender: &NodeId, message: &SigningGenerationMessage) -> Result<(), Error> {
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
			});
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
	pub fn on_partial_signature_requested(&self, sender: &NodeId, message: &RequestPartialSignature) -> Result<(), Error> {
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
		let key_version = key_share.version(data.version.as_ref().ok_or(Error::InvalidMessage)?)
			.map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();
		let signing_job = SigningJob::new_on_slave(self.core.meta.self_node_id.clone(), key_share.clone(), key_version, joint_public_and_secret.0, joint_public_and_secret.1)?;
		let signing_transport = self.core.signing_transport();

		data.consensus_session.on_job_request(sender, PartialSigningRequest {
			id: message.request_id.clone().into(),
			message_hash: message.message_hash.clone().into(),
			other_nodes_ids: message.nodes.iter().cloned().map(Into::into).collect(),
		}, signing_job, signing_transport)
	}

	/// When partial signature is received.
	pub fn on_partial_signature(&self, sender: &NodeId, message: &PartialSignature) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		data.consensus_session.on_job_response(sender, PartialSigningResponse {
			request_id: message.request_id.clone().into(),
			partial_signature: message.partial_signature.clone().into(),
		})?;

		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// send compeltion signal to all nodes, except for rejected nodes
		for node in data.consensus_session.consensus_non_rejected_nodes() {
			self.core.cluster.send(&node, Message::Signing(SigningMessage::SigningSessionCompleted(SigningSessionCompleted {
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
	pub fn on_session_completed(&self, sender: &NodeId, message: &SigningSessionCompleted) -> Result<(), Error> {
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
				Some(node) => data.consensus_session.on_node_error(node),
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
				Ok(signature) => core.cluster.send(&master, Message::Signing(SigningMessage::SigningSessionDelegationCompleted(SigningSessionDelegationCompleted {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					signature_c: signature.0.clone().into(),
					signature_s: signature.1.clone().into(),
				}))),
				Err(error) => core.cluster.send(&master, Message::Signing(SigningMessage::SigningSessionError(SigningSessionError {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					error: error.clone().into(),
				}))),
			};
		}

		data.result = Some(result);
		core.completed.notify_all();
	}
}

impl ClusterSession for SessionImpl {
	type Id = SessionIdWithSubSession;

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
			let message = Message::Signing(SigningMessage::SigningSessionError(SigningSessionError {
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
			Message::Signing(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl SessionKeyGenerationTransport {
	fn map_message(&self, message: Message) -> Result<Message, Error> {
		match message {
			Message::Generation(message) => Ok(Message::Signing(SigningMessage::SigningGenerationMessage(SigningGenerationMessage {
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

		let key_version = key_share.version(version).map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();
		let signing_job = SigningJob::new_on_master(self.meta.self_node_id.clone(), key_share.clone(), key_version, session_public, session_secret_share, message_hash)?;
		consensus_session.disseminate_jobs(signing_job, self.signing_transport(), false)
	}
}

impl JobTransport for SigningConsensusTransport {
	type PartialJobRequest=Signature;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Signature) -> Result<(), Error> {
		let version = self.version.as_ref()
			.expect("send_partial_request is called on initialized master node only; version is filled in before initialization starts on master node; qed");
		self.cluster.send(node, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
				requestor_signature: request.into(),
				version: version.clone().into(),
			})
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
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
	type PartialJobRequest=PartialSigningRequest;
	type PartialJobResponse=PartialSigningResponse;

	fn send_partial_request(&self, node: &NodeId, request: PartialSigningRequest) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::RequestPartialSignature(RequestPartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			request_id: request.id.into(),
			message_hash: request.message_hash.into(),
			nodes: request.other_nodes_ids.into_iter().map(Into::into).collect(),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: PartialSigningResponse) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::PartialSignature(PartialSignature {
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
	use std::collections::{BTreeMap, VecDeque};
	use ethereum_types::H256;
	use ethkey::{self, Random, Generator, Public, Secret, KeyPair};
	use acl_storage::DummyAclStorage;
	use key_server_cluster::{NodeId, DummyKeyStorage, DocumentKeyShare, DocumentKeyShareVersion, SessionId, SessionMeta, Error, KeyStorage};
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::MessageLoop as KeyGenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, ConsensusMessage, ConfirmConsensusInitialization,
		SigningGenerationMessage, GenerationMessage, ConfirmInitialization, InitializeSession, RequestPartialSignature};
	use key_server_cluster::signing_session::{SessionImpl, SessionState, SessionParams};

	struct Node {
		pub node_id: NodeId,
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub requester: KeyPair,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
		pub acl_storages: Vec<Arc<DummyAclStorage>>,
		pub version: H256,
	}

	impl MessageLoop {
		pub fn new(gl: &KeyGenerationMessageLoop) -> Self {
			let version = gl.nodes.values().nth(0).unwrap().key_storage.get(&Default::default()).unwrap().unwrap().versions.iter().last().unwrap().hash;
			let mut nodes = BTreeMap::new();
			let session_id = gl.session_id.clone();
			let requester = Random.generate().unwrap();
			let signature = Some(ethkey::sign(requester.secret(), &SessionId::default()).unwrap());
			let master_node_id = gl.nodes.keys().nth(0).unwrap().clone();
			let mut acl_storages = Vec::new();
			for (i, (gl_node_id, gl_node)) in gl.nodes.iter().enumerate() {
				let acl_storage = Arc::new(DummyAclStorage::default());
				acl_storages.push(acl_storage.clone());
				let cluster = Arc::new(DummyCluster::new(gl_node_id.clone()));
				let session = SessionImpl::new(SessionParams {
					meta: SessionMeta {
						id: session_id.clone(),
						self_node_id: gl_node_id.clone(),
						master_node_id: master_node_id.clone(),
						threshold: gl_node.key_storage.get(&session_id).unwrap().unwrap().threshold,
					},
					access_key: "834cb736f02d9c968dfaf0c37658a1d86ff140554fc8b59c9fdad5a8cf810eec".parse().unwrap(),
					key_share: Some(gl_node.key_storage.get(&session_id).unwrap().unwrap()),
					acl_storage: acl_storage,
					cluster: cluster.clone(),
					nonce: 0,
				}, if i == 0 { signature.clone() } else { None }).unwrap();
				nodes.insert(gl_node_id.clone(), Node { node_id: gl_node_id.clone(), cluster: cluster, key_storage: gl_node.key_storage.clone(), session: session });
			}

			let nodes_ids: Vec<_> = nodes.keys().cloned().collect();
			for node in nodes.values() {
				for node_id in &nodes_ids {
					node.cluster.add_node(node_id.clone());
				}
			}

			MessageLoop {
				session_id: session_id,
				requester: requester,
				nodes: nodes,
				queue: VecDeque::new(),
				acl_storages: acl_storages,
				version: version,
			}
		}

		pub fn master(&self) -> &SessionImpl {
			&self.nodes.values().nth(0).unwrap().session
		}

		pub fn take_message(&mut self) -> Option<(NodeId, NodeId, Message)> {
			self.nodes.values()
				.filter_map(|n| n.cluster.take_message().map(|m| (n.node_id.clone(), m.0, m.1)))
				.nth(0)
				.or_else(|| self.queue.pop_front())
		}

		pub fn process_message(&mut self, mut msg: (NodeId, NodeId, Message)) -> Result<(), Error> {
			let mut is_queued_message = false;
			loop {
				match self.nodes[&msg.1].session.on_message(&msg.0, &msg.2) {
					Ok(_) => {
						if let Some(message) = self.queue.pop_front() {
							msg = message;
							is_queued_message = true;
							continue;
						}
						return Ok(());
					},
					Err(Error::TooEarlyForRequest) => {
						if is_queued_message {
							self.queue.push_front(msg);
						} else {
							self.queue.push_back(msg);
						}
						return Ok(());
					},
					Err(err) => return Err(err),
				}
			}
		}

		pub fn run_until<F: Fn(&MessageLoop) -> bool>(&mut self, predicate: F) -> Result<(), Error> {
			while let Some((from, to, message)) = self.take_message() {
				if predicate(self) {
					return Ok(());
				}

				self.process_message((from, to, message))?;
			}

			unreachable!("either wrong predicate, or failing test")
		}
	}

	fn prepare_signing_sessions(threshold: usize, num_nodes: usize) -> (KeyGenerationMessageLoop, MessageLoop) {
		// run key generation sessions
		let mut gl = KeyGenerationMessageLoop::new(num_nodes);
		gl.master().initialize(Public::default(), threshold, gl.nodes.keys().cloned().collect()).unwrap();
		while let Some((from, to, message)) = gl.take_message() {
			gl.process_message((from, to, message)).unwrap();
		}

		// run signing session
		let sl = MessageLoop::new(&gl);
		(gl, sl)
	}

	#[test]
	fn complete_gen_sign_session() {
		let test_cases = [(0, 1), (0, 5), (2, 5), (3, 5)];
		for &(threshold, num_nodes) in &test_cases {
			let (gl, mut sl) = prepare_signing_sessions(threshold, num_nodes);

			// run signing session
			let message_hash = H256::from(777);
			sl.master().initialize(sl.version.clone(), message_hash).unwrap();
			while let Some((from, to, message)) = sl.take_message() {
				sl.process_message((from, to, message)).unwrap();
			}

			// verify signature
			let public = gl.master().joint_public_and_secret().unwrap().unwrap().0;
			let signature = sl.master().wait().unwrap();
			assert!(math::verify_signature(&public, &signature, &message_hash).unwrap());
		}
	}

	#[test]
	fn constructs_in_cluster_of_single_node() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(self_node_id, Random.generate().unwrap().secret().clone());
		match SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: SessionId::default(),
				self_node_id: self_node_id.clone(),
				master_node_id: self_node_id.clone(),
				threshold: 0,
			},
			access_key: Random.generate().unwrap().secret().clone(),
			key_share: Some(DocumentKeyShare {
				author: Public::default(),
				threshold: 0,
				public: Default::default(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
				versions: vec![DocumentKeyShareVersion {
					hash: Default::default(),
					id_numbers: nodes,
					secret_share: Random.generate().unwrap().secret().clone(),
				}],
			}),
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
			nonce: 0,
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())) {
			Ok(_) => (),
			_ => panic!("unexpected"),
		}
	}

	#[test]
	fn fails_to_initialize_if_does_not_have_a_share() {
		let self_node_id = Random.generate().unwrap().public().clone();
		let session = SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: SessionId::default(),
				self_node_id: self_node_id.clone(),
				master_node_id: self_node_id.clone(),
				threshold: 0,
			},
			access_key: Random.generate().unwrap().secret().clone(),
			key_share: None,
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
			nonce: 0,
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())).unwrap();
		assert_eq!(session.initialize(Default::default(), Default::default()), Err(Error::InvalidMessage));
	}

	#[test]
	fn fails_to_initialize_if_threshold_is_wrong() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(self_node_id.clone(), Random.generate().unwrap().secret().clone());
		nodes.insert(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone());
		let session = SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: SessionId::default(),
				self_node_id: self_node_id.clone(),
				master_node_id: self_node_id.clone(),
				threshold: 2,
			},
			access_key: Random.generate().unwrap().secret().clone(),
			key_share: Some(DocumentKeyShare {
				author: Public::default(),
				threshold: 2,
				public: Default::default(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
				versions: vec![DocumentKeyShareVersion {
					hash: Default::default(),
					id_numbers: nodes,
					secret_share: Random.generate().unwrap().secret().clone(),
				}],
			}),
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
			nonce: 0,
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())).unwrap();
		assert_eq!(session.initialize(Default::default(), Default::default()), Err(Error::ConsensusUnreachable));
	}

	#[test]
	fn fails_to_initialize_when_already_initialized() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().initialize(sl.version.clone(), 777.into()), Ok(()));
		assert_eq!(sl.master().initialize(sl.version.clone(), 777.into()), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn does_not_fail_when_consensus_message_received_after_consensus_established() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();
		// consensus is established
		sl.run_until(|sl| sl.master().state() == SessionState::SessionKeyGeneration).unwrap();
		// but 3rd node continues to send its messages
		// this should not fail session
		let consensus_group = sl.master().data.lock().consensus_session.select_consensus_group().unwrap().clone();
		let mut had_3rd_message = false;
		while let Some((from, to, message)) = sl.take_message() {
			if !consensus_group.contains(&from) {
				had_3rd_message = true;
				sl.process_message((from, to, message)).unwrap();
			}
		}
		assert!(had_3rd_message);
	}

	#[test]
	fn fails_when_consensus_message_is_received_when_not_initialized() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().on_consensus_message(sl.nodes.keys().nth(1).unwrap(), &SigningConsensusMessage {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 0,
			message: ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: true,
			}),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn fails_when_generation_message_is_received_when_not_initialized() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().on_generation_message(sl.nodes.keys().nth(1).unwrap(), &SigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 0,
			message: GenerationMessage::ConfirmInitialization(ConfirmInitialization {
				session: SessionId::default().into(),
				session_nonce: 0,
				derived_point: Public::default().into(),
			}),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn fails_when_generation_sesson_is_initialized_by_slave_node() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();
		sl.run_until(|sl| sl.master().state() == SessionState::SessionKeyGeneration).unwrap();

		let slave2_id = sl.nodes.keys().nth(2).unwrap().clone();
		let slave1 = &sl.nodes.values().nth(1).unwrap().session;

		assert_eq!(slave1.on_generation_message(&slave2_id, &SigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 0,
			message: GenerationMessage::InitializeSession(InitializeSession {
				session: SessionId::default().into(),
				session_nonce: 0,
				author: Public::default().into(),
				nodes: BTreeMap::new(),
				threshold: 1,
				derived_point: Public::default().into(),
			})
		}), Err(Error::InvalidMessage));
	}

	#[test]
	fn fails_when_signature_requested_when_not_initialized() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		let slave1 = &sl.nodes.values().nth(1).unwrap().session;
		assert_eq!(slave1.on_partial_signature_requested(sl.nodes.keys().nth(0).unwrap(), &RequestPartialSignature {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 0,
			request_id: Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
			message_hash: H256::default().into(),
			nodes: Default::default(),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn fails_when_signature_requested_by_slave_node() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().on_partial_signature_requested(sl.nodes.keys().nth(1).unwrap(), &RequestPartialSignature {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 0,
			request_id: Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
			message_hash: H256::default().into(),
			nodes: Default::default(),
		}), Err(Error::InvalidMessage));
	}

	#[test]
	fn failed_signing_session() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 2 of 3 nodes disagee
		sl.acl_storages[1].prohibit(sl.requester.public().clone(), SessionId::default());
		sl.acl_storages[2].prohibit(sl.requester.public().clone(), SessionId::default());

		// then consensus is unreachable
		assert_eq!(sl.run_until(|_| false), Err(Error::ConsensusUnreachable));
	}

	#[test]
	fn complete_signing_session_with_single_node_failing() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 1 of 3 nodes disagee
		sl.acl_storages[1].prohibit(sl.requester.public().clone(), SessionId::default());

		// then consensus reachable, but single node will disagree
		while let Some((from, to, message)) = sl.take_message() {
			sl.process_message((from, to, message)).unwrap();
		}

		let data = sl.master().data.lock();
		match data.result {
			Some(Ok(_)) => (),
			_ => unreachable!(),
		}
	}

	#[test]
	fn complete_signing_session_with_acl_check_failed_on_master() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// we need at least 2-of-3 nodes to agree to reach consensus
		// let's say 1 of 3 nodes disagee
		sl.acl_storages[0].prohibit(sl.requester.public().clone(), SessionId::default());

		// then consensus reachable, but single node will disagree
		while let Some((from, to, message)) = sl.take_message() {
			sl.process_message((from, to, message)).unwrap();
		}

		let data = sl.master().data.lock();
		match data.result {
			Some(Ok(_)) => (),
			_ => unreachable!(),
		}
	}

	#[test]
	fn signing_message_fails_when_nonce_is_wrong() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().process_message(sl.nodes.keys().nth(1).unwrap(), &SigningMessage::SigningGenerationMessage(SigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			session_nonce: 10,
			message: GenerationMessage::ConfirmInitialization(ConfirmInitialization {
				session: SessionId::default().into(),
				session_nonce: 0,
				derived_point: Public::default().into(),
			}),
		})), Err(Error::ReplayProtection));
	}

	#[test]
	fn signing_works_when_delegated_to_other_node() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);

		// let's say node1 doesn't have a share && delegates decryption request to node0
		// initially session is created on node1 => node1 is master for itself, but for other nodes node0 is still master
		let actual_master = sl.nodes.keys().nth(0).cloned().unwrap();
		let requested_node = sl.nodes.keys().skip(1).nth(0).cloned().unwrap();
		let version = sl.nodes[&actual_master].key_storage.get(&Default::default()).unwrap().unwrap().last_version().unwrap().hash.clone();
		sl.nodes[&requested_node].key_storage.remove(&Default::default()).unwrap();
		sl.nodes.get_mut(&requested_node).unwrap().session.core.key_share = None;
		sl.nodes.get_mut(&requested_node).unwrap().session.core.meta.master_node_id = sl.nodes[&requested_node].session.core.meta.self_node_id.clone();
		sl.nodes[&requested_node].session.data.lock().consensus_session.consensus_job_mut().executor_mut().set_requester_signature(
			sl.nodes[&actual_master].session.data.lock().consensus_session.consensus_job().executor().requester_signature().unwrap().clone()
		);

		// now let's try to do a decryption
		sl.nodes[&requested_node].session.delegate(actual_master, version, Default::default()).unwrap();

		// then consensus reachable, but single node will disagree
		while let Some((from, to, message)) = sl.take_message() {
			sl.process_message((from, to, message)).unwrap();
		}
	}

	#[test]
	fn signing_works_when_share_owners_are_isolated() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);

		// we need 2 out of 3 nodes to agree to do a decryption
		// let's say that 1 of these nodes (master) is isolated
		let isolated_node_id = sl.nodes.keys().skip(2).nth(0).cloned().unwrap();
		for node in sl.nodes.values() {
			node.cluster.remove_node(&isolated_node_id);
		}

		// now let's try to do a signing
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// then consensus reachable, but single node will disagree
		while let Some((from, to, message)) = sl.take_message() {
			sl.process_message((from, to, message)).unwrap();
		}

		let data = sl.master().data.lock();
		match data.result {
			Some(Ok(_)) => (),
			_ => unreachable!(),
		}
	}
}
