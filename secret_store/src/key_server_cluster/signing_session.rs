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

use std::collections::{BTreeSet, VecDeque};
use std::mem::swap;
use std::ops::DerefMut;
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use ethkey::{self, Public, Secret, Signature};
use util::H256;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::consensus::Consensus;
use key_server_cluster::consensus_session::{ConsensusSession, AclConsensusChecker, SessionParams as ConsensusSessionParams,
	SessionState as ConsensusSessionState, SessionAction as ConsensusSessionAction};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	Session as GenerationSessionApi};
use key_server_cluster::math;
use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, SigningGenerationMessage,
	RequestPartialSignature, PartialSignature, SigningSessionCompleted, GenerationMessage, ConsensusMessage, SigningSessionError};

pub use key_server_cluster::decryption_session::DecryptionSessionId as SigningSessionId;

/// Signing session API.
pub trait Session: Send + Sync + 'static {
	/// Get session state.
	fn state(&self) -> SessionState;
	/// Wait until session is completed. Returns signed message.
	fn wait(&self) -> Result<(Secret, Secret), Error>;
}

/// Distributed signing session.
/// Based on "Efficient Multi-Party Digital Signature using Adaptive Secret Sharing for Low-Power Devices in Wireless Network" paper.
/// Brief overview:
/// 1) initialization: master node (which has received request for signing the message) requests all other nodes to sign the message
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor has access to the private key
/// 3) partial signing: every node which has succussfully checked access for the requestor do a partial signing
/// 4) signing: master node receives all partial signatures of the secret and computes the signature
pub struct SessionImpl {
	/// Key generation session id.
	id: SessionId,
	/// Signing session access key.
	access_key: Secret,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Encrypted data.
	encrypted_data: DocumentKeyShare,
	/// ACL storate to check access to the resource.
	acl_storage: Arc<AclStorage>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	cluster: Arc<Cluster>,
	/// SessionImpl completion condvar.
	completed: Condvar,
	/// Mutable session data.
	data: Mutex<SessionData>,
}

/// SessionImpl creation parameters
pub struct SessionParams {
	/// SessionImpl identifier.
	pub id: SessionId,
	/// SessionImpl access key.
	pub access_key: Secret,
	/// Id of node, on which this session is running.
	pub self_node_id: Public,
	/// Encrypted data (result of running encryption_session::SessionImpl).
	pub encrypted_data: DocumentKeyShare,
	/// Key storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
}

/// Mutable data of signing session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,
	/// Public key of requestor.
	requestor: Option<Public>,
	/// Hash of the message to sign.
	message_hash: Option<H256>,
	/// Signing consensus group.
	consensus: Option<Consensus<Secret>>,

	// === Values, filled when consensus is establishing ===
	/// Consensus session.
	consensus_session: Option<ConsensusSession<AclConsensusChecker>>,

	// === Values, filled when session key is generating ===
	/// Signing cluster subgroup.
	generation_cluster: Option<Arc<SigningCluster>>,
	/// Session key generation session.
	generation_session: Option<GenerationSession>,
	/// Generated session public key.
	session_joint_public: Option<Public>,
	/// Generated session secret coefficient.
	session_secret_coeff: Option<Secret>,

	/// === Values, filled during final decryption ===
	/// Decrypted secret
	signed_message: Option<Result<(Secret, Secret), Error>>,
}

#[derive(Debug, Clone, PartialEq)]
/// Distributed key generation session state.
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Establishing consensus.
	EstablishingConsensus,
	/// Consensus established
	EstablishedConsensus,

	/// === One-time key genration states ===
	/// Generating one-time key.
	SessionKeyGeneration,
	/// One-time key generated.
	SessionKeyGenerated,

	// === Signature generation states ===
	/// Waiting for partial signatures.
	WaitingForPartialSignature,
	/// Waiting for partial signature request.
	WaitingForPartialSignatureRequest,

	// === Final states of the session ===
	/// Signing is completed.
	Finished,
	/// Signing is failed.
	Failed,
}

/// Signing group of cluster nodes.
struct SigningCluster {
	/// Original cluster reference.
	cluster: Arc<Cluster>,
	/// This node id.
	self_node_id: NodeId,
	/// Signing group.
	nodes: BTreeSet<NodeId>,
	/// Generation session messages.
	messages: Mutex<VecDeque<(NodeId, Message)>>,
}

