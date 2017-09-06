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
use bigint::hash::H256;
use key_server_cluster::{Error, NodeId, SessionId, SessionMeta, AclStorage, DocumentKeyShare};
use key_server_cluster::cluster::{Cluster};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionParams as GenerationSessionParams,
	Session as GenerationSessionApi, SessionState as GenerationSessionState};
use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, SigningGenerationMessage,
	RequestPartialSignature, PartialSignature, SigningSessionCompleted, GenerationMessage, ConsensusMessage, SigningSessionError,
	InitializeConsensusSession, ConfirmConsensusInitialization};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::signing_job::{PartialSigningRequest, PartialSigningResponse, SigningJob};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

pub use key_server_cluster::decryption_session::DecryptionSessionId as SigningSessionId;

/// Signing session API.
pub trait Session: Send + Sync + 'static {
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
	pub key_share: DocumentKeyShare,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	pub cluster: Arc<Cluster>,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
}

/// Signing consensus session type.
type SigningConsensusSession = ConsensusSession<SigningConsensusTransport, SigningJob, SigningJobTransport>;

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Message hash.
	pub message_hash: Option<H256>,
	/// Consensus-based signing session.
	pub consensus_session: SigningConsensusSession,
	/// Session key generation session.
	pub generation_session: Option<GenerationSession>,
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
	pub key_share: DocumentKeyShare,
	/// ACL storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
}

/// Signing consensus transport.
struct SigningConsensusTransport {
	/// Session id.
	id: SessionId,
	/// Session access key.
	access_key: Secret,
	/// Cluster.
	cluster: Arc<Cluster>,
}

/// Signing key generation transport.
struct SessionKeyGenerationTransport {
	/// Session access key.
	access_key: Secret,
	/// Cluster.
	cluster: Arc<Cluster>,
	/// Other nodes ids.
	other_nodes_ids: BTreeSet<NodeId>,
}

