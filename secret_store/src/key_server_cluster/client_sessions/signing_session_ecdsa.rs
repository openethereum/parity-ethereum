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

use std::collections::{BTreeSet, BTreeMap};
use std::collections::btree_map::Entry;
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use ethkey::{Public, Secret, Signature, sign};
use ethereum_types::H256;
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, AclStorage, DocumentKeyShare, Requester};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::{SessionIdWithSubSession, ClusterSession};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	SessionState as GenerationSessionState};
use key_server_cluster::math;
use key_server_cluster::message::{Message, EcdsaSigningMessage, EcdsaSigningConsensusMessage, EcdsaSignatureNonceGenerationMessage,
	EcdsaInversionNonceGenerationMessage, EcdsaInversionZeroGenerationMessage, EcdsaSigningInversedNonceCoeffShare,
	EcdsaRequestPartialSignature, EcdsaPartialSignature, EcdsaSigningSessionCompleted, GenerationMessage,
	ConsensusMessage, EcdsaSigningSessionError, InitializeConsensusSession, ConfirmConsensusInitialization,
	EcdsaSigningSessionDelegation, EcdsaSigningSessionDelegationCompleted};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::key_access_job::KeyAccessJob;
use key_server_cluster::jobs::signing_job_ecdsa::{EcdsaPartialSigningRequest, EcdsaPartialSigningResponse, EcdsaSigningJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Distributed ECDSA-signing session.
/// Based on "A robust threshold elliptic curve digital signature providing a new verifiable secret sharing scheme" paper.
/// WARNING: can only be used if 2*t < N is true for key generation scheme
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
type SigningConsensusSession = ConsensusSession<KeyAccessJob, SigningConsensusTransport, EcdsaSigningJob, SigningJobTransport>;

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
	/// Signature nonce generation session.
	pub sig_nonce_generation_session: Option<GenerationSession>,
	/// Inversion nonce generation session.
	pub inv_nonce_generation_session: Option<GenerationSession>,
	/// Inversion zero generation session.
	pub inv_zero_generation_session: Option<GenerationSession>,
	/// Inversed nonce coefficient shares.
	pub inversed_nonce_coeff_shares: Option<BTreeMap<NodeId, Secret>>,
	/// Delegation status.
	pub delegation_status: Option<DelegationStatus>,
	/// Decryption result.
	pub result: Option<Result<Signature, Error>>,
}

/// Signing session state.
#[derive(Debug, PartialEq)]
pub enum SessionState {
	/// Consensus is establishing.
	ConsensusEstablishing,
	/// Nonces (signature, inversion && zero) are generating.
	NoncesGenerating,
	/// Waiting for inversed nonce shares.
	WaitingForInversedNonceShares,
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
struct NonceGenerationTransport<F: Fn(SessionId, Secret, u64, GenerationMessage) -> EcdsaSigningMessage + Send + Sync> {
	/// Session id.
	id: SessionId,
	/// Session access key.
	access_key: Secret,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
	/// Other nodes ids.
	other_nodes_ids: BTreeSet<NodeId>,
	/// Message mapping function.
	map: F,
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
	pub fn new(params: SessionParams, requester: Option<Requester>) -> Result<Self, Error> {
		debug_assert_eq!(params.meta.threshold, params.key_share.as_ref().map(|ks| ks.threshold).unwrap_or_default());

		let consensus_transport = SigningConsensusTransport {
			id: params.meta.id.clone(),
			access_key: params.access_key.clone(),
			nonce: params.nonce,
			version: None,
			cluster: params.cluster.clone(),
		};
		let consensus_session = ConsensusSession::new(ConsensusSessionParams {
			// this session requires responses from 2 * t nodes
			meta: SessionMeta {
				id: params.meta.id,
				master_node_id: params.meta.master_node_id,
				self_node_id: params.meta.self_node_id,
				threshold: params.meta.threshold * 2,
			},
			consensus_executor: match requester {
				Some(requester) => KeyAccessJob::new_on_master(params.meta.id.clone(), params.acl_storage.clone(), requester),
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
				sig_nonce_generation_session: None,
				inv_nonce_generation_session: None,
				inv_zero_generation_session: None,
				inversed_nonce_coeff_shares: None,
				delegation_status: None,
				result: None,
			}),
		})
	}

