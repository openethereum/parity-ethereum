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
use ethereum_types::H256;
use ethkey::{Secret, Signature};
use key_server_cluster::{Error, AclStorage, DocumentKeyShare, NodeId, SessionId, EncryptedDocumentKeyShadow, SessionMeta};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::{SessionIdWithSubSession, ClusterSession};
use key_server_cluster::message::{Message, DecryptionMessage, DecryptionConsensusMessage, RequestPartialDecryption,
	PartialDecryption, DecryptionSessionError, DecryptionSessionCompleted, ConsensusMessage, InitializeConsensusSession,
	ConfirmConsensusInitialization, DecryptionSessionDelegation, DecryptionSessionDelegationCompleted};
use key_server_cluster::jobs::job_session::{JobSession, JobTransport};
use key_server_cluster::jobs::key_access_job::KeyAccessJob;
use key_server_cluster::jobs::decryption_job::{PartialDecryptionRequest, PartialDecryptionResponse, DecryptionJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Distributed decryption session.
/// Based on "ECDKG: A Distributed Key Generation Protocol Based on Elliptic Curve Discrete Logarithm" paper:
/// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.124.4128&rep=rep1&type=pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for decrypting the secret) requests all other nodes to decrypt the secret
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor has access to the document
/// 3) partial decryption: every node which has succussfully checked access for the requestor do a partial decryption
/// 4) decryption: master node receives all partial decryptions of the secret and restores the secret
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
	/// Decryption session access key.
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

/// Decryption consensus session type.
type DecryptionConsensusSession = ConsensusSession<KeyAccessJob, DecryptionConsensusTransport, DecryptionJob, DecryptionJobTransport>;
/// Broadcast decryption job session type.
type BroadcastDecryptionJobSession = JobSession<DecryptionJob, DecryptionJobTransport>;

/// Mutable session data.
struct SessionData {
	/// Key version to use for decryption.
	pub version: Option<H256>,
	/// Consensus-based decryption session.
	pub consensus_session: DecryptionConsensusSession,
	/// Broadcast decryption job.
	pub broadcast_job_session: Option<BroadcastDecryptionJobSession>,
	/// Is shadow decryption requested?
	pub is_shadow_decryption: Option<bool>,
	/// Decryption result must be reconstructed on all participating nodes. This is useful
	/// for service contract API so that all nodes from consensus group can confirm decryption.
	pub is_broadcast_session: Option<bool>,
	/// Delegation status.
	pub delegation_status: Option<DelegationStatus>,
	/// Decryption result.
	pub result: Option<Result<EncryptedDocumentKeyShadow, Error>>,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Session access key.
	pub access_key: Secret,
	/// Key share.
	pub key_share: Option<DocumentKeyShare>,
	/// ACL storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Session nonce.
	pub nonce: u64,
}

/// Decryption consensus transport.
struct DecryptionConsensusTransport {
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

/// Decryption job transport
struct DecryptionJobTransport {
	/// Session id.
	id: SessionId,
	//// Session access key.
	access_key: Secret,
	/// Session-level nonce.
	nonce: u64,
	/// Is this a broadcast transport? If true, requests are not send and responses are sent only to non-master nodes.
	is_broadcast_transport: bool,
	/// Master node id.
	master_node_id: NodeId,
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
	/// Create new decryption session.
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
		debug_assert_eq!(params.meta.threshold, params.key_share.as_ref().map(|ks| ks.threshold).unwrap_or_default());

		// check that common_point and encrypted_point are already set
		if let Some(key_share) = params.key_share.as_ref() {
			// encrypted data must be set
			if key_share.common_point.is_none() || key_share.encrypted_point.is_none() {
				return Err(Error::NotStartedSessionId);
			}
		}

		let consensus_transport = DecryptionConsensusTransport {
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
				version: None,
				consensus_session: consensus_session,
				broadcast_job_session: None,
				is_shadow_decryption: None,
				is_broadcast_session: None,
				delegation_status: None,
				result: None,
			}),
		})
	}

	/// Get this node id.
	#[cfg(test)]
	pub fn node(&self) -> &NodeId {
		&self.core.meta.self_node_id
	}

	/// Get this session access key.
	#[cfg(test)]
	pub fn access_key(&self) -> &Secret {
		&self.core.access_key
	}

	/// Get session state.
	#[cfg(test)]
	pub fn state(&self) -> ConsensusSessionState {
		self.data.lock().consensus_session.state()
	}

	/// Get decrypted secret
	#[cfg(test)]
	pub fn decrypted_secret(&self) -> Option<Result<EncryptedDocumentKeyShadow, Error>> {
		self.data.lock().result.clone()
	}

	/// Wait for session completion.
	pub fn wait(&self) -> Result<EncryptedDocumentKeyShadow, Error> {
		Self::wait_session(&self.core.completed, &self.data, None, |data| data.result.clone())
	}

	/// Delegate session to other node.
	pub fn delegate(&self, master: NodeId, version: H256, is_shadow_decryption: bool, is_broadcast_session: bool) -> Result<(), Error> {
		if self.core.meta.master_node_id != self.core.meta.self_node_id {
			return Err(Error::InvalidStateForRequest);
		}

		let mut data = self.data.lock();
		if data.consensus_session.state() != ConsensusSessionState::WaitingForInitialization || data.delegation_status.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		data.consensus_session.consensus_job_mut().executor_mut().set_has_key_share(false);
		self.core.cluster.send(&master, Message::Decryption(DecryptionMessage::DecryptionSessionDelegation(DecryptionSessionDelegation {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.access_key.clone().into(),
			session_nonce: self.core.nonce,
			requestor_signature: data.consensus_session.consensus_job().executor().requester_signature()
				.expect("signature is passed to master node on creation; session can be delegated from master node only; qed")
				.clone().into(),
			version: version.into(),
			is_shadow_decryption: is_shadow_decryption,
			is_broadcast_session: is_broadcast_session,
		})))?;
		data.delegation_status = Some(DelegationStatus::DelegatedTo(master));
		Ok(())
	}

	/// Initialize decryption session on master node.
	pub fn initialize(&self, version: H256, is_shadow_decryption: bool, is_broadcast_session: bool) -> Result<(), Error> {
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
		data.is_shadow_decryption = Some(is_shadow_decryption);
		data.is_broadcast_session = Some(is_broadcast_session);
		data.consensus_session.initialize(consensus_nodes)?;

		if data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished {
			Self::disseminate_jobs(&self.core, &mut *data, &version, is_shadow_decryption, is_broadcast_session)?;

			debug_assert!(data.consensus_session.state() == ConsensusSessionState::Finished);
			let result = data.consensus_session.result()?;
			Self::set_decryption_result(&self.core, &mut *data, Ok(result));
		}

		Ok(())
	}

	/// Process decryption message.
	pub fn process_message(&self, sender: &NodeId, message: &DecryptionMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&DecryptionMessage::DecryptionConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&DecryptionMessage::RequestPartialDecryption(ref message) =>
				self.on_partial_decryption_requested(sender, message),
			&DecryptionMessage::PartialDecryption(ref message) =>
				self.on_partial_decryption(sender, message),
			&DecryptionMessage::DecryptionSessionError(ref message) =>
				self.process_node_error(Some(&sender), Error::Io(message.error.clone())),
			&DecryptionMessage::DecryptionSessionCompleted(ref message) =>
				self.on_session_completed(sender, message),
			&DecryptionMessage::DecryptionSessionDelegation(ref message) =>
				self.on_session_delegated(sender, message),
			&DecryptionMessage::DecryptionSessionDelegationCompleted(ref message) =>
				self.on_session_delegation_completed(sender, message),
		}
	}

	/// When session is delegated to this node.
	pub fn on_session_delegated(&self, sender: &NodeId, message: &DecryptionSessionDelegation) -> Result<(), Error> {
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

		self.initialize(message.version.clone().into(), message.is_shadow_decryption, message.is_broadcast_session)
	}

	/// When delegated session is completed on other node.
	pub fn on_session_delegation_completed(&self, sender: &NodeId, message: &DecryptionSessionDelegationCompleted) -> Result<(), Error> {
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

		Self::set_decryption_result(&self.core, &mut *data, Ok(EncryptedDocumentKeyShadow {
				decrypted_secret: message.decrypted_secret.clone().into(),
				common_point: message.common_point.clone().map(Into::into),
				decrypt_shadows: message.decrypt_shadows.clone().map(Into::into),
			}));

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &DecryptionConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);

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

		let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
		let is_shadow_decryption = data.is_shadow_decryption
			.expect("we are on master node; on master node is_shadow_decryption is filled in initialize(); on_consensus_message follows initialize (state check in consensus_session); qed");
		let is_broadcast_session = data.is_broadcast_session
			.expect("we are on master node; on master node is_broadcast_session is filled in initialize(); on_consensus_message follows initialize (state check in consensus_session); qed");
		Self::disseminate_jobs(&self.core, &mut *data, &version, is_shadow_decryption, is_broadcast_session)
	}

	/// When partial decryption is requested.
	pub fn on_partial_decryption_requested(&self, sender: &NodeId, message: &RequestPartialDecryption) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let key_share = match self.core.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let mut data = self.data.lock();
		let key_version = key_share.version(data.version.as_ref().ok_or(Error::InvalidMessage)?)
			.map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();
		let requester = data.consensus_session.consensus_job().executor().requester()?.ok_or(Error::InvalidStateForRequest)?.clone();
		let decryption_job = DecryptionJob::new_on_slave(self.core.meta.self_node_id.clone(), self.core.access_key.clone(), requester, key_share.clone(), key_version)?;
		let decryption_transport = self.core.decryption_transport(false);

		// respond to request
		data.consensus_session.on_job_request(sender, PartialDecryptionRequest {
			id: message.request_id.clone().into(),
			is_shadow_decryption: message.is_shadow_decryption,
			is_broadcast_session: message.is_broadcast_session,
			other_nodes_ids: message.nodes.iter().cloned().map(Into::into).collect(),
		}, decryption_job, decryption_transport)?;

		// ...and prepare decryption job session if we need to broadcast result
		if message.is_broadcast_session {
			let consensus_group: BTreeSet<_> = message.nodes.iter().cloned().map(Into::into).collect();
			let broadcast_decryption_job = DecryptionJob::new_on_master(self.core.meta.self_node_id.clone(),
				self.core.access_key.clone(), requester, key_share.clone(), key_version,
				message.is_shadow_decryption, message.is_broadcast_session)?;
			Self::create_broadcast_decryption_job(&self.core, &mut *data, consensus_group, broadcast_decryption_job,
				message.request_id.clone().into())?;
		}

		Ok(())
	}

	/// When partial decryption is received.
	pub fn on_partial_decryption(&self, sender: &NodeId, message: &PartialDecryption) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		if self.core.meta.self_node_id == self.core.meta.master_node_id {
			data.consensus_session.on_job_response(sender, PartialDecryptionResponse {
				request_id: message.request_id.clone().into(),
				shadow_point: message.shadow_point.clone().into(),
				decrypt_shadow: message.decrypt_shadow.clone(),
			})?;
		} else {
			match data.broadcast_job_session.as_mut() {
				Some(broadcast_job_session) => broadcast_job_session.on_partial_response(sender, PartialDecryptionResponse {
					request_id: message.request_id.clone().into(),
					shadow_point: message.shadow_point.clone().into(),
					decrypt_shadow: message.decrypt_shadow.clone(),
				})?,
				None => return Err(Error::TooEarlyForRequest),
			}
		}

		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// send compeltion signal to all nodes, except for rejected nodes
		for node in data.consensus_session.consensus_non_rejected_nodes() {
			self.core.cluster.send(&node, Message::Decryption(DecryptionMessage::DecryptionSessionCompleted(DecryptionSessionCompleted {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.access_key.clone().into(),
				session_nonce: self.core.nonce,
			})))?;
		}

		let result = data.consensus_session.result()?;
		Self::set_decryption_result(&self.core, &mut *data, Ok(result));

		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&self, sender: &NodeId, message: &DecryptionSessionCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		// if it is a broadcast session, wait for all answers before completing the session
		let decryption_result = match data.broadcast_job_session.as_ref() {
			Some(broadcast_job_session) => {
				if !broadcast_job_session.is_result_ready() {
					return Err(Error::TooEarlyForRequest);
				}

				Some(broadcast_job_session.result())
			},
			None => None,
		};
		if let Some(decryption_result) = decryption_result {
			Self::set_decryption_result(&self.core, &mut *data, decryption_result);
		}

		data.consensus_session.on_session_completed(sender)
	}

	/// Process error from the other node.
	fn process_node_error(&self, node: Option<&NodeId>, error: Error) -> Result<(), Error> {
		let mut data = self.data.lock();
		let is_self_node_error = node.map(|n| n == &self.core.meta.self_node_id).unwrap_or(false);
		// error is always fatal if coming from this node
		if is_self_node_error {
			Self::set_decryption_result(&self.core, &mut *data, Err(error.clone()));
			return Err(error);
		}

		match {
			match node {
				Some(node) => data.consensus_session.on_node_error(node),
				None => data.consensus_session.on_session_timeout(),
			}
		} {
			Ok(false) => Ok(()),
			Ok(true) => {
				let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
				let proof = "on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when is_shadow_decryption.is_some(); qed";
				let is_shadow_decryption = data.is_shadow_decryption.expect(proof);
				let is_broadcast_session = data.is_broadcast_session.expect(proof);
				let disseminate_result = Self::disseminate_jobs(&self.core, &mut *data, &version, is_shadow_decryption, is_broadcast_session);
				match disseminate_result {
					Ok(()) => Ok(()),
					Err(err) => {
						warn!("{}: decryption session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);

						Self::set_decryption_result(&self.core, &mut *data, Err(err.clone()));
						Err(err)
					}
				}
			},
			Err(err) => {
				warn!("{}: decryption session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);

				Self::set_decryption_result(&self.core, &mut *data, Err(err.clone()));
				Err(err)
			},
		}
	}

	/// Disseminate jobs on session master.
	fn disseminate_jobs(core: &SessionCore, data: &mut SessionData, version: &H256, is_shadow_decryption: bool, is_broadcast_session: bool) -> Result<(), Error> {
		let key_share = match core.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let key_version = key_share.version(version).map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();
		let requester = data.consensus_session.consensus_job().executor().requester()?.ok_or(Error::InvalidStateForRequest)?.clone();
		let consensus_group = data.consensus_session.select_consensus_group()?.clone();
		let decryption_job = DecryptionJob::new_on_master(core.meta.self_node_id.clone(),
			core.access_key.clone(), requester, key_share.clone(), key_version,
			is_shadow_decryption, is_broadcast_session)?;
		let decryption_request_id = decryption_job.request_id().clone().expect("TODO");
		let decryption_transport = core.decryption_transport(false);
		data.consensus_session.disseminate_jobs(decryption_job, decryption_transport, data.is_broadcast_session.expect("TODO"))?;

		// ...and prepare decryption job session if we need to broadcast result
		if data.is_broadcast_session.expect("TODO") {
			let broadcast_decryption_job = DecryptionJob::new_on_master(core.meta.self_node_id.clone(),
				core.access_key.clone(), requester, key_share.clone(), key_version, is_shadow_decryption, is_broadcast_session)?;
			Self::create_broadcast_decryption_job(&core, data, consensus_group, broadcast_decryption_job,
				decryption_request_id)?;
		}

		Ok(())
	}

	/// Create broadcast decryption job.
	fn create_broadcast_decryption_job(core: &SessionCore, data: &mut SessionData, mut consensus_group: BTreeSet<NodeId>, mut job: DecryptionJob, request_id: Secret) -> Result<(), Error> {
		consensus_group.insert(core.meta.self_node_id.clone());
		job.set_request_id(request_id.clone().into());

		let transport = core.decryption_transport(true);
		let mut job_session = JobSession::new(SessionMeta {
			id: core.meta.id.clone(),
			master_node_id: core.meta.self_node_id.clone(),
			self_node_id: core.meta.self_node_id.clone(),
			threshold: core.meta.threshold,
		}, job, transport);
		job_session.initialize(consensus_group, core.meta.self_node_id != core.meta.master_node_id)?;
		data.broadcast_job_session = Some(job_session);

		Ok(())
	}

	/// Set decryption result.
	fn set_decryption_result(core: &SessionCore, data: &mut SessionData, result: Result<EncryptedDocumentKeyShadow, Error>) {
		if let Some(DelegationStatus::DelegatedFrom(master, nonce)) = data.delegation_status.take() {
			// error means can't communicate => ignore it
			let _ = match result.as_ref() {
				Ok(document_key) => core.cluster.send(&master, Message::Decryption(DecryptionMessage::DecryptionSessionDelegationCompleted(DecryptionSessionDelegationCompleted {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					decrypted_secret: document_key.decrypted_secret.clone().into(),
					common_point: document_key.common_point.clone().map(Into::into),
					decrypt_shadows: document_key.decrypt_shadows.clone(),
				}))),
				Err(error) => core.cluster.send(&master, Message::Decryption(DecryptionMessage::DecryptionSessionError(DecryptionSessionError {
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
		"decryption"
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
			let message = Message::Decryption(DecryptionMessage::DecryptionSessionError(DecryptionSessionError {
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
			Message::Decryption(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl SessionCore {
	pub fn decryption_transport(&self, is_broadcast_transport: bool) -> DecryptionJobTransport {
		DecryptionJobTransport {
			id: self.meta.id.clone(),
			access_key: self.access_key.clone(),
			nonce: self.nonce,
			is_broadcast_transport: is_broadcast_transport,
			master_node_id: self.meta.master_node_id.clone(),
			cluster: self.cluster.clone(),
		}
	}
}

impl JobTransport for DecryptionConsensusTransport {
	type PartialJobRequest=Signature;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Signature) -> Result<(), Error> {
		let version = self.version.as_ref()
			.expect("send_partial_request is called on initialized master node only; version is filled in before initialization starts on master node; qed");
		self.cluster.send(node, Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(DecryptionConsensusMessage {
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
		self.cluster.send(node, Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(DecryptionConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessage::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			})
		})))
	}
}

impl JobTransport for DecryptionJobTransport {
	type PartialJobRequest=PartialDecryptionRequest;
	type PartialJobResponse=PartialDecryptionResponse;

	fn send_partial_request(&self, node: &NodeId, request: PartialDecryptionRequest) -> Result<(), Error> {
		if !self.is_broadcast_transport {
			self.cluster.send(node, Message::Decryption(DecryptionMessage::RequestPartialDecryption(RequestPartialDecryption {
				session: self.id.clone().into(),
				sub_session: self.access_key.clone().into(),
				session_nonce: self.nonce,
				request_id: request.id.into(),
				is_shadow_decryption: request.is_shadow_decryption,
				is_broadcast_session: request.is_broadcast_session,
				nodes: request.other_nodes_ids.into_iter().map(Into::into).collect(),
			})))?;
		}

		Ok(())
	}

	fn send_partial_response(&self, node: &NodeId, response: PartialDecryptionResponse) -> Result<(), Error> {
		if !self.is_broadcast_transport || *node != self.master_node_id {
			self.cluster.send(node, Message::Decryption(DecryptionMessage::PartialDecryption(PartialDecryption {
				session: self.id.clone().into(),
				sub_session: self.access_key.clone().into(),
				session_nonce: self.nonce,
				request_id: response.request_id.into(),
				shadow_point: response.shadow_point.into(),
				decrypt_shadow: response.decrypt_shadow,
			})))?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{BTreeMap, VecDeque};
	use acl_storage::DummyAclStorage;
	use ethkey::{self, KeyPair, Random, Generator, Public, Secret};
	use key_server_cluster::{NodeId, DocumentKeyShare, DocumentKeyShareVersion, SessionId, Error, EncryptedDocumentKeyShadow, SessionMeta};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::decryption_session::{SessionImpl, SessionParams};
	use key_server_cluster::message::{self, Message, DecryptionMessage};
	use key_server_cluster::math;
	use key_server_cluster::jobs::consensus_session::ConsensusSessionState;

	const SECRET_PLAIN: &'static str = "d2b57ae7619e070af0af6bc8c703c0cd27814c54d5d6a999cacac0da34ede279ca0d9216e85991029e54e2f0c92ee0bd30237725fa765cbdbfc4529489864c5f";

	fn prepare_decryption_sessions() -> (KeyPair, Vec<Arc<DummyCluster>>, Vec<Arc<DummyAclStorage>>, Vec<SessionImpl>) {
		// prepare encrypted data + cluster configuration for scheme 4-of-5
		let session_id = SessionId::default();
		let access_key = Random.generate().unwrap().secret().clone();
		let secret_shares: Vec<Secret> = vec![
			"834cb736f02d9c968dfaf0c37658a1d86ff140554fc8b59c9fdad5a8cf810eec".parse().unwrap(),
			"5a3c1d90fafafa66bb808bcc464354a98b05e6b2c95b5f609d4511cdd1b17a0b".parse().unwrap(),
			"71bf61e7848e08e3a8486c308ce521bdacfebcf9116a0151447eb301f3a2d0e9".parse().unwrap(),
			"80c0e5e2bea66fa9b2e07f7ce09630a9563e8242446d5ee63221feb09c4338f4".parse().unwrap(),
			"c06546b5669877ba579ca437a5602e89425c53808c708d44ccd6afcaa4610fad".parse().unwrap(),
		];
		let id_numbers: Vec<(NodeId, Secret)> = vec![
			("b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".into(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()),
			("1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb".into(),
				"00125d85a05e5e63e214cb60fe63f132eec8a103aa29266b7e6e6c5b7597230b".parse().unwrap()),
			("99e82b163b062d55a64085bacfd407bb55f194ba5fb7a1af9c34b84435455520f1372e0e650a4f91aed0058cb823f62146ccb5599c8d13372c300dea866b69fc".into(),
				"f43ac0fba42a5b6ed95707d2244659e89ba877b1c9b82c0d0a9dcf834e80fc62".parse().unwrap()),
			("7e05df9dd077ec21ed4bc45c9fe9e0a43d65fa4be540630de615ced5e95cf5c3003035eb713317237d7667feeeb64335525158f5f7411f67aca9645169ea554c".into(),
				"5a324938dfb2516800487d25ab7289ba8ec38811f77c3df602e4e65e3c9acd9f".parse().unwrap()),
			("321977760d1d8e15b047a309e4c7fe6f355c10bb5a06c68472b676926427f69f229024fa2692c10da167d14cdc77eb95d0fce68af0a0f704f0d3db36baa83bb2".into(),
				"12cf422d50002d04e52bd4906fd7f5f235f051ca36abfe37e061f8da248008d8".parse().unwrap()),
		];
		let common_point: Public = "6962be696e1bcbba8e64cc7fddf140f854835354b5804f3bb95ae5a2799130371b589a131bd39699ac7174ccb35fc4342dab05331202209582fc8f3a40916ab0".into();
		let encrypted_point: Public = "b07031982bde9890e12eff154765f03c56c3ab646ad47431db5dd2d742a9297679c4c65b998557f8008469afd0c43d40b6c5f6c6a1c7354875da4115237ed87a".into();
		let encrypted_datas: Vec<_> = (0..5).map(|i| DocumentKeyShare {
			author: Public::default(),
			threshold: 3,
			public: Default::default(),
			common_point: Some(common_point.clone()),
			encrypted_point: Some(encrypted_point.clone()),
			versions: vec![DocumentKeyShareVersion {
				hash: Default::default(),
				id_numbers: id_numbers.clone().into_iter().collect(),
				secret_share: secret_shares[i].clone(),
			}],
		}).collect();
		let acl_storages: Vec<_> = (0..5).map(|_| Arc::new(DummyAclStorage::default())).collect();
		let clusters: Vec<_> = (0..5).map(|i| {
			let cluster = Arc::new(DummyCluster::new(id_numbers.iter().nth(i).clone().unwrap().0));
			for id_number in &id_numbers {
				cluster.add_node(id_number.0.clone());
			}
			cluster
		}).collect();
		let requester = Random.generate().unwrap();
		let signature = Some(ethkey::sign(requester.secret(), &SessionId::default()).unwrap());
		let sessions: Vec<_> = (0..5).map(|i| SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: session_id.clone(),
				self_node_id: id_numbers.iter().nth(i).clone().unwrap().0,
				master_node_id: id_numbers.iter().nth(0).clone().unwrap().0,
				threshold: encrypted_datas[i].threshold,
			},
			access_key: access_key.clone(),
			key_share: Some(encrypted_datas[i].clone()),
			acl_storage: acl_storages[i].clone(),
			cluster: clusters[i].clone(),
			nonce: 0,
		}, if i == 0 { signature.clone() } else { None }).unwrap()).collect();

		(requester, clusters, acl_storages, sessions)
	}

	fn do_messages_exchange(clusters: &[Arc<DummyCluster>], sessions: &[SessionImpl]) -> Result<(), Error> {
		do_messages_exchange_until(clusters, sessions, |_, _, _| false)
	}

	fn do_messages_exchange_until<F>(clusters: &[Arc<DummyCluster>], sessions: &[SessionImpl], mut cond: F) -> Result<(), Error> where F: FnMut(&NodeId, &NodeId, &Message) -> bool {
		let mut queue: VecDeque<(NodeId, NodeId, Message)> = VecDeque::new();
		while let Some((mut from, mut to, mut message)) = clusters.iter().filter_map(|c| c.take_message().map(|(to, msg)| (c.node(), to, msg))).next() {
			if cond(&from, &to, &message) {
				break;
			}

			let mut is_queued_message = false;
			loop {
				let session = &sessions[sessions.iter().position(|s| s.node() == &to).unwrap()];
				match session.on_message(&from, &message) {
					Ok(_) => {
						if let Some(qmessage) = queue.pop_front() {
							from = qmessage.0;
							to = qmessage.1;
							message = qmessage.2;
							is_queued_message = true;
							continue;
						}
						break;
					},
					Err(Error::TooEarlyForRequest) => {
						if is_queued_message {
							queue.push_front((from, to, message));
						} else {
							queue.push_back((from, to, message));
						}
						break;
					},
					Err(err) => return Err(err),
				}
			}
		}

		Ok(())
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
		assert_eq!(session.initialize(Default::default(), false, false), Err(Error::InvalidMessage));
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
		assert_eq!(session.initialize(Default::default(), false, false), Err(Error::ConsensusUnreachable));
	}

	#[test]
	fn fails_to_initialize_when_already_initialized() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].initialize(Default::default(), false, false).unwrap(), ());
		assert_eq!(sessions[0].initialize(Default::default(), false, false).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_initialization_when_already_initialized() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].initialize(Default::default(), false, false).unwrap(), ());
		assert_eq!(sessions[0].on_consensus_message(sessions[1].node(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				session_nonce: 0,
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
					version: Default::default(),
				}),
			}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_partial_decrypt_if_requested_by_slave() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_consensus_message(sessions[0].node(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				session_nonce: 0,
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
					version: Default::default(),
				}),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[2].node(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			session_nonce: 0,
			request_id: Random.generate().unwrap().secret().clone().into(),
			is_shadow_decryption: false,
			is_broadcast_session: false,
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(4).collect(),
		}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_partial_decrypt_if_wrong_number_of_nodes_participating() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_consensus_message(sessions[0].node(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				session_nonce: 0,
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
					version: Default::default(),
				}),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[0].node(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			session_nonce: 0,
			request_id: Random.generate().unwrap().secret().clone().into(),
			is_shadow_decryption: false,
			is_broadcast_session: false,
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(2).collect(),
		}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_accept_partial_decrypt_if_not_waiting() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].on_partial_decryption(sessions[1].node(), &message::PartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			session_nonce: 0,
			request_id: Random.generate().unwrap().secret().clone().into(),
			shadow_point: Random.generate().unwrap().public().clone().into(),
			decrypt_shadow: None,
		}).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_partial_decrypt_twice() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		let mut pd_from = None;
		let mut pd_msg = None;
		do_messages_exchange_until(&clusters, &sessions, |from, _, msg| match msg {
			&Message::Decryption(DecryptionMessage::PartialDecryption(ref msg)) => {
				pd_from = Some(from.clone());
				pd_msg = Some(msg.clone());
				true
			},
			_ => false,
		}).unwrap();

		assert_eq!(sessions[0].on_partial_decryption(pd_from.as_ref().unwrap(), &pd_msg.clone().unwrap()).unwrap(), ());
		assert_eq!(sessions[0].on_partial_decryption(pd_from.as_ref().unwrap(), &pd_msg.unwrap()).unwrap_err(), Error::InvalidNodeForRequest);
	}

	#[test]
	fn decryption_fails_on_session_timeout() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert!(sessions[0].decrypted_secret().is_none());
		sessions[0].on_session_timeout();
		assert_eq!(sessions[0].decrypted_secret().unwrap().unwrap_err(), Error::ConsensusUnreachable);
	}

	#[test]
	fn node_is_marked_rejected_when_timed_out_during_initialization_confirmation() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		// 1 node disconnects => we still can recover secret
		sessions[0].on_node_timeout(sessions[1].node());
		assert!(sessions[0].data.lock().consensus_session.consensus_job().rejects().contains(sessions[1].node()));
		assert!(sessions[0].state() == ConsensusSessionState::EstablishingConsensus);

		// 2 node are disconnected => we can not recover secret
		sessions[0].on_node_timeout(sessions[2].node());
		assert!(sessions[0].state() == ConsensusSessionState::Failed);
	}

	#[test]
	fn session_does_not_fail_if_rejected_node_disconnects() {
		let (_, clusters, acl_storages, sessions) = prepare_decryption_sessions();
		let key_pair = Random.generate().unwrap();

		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == ConsensusSessionState::WaitingForPartialResults).unwrap();

		// 1st node disconnects => ignore this
		sessions[0].on_node_timeout(sessions[1].node());
		assert_eq!(sessions[0].state(), ConsensusSessionState::EstablishingConsensus);
	}

	#[test]
	fn session_does_not_fail_if_requested_node_disconnects() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == ConsensusSessionState::WaitingForPartialResults).unwrap();

		// 1 node disconnects => we still can recover secret
		sessions[0].on_node_timeout(sessions[1].node());
		assert!(sessions[0].state() == ConsensusSessionState::EstablishingConsensus);

		// 2 node are disconnected => we can not recover secret
		sessions[0].on_node_timeout(sessions[2].node());
		assert!(sessions[0].state() == ConsensusSessionState::Failed);
	}

	#[test]
	fn session_does_not_fail_if_node_with_shadow_point_disconnects() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == ConsensusSessionState::WaitingForPartialResults
			&& sessions[0].data.lock().consensus_session.computation_job().responses().len() == 2).unwrap();

		// disconnects from the node which has already sent us its own shadow point
		let disconnected = sessions[0].data.lock().
			consensus_session.computation_job().responses().keys()
			.filter(|n| *n != sessions[0].node())
			.cloned().nth(0).unwrap();
		sessions[0].on_node_timeout(&disconnected);
		assert_eq!(sessions[0].state(), ConsensusSessionState::EstablishingConsensus);
	}

	#[test]
	fn session_restarts_if_confirmed_node_disconnects() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == ConsensusSessionState::WaitingForPartialResults).unwrap();

		// disconnects from the node which has already confirmed its participation
		let disconnected = sessions[0].data.lock().consensus_session.computation_job().requests().iter().cloned().nth(0).unwrap();
		sessions[0].on_node_timeout(&disconnected);
		assert_eq!(sessions[0].state(), ConsensusSessionState::EstablishingConsensus);
		assert!(sessions[0].data.lock().consensus_session.computation_job().rejects().contains(&disconnected));
		assert!(!sessions[0].data.lock().consensus_session.computation_job().requests().contains(&disconnected));
	}

	#[test]
	fn session_does_not_fail_if_non_master_node_disconnects_from_non_master_node() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == ConsensusSessionState::WaitingForPartialResults).unwrap();

		// disconnects from the node which has already confirmed its participation
		sessions[1].on_node_timeout(sessions[2].node());
		assert!(sessions[0].state() == ConsensusSessionState::WaitingForPartialResults);
		assert!(sessions[1].state() == ConsensusSessionState::ConsensusEstablished);
	}

	#[test]
	fn complete_dec_session() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange(&clusters, &sessions).unwrap();

		// now check that:
		// 1) 5 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == ConsensusSessionState::Finished).count(), 5);
		// 2) 1 session has decrypted key value
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));

		assert_eq!(sessions[0].decrypted_secret().unwrap().unwrap(), EncryptedDocumentKeyShadow {
			decrypted_secret: SECRET_PLAIN.into(),
			common_point: None,
			decrypt_shadows: None,
		});
	}

	#[test]
	fn complete_shadow_dec_session() {
		let (key_pair, clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		sessions[0].initialize(Default::default(), true, false).unwrap();

		do_messages_exchange(&clusters, &sessions).unwrap();

		// now check that:
		// 1) 5 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == ConsensusSessionState::Finished).count(), 5);
		// 2) 1 session has decrypted key value
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));

		let decrypted_secret = sessions[0].decrypted_secret().unwrap().unwrap();
		// check that decrypted_secret != SECRET_PLAIN
		assert!(decrypted_secret.decrypted_secret != SECRET_PLAIN.into());
		// check that common point && shadow coefficients are returned
		assert!(decrypted_secret.common_point.is_some());
		assert!(decrypted_secret.decrypt_shadows.is_some());
		// check that KS client is able to restore original secret
		use ethcrypto::DEFAULT_MAC;
		use ethcrypto::ecies::decrypt;
		let decrypt_shadows: Vec<_> = decrypted_secret.decrypt_shadows.unwrap().into_iter()
			.map(|c| Secret::from_slice(&decrypt(key_pair.secret(), &DEFAULT_MAC, &c).unwrap()))
			.collect();
		let decrypted_secret = math::decrypt_with_shadow_coefficients(decrypted_secret.decrypted_secret, decrypted_secret.common_point.unwrap(), decrypt_shadows).unwrap();
		assert_eq!(decrypted_secret, SECRET_PLAIN.into());
	}

	#[test]
	fn failed_dec_session() {
		let (key_pair, clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		sessions[0].initialize(Default::default(), false, false).unwrap();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 2 of these nodes are disagree
		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		acl_storages[2].prohibit(key_pair.public().clone(), SessionId::default());

		assert_eq!(do_messages_exchange(&clusters, &sessions).unwrap_err(), Error::ConsensusUnreachable);

		// check that 3 nodes have failed state
		assert_eq!(sessions[0].state(), ConsensusSessionState::Failed);
		assert_eq!(sessions.iter().filter(|s| s.state() == ConsensusSessionState::Failed).count(), 3);
	}

	#[test]
	fn complete_dec_session_with_acl_check_failed_on_master() {
		let (key_pair, clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 1 of these nodes (master) is disagree
		acl_storages[0].prohibit(key_pair.public().clone(), SessionId::default());

		// now let's try to do a decryption
		sessions[0].initialize(Default::default(), false, false).unwrap();

		do_messages_exchange(&clusters, &sessions).unwrap();

		// now check that:
		// 1) 4 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == ConsensusSessionState::Finished).count(), 5);
		// 2) 1 session has decrypted key value
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));
		assert_eq!(sessions[0].decrypted_secret().unwrap().unwrap(), EncryptedDocumentKeyShadow {
			decrypted_secret: SECRET_PLAIN.into(),
			common_point: None,
			decrypt_shadows: None,
		});
	}

	#[test]
	fn decryption_message_fails_when_nonce_is_wrong() {
		let (_, _, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].process_message(sessions[0].node(), &message::DecryptionMessage::DecryptionSessionCompleted(
			message::DecryptionSessionCompleted {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				session_nonce: 10,
			}
		)), Err(Error::ReplayProtection));
	}

	#[test]
	fn decryption_works_when_delegated_to_other_node() {
		let (_, clusters, _, mut sessions) = prepare_decryption_sessions();

		// let's say node1 doesn't have a share && delegates decryption request to node0
		// initially session is created on node1 => node1 is master for itself, but for other nodes node0 is still master
		sessions[1].core.meta.master_node_id = sessions[1].core.meta.self_node_id.clone();
		sessions[1].data.lock().consensus_session.consensus_job_mut().executor_mut().set_requester_signature(
			sessions[0].data.lock().consensus_session.consensus_job().executor().requester_signature().unwrap().clone()
		);

		// now let's try to do a decryption
		sessions[1].delegate(sessions[0].core.meta.self_node_id.clone(), Default::default(), false, false).unwrap();
		do_messages_exchange(&clusters, &sessions).unwrap();

		// now check that:
		// 1) 4 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == ConsensusSessionState::Finished).count(), 4);
		// 2) 1 session has decrypted key value
		assert_eq!(sessions[1].decrypted_secret().unwrap().unwrap(), EncryptedDocumentKeyShadow {
			decrypted_secret: SECRET_PLAIN.into(),
			common_point: None,
			decrypt_shadows: None,
		});
	}

	#[test]
	fn decryption_works_when_share_owners_are_isolated() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 1 of these nodes (master) is isolated
		let isolated_node_id = sessions[4].core.meta.self_node_id.clone();
		for cluster in &clusters {
			cluster.remove_node(&isolated_node_id);
		}

		// now let's try to do a decryption
		sessions[0].initialize(Default::default(), false, false).unwrap();
		do_messages_exchange(&clusters, &sessions).unwrap();

		assert_eq!(sessions[0].decrypted_secret().unwrap().unwrap(), EncryptedDocumentKeyShadow {
			decrypted_secret: SECRET_PLAIN.into(),
			common_point: None,
			decrypt_shadows: None,
		});
	}

	#[test]
	fn decryption_result_restored_on_all_nodes_if_broadcast_session_is_completed() {
		let (_, clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(Default::default(), false, true).unwrap();
		do_messages_exchange(&clusters, &sessions).unwrap();

		// decryption result must be the same and available on 4 nodes
		let result = sessions[0].decrypted_secret();
		assert!(result.clone().unwrap().is_ok());
		assert_eq!(3, sessions.iter().skip(1).filter(|s| s.decrypted_secret() == result).count());
		assert_eq!(1, sessions.iter().skip(1).filter(|s| s.decrypted_secret().is_none()).count());
	}
}