impl SessionImpl {
	/// Create new decryption session.
	pub fn new(params: SessionParams) -> Result<Self, Error> {
		check_encrypted_data(&params.self_node_id, &params.encrypted_data)?;

		Ok(SessionImpl {
			id: params.id,
			access_key: params.access_key,
			self_node_id: params.self_node_id,
			encrypted_data: params.encrypted_data,
			acl_storage: params.acl_storage,
			cluster: params.cluster,
			completed: Condvar::new(),
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				master: None,
				requestor: None,
				message_hash: None,
				consensus_session: None,
				consensus: None,
				generation_cluster: None,
				generation_session: None,
				session_joint_public: None,
				session_secret_coeff: None,
				signed_message: None,
			})
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Initialize signing session.
	pub fn initialize(&self, requestor_signature: Signature, message_hash: H256) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		data.state = SessionState::EstablishingConsensus;
		data.master = Some(self.node().clone());
		data.requestor = Some(requestor_public.clone());
		data.message_hash = Some(message_hash);

		// create consensus session
		let mut consensus = Consensus::new(self.encrypted_data.threshold, self.encrypted_data.id_numbers.keys().cloned().collect())?;
		let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
			id: self.id.clone(),
			self_node_id: self.self_node_id.clone(),
			master_node_id: self.self_node_id.clone(),
			consensus_checker: AclConsensusChecker::new(self.acl_storage.clone()),
		})?;

		// start consensus session
		let consensus_action = consensus_session.initialize(requestor_signature, &mut consensus)?;
		data.consensus = Some(consensus);
		data.consensus_session = Some(consensus_session);

		// process consensus action
		SessionImpl::process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if single node is required to sign message, proceed
		if data.state == SessionState::EstablishedConsensus {
			SessionImpl::start_generating_session_key(self.self_node_id.clone(), &self.encrypted_data, &self.cluster, &mut *data)?;
			SessionImpl::process_generation_session_action(&self.id, &self.access_key, &self.completed, &mut *data)?;
			SessionImpl::start_waiting_for_partial_signing(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)?;
			SessionImpl::do_signing(&mut *data)?;
			self.completed.notify_all();
		}

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: NodeId, message: &SigningConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);

		let mut data = self.data.lock();

		// if we are waiting for initialization
		if data.state == SessionState::WaitingForInitialization {
			data.master = Some(sender.clone());
			data.state = SessionState::EstablishingConsensus;
			data.consensus = Some(Consensus::new(self.encrypted_data.threshold, self.encrypted_data.id_numbers.keys().cloned().collect())?);
			data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
				id: self.id.clone(),
				self_node_id: self.self_node_id.clone(),
				master_node_id: sender.clone(),
				consensus_checker: AclConsensusChecker::new(self.acl_storage.clone()),
			})?);
		}

		// check state
		if data.state != SessionState::EstablishingConsensus {
			// consensus is already established => mark node as confirmed (for restart case) and ignore
			return data.consensus.as_mut().map(|consensus| consensus.accept_offer(&sender)).unwrap_or(Ok(()));
		}

		// process message
		let consensus_action = {
			let mut data = data.deref_mut();
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidStateForRequest)?;
			match message.message {
				ConsensusMessage::InitializeConsensusSession(ref message) => {
					let requestor = ethkey::recover(&message.requestor_signature, &self.id)?;
					data.requestor = Some(requestor.clone());
					consensus_session.on_initialize_session(sender, &requestor)?
				},
				ConsensusMessage::ConfirmConsensusInitialization(ref message) => {
					let consensus = data.consensus.as_mut().ok_or(Error::InvalidStateForRequest)?;
					consensus_session.on_confirm_initialization(sender, message.is_confirmed, consensus)?
				},
			}
		};
		SessionImpl::process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if consensus is established, start generating session key on master
		if data.state != SessionState::EstablishedConsensus || data.master.as_ref() != Some(&self.self_node_id) {
			return Ok(());
		}

		SessionImpl::start_generating_session_key(self.self_node_id.clone(), &self.encrypted_data, &self.cluster, &mut *data)?;
		SessionImpl::process_generation_session_action(&self.id, &self.access_key, &self.completed, &mut *data)
	}

	/// When session key related message is received.
	pub fn on_generation_message(&self, sender: NodeId, message: &SigningGenerationMessage) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::EstablishingConsensus {
			// on 'slave' nodes, consensus is established when session key generation starts
			if data.master.as_ref() != Some(&sender) {
				return Err(Error::InvalidMessage);
			}

			data.state = SessionState::EstablishedConsensus;
		}
		if data.state == SessionState::EstablishedConsensus {
			match message.message {
				GenerationMessage::InitializeSession(ref message) => {
					// check message
					if data.master.as_ref() != Some(&sender) {
						return Err(Error::InvalidMessage);
					}

					// update state
					data.state = SessionState::SessionKeyGeneration;

					// cluster for generation session would only include nodes, which has fodmed consensus group
					let generation_cluster = Arc::new(SigningCluster::new(self.cluster.clone(), self.self_node_id.clone(), message.nodes.keys().cloned().map(Into::into).collect()));
					data.generation_cluster = Some(generation_cluster.clone());

					// create generation session
					data.generation_session = Some(GenerationSession::new(GenerationSessionParams {
						id: message.session.clone().into(),
						self_node_id: self.self_node_id.clone(),
						key_storage: None,
						cluster: generation_cluster,
					}));
				},
				_ => return Err(Error::InvalidStateForRequest),
			}
		}
		// do not check for other states, as completion message can come after moving to other states
		// if generation session exists, let it process message

		// process message
		let is_generation_completed = data.session_joint_public.is_some();
		{
			let generation_session = data.generation_session.as_ref().ok_or(Error::InvalidStateForRequest)?;
			match message.message {
				GenerationMessage::InitializeSession(ref message) =>
					generation_session.on_initialize_session(sender, message)?,
				GenerationMessage::ConfirmInitialization(ref message) =>
					generation_session.on_confirm_initialization(sender, message)?,
				GenerationMessage::CompleteInitialization(ref message) =>
					generation_session.on_complete_initialization(sender, message)?,
				GenerationMessage::KeysDissemination(ref message) =>
					generation_session.on_keys_dissemination(sender, message)?,
				GenerationMessage::PublicKeyShare(ref message) =>
					generation_session.on_public_key_share(sender, message)?,
				GenerationMessage::SessionError(ref message) =>
					generation_session.on_session_error(sender, message)?,
				GenerationMessage::SessionCompleted(ref message) =>
					generation_session.on_session_completed(sender, message)?,
			}
		}
		SessionImpl::process_generation_session_action(&self.id, &self.access_key, &self.completed, &mut *data)?;

		// if session key generated just now => start generating partial signatures
		if data.state != SessionState::SessionKeyGenerated || is_generation_completed {
			return Ok(());
		}

		SessionImpl::start_waiting_for_partial_signing(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)
	}

	/// When partial signature is requested.
	pub fn on_partial_signature_requested(&self, sender: NodeId, message: &RequestPartialSignature) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.master != Some(sender) {
			return Err(Error::InvalidMessage);
		}
		if data.state != SessionState::WaitingForPartialSignatureRequest {
			return Err(Error::InvalidStateForRequest);
		}

		// update data
		data.message_hash = Some(message.message_hash.clone().into());

		// calculate partial signature
		let session_joint_public = data.session_joint_public.as_ref().expect("we are in SessionKeyGenerated state; public is generated during SessionKeyGenerating; qed");
		let session_secret_coeff = data.session_secret_coeff.as_ref().expect("we are in SessionKeyGenerated state; coeff is generated during SessionKeyGenerating; qed");
		let nodes: BTreeSet<_> = message.nodes.iter().cloned().map(Into::into).filter(|n| n != &self.self_node_id).collect();
		let partial_signature = SessionImpl::do_partial_signing(&self.self_node_id, &message.message_hash.clone().into(), &self.encrypted_data, &nodes, session_joint_public, session_secret_coeff)?;

		self.cluster.send(&sender, Message::Signing(SigningMessage::PartialSignature(PartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			request_id: message.request_id.clone(),
			partial_signature: partial_signature.into(),
		})))?;

		// master could ask us for another partial signature in case of restart
		// => no state change is required

		Ok(())
	}

	/// When partial signature is received.
	pub fn on_partial_signature(&self, sender: NodeId, message: &PartialSignature) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForPartialSignature {
			return Err(Error::InvalidStateForRequest);
		}

		// check partial signature
		// TODO: check_signature_share()

		// remember partial signature
		{
			let consensus = data.consensus.as_mut().ok_or(Error::InvalidStateForRequest)?;
			consensus.job_response_received(&sender, &message.request_id.clone().into(), message.partial_signature.clone().into())?;

			// check if we have enough shadow points to decrypt the secret
			if !consensus.is_completed() {
				return Ok(());
			}
		}

		// notify all other nodes about session completion
		self.cluster.broadcast(Message::Signing(SigningMessage::SigningSessionCompleted(SigningSessionCompleted {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
		})))?;

		// do signing
		SessionImpl::do_signing(&mut *data)?;
		self.completed.notify_all();

		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&self, sender: NodeId, message: &SigningSessionCompleted) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		if data.master != Some(sender) {
			return Err(Error::InvalidMessage);
		}
		// it is up to master node to decide when to complete session
		// => we could only fail if already failed
		if data.state == SessionState::Failed {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		data.state = SessionState::Finished;

		Ok(())
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: NodeId, message: &SigningSessionError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: signing session failed with error: {:?} from {}", self.node(), message.error, sender);

		data.state = SessionState::Failed;
		data.signed_message = Some(Err(Error::Io(message.error.clone())));
		self.completed.notify_all();

		Ok(())
	}

	/// Process nested consensus session action.
	fn process_consensus_session_action(id: &SessionId, access_key: &Secret, cluster: &Arc<Cluster>, completed: &Condvar, data: &mut SessionData, action: ConsensusSessionAction) -> Result<(), Error> {
		match action {
			ConsensusSessionAction::BroadcastMessage(message) => {
				cluster.broadcast(Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
					session: id.clone().into(),
					sub_session: access_key.clone().into(),
					message: message,
				})))?
			},
			ConsensusSessionAction::SendMessage(to, message) => {
				cluster.send(&to, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
					session: id.clone().into(),
					sub_session: access_key.clone().into(),
					message: message,
				})))?
			},
			ConsensusSessionAction::CheckStatus => (),
		}

		match data.consensus_session.as_ref()
			.expect("we are processing consensus session action; action is a result of processing message by session; qed")
			.state() {
			ConsensusSessionState::Finished => data.state = SessionState::EstablishedConsensus,
			ConsensusSessionState::Failed => {
				data.state = SessionState::Failed;
				data.signed_message = Some(Err(Error::ConsensusUnreachable));
				completed.notify_all();
			},
			_ => (),
		}

		Ok(())
	}

	/// Start generating one-time session key.
	fn start_generating_session_key(self_node_id: NodeId, encrypted_data: &DocumentKeyShare, cluster: &Arc<Cluster>, data: &mut SessionData) -> Result<(), Error> {
		// update state
		data.state = SessionState::SessionKeyGeneration;

		// select nodes to make signature
		let mut consensus = data.consensus.as_mut().expect("consensus is filled during initialization phase; key generation phase follows initialization; qed");
		consensus.activate()?;
		let (_, selected_nodes) = consensus.select_nodes(&self_node_id)?;

		// create generation session
		let generation_cluster = Arc::new(SigningCluster::new(cluster.clone(), self_node_id.clone(), selected_nodes.clone()));
		let generation_session = GenerationSession::new(GenerationSessionParams {
			id: H256::default(), // doesn't matter
			self_node_id: self_node_id.clone(),
			key_storage: None,
			cluster: generation_cluster.clone(),
		});

		// start generation session
		generation_session.initialize(Public::default(), // doesn't matter
			encrypted_data.threshold, selected_nodes.clone())?;
		data.generation_cluster = Some(generation_cluster);
		data.generation_session = Some(generation_session);

		Ok(())
	}

	/// Process nested key generation session action.
	fn process_generation_session_action(id: &SessionId, access_key: &Secret, completed: &Condvar, data: &mut SessionData) -> Result<(), Error> {
		// it only makes sense to process actions if session is currently active
		if data.state != SessionState::SessionKeyGeneration {
			return Ok(());
		}

		// send every scheduled message
		{
			let generation_cluster = data.generation_cluster.as_ref()
				.expect("generation cluster is crated when SessionKeyGeneration state starts; we are in SessionKeyGeneration state; qed");
			for (to, message) in generation_cluster.messages() {
				match message {
					Message::Generation(message) => generation_cluster.cluster().send(&to, Message::Signing(SigningMessage::SigningGenerationMessage(SigningGenerationMessage {
						session: id.clone().into(),
						sub_session: access_key.clone().into(),
						message: message,
					})))?,
					_ => unreachable!("generation session only sends generation messages"),
				}
			}
		}

		// and now check if session key is generated
		let generation_session = data.generation_session.as_ref()
			.expect("generation session is crated when SessionKeyGeneration state starts; we are in SessionKeyGeneration state; qed");
		match generation_session.joint_public_and_secret() {
			Some(Ok(session_joint_public_and_secret)) => {
				data.state = SessionState::SessionKeyGenerated;
				data.session_joint_public = Some(session_joint_public_and_secret.0);
				data.session_secret_coeff = Some(session_joint_public_and_secret.1);
				Ok(())
			},
			Some(Err(err)) => {
				data.state = SessionState::Failed;
				data.signed_message = Some(Err(err));
				completed.notify_all();
				Ok(())
			},
			None => Ok(()),
		}
	}

	/// Start waiting for partial signatures/partial signatures requests.
	fn start_waiting_for_partial_signing(self_node_id: &NodeId, session_id: SessionId, access_key: Secret, cluster: &Arc<Cluster>, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		if data.master.as_ref() != Some(self_node_id) {
			// if we are on the slave node, wait for partial signature requests
			data.state = SessionState::WaitingForPartialSignatureRequest;
			return Ok(());
		}

		// update state
		data.state = SessionState::WaitingForPartialSignature;

		// send jobs to all selected nodes
		let consensus = data.consensus.as_mut().expect("consensus is created on initialization phase; partial signing phase follows initialization; qed");
		let (request_id, mut confirmed_nodes) = consensus.selected_nodes().map(|(r, n)| (r.clone(), n.clone()))?;

		// send requests
		let message_hash = data.message_hash.as_ref().expect("message_hash on master is filled in initialization phase; this is master node; qed");
		for node in confirmed_nodes.iter().filter(|n| n != &self_node_id) {
			consensus.job_request_sent(node)?;
			cluster.send(node, Message::Signing(SigningMessage::RequestPartialSignature(RequestPartialSignature {
				session: session_id.clone().into(),
				sub_session: access_key.clone().into(),
				request_id: request_id.clone().into(),
				message_hash: message_hash.clone().into(),
				nodes: confirmed_nodes.iter().cloned().map(Into::into).collect(),
			})))?;
		}

		// confirmation from this node, if this node is in consensus group
		if confirmed_nodes.remove(self_node_id) {
			let signing_result = {
				let session_joint_public = data.session_joint_public.as_ref().expect("session key is generated on key generation phase; partial signing phase follows initialization; qed");
				let session_secret_coeff = data.session_secret_coeff.as_ref().expect("sessin coeff is generated on key generation phase; partial signing phase follows initialization; qed");
				SessionImpl::do_partial_signing(self_node_id, message_hash, encrypted_data, &confirmed_nodes, session_joint_public, session_secret_coeff)?
			};

			consensus.job_request_sent(&self_node_id)?;
			consensus.job_response_received(&self_node_id, &request_id, signing_result)?;
		}

		Ok(())
	}

	/// Compute partial signature.
	fn do_partial_signing(self_node_id: &NodeId, message_hash: &H256, encrypted_data: &DocumentKeyShare, session_nodes: &BTreeSet<NodeId>, session_joint_public: &Public, session_secret_coeff: &Secret) -> Result<Secret, Error> {
		debug_assert!(!session_nodes.contains(self_node_id));
		debug_assert!(session_nodes.len() == encrypted_data.threshold);

		let combined_hash = math::combine_message_hash_with_public(&message_hash, &session_joint_public)?;
		math::compute_signature_share(
			&combined_hash,
			&session_secret_coeff,
			&encrypted_data.secret_share,
			&encrypted_data.id_numbers[self_node_id],
			session_nodes.iter().map(|n| &encrypted_data.id_numbers[n])
		)
	}

	/// Compute signature
	fn do_signing(data: &mut SessionData) -> Result<(), Error> {
		let message_hash = data.message_hash.as_ref().expect("message_hash on master is filled in initialization phase; this is master node; qed");
		let session_joint_public = data.session_joint_public.as_ref().expect("session key is generated on key generation phase; signing phase follows initialization; qed");
		let partial_signatures = data.consensus.as_ref().expect("consensus on master is filled in initialization phase; this is master node; qed").job_responses()?.values();

		let signature_c = math::combine_message_hash_with_public(message_hash, session_joint_public)?;
		let signature_s = math::compute_signature(partial_signatures)?;
	
		data.signed_message = Some(Ok((signature_c, signature_s)));

		Ok(())
	}
}