	/// Wait for session completion.
	pub fn wait(&self) -> Result<Signature, Error> {
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
		self.core.cluster.send(&master, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionDelegation(EcdsaSigningSessionDelegation {
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
			Some(key_share) => key_share.version(&version).map_err(|e| Error::KeyStorage(e.into()))?,
		};

		// select nodes to participate in consensus etablish session
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

		// start consensus establish sesssion
		data.consensus_session.consensus_job_mut().transport_mut().version = Some(version.clone());
		data.version = Some(version.clone());
		data.message_hash = Some(message_hash);
		data.consensus_session.initialize(consensus_nodes)?;

		// consensus established => threshold is 0 => we can generate signature on this node
		if data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished {
			data.result = Some(sign(&key_version.secret_share, &message_hash).map_err(Into::into));
			self.core.completed.notify_all();
		}

		Ok(())
	}

	/// Process signing message.
	pub fn process_message(&self, sender: &NodeId, message: &EcdsaSigningMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(ref message) =>
				self.on_signature_nonce_generation_message(sender, message),
			&EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(ref message) =>
				self.on_inversion_nonce_generation_message(sender, message),
			&EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(ref message) =>
				self.on_inversion_zero_generation_message(sender, message),
			&EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(ref message) =>
				self.on_inversed_nonce_coeff_share(sender, message),
			&EcdsaSigningMessage::EcdsaRequestPartialSignature(ref message) =>
				self.on_partial_signature_requested(sender, message),
			&EcdsaSigningMessage::EcdsaPartialSignature(ref message) =>
				self.on_partial_signature(sender, message),
			&EcdsaSigningMessage::EcdsaSigningSessionError(ref message) =>
				self.process_node_error(Some(&sender), Error::Io(message.error.clone())),
			&EcdsaSigningMessage::EcdsaSigningSessionCompleted(ref message) =>
				self.on_session_completed(sender, message),
			&EcdsaSigningMessage::EcdsaSigningSessionDelegation(ref message) =>
				self.on_session_delegated(sender, message),
			&EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(ref message) =>
				self.on_session_delegation_completed(sender, message),
		}
	}

	/// When session is delegated to this node.
	pub fn on_session_delegated(&self, sender: &NodeId, message: &EcdsaSigningSessionDelegation) -> Result<(), Error> {
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
	pub fn on_session_delegation_completed(&self, sender: &NodeId, message: &EcdsaSigningSessionDelegationCompleted) -> Result<(), Error> {
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

		Self::set_signing_result(&self.core, &mut *data, Ok(message.signature.clone().into()));

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &EcdsaSigningConsensusMessage) -> Result<(), Error> {
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

		let key_share = self.core.key_share.as_ref()
			.expect("this is master node; master node is selected so that it has key version; qed");
		let key_version = key_share.version(data.version.as_ref()
			.expect("this is master node; master node is selected so that it has key version; qed")
		).map_err(|e| Error::KeyStorage(e.into()))?;

		let consensus_group = data.consensus_session.select_consensus_group()?.clone();
		let mut other_consensus_group_nodes = consensus_group.clone();
		other_consensus_group_nodes.remove(&self.core.meta.self_node_id);
		let consensus_group_map: BTreeMap<_, _> = consensus_group.iter().map(|n| (n.clone(), key_version.id_numbers[n].clone())).collect();

		// start generation of signature nonce
		let sig_nonce_generation_session = Self::start_generation_session(&self.core, &other_consensus_group_nodes,
			|s, k, n, m| EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(
				EcdsaSignatureNonceGenerationMessage {
					session: s.into(),
					sub_session: k.into(),
					session_nonce: n,
					message: m,
				}));
		sig_nonce_generation_session.initialize(Public::default(), false, key_share.threshold, consensus_group_map.clone().into())?;
		data.sig_nonce_generation_session = Some(sig_nonce_generation_session);

		// start generation of inversed nonce computation session
		let inv_nonce_generation_session = Self::start_generation_session(&self.core, &other_consensus_group_nodes,
			move |s, k, n, m| EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(
				EcdsaInversionNonceGenerationMessage {
					session: s.into(),
					sub_session: k.into(),
					session_nonce: n,
					message: m,
				}));
		inv_nonce_generation_session.initialize(Public::default(), false, key_share.threshold, consensus_group_map.clone().into())?;
		data.inv_nonce_generation_session = Some(inv_nonce_generation_session);

		// start generation of zero-secret shares for inversed nonce computation session
		let inv_zero_generation_session = Self::start_generation_session(&self.core, &other_consensus_group_nodes,
			move |s, k, n, m| EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(
				EcdsaInversionZeroGenerationMessage {
					session: s.into(),
					sub_session: k.into(),
					session_nonce: n,
					message: m,
				}));
		inv_zero_generation_session.initialize(Public::default(), true, key_share.threshold * 2, consensus_group_map.clone().into())?;
		data.inv_zero_generation_session = Some(inv_zero_generation_session);

		data.state = SessionState::NoncesGenerating;

		Ok(())
	}

	/// When signature nonce generation message is received.
	pub fn on_signature_nonce_generation_message(&self, sender: &NodeId, message: &EcdsaSignatureNonceGenerationMessage) -> Result<(), Error> {
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

			data.sig_nonce_generation_session = Some(Self::start_generation_session(&self.core, &other_consensus_group_nodes,
				|s, k, n, m| EcdsaSigningMessage::EcdsaSignatureNonceGenerationMessage(
					EcdsaSignatureNonceGenerationMessage {
						session: s.into(),
						sub_session: k.into(),
						session_nonce: n,
						message: m,
					})));

			data.state = SessionState::NoncesGenerating;
		}

		{
			let generation_session = data.sig_nonce_generation_session.as_ref().ok_or(Error::InvalidStateForRequest)?;
			let is_key_generating = generation_session.state() != GenerationSessionState::Finished;
			generation_session.process_message(sender, &message.message)?;

			let is_key_generated = generation_session.state() == GenerationSessionState::Finished;
			if !is_key_generating || !is_key_generated {
				return Ok(());
			}
		}

		if !Self::check_nonces_generated(&*data) {
			return Ok(());
		}

		Self::send_inversed_nonce_coeff_share(&self.core, &mut*data)?;
		data.state = if self.core.meta.master_node_id != self.core.meta.self_node_id {
			SessionState::SignatureComputing
		} else {
			SessionState::WaitingForInversedNonceShares
		};

		Ok(())
	}

	/// When inversion nonce generation message is received.
	pub fn on_inversion_nonce_generation_message(&self, sender: &NodeId, message: &EcdsaInversionNonceGenerationMessage) -> Result<(), Error> {
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

			data.inv_nonce_generation_session = Some(Self::start_generation_session(&self.core, &other_consensus_group_nodes,
				|s, k, n, m| EcdsaSigningMessage::EcdsaInversionNonceGenerationMessage(
					EcdsaInversionNonceGenerationMessage {
						session: s.into(),
						sub_session: k.into(),
						session_nonce: n,
						message: m,
					})));

			data.state = SessionState::NoncesGenerating;
		}

		{
			let generation_session = data.inv_nonce_generation_session.as_ref().ok_or(Error::InvalidStateForRequest)?;
			let is_key_generating = generation_session.state() != GenerationSessionState::Finished;
			generation_session.process_message(sender, &message.message)?;

			let is_key_generated = generation_session.state() == GenerationSessionState::Finished;
			if !is_key_generating || !is_key_generated {
				return Ok(());
			}
		}

		if !Self::check_nonces_generated(&*data) {
			return Ok(());
		}

		Self::send_inversed_nonce_coeff_share(&self.core, &mut*data)?;
		data.state = if self.core.meta.master_node_id != self.core.meta.self_node_id {
			SessionState::SignatureComputing
		} else {
			SessionState::WaitingForInversedNonceShares
		};

		Ok(())
	}

