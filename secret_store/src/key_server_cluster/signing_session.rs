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
use std::cmp::{Ordering, Ord, PartialOrd};
use std::sync::Arc;
use std::time;
use parking_lot::{Mutex, Condvar};
use ethkey::{self, Public, Secret, Signature};
use util::{self, H256};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::consensus_session::{ConsensusSession, Consensus, SessionParams as ConsensusSessionParams,
	SessionState as ConsensusSessionState, SessionAction as ConsensusSessionAction};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	SessionState as GenerationSessionState, Session as GenerationSessionApi};
use key_server_cluster::math;
use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, SigningGenerationMessage,
	RequestPartialSignature, PartialSignature, SigningSessionCompleted, GenerationMessage, ConsensusMessage};

/// Signing session API.
pub trait Session: Send + Sync + 'static {
	/// Get generation session state.
	fn state(&self) -> SessionState;
	/// Wait until session is completed. Returns signed message.
	fn wait(&self, timeout: Option<time::Duration>) -> Result<util::Bytes, Error>;
}

/// Signing session.
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

/// Signing session Id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningSessionId {
	/// Encryption session id.
	pub id: SessionId,
	/// Signing session access key.
	pub access_key: Secret,
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

	// === Values, filled when consensus is establishing ===
	/// Consensus session.
	consensus_session: Option<ConsensusSession>,
	/// Consensus params.
	consensus: Option<Consensus>,

	// === Values, filled when session key is generating ===
	/// Signing cluster subgroup.
	generation_cluster: Option<Arc<SigningCluster>>,
	/// Session key generation session.
	generation_session: Option<GenerationSession>,
	/// Generated session public key.
	session_joint_public: Option<Public>,
	/// Generated session secret coefficient.
	session_secret_coeff: Option<Secret>,

	// === Values, filled when partial signatures are generating ===
	/// Nodes which have agreed to make partial signatures.
	confirmed_nodes: BTreeSet<NodeId>,
	/// Active partial requests.
	partial_requests: BTreeSet<NodeId>,
	/// Partial signatures.
	partial_signatures: VecDeque<Secret>,

	/// === Values, filled during final decryption ===
	/// Decrypted secret
	signed_message: Option<Result<(Secret, Secret), Error>>,
}

#[derive(Debug, Clone)]
/// Mutable node-specific data.
struct NodeData {
	/// Random unique scalar. Persistent.
	pub id_number: Secret,
}