impl ClusterSession for SessionImpl {
	fn is_finished(&self) -> bool {
		let data = self.data.lock();
		data.state == SessionState::Failed
			|| data.state == SessionState::Finished
	}

	fn on_node_timeout(&self, node: &NodeId) {
		let mut data = self.data.lock();

		let is_self_master = data.master.as_ref() == Some(self.node());
		let is_other_master = data.master.as_ref() == Some(node);
		// if this is master node, we might have to restart
		if is_self_master {
			let is_restart_required = match data.consensus.as_mut() {
				None => false,
				Some(consensus) => match consensus.node_timeouted(node) {
					Ok(false) => return,
					Ok(true) => true,
					Err(_) => false, //fall through
				},
			};
			if is_restart_required {
				if SessionImpl::start_waiting_for_partial_signing(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data).is_ok() {
					return;
				}
			}
		} else if !is_other_master {
			// disconnected from non-master node on non-master node
			// => this does not affect this session
			return;
		}
		// else: disconnecting from master node means failure

		warn!("{}: signing session failed because {} connection has timeouted", self.node(), node);

		data.state = SessionState::Failed;
		data.signed_message = Some(Err(Error::NodeDisconnected));
		self.completed.notify_all();
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		let is_self_master = data.master.as_ref() == Some(self.node());
		// if this is master node, we might have to restart
		if is_self_master {
			let is_restart_required = match data.consensus.as_mut() {
				None => false,
				Some(consensus) => match consensus.session_timeouted() {
					Ok(_) => true,
					Err(_) => false,
				},
			};
			if is_restart_required {
				if SessionImpl::start_waiting_for_partial_signing(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data).is_ok() {
					return;
				}
			}
		}

		warn!("{}: signing session failed with timeout", self.node());

		data.state = SessionState::Failed;
		data.signed_message = Some(Err(Error::NodeDisconnected));
		self.completed.notify_all();
	}
}