	/// When inversion zero generation message is received.
	pub fn on_inversion_zero_generation_message(&self, sender: &NodeId, message: &EcdsaInversionZeroGenerationMessage) -> Result<(), Error> {
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

			data.inv_zero_generation_session = Some(Self::start_generation_session(&self.core, &other_consensus_group_nodes,
				|s, k, n, m| EcdsaSigningMessage::EcdsaInversionZeroGenerationMessage(
					EcdsaInversionZeroGenerationMessage {
						session: s.into(),
						sub_session: k.into(),
						session_nonce: n,
						message: m,
					})));

			data.state = SessionState::NoncesGenerating;
		}

		{
			let generation_session = data.inv_zero_generation_session.as_ref().ok_or(Error::InvalidStateForRequest)?;
			let is_key_generating = generation_session.state() != GenerationSessionState::Finished;
			generation_session.process_message(sender, &message.message)?;

			let is_key_generated = generation_session.state() == GenerationSessionState::Finished;
			if !is_key_generating || !is_key_generated {
				return Ok(());
			}
		}

		if !Self::check_nonces_generated(&*data) {
			return Ok(());
		}

		Self::send_inversed_nonce_coeff_share(&self.core, &mut*data)?;
		data.state = if self.core.meta.master_node_id != self.core.meta.self_node_id {
			SessionState::SignatureComputing
		} else {
			SessionState::WaitingForInversedNonceShares
		};