/// Signing job transport
struct SigningJobTransport {
	/// Session id.
	id: SessionId,
	//// Session access key.
	access_key: Secret,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl SessionImpl {
	/// Create new signing session.
	pub fn new(params: SessionParams, requester_signature: Option<Signature>) -> Result<Self, Error> {
		debug_assert_eq!(params.meta.threshold, params.key_share.threshold);
		debug_assert_eq!(params.meta.self_node_id == params.meta.master_node_id, requester_signature.is_some());

		use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

		// check nodes and threshold
		let nodes = params.key_share.id_numbers.keys().cloned().collect();
		check_cluster_nodes(&params.meta.self_node_id, &nodes)?;
		check_threshold(params.key_share.threshold, &nodes)?;

		let consensus_transport = SigningConsensusTransport {
			id: params.meta.id.clone(),
			access_key: params.access_key.clone(),
			cluster: params.cluster.clone(),
		};

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				access_key: params.access_key,
				key_share: params.key_share,
				cluster: params.cluster,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::ConsensusEstablishing,
				message_hash: None,
				consensus_session: match requester_signature {
					Some(requester_signature) => ConsensusSession::new_on_master(ConsensusSessionParams {
						meta: params.meta,
						acl_storage: params.acl_storage.clone(),
						consensus_transport: consensus_transport,
					}, requester_signature)?,
					None => ConsensusSession::new_on_slave(ConsensusSessionParams {
						meta: params.meta,
						acl_storage: params.acl_storage.clone(),
						consensus_transport: consensus_transport,
					})?,
				},
				generation_session: None,
				result: None,
			}),
		})
	}

	/// Get session state.
	#[cfg(test)]
	pub fn state(&self) -> SessionState {
		self.data.lock().state
	}

	/// Initialize signing session on master node.
	pub fn initialize(&self, message_hash: H256) -> Result<(), Error> {
		let mut data = self.data.lock();
		data.message_hash = Some(message_hash);
		data.consensus_session.initialize(self.core.key_share.id_numbers.keys().cloned().collect())?;

		if data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished {
			let generation_session = GenerationSession::new(GenerationSessionParams {
				id: self.core.meta.id.clone(),
				self_node_id: self.core.meta.self_node_id.clone(),
				key_storage: None,
				cluster: Arc::new(SessionKeyGenerationTransport {
					access_key: self.core.access_key.clone(),
					cluster: self.core.cluster.clone(),
					other_nodes_ids: BTreeSet::new()
				}),
			});
			generation_session.initialize(Public::default(), 0, vec![self.core.meta.self_node_id.clone()].into_iter().collect())?;

			debug_assert_eq!(generation_session.state(), GenerationSessionState::WaitingForGenerationConfirmation);
			let joint_public_and_secret = generation_session
				.joint_public_and_secret()
				.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")?;
			data.generation_session = Some(generation_session);
			data.state = SessionState::SignatureComputing;

			self.core.disseminate_jobs(&mut data.consensus_session, joint_public_and_secret.0, joint_public_and_secret.1, message_hash)?;

			debug_assert!(data.consensus_session.state() == ConsensusSessionState::Finished);
			data.result = Some(Ok(data.consensus_session.result()?));
			self.core.completed.notify_all();
		}

		Ok(())
	}

	/// Process signing message.
	pub fn process_message(&self, sender: &NodeId, message: &SigningMessage) -> Result<(), Error> {
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
				self.on_session_error(sender, message),
			&SigningMessage::SigningSessionCompleted(ref message) =>
				self.on_session_completed(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &SigningConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();
		let is_establishing_consensus = data.consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
		data.consensus_session.on_consensus_message(&sender, &message.message)?;

		let is_consensus_established = data.consensus_session.state() == ConsensusSessionState::ConsensusEstablished;
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		let consensus_group = data.consensus_session.select_consensus_group()?.clone();
		let mut other_consensus_group_nodes = consensus_group.clone();
		other_consensus_group_nodes.remove(&self.core.meta.self_node_id);

		let generation_session = GenerationSession::new(GenerationSessionParams {
			id: self.core.meta.id.clone(),
			self_node_id: self.core.meta.self_node_id.clone(),
			key_storage: None,
			cluster: Arc::new(SessionKeyGenerationTransport {
				access_key: self.core.access_key.clone(),
				cluster: self.core.cluster.clone(),
				other_nodes_ids: other_consensus_group_nodes,
			}),
		});
		generation_session.initialize(Public::default(), self.core.key_share.threshold, consensus_group)?;
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
				return Err(Error::InvalidMessage);
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
					other_nodes_ids: other_consensus_group_nodes
				}),
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

		let message_hash = data.message_hash
			.expect("we are on master node; on master node message_hash is filled in initialize(); on_generation_message follows initialize; qed");
		let joint_public_and_secret = data.generation_session.as_ref()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")
			.joint_public_and_secret()
			.expect("session key is generated before signature is computed; we are in SignatureComputing state; qed")?;
		self.core.disseminate_jobs(&mut data.consensus_session, joint_public_and_secret.0, joint_public_and_secret.1, message_hash)
	}

	/// When partial signature is requested.
	pub fn on_partial_signature_requested(&self, sender: &NodeId, message: &RequestPartialSignature) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

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
		let signing_job = SigningJob::new_on_slave(self.core.meta.self_node_id.clone(), self.core.key_share.clone(), joint_public_and_secret.0, joint_public_and_secret.1)?;
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
			})))?;
		}

		data.result = Some(Ok(data.consensus_session.result()?));
		self.core.completed.notify_all();

		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&self, sender: &NodeId, message: &SigningSessionCompleted) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.access_key == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		self.data.lock().consensus_session.on_session_completed(sender)
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &SigningSessionError) -> Result<(), Error> {
		self.process_node_error(Some(&sender), &message.error)
	}

	/// Process error from the other node.
	fn process_node_error(&self, node: Option<&NodeId>, error: &String) -> Result<(), Error> {
		let mut data = self.data.lock();
		match {
			match node {
				Some(node) => data.consensus_session.on_node_error(node),
				None => data.consensus_session.on_session_timeout(),
			}
		} {
			Ok(false) => Ok(()),
			Ok(true) => {
				let message_hash = data.message_hash.as_ref().cloned()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed");
				let joint_public_and_secret = data.generation_session.as_ref()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed")
					.joint_public_and_secret()
					.expect("on_node_error returned true; this means that jobs must be REsent; this means that jobs already have been sent; jobs are sent when message_hash.is_some(); qed")?;
				let disseminate_result = self.core.disseminate_jobs(&mut data.consensus_session, joint_public_and_secret.0, joint_public_and_secret.1, message_hash);
				match disseminate_result {
					Ok(()) => Ok(()),
					Err(err) => {
						warn!("{}: signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);

						data.result = Some(Err(err.clone()));
						self.core.completed.notify_all();
						Err(err)
					}
				}
			},
			Err(err) => {
				warn!("{}: signing session failed with error: {:?} from {:?}", &self.core.meta.self_node_id, error, node);

				data.result = Some(Err(err.clone()));
				self.core.completed.notify_all();
				Err(err)
			},
		}
	}
}