impl SigningCluster {
	pub fn new(cluster: Arc<Cluster>, self_node_id: NodeId, subset: BTreeSet<NodeId>) -> Self {
		SigningCluster {
			cluster: cluster,
			self_node_id: self_node_id,
			nodes: subset,
			messages: Mutex::new(VecDeque::new()),
		}
	}

	pub fn cluster(&self) -> &Arc<Cluster> {
		&self.cluster
	}

	pub fn messages(&self) -> VecDeque<(NodeId, Message)> {
		let mut lock = self.messages.lock();
		let mut messages = VecDeque::new();
		swap(&mut messages, &mut *lock);
		messages
	}
}

impl Cluster for SigningCluster {
	fn broadcast(&self, message: Message) -> Result<(), Error> {
		let mut messages = self.messages.lock();
		for node in &self.nodes {
			if node != &self.self_node_id {
				messages.push_back((node.clone(), message.clone()));
			}
		}
		Ok(())
	}

	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error> {
		self.messages.lock().push_back((to.clone(), message.clone()));
		Ok(())
	}
}

impl Session for SessionImpl {
	fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	fn wait(&self) -> Result<(Secret, Secret), Error> {
		let mut data = self.data.lock();
		if !data.signed_message.is_some() {
			self.completed.wait(&mut data);
		}

		data.signed_message.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when signed_message.is_some(); qed")
			.clone()
	}
}