		Ok(())
	}

	/// When inversed nonce share is received.
	pub fn on_inversed_nonce_coeff_share(&self, sender: &NodeId, message: &EcdsaSigningInversedNonceCoeffShare) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}
		match data.state {
			SessionState::WaitingForInversedNonceShares => (),
			SessionState::NoncesGenerating => return Err(Error::TooEarlyForRequest),
			_ => return Err(Error::InvalidStateForRequest),
		}

		let inversed_nonce_coeff = {
			let consensus_group = data.consensus_session.select_consensus_group()?.clone();
			{
				let inversed_nonce_coeff_shares = data.inversed_nonce_coeff_shares.as_mut()
					.expect("we are in WaitingForInversedNonceShares state; inversed_nonce_coeff_shares are filled before this state; qed");
				match inversed_nonce_coeff_shares.entry(sender.clone()) {
					Entry::Occupied(_) => return Err(Error::InvalidStateForRequest),
					Entry::Vacant(entry) => {
						entry.insert(message.inversed_nonce_coeff_share.clone().into());
					},
				}

				if consensus_group.iter().any(|n| !inversed_nonce_coeff_shares.contains_key(n)) {
					return Ok(());
				}
			}

			Self::compute_inversed_nonce_coeff(&self.core, &*data)?
		};


		let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
		let message_hash = data.message_hash
			.expect("we are on master node; on master node message_hash is filled in initialize(); on_generation_message follows initialize; qed");

		let nonce_exists_proof = "nonce is generated before signature is computed; we are in SignatureComputing state; qed";
		let sig_nonce_public = data.sig_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.0;
		let inv_nonce_share = data.inv_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.2;

		self.core.disseminate_jobs(&mut data.consensus_session, &version, sig_nonce_public, inv_nonce_share, inversed_nonce_coeff, message_hash)
	}

	/// When partial signature is requested.
	pub fn on_partial_signature_requested(&self, sender: &NodeId, message: &EcdsaRequestPartialSignature) -> Result<(), Error> {
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

		let nonce_exists_proof = "nonce is generated before signature is computed; we are in SignatureComputing state; qed";
		let sig_nonce_public = data.sig_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.0;
		let inv_nonce_share = data.inv_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.2;

		let version = data.version.as_ref().ok_or(Error::InvalidMessage)?.clone();
		let key_version = key_share.version(&version).map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();

		let signing_job = EcdsaSigningJob::new_on_slave(key_share.clone(), key_version, sig_nonce_public, inv_nonce_share)?;
		let signing_transport = self.core.signing_transport();

		data.consensus_session.on_job_request(sender, EcdsaPartialSigningRequest {
			id: message.request_id.clone().into(),
			inversed_nonce_coeff: message.inversed_nonce_coeff.clone().into(),
			message_hash: message.message_hash.clone().into(),
		}, signing_job, signing_transport)
	}

	/// When partial signature is received.
	pub fn on_partial_signature(&self, sender: &NodeId, message: &EcdsaPartialSignature) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		data.consensus_session.on_job_response(sender, EcdsaPartialSigningResponse {
			request_id: message.request_id.clone().into(),
			partial_signature_s: message.partial_signature_s.clone().into(),
		})?;

		if data.consensus_session.state() != ConsensusSessionState::Finished {
			return Ok(());
		}

		// send compeltion signal to all nodes, except for rejected nodes
		for node in data.consensus_session.consensus_non_rejected_nodes() {
			self.core.cluster.send(&node, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionCompleted(EcdsaSigningSessionCompleted {
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
	pub fn on_session_completed(&self, sender: &NodeId, message: &EcdsaSigningSessionCompleted) -> Result<(), Error> {
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

				let nonce_exists_proof = "on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when nonces generation has completed; qed";
				let sig_nonce_public = data.sig_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.0;
				let inv_nonce_share = data.inv_nonce_generation_session.as_ref().expect(nonce_exists_proof).joint_public_and_secret().expect(nonce_exists_proof)?.2;

				let inversed_nonce_coeff = Self::compute_inversed_nonce_coeff(&self.core, &*data)?;

				let disseminate_result = self.core.disseminate_jobs(&mut data.consensus_session, &version, sig_nonce_public, inv_nonce_share, inversed_nonce_coeff, message_hash);
				match disseminate_result {
					Ok(()) => Ok(()),
					Err(err) => {
						warn!("{}: ECDSA signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);
						Self::set_signing_result(&self.core, &mut *data, Err(err.clone()));
						Err(err)
					}
				}
			},
			Err(err) => {
				warn!("{}: ECDSA signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);
				Self::set_signing_result(&self.core, &mut *data, Err(err.clone()));
				Err(err)
			},
		}
	}

	/// Start generation session.
	fn start_generation_session<F>(core: &SessionCore, other_consensus_group_nodes: &BTreeSet<NodeId>, map_message: F) -> GenerationSession
		where F: Fn(SessionId, Secret, u64, GenerationMessage) -> EcdsaSigningMessage + Send + Sync + 'static {
		GenerationSession::new(GenerationSessionParams {
			id: core.meta.id.clone(),
			self_node_id: core.meta.self_node_id.clone(),
			key_storage: None,
			cluster: Arc::new(NonceGenerationTransport {
				id: core.meta.id.clone(),
				access_key: core.access_key.clone(),
				nonce: core.nonce,
				cluster: core.cluster.clone(),
				other_nodes_ids: other_consensus_group_nodes.clone(),
				map: map_message,
			}),
			nonce: None,
		})
	}

	/// Set signing session result.
	fn set_signing_result(core: &SessionCore, data: &mut SessionData, result: Result<Signature, Error>) {
		if let Some(DelegationStatus::DelegatedFrom(master, nonce)) = data.delegation_status.take() {
			// error means can't communicate => ignore it
			let _ = match result.as_ref() {
				Ok(signature) => core.cluster.send(&master, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionDelegationCompleted(EcdsaSigningSessionDelegationCompleted {
					session: core.meta.id.clone().into(),
					sub_session: core.access_key.clone().into(),
					session_nonce: nonce,
					signature: signature.clone().into(),
				}))),
				Err(error) => core.cluster.send(&master, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionError(EcdsaSigningSessionError {
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

	/// Check if all nonces are generated.
	fn check_nonces_generated(data: &SessionData) -> bool {
		let expect_proof = "check_nonces_generated is called when som nonce-gen session is completed;
			all nonce-gen sessions are created at once; qed";
		let sig_nonce_generation_session = data.sig_nonce_generation_session.as_ref().expect(expect_proof);
		let inv_nonce_generation_session = data.inv_nonce_generation_session.as_ref().expect(expect_proof);
		let inv_zero_generation_session = data.inv_zero_generation_session.as_ref().expect(expect_proof);

		sig_nonce_generation_session.state() == GenerationSessionState::Finished
			&& inv_nonce_generation_session.state() == GenerationSessionState::Finished
			&& inv_zero_generation_session.state() == GenerationSessionState::Finished
	}

	/// Broadcast inversed nonce share.
	fn send_inversed_nonce_coeff_share(core: &SessionCore, data: &mut SessionData) -> Result<(), Error> {
		let proof = "inversed nonce coeff share is sent after nonces generation is completed; qed";

		let sig_nonce_generation_session = data.sig_nonce_generation_session.as_ref().expect(proof);
		let sig_nonce = sig_nonce_generation_session.joint_public_and_secret().expect(proof).expect(proof).2;

		let inv_nonce_generation_session = data.inv_nonce_generation_session.as_ref().expect(proof);
		let inv_nonce = inv_nonce_generation_session.joint_public_and_secret().expect(proof).expect(proof).2;

		let inv_zero_generation_session = data.inv_zero_generation_session.as_ref().expect(proof);
		let inv_zero = inv_zero_generation_session.joint_public_and_secret().expect(proof).expect(proof).2;

		let inversed_nonce_coeff_share = math::compute_ecdsa_inversed_secret_coeff_share(&sig_nonce, &inv_nonce, &inv_zero)?;
		if core.meta.self_node_id == core.meta.master_node_id {
			let mut inversed_nonce_coeff_shares = BTreeMap::new();
			inversed_nonce_coeff_shares.insert(core.meta.self_node_id.clone(), inversed_nonce_coeff_share);
			data.inversed_nonce_coeff_shares = Some(inversed_nonce_coeff_shares);
			Ok(())
		} else {
			core.cluster.send(&core.meta.master_node_id, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningInversedNonceCoeffShare(EcdsaSigningInversedNonceCoeffShare {
				session: core.meta.id.clone().into(),
				sub_session: core.access_key.clone().into(),
				session_nonce: core.nonce,
				inversed_nonce_coeff_share: inversed_nonce_coeff_share.into(),
			})))
		}
	}

	/// Compute inversed nonce coefficient on master node.
	fn compute_inversed_nonce_coeff(core: &SessionCore, data: &SessionData) -> Result<Secret, Error> {
		let proof = "inversed nonce coeff is computed on master node; key version exists on master node";
		let key_share = core.key_share.as_ref().expect(proof);
		let key_version = key_share.version(data.version.as_ref().expect(proof)).expect(proof);

		let proof = "inversed nonce coeff is computed after all shares are received; qed";
		let inversed_nonce_coeff_shares = data.inversed_nonce_coeff_shares.as_ref().expect(proof);

		math::compute_ecdsa_inversed_secret_coeff_from_shares(key_share.threshold,
			&inversed_nonce_coeff_shares.keys().map(|n| key_version.id_numbers[n].clone()).collect::<Vec<_>>(),
			&inversed_nonce_coeff_shares.values().cloned().collect::<Vec<_>>())
	}
}

impl ClusterSession for SessionImpl {
	type Id = SessionIdWithSubSession;

	fn type_name() -> &'static str {
		"ecdsa_signing"
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
			let message = Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionError(EcdsaSigningSessionError {
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
			Message::EcdsaSigning(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl<F> NonceGenerationTransport<F> where F: Fn(SessionId, Secret, u64, GenerationMessage) -> EcdsaSigningMessage + Send + Sync {
	fn map_message(&self, message: Message) -> Result<Message, Error> {
		match message {
			Message::Generation(message) => Ok(Message::EcdsaSigning((self.map)(self.id.clone(), self.access_key.clone(), self.nonce, message))),
			_ => Err(Error::InvalidMessage),
		}
	}
}

impl<F> Cluster for NonceGenerationTransport<F> where F: Fn(SessionId, Secret, u64, GenerationMessage) -> EcdsaSigningMessage + Send + Sync {
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

	pub fn disseminate_jobs(&self, consensus_session: &mut SigningConsensusSession, version: &H256, nonce_public: Public, inv_nonce_share: Secret, inversed_nonce_coeff: Secret, message_hash: H256) -> Result<(), Error> {
		let key_share = match self.key_share.as_ref() {
			None => return Err(Error::InvalidMessage),
			Some(key_share) => key_share,
		};

		let key_version = key_share.version(version).map_err(|e| Error::KeyStorage(e.into()))?.hash.clone();
		let signing_job = EcdsaSigningJob::new_on_master(key_share.clone(), key_version, nonce_public, inv_nonce_share, inversed_nonce_coeff, message_hash)?;
		consensus_session.disseminate_jobs(signing_job, self.signing_transport(), false)
	}
}

impl JobTransport for SigningConsensusTransport {
	type PartialJobRequest=Requester;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Requester) -> Result<(), Error> {
		let version = self.version.as_ref()
			.expect("send_partial_request is called on initialized master node only; version is filled in before initialization starts on master node; qed");
		self.cluster.send(node, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningConsensusMessage(EcdsaSigningConsensusMessage {
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
		self.cluster.send(node, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningConsensusMessage(EcdsaSigningConsensusMessage {
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
	type PartialJobRequest=EcdsaPartialSigningRequest;
	type PartialJobResponse=EcdsaPartialSigningResponse;

	fn send_partial_request(&self, node: &NodeId, request: EcdsaPartialSigningRequest) -> Result<(), Error> {
		self.cluster.send(node, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaRequestPartialSignature(EcdsaRequestPartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			request_id: request.id.into(),
			inversed_nonce_coeff: request.inversed_nonce_coeff.into(),
			message_hash: request.message_hash.into(),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: EcdsaPartialSigningResponse) -> Result<(), Error> {
		self.cluster.send(node, Message::EcdsaSigning(EcdsaSigningMessage::EcdsaPartialSignature(EcdsaPartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			session_nonce: self.nonce,
			request_id: response.request_id.into(),
			partial_signature_s: response.partial_signature_s.into(),
		})))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{BTreeSet, BTreeMap, VecDeque};
	use ethereum_types::H256;
	use ethkey::{self, Random, Generator, Public, KeyPair, verify_public};
	use acl_storage::DummyAclStorage;
	use key_server_cluster::{NodeId, DummyKeyStorage, SessionId, SessionMeta, Error, KeyStorage};
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::MessageLoop as KeyGenerationMessageLoop;
	use key_server_cluster::message::Message;
	use key_server_cluster::signing_session_ecdsa::{SessionImpl, SessionParams};

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
				}, if i == 0 { signature.clone().map(Into::into) } else { None }).unwrap();
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
	}

	fn prepare_signing_sessions(threshold: usize, num_nodes: usize) -> (KeyGenerationMessageLoop, MessageLoop) {
		// run key generation sessions
		let mut gl = KeyGenerationMessageLoop::new(num_nodes);
		gl.master().initialize(Public::default(), false, threshold, gl.nodes.keys().cloned().collect::<BTreeSet<_>>().into()).unwrap();
		while let Some((from, to, message)) = gl.take_message() {
			gl.process_message((from, to, message)).unwrap();
		}

		// run signing session
		let sl = MessageLoop::new(&gl);
		(gl, sl)
	}

	#[test]
	fn failed_gen_ecdsa_sign_session_when_threshold_is_too_low() {
		let test_cases = [(1, 2), (2, 4), (3, 6), (4, 6)];
		for &(threshold, num_nodes) in &test_cases {
			let (_, sl) = prepare_signing_sessions(threshold, num_nodes);

			// run signing session
			let message_hash = H256::random();
			assert_eq!(sl.master().initialize(sl.version.clone(), message_hash).unwrap_err(), Error::ConsensusUnreachable);
		}
	}

	#[test]
	fn complete_gen_ecdsa_sign_session() {
		let test_cases = [(0, 1), (2, 5), (2, 6), (3, 11), (4, 11)];
		for &(threshold, num_nodes) in &test_cases {
			let (gl, mut sl) = prepare_signing_sessions(threshold, num_nodes);
			let key_pair = gl.compute_key_pair(threshold);

			// run signing session
			let message_hash = H256::random();
			sl.master().initialize(sl.version.clone(), message_hash).unwrap();
			while let Some((from, to, message)) = sl.take_message() {
				sl.process_message((from, to, message)).unwrap();
			}

			// verify signature
			let signature = sl.master().wait().unwrap();
			assert!(verify_public(key_pair.public(), &signature, &message_hash).unwrap());
		}
	}

	#[test]
	fn ecdsa_complete_signing_session_with_single_node_failing() {
		let (_, mut sl) = prepare_signing_sessions(1, 4);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// we need at least 3-of-4 nodes to agree to reach consensus
		// let's say 1 of 4 nodes disagee
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
	fn ecdsa_complete_signing_session_with_acl_check_failed_on_master() {
		let (_, mut sl) = prepare_signing_sessions(1, 4);
		sl.master().initialize(sl.version.clone(), 777.into()).unwrap();

		// we need at least 3-of-4 nodes to agree to reach consensus
		// let's say 1 of 4 nodes disagee
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
	fn ecdsa_signing_works_when_delegated_to_other_node() {
		let (_, mut sl) = prepare_signing_sessions(1, 4);

		// let's say node1 doesn't have a share && delegates decryption request to node0
		// initially session is created on node1 => node1 is master for itself, but for other nodes node0 is still master
		let actual_master = sl.nodes.keys().nth(0).cloned().unwrap();
		let requested_node = sl.nodes.keys().skip(1).nth(0).cloned().unwrap();
		let version = sl.nodes[&actual_master].key_storage.get(&Default::default()).unwrap().unwrap().last_version().unwrap().hash.clone();
		sl.nodes[&requested_node].key_storage.remove(&Default::default()).unwrap();
		sl.nodes.get_mut(&requested_node).unwrap().session.core.key_share = None;
		sl.nodes.get_mut(&requested_node).unwrap().session.core.meta.master_node_id = sl.nodes[&requested_node].session.core.meta.self_node_id.clone();
		sl.nodes[&requested_node].session.data.lock().consensus_session.consensus_job_mut().executor_mut().set_requester(
			sl.nodes[&actual_master].session.data.lock().consensus_session.consensus_job().executor().requester().unwrap().clone()
		);

		// now let's try to do a decryption
		sl.nodes[&requested_node].session.delegate(actual_master, version, H256::random()).unwrap();

		// then consensus reachable, but single node will disagree
		while let Some((from, to, message)) = sl.take_message() {
			sl.process_message((from, to, message)).unwrap();
		}
	}

	#[test]
	fn ecdsa_signing_works_when_share_owners_are_isolated() {
		let (_, mut sl) = prepare_signing_sessions(2, 6);

		// we need 5 out of 6 nodes to agree to do a decryption
		// let's say that 1 of these nodes (master) is isolated
		let isolated_node_id = sl.nodes.keys().skip(2).nth(0).cloned().unwrap();
		for node in sl.nodes.values() {
			node.cluster.remove_node(&isolated_node_id);
		}

		// now let's try to do a signing
		sl.master().initialize(sl.version.clone(), H256::random()).unwrap();

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