impl ClusterSession for SessionImpl {
	fn is_finished(&self) -> bool {
		let data = self.data.lock();
		data.consensus_session.state() == ConsensusSessionState::Failed
			|| data.consensus_session.state() == ConsensusSessionState::Finished
	}

	fn on_node_timeout(&self, node: &NodeId) {
		// ignore error, only state matters
		let _ = self.process_node_error(Some(node), &Error::NodeDisconnected.into());
	}

	fn on_session_timeout(&self) {
		// ignore error, only state matters
		let _ = self.process_node_error(None, &Error::NodeDisconnected.into());
	}
}

impl Session for SessionImpl {
	fn wait(&self) -> Result<(Secret, Secret), Error> {
		let mut data = self.data.lock();
		if !data.result.is_some() {
			self.core.completed.wait(&mut data);
		}

		data.result.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when result.is_some(); qed")
			.clone()
	}
}

impl SessionKeyGenerationTransport {
	fn map_message(&self, message: Message) -> Result<Message, Error> {
		match message {
			Message::Generation(message) => Ok(Message::Signing(SigningMessage::SigningGenerationMessage(SigningGenerationMessage {
				session: message.session_id().clone().into(),
				sub_session: self.access_key.clone().into(),
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
}

impl SessionCore {
	pub fn signing_transport(&self) -> SigningJobTransport {
		SigningJobTransport {
			id: self.meta.id.clone(),
			access_key: self.access_key.clone(),
			cluster: self.cluster.clone()
		}
	}

	pub fn disseminate_jobs(&self, consensus_session: &mut SigningConsensusSession, session_public: Public, session_secret_share: Secret, message_hash: H256) -> Result<(), Error> {
		let signing_job = SigningJob::new_on_master(self.meta.self_node_id.clone(), self.key_share.clone(), session_public, session_secret_share, message_hash)?;
		consensus_session.disseminate_jobs(signing_job, self.signing_transport())
	}
}

impl JobTransport for SigningConsensusTransport {
	type PartialJobRequest=Signature;
	type PartialJobResponse=bool;

	fn send_partial_request(&self, node: &NodeId, request: Signature) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			message: ConsensusMessage::InitializeConsensusSession(InitializeConsensusSession {
				requestor_signature: request.into(),
			})
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::SigningConsensusMessage(SigningConsensusMessage {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
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
			request_id: request.id.into(),
			message_hash: request.message_hash.into(),
			nodes: request.other_nodes_ids.into_iter().map(Into::into).collect(),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: PartialSigningResponse) -> Result<(), Error> {
		self.cluster.send(node, Message::Signing(SigningMessage::PartialSignature(PartialSignature {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
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
	use bigint::hash::H256;
	use ethkey::{self, Random, Generator, Public, Secret, KeyPair};
	use acl_storage::DummyAclStorage;
	use key_server_cluster::{NodeId, DocumentKeyShare, SessionId, SessionMeta, Error, KeyStorage};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::{Session as GenerationSession};
	use key_server_cluster::generation_session::tests::MessageLoop as KeyGenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::{Message, SigningMessage, SigningConsensusMessage, ConsensusMessage, ConfirmConsensusInitialization,
		SigningGenerationMessage, GenerationMessage, ConfirmInitialization, InitializeSession, RequestPartialSignature};
	use key_server_cluster::signing_session::{Session, SessionImpl, SessionState, SessionParams};

	struct Node {
		pub node_id: NodeId,
		pub cluster: Arc<DummyCluster>,
		pub session: SessionImpl,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub requester: KeyPair,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
		pub acl_storages: Vec<Arc<DummyAclStorage>>,
	}

	impl MessageLoop {
		pub fn new(gl: &KeyGenerationMessageLoop) -> Self {
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
						threshold: gl_node.key_storage.get(&session_id).unwrap().threshold,
					},
					access_key: "834cb736f02d9c968dfaf0c37658a1d86ff140554fc8b59c9fdad5a8cf810eec".parse().unwrap(),
					key_share: gl_node.key_storage.get(&session_id).unwrap(),
					acl_storage: acl_storage,
					cluster: cluster.clone(),
				}, if i == 0 { signature.clone() } else { None }).unwrap();
				nodes.insert(gl_node_id.clone(), Node { node_id: gl_node_id.clone(), cluster: cluster, session: session });
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
				match {
					match msg.2 {
						Message::Signing(SigningMessage::SigningConsensusMessage(ref message)) => self.nodes[&msg.1].session.on_consensus_message(&msg.0, &message),
						Message::Signing(SigningMessage::SigningGenerationMessage(ref message)) => self.nodes[&msg.1].session.on_generation_message(&msg.0, &message),
						Message::Signing(SigningMessage::RequestPartialSignature(ref message)) => self.nodes[&msg.1].session.on_partial_signature_requested(&msg.0, &message),
						Message::Signing(SigningMessage::PartialSignature(ref message)) => self.nodes[&msg.1].session.on_partial_signature(&msg.0, &message),
						Message::Signing(SigningMessage::SigningSessionCompleted(ref message)) => self.nodes[&msg.1].session.on_session_completed(&msg.0, &message),
						_ => panic!("unexpected"),
					}
				} {
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
			sl.master().initialize(message_hash).unwrap();
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
			key_share: DocumentKeyShare {
				author: Public::default(),
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())) {
			Ok(_) => (),
			_ => panic!("unexpected"),
		}
	}

	#[test]
	fn fails_to_construct_if_not_a_part_of_cluster() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone());
		nodes.insert(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone());
		match SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: SessionId::default(),
				self_node_id: self_node_id.clone(),
				master_node_id: self_node_id.clone(),
				threshold: 0,
			},
			access_key: Random.generate().unwrap().secret().clone(),
			key_share: DocumentKeyShare {
				author: Public::default(),
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())) {
			Err(Error::InvalidNodesConfiguration) => (),
			_ => panic!("unexpected"),
		}
	}

	#[test]
	fn fails_to_construct_if_threshold_is_wrong() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(self_node_id.clone(), Random.generate().unwrap().secret().clone());
		nodes.insert(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone());
		match SessionImpl::new(SessionParams {
			meta: SessionMeta {
				id: SessionId::default(),
				self_node_id: self_node_id.clone(),
				master_node_id: self_node_id.clone(),
				threshold: 2,
			},
			access_key: Random.generate().unwrap().secret().clone(),
			key_share: DocumentKeyShare {
				author: Public::default(),
				threshold: 2,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}, Some(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap())) {
			Err(Error::InvalidThreshold) => (),
			_ => panic!("unexpected"),
		}
	}

	#[test]
	fn fails_to_initialize_when_already_initialized() {
		let (_, sl) = prepare_signing_sessions(1, 3);
		assert_eq!(sl.master().initialize(777.into()), Ok(()));
		assert_eq!(sl.master().initialize(777.into()), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn does_not_fail_when_consensus_message_received_after_consensus_established() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(777.into()).unwrap();
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
			message: GenerationMessage::ConfirmInitialization(ConfirmInitialization {
				session: SessionId::default().into(),
				derived_point: Public::default().into(),
			}),
		}), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn fails_when_generation_sesson_is_initialized_by_slave_node() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(777.into()).unwrap();
		sl.run_until(|sl| sl.master().state() == SessionState::SessionKeyGeneration).unwrap();

		let slave2_id = sl.nodes.keys().nth(2).unwrap().clone();
		let slave1 = &sl.nodes.values().nth(1).unwrap().session;

		assert_eq!(slave1.on_generation_message(&slave2_id, &SigningGenerationMessage {
			session: SessionId::default().into(),
			sub_session: sl.master().core.access_key.clone().into(),
			message: GenerationMessage::InitializeSession(InitializeSession {
				session: SessionId::default().into(),
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
			request_id: Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
			message_hash: H256::default().into(),
			nodes: Default::default(),
		}), Err(Error::InvalidMessage));
	}

	#[test]
	fn failed_signing_session() {
		let (_, mut sl) = prepare_signing_sessions(1, 3);
		sl.master().initialize(777.into()).unwrap();

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
		sl.master().initialize(777.into()).unwrap();

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
		sl.master().initialize(777.into()).unwrap();

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
}