#[derive(Debug, Clone, PartialEq)]
/// Distributed key generation session state.
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Establishing consensus.
	EstablishingConsensus,

	// === Intermediate states ===
	/// Generating one-time key.
	SessionKeyGeneration,
	/// Waiting for partial signatures.
	WaitingForPartialSignature,

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
				confirmed_nodes: BTreeSet::new(),
				partial_requests: BTreeSet::new(),
				partial_signatures: VecDeque::new(),
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
		//data.requestor = Some(requestor_public.clone());
		data.message_hash = Some(message_hash);

		// create consensus session
		let consensus_session = ConsensusSession::new(ConsensusSessionParams {
			id: self.id.clone(),
			self_node_id: self.self_node_id.clone(),
			encrypted_data: self.encrypted_data.clone(),
			acl_storage: self.acl_storage.clone(),
		})?;

		// start consensus session
		let consensus_action = consensus_session.initialize(requestor_signature)?;
		data.consensus_session = Some(consensus_session);
		SessionImpl::process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if single node is required to sign message, proceed
		if data.state != SessionState::Failed && data.consensus.is_some() {
					//data.state = SessionState::Finished;
					//SessionImpl::start_waiting_for_partial_signature(self.node().clone(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)?;
					//SessionImpl::make_signature(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
					//self.completed.notify_all();
		}

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: NodeId, message: &SigningConsensusMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// if we are waiting for initialization
		if data.state == SessionState::WaitingForInitialization {
			data.master = Some(sender.clone());
			//data.requestor = Some(requestor_public);
			data.state = SessionState::EstablishingConsensus;
			data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
				id: self.id.clone(),
				self_node_id: self.self_node_id.clone(),
				encrypted_data: self.encrypted_data.clone(),
				acl_storage: self.acl_storage.clone(),
			})?);
		}

		// check state
		if data.state != SessionState::EstablishingConsensus {
			return Err(Error::InvalidStateForRequest);
		}
		// TODO: check master node + etc

		// process message
		let consensus_action = match message.message {
			ConsensusMessage::InitializeConsensusSession(ref message) =>
				data.consensus_session.as_ref().expect("TODO").on_initialize_session(sender, &message)?,
			ConsensusMessage::ConfirmConsensusInitialization(ref message) =>
				data.consensus_session.as_ref().expect("TODO").on_confirm_initialization(sender, &message)?,
		};
		SessionImpl::process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if consensus is reached, start generating session key
		if data.state == SessionState::Failed || data.consensus.is_some() {
			return Ok(());
		}
		SessionImpl::start_generating_session_key(self.self_node_id.clone(), &self.encrypted_data, &self.cluster, &mut *data)?;
		SessionImpl::process_generation_session_action(&self.id, &self.access_key, &self.completed, &mut *data)
	}

	/// When session key related message is received.
	pub fn on_generation_message(&self, sender: NodeId, message: &SigningGenerationMessage) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::EstablishingConsensus {
			return Err(Error::InvalidStateForRequest);
		}
		// TODO: check master node + etc

		// process message
		match message.message {
			GenerationMessage::InitializeSession(ref message) => {
				// if we are NOT part of consensus
				if data.consensus_session.as_ref().expect("TODO").state() != ConsensusSessionState::Finished {
					return Err(Error::InvalidStateForRequest);
				}

				// update state
				data.state = SessionState::SessionKeyGeneration;

				// create generation session
				let generation_cluster = Arc::new(SigningCluster::new(self.cluster.clone(), message.nodes.keys().cloned().map(Into::into).collect()));
				data.generation_cluster = Some(generation_cluster.clone());
				data.generation_session = Some(GenerationSession::new(GenerationSessionParams {
					id: message.session.clone().into(),
					self_node_id: self.self_node_id.clone(),
					key_storage: None,
					cluster: generation_cluster,
				}));

				// process initialization message
				data.generation_session.as_ref().expect("TODO").on_initialize_session(sender, &message)?
			},
			GenerationMessage::ConfirmInitialization(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_confirm_initialization(sender, &message)?,
			GenerationMessage::CompleteInitialization(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_complete_initialization(sender, &message)?,
			GenerationMessage::KeysDissemination(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_keys_dissemination(sender, &message)?,
			GenerationMessage::PublicKeyShare(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_public_key_share(sender, &message)?,
			GenerationMessage::SessionError(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_session_error(sender, &message)?,
			GenerationMessage::SessionCompleted(ref message) =>
				data.generation_session.as_ref().expect("TODO").on_session_completed(sender, &message)?,
		};
		SessionImpl::process_generation_session_action(&self.id, &self.access_key, &self.completed, &mut *data)?;

		// if session key generated is not yet completed => continue
		if data.state == SessionState::Failed || !data.session_joint_public.is_some() {
			return Ok(());
		}

		// else ask other nodes to generate partial signatures
		SessionImpl::start_waiting_for_partial_signing(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)
	}

	fn on_partial_signature_requested(&self, sender: NodeId, message: &RequestPartialSignature) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		// TODO: check message
		//if message.nodes.len() != self.encrypted_data.threshold + 1 {
		//	return Err(Error::InvalidMessage);
		//}

		let data = self.data.lock();

		// check state
		if data.master != Some(sender) {
			return Err(Error::InvalidMessage);
		}
		if data.state != SessionState::EstablishingConsensus && data.generation_session.as_ref().expect("TODO").state() != GenerationSessionState::Finished {
			return Err(Error::InvalidStateForRequest);
		}

		// calculate partial signature
		let message_hash = data.message_hash.as_ref().expect("TODO");
		let session_joint_public = data.session_joint_public.as_ref().expect("TODO");
		let session_secret_coeff = data.session_secret_coeff.as_ref().expect("TODO");
		let partial_signature = SessionImpl::do_partial_signing(&self.self_node_id, message_hash, &self.encrypted_data, &data.confirmed_nodes, session_joint_public, session_secret_coeff)?;

		self.cluster.send(&sender, Message::Signing(SigningMessage::PartialSignature(PartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
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

		if !data.partial_requests.remove(&sender) {
			return Err(Error::InvalidStateForRequest);
		}
		data.partial_signatures.push_back(message.partial_signature.clone().into());

		// check if we have enough shadow points to decrypt the secret
		if data.partial_signatures.len() != self.encrypted_data.threshold + 1 {
			return Ok(());
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

	fn process_consensus_session_action(id: &SessionId, access_key: &Secret, cluster: &Arc<Cluster>, completed: &Condvar, data: &mut SessionData, action: ConsensusSessionAction) -> Result<(), Error> {
		match action {
			ConsensusSessionAction::BroadcastMessage(message) => {
				cluster.broadcast(Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
					session: id.clone().into(),
					sub_session: access_key.clone().into(),
					message: message,
				})))
			},
			ConsensusSessionAction::SendMessage(to, message) => {
				cluster.send(&to, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
					session: id.clone().into(),
					sub_session: access_key.clone().into(),
					message: message,
				})))
			},
			ConsensusSessionAction::CheckStatus => match data.consensus_session.as_ref().expect("TODO").consensus() {
				Some(Ok(consensus)) => {
					data.consensus = Some(consensus.clone());
					Ok(())
				},
				Some(Err(err)) => {
					data.state = SessionState::Failed;
					data.signed_message = Some(Err(err));
					completed.notify_all();
					Ok(())
				},
				None => Ok(()),
			},
		}
	}

	fn process_generation_session_action(id: &SessionId, access_key: &Secret, completed: &Condvar, data: &mut SessionData) -> Result<(), Error> {
		let generation_cluster = data.generation_cluster.as_ref().expect("TODO").clone();
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

		match data.generation_session.as_ref().expect("TODO").joint_public_key() {
			Some(Ok(session_joint_public)) => {
				data.session_joint_public = Some(session_joint_public);
				data.session_secret_coeff = Some(data.generation_session.as_ref().expect("TODO").secret_coeff().expect("TODO").expect("TODO"));
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

	fn start_generating_session_key(self_node_id: NodeId, encrypted_data: &DocumentKeyShare, cluster: &Arc<Cluster>, data: &mut SessionData) -> Result<(), Error> {
		// select nodes to make signature
		let confirmed_nodes: BTreeSet<_> = data.consensus.as_ref().expect("TODO").confirmed_nodes.clone();
		let confirmed_nodes: BTreeSet<_> = confirmed_nodes.difference(&data.consensus.as_ref().expect("TODO").rejected_nodes).cloned().collect();

		// create generation session
		let generation_cluster = Arc::new(SigningCluster::new(cluster.clone(), confirmed_nodes.clone()));
		let generation_session = GenerationSession::new(GenerationSessionParams {
			id: H256::default(), // doesn't matter
			self_node_id: self_node_id.clone(),
			key_storage: None,
			cluster: generation_cluster.clone(),
		});

		// start generation session
		let result = generation_session.initialize(data.requestor.as_ref().expect("TODO").clone(), encrypted_data.threshold, confirmed_nodes)?;
		data.generation_cluster = Some(generation_cluster);
		data.generation_session = Some(generation_session);

		Ok(())
	}

	fn start_waiting_for_partial_signing(self_node_id: &NodeId, session_id: SessionId, access_key: Secret, cluster: &Arc<Cluster>, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		// nodes which have formed consensus group
		let confirmed_nodes: BTreeSet<_> = data.consensus.as_ref().expect("TODO").confirmed_nodes.clone();
		let confirmed_nodes: BTreeSet<_> = confirmed_nodes.difference(&data.consensus.as_ref().expect("TODO").rejected_nodes).cloned().collect();

		// send requests
		data.partial_requests.clear();
		data.partial_signatures.clear();
		for node in confirmed_nodes.iter().filter(|n| n != &self_node_id) {
			data.partial_requests.insert(node.clone());
			cluster.send(node, Message::Signing(SigningMessage::RequestPartialSignature(RequestPartialSignature {
				session: session_id.clone().into(),
				sub_session: access_key.clone().into(),
				message_hash: data.message_hash.as_ref().expect("TODO").clone().into(),
			})))?;
		}

		// confirmation from this node
		data.state = SessionState::WaitingForPartialSignature;
		data.confirmed_nodes = confirmed_nodes;
		if data.confirmed_nodes.remove(&self_node_id) {
			let signing_result = {
				let message_hash = data.message_hash.as_ref().expect("TODO");
				let session_joint_public = data.session_joint_public.as_ref().expect("TODO");
				let session_secret_coeff = data.session_secret_coeff.as_ref().expect("TODO");
				SessionImpl::do_partial_signing(self_node_id, message_hash, encrypted_data, &data.confirmed_nodes, session_joint_public, session_secret_coeff)?
			};
			data.partial_signatures.push_back(signing_result);
		}

		Ok(())
	}

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

	fn do_signing(data: &mut SessionData) -> Result<(), Error> {
		let message_hash = data.message_hash.as_ref().expect("TODO");
		let session_joint_public = data.session_joint_public.as_ref().expect("TODO");
		
		let signature_c = math::combine_message_hash_with_public(message_hash, session_joint_public)?;
		let signature_s = math::compute_signature(&data.partial_signatures[0], data.partial_signatures.iter().skip(1))?;
	
		data.signed_message = Some(Ok((signature_c, signature_s)));

		Ok(())
	}
}

impl ClusterSession for SessionImpl {
	fn is_finished(&self) -> bool {
		unimplemented!()
	}

	fn on_session_timeout(&self) {
		unimplemented!()
	}

	fn on_node_timeout(&self, node_id: &NodeId) {
		unimplemented!()
	}
}

impl SigningCluster {
	pub fn new(cluster: Arc<Cluster>, subset: BTreeSet<NodeId>) -> Self {
		SigningCluster {
			cluster: cluster,
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
			messages.push_back((node.clone(), message.clone()));
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
		unimplemented!()
	}

	fn wait(&self, timeout: Option<time::Duration>) -> Result<util::Bytes, Error> {
		unimplemented!()
	}
}

impl SigningSessionId {
	/// Create new decryption session Id.
	pub fn new(session_id: SessionId, sub_session_id: Secret) -> Self {
		SigningSessionId {
			id: session_id,
			access_key: sub_session_id,
		}
	}
}

impl PartialOrd for SigningSessionId {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for SigningSessionId {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.id.cmp(&other.id) {
			Ordering::Equal => self.access_key.cmp(&other.access_key),
			r @ _ => r,
		}
	}
}

fn check_encrypted_data(self_node_id: &Public, encrypted_data: &DocumentKeyShare) -> Result<(), Error> {
	use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)
}