fn check_encrypted_data(self_node_id: &Public, encrypted_data: &DocumentKeyShare) -> Result<(), Error> {
	use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{BTreeMap, VecDeque};
	use ethkey::{Random, Generator, Public, sign};
	use util::H256;
	use super::super::super::acl_storage::tests::DummyAclStorage;
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::{Session as GenerationSession};
	use key_server_cluster::generation_session::tests::MessageLoop as KeyGenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::{Message, SigningMessage};
	use key_server_cluster::signing_session::{Session, SessionImpl, SessionParams};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub session: SessionImpl,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	impl MessageLoop {
		pub fn new(gl: &KeyGenerationMessageLoop) -> Self {
			let mut nodes = BTreeMap::new();
			let session_id = gl.session_id.clone();
			for (gl_node_id, gl_node) in &gl.nodes {
				let acl_storage = Arc::new(DummyAclStorage::default());
				let cluster = Arc::new(DummyCluster::new(gl_node_id.clone()));
				let session = SessionImpl::new(SessionParams {
					id: session_id.clone(),
					access_key: "834cb736f02d9c968dfaf0c37658a1d86ff140554fc8b59c9fdad5a8cf810eec".parse().unwrap(),
					self_node_id: gl_node_id.clone(),
					encrypted_data: gl_node.key_storage.get(&session_id).unwrap(),
					acl_storage: acl_storage,
					cluster: cluster.clone(),
				}).unwrap();
				nodes.insert(gl_node_id.clone(), Node { cluster: cluster, session: session });
			}

			let nodes_ids: Vec<_> = nodes.keys().cloned().collect();
			for node in nodes.values() {
				for node_id in &nodes_ids {
					node.cluster.add_node(node_id.clone());
				}
			}

			MessageLoop {
				session_id: session_id,
				nodes: nodes,
				queue: VecDeque::new(),
			}
		}

		pub fn master(&self) -> &SessionImpl {
			&self.nodes.values().nth(0).unwrap().session
		}

		pub fn take_message(&mut self) -> Option<(NodeId, NodeId, Message)> {
			self.nodes.values()
				.filter_map(|n| n.cluster.take_message().map(|m| (n.session.node().clone(), m.0, m.1)))
				.nth(0)
				.or_else(|| self.queue.pop_front())
		}

		pub fn process_message(&mut self, msg: (NodeId, NodeId, Message)) -> Result<(), Error> {
			match {
				match msg.2 {
					Message::Signing(SigningMessage::SigningConsensusMessage(ref message)) => self.nodes[&msg.1].session.on_consensus_message(msg.0.clone(), &message),
					Message::Signing(SigningMessage::SigningGenerationMessage(ref message)) => self.nodes[&msg.1].session.on_generation_message(msg.0.clone(), &message),
					Message::Signing(SigningMessage::RequestPartialSignature(ref message)) => self.nodes[&msg.1].session.on_partial_signature_requested(msg.0.clone(), &message),
					Message::Signing(SigningMessage::PartialSignature(ref message)) => self.nodes[&msg.1].session.on_partial_signature(msg.0.clone(), &message),
					Message::Signing(SigningMessage::SigningSessionCompleted(ref message)) => self.nodes[&msg.1].session.on_session_completed(msg.0.clone(), &message),
					_ => panic!("unexpected"),
				}
			} {
				Ok(_) => Ok(()),
				Err(Error::TooEarlyForRequest) => {
					self.queue.push_back(msg);
					Ok(())
				},
				Err(err) => Err(err),
			}
		}
	}

	#[test]
	fn complete_gen_sign_session() {
		//let test_cases = [(0, 1)];
		let test_cases = [(1, 3)];
		for &(threshold, num_nodes) in &test_cases {
			// run key generation sessions
			let mut gl = KeyGenerationMessageLoop::new(num_nodes);
			gl.master().initialize(Public::default(), threshold, gl.nodes.keys().cloned().collect()).unwrap();
			while let Some((from, to, message)) = gl.take_message() {
				gl.process_message((from, to, message)).unwrap();
			}

			// run signing session
			let requestor_pair = Random.generate().unwrap();
			let requestor_signature = sign(&requestor_pair.secret(), &SessionId::default()).unwrap();
			let message_hash = H256::from(777);
			let mut sl = MessageLoop::new(&gl);
			sl.master().initialize(requestor_signature, message_hash).unwrap();
			while let Some((from, to, message)) = sl.take_message() {
				sl.process_message((from, to, message)).unwrap();
			}

			// verify signature
			let public = gl.master().joint_public_and_secret().unwrap().unwrap().0;
			let signature = sl.master().wait().unwrap();
			assert!(math::verify_signature(&public, &signature, &message_hash).unwrap());
		}
	}
}
