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

use std::cmp::{Ord, PartialOrd, Ordering};
use std::collections::{BTreeSet, BTreeMap};
use std::ops::DerefMut;
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use ethcrypto::ecies::encrypt;
use ethcrypto::DEFAULT_MAC;
use ethkey::{self, Secret, Public, Signature};
use key_server_cluster::{Error, AclStorage, DocumentKeyShare, NodeId, SessionId, EncryptedDocumentKeyShadow};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::consensus::Consensus;
use key_server_cluster::consensus_session::{ConsensusSession, AclConsensusChecker, SessionParams as ConsensusSessionParams,
	SessionState as ConsensusSessionState, SessionAction as ConsensusSessionAction};
use key_server_cluster::math;
use key_server_cluster::message::{Message, DecryptionMessage, DecryptionConsensusMessage, RequestPartialDecryption,
	PartialDecryption, DecryptionSessionError, DecryptionSessionCompleted, ConsensusMessage};

/// Decryption session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed. Returns distributely restored secret key.
	fn wait(&self) -> Result<EncryptedDocumentKeyShadow, Error>;
}

/// Distributed decryption session.
/// Based on "ECDKG: A Distributed Key Generation Protocol Based on Elliptic Curve Discrete Logarithm" paper:
/// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.124.4128&rep=rep1&type=pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for decrypting the secret) requests all other nodes to decrypt the secret
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor has access to the document
/// 3) partial decryption: every node which has succussfully checked access for the requestor do a partial decryption
/// 4) decryption: master node receives all partial decryptions of the secret and restores the secret
pub struct SessionImpl {
	/// Encryption session id.
	id: SessionId,
	/// Decryption session access key.
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

/// Decryption session Id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecryptionSessionId {
	/// Encryption session id.
	pub id: SessionId,
	/// Decryption session access key.
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
	/// ACL storage.
	pub acl_storage: Arc<AclStorage>,
	/// Cluster
	pub cluster: Arc<Cluster>,
}

#[derive(Debug, Clone)]
/// Partial decryption result.
struct PartialDecryptionResult {
	/// Shadow point.
	pub shadow_point: Public,
	/// Decryption shadow coefficient, if requested.
	pub decrypt_shadow: Option<Vec<u8>>,
}

/// Mutable data of encryption (distributed key generation) session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,
	/// Public key of requestor.
	requestor: Option<Public>,
	/// Is shadow decryption requested?
	is_shadow_decryption: Option<bool>,
	/// Decryption consensus group.
	consensus: Option<Consensus<PartialDecryptionResult>>,

	// === Values, filled when consensus is establishing ===
	/// Consensus session.
	consensus_session: Option<ConsensusSession<AclConsensusChecker>>,

	/// === Values, filled during final decryption ===
	/// Decrypted secret
	decrypted_secret: Option<Result<EncryptedDocumentKeyShadow, Error>>,
}

#[derive(Debug, Clone, PartialEq)]
/// Decryption session data.
pub enum SessionState {
	// === Initialization states ===
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Establishing consensus.
	EstablishingConsensus,
	/// Consensus established
	EstablishedConsensus,

	// === Decryption states ===
	/// Waiting for partial decrypion request.
	WaitingForPartialDecryptionRequest,
	/// Waiting for partial decryption responses.
	WaitingForPartialDecryption,

	// === Final states of the session ===
	/// Decryption session is finished for this node.
	Finished,
	/// Decryption session is failed for this node.
	Failed,
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
				is_shadow_decryption: None,
				consensus: None,
				consensus_session: None,
				decrypted_secret: None,
			})
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Get current session state.
	pub fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	#[cfg(test)]
	/// Get this session access key.
	pub fn access_key(&self) -> &Secret {
		&self.access_key
	}

	#[cfg(test)]
	/// Get decrypted secret
	pub fn decrypted_secret(&self) -> Option<Result<EncryptedDocumentKeyShadow, Error>> {
		self.data.lock().decrypted_secret.clone()
	}

	/// Initialize decryption session.
	pub fn initialize(&self, requestor_signature: Signature, is_shadow_decryption: bool) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
println!("=== 0");
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		data.master = Some(self.node().clone());
		data.state = SessionState::EstablishingConsensus;
		data.requestor = Some(requestor_public.clone());
		data.is_shadow_decryption = Some(is_shadow_decryption);

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
			SessionImpl::start_waiting_for_partial_decryption(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)?;
			SessionImpl::do_decryption(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
			self.completed.notify_all();
		}

		Ok(())
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: NodeId, message: &DecryptionConsensusMessage) -> Result<(), Error> {
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
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidStateForRequest).map_err(|e| { println!("=== 1");e })?;
			match message.message {
				ConsensusMessage::InitializeConsensusSession(ref message) => {
					let requestor = ethkey::recover(&message.requestor_signature, &self.id)?;
					data.requestor = Some(requestor.clone());
					consensus_session.on_initialize_session(sender, &requestor)?
				},
				ConsensusMessage::ConfirmConsensusInitialization(ref message) => {
					let consensus = data.consensus.as_mut().ok_or(Error::InvalidStateForRequest).map_err(|e| { println!("=== 2"); e })?;
					consensus_session.on_confirm_initialization(sender, message.is_confirmed, consensus)?
				},
			}
		};
		SessionImpl::process_consensus_session_action(&self.id, &self.access_key, &self.cluster, &self.completed, &mut *data, consensus_action)?;

		// if consensus is established and we are on master node => ask for partial decryption
println!("====================== before state check: {:?}", data.state);
		if data.state != SessionState::EstablishedConsensus {
			return Ok(());
		}
println!("====================== before start_waiting_for_partial_decryption");
		SessionImpl::start_waiting_for_partial_decryption(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)
	}

	pub fn on_partial_decryption_requested(&self, sender: NodeId, message: &RequestPartialDecryption) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		// check message
		if message.nodes.len() != self.encrypted_data.threshold + 1 {
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		// check state
		if data.master != Some(sender) {
			return Err(Error::InvalidMessage);
		}
		if data.state == SessionState::EstablishingConsensus {
			// for slave nodes consensus is established when partial decryption request is received
			data.state = SessionState::WaitingForPartialDecryptionRequest;
		}
		if data.state != SessionState::WaitingForPartialDecryptionRequest {
println!("=== 3: {:?}", data.state);
			return Err(Error::InvalidStateForRequest);
		}

		// calculate shadow point
		let decryption_result = {
			let requestor = data.requestor.as_ref().expect("requestor public is filled during initialization; WaitingForPartialDecryptionRequest follows initialization; qed");
			let nodes = message.nodes.iter().cloned().map(Into::into).collect();
			do_partial_decryption(self.node(), &requestor, message.is_shadow_decryption, &nodes, &self.access_key, &self.encrypted_data)?
		};
		self.cluster.send(&sender, Message::Decryption(DecryptionMessage::PartialDecryption(PartialDecryption {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			shadow_point: decryption_result.shadow_point.into(),
			decrypt_shadow: decryption_result.decrypt_shadow,
		})))?;

		// master could ask us for another partial decryption in case of restart
		// => no state change is required

		Ok(())
	}

	/// When partial decryption is received.
	pub fn on_partial_decryption(&self, sender: NodeId, message: &PartialDecryption) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForPartialDecryption {
println!("=== 4");
			return Err(Error::InvalidStateForRequest);
		}

		// remember partial signature
		{
			let consensus = data.consensus.as_mut().ok_or(Error::InvalidStateForRequest).map_err(|e| { println!("=== 5"); e })?;
			consensus.job_response_received(&sender, PartialDecryptionResult {
				shadow_point: message.shadow_point.clone().into(),
				decrypt_shadow: message.decrypt_shadow.clone(),
			})?;

			// check if we have enough shadow points to decrypt the secret
			if !consensus.is_completed() {
				return Ok(());
			}
		}

		// notify all other nodes about session completion
		self.cluster.broadcast(Message::Decryption(DecryptionMessage::DecryptionSessionCompleted(DecryptionSessionCompleted {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
		})))?;

		// do decryption
		SessionImpl::do_decryption(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
		self.completed.notify_all();

		Ok(())
	}

	/// When session is completed.
	pub fn on_session_completed(&self, sender: NodeId, message: &DecryptionSessionCompleted) -> Result<(), Error> {
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
	pub fn on_session_error(&self, sender: NodeId, message: &DecryptionSessionError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: decryption session failed with error: {:?} from {}", self.node(), message.error, sender);

		data.state = SessionState::Failed;
		data.decrypted_secret = Some(Err(Error::Io(message.error.clone())));
		self.completed.notify_all();

		Ok(())
	}

	/// Process nested consensus session action.
	fn process_consensus_session_action(id: &SessionId, access_key: &Secret, cluster: &Arc<Cluster>, completed: &Condvar, data: &mut SessionData, action: ConsensusSessionAction) -> Result<(), Error> {
		match action {
			ConsensusSessionAction::BroadcastMessage(message) => {
				cluster.broadcast(Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(DecryptionConsensusMessage {
					session: id.clone().into(),
					sub_session: access_key.clone().into(),
					message: message,
				})))?
			},
			ConsensusSessionAction::SendMessage(to, message) => {
				cluster.send(&to, Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(DecryptionConsensusMessage {
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
				data.decrypted_secret = Some(Err(Error::ConsensusUnreachable));
				completed.notify_all();
			},
			_ => (),
		}

		Ok(())
	}

	fn start_waiting_for_partial_decryption(self_node_id: &NodeId, session_id: SessionId, access_key: Secret, cluster: &Arc<Cluster>, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		if data.master.as_ref() != Some(self_node_id) {
			// if we are on the slave node, wait for partial decryption requests
			data.state = SessionState::WaitingForPartialDecryptionRequest;
			return Ok(());
		}

		// update state
		data.state = SessionState::WaitingForPartialDecryption;

		// send jobs to all selected nodes
		let consensus = data.consensus.as_mut().expect("consensus is created on initialization phase; partial decryption phase follows initialization; qed");
		consensus.activate()?;
		let mut confirmed_nodes = consensus.select_nodes()?.clone();

		// send requests
		for node in confirmed_nodes.iter().filter(|n| n != &self_node_id) {
			consensus.job_request_sent(node)?;
			cluster.send(node, Message::Decryption(DecryptionMessage::RequestPartialDecryption(RequestPartialDecryption {
				session: session_id.clone().into(),
				sub_session: access_key.clone().into(),
				is_shadow_decryption: data.is_shadow_decryption.expect("TODO"),
				nodes: confirmed_nodes.iter().cloned().map(Into::into).collect(),
			})))?;
		}

		// confirmation from this node, if this node is in consensus group
		if confirmed_nodes.remove(self_node_id) {
			let decryption_result = {
				let requestor = data.requestor.as_ref().expect("requestor public is filled during initialization; WaitingForPartialDecryption follows initialization; qed");
				let is_shadow_decryption = data.is_shadow_decryption.expect("is_shadow_decryption is filled during initialization; WaitingForPartialDecryption follows initialization; qed");
				do_partial_decryption(self_node_id, &requestor, is_shadow_decryption, &confirmed_nodes, &access_key, &encrypted_data)?
			};

			consensus.job_request_sent(self_node_id)?;
			consensus.job_response_received(self_node_id, decryption_result)?;
		}

		Ok(())
	}

	fn do_decryption(access_key: Secret, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		// decrypt the secret using shadow points
		let job_responses = data.consensus.as_ref().expect("TODO").job_responses()?;
		let joint_shadow_point = math::compute_joint_shadow_point(job_responses.values().map(|s| &s.shadow_point))?;
		let encrypted_point = encrypted_data.encrypted_point.as_ref().expect("checked at the beginning of the session; immutable; qed");
		let common_point = encrypted_data.common_point.as_ref().expect("checked at the beginning of the session; immutable; qed");
		let decrypted_secret = math::decrypt_with_joint_shadow(encrypted_data.threshold, &access_key, encrypted_point, &joint_shadow_point)?;
		let is_shadow_decryption = data.is_shadow_decryption.expect("is_shadow_decryption is filled during initialization; decryption follows initialization; qed");
		let (common_point, decrypt_shadows) = if is_shadow_decryption {
			(
				Some(math::make_common_shadow_point(encrypted_data.threshold, common_point.clone())?),
				Some(job_responses.values()
					.map(|s| s.decrypt_shadow.as_ref().expect("decrypt_shadow is filled during partial decryption; decryption follows partial decryption; qed").clone())
					.collect())
			)
		} else {
			(None, None)
		};
		data.decrypted_secret = Some(Ok(EncryptedDocumentKeyShadow {
			decrypted_secret: decrypted_secret,
			common_point: common_point,
			decrypt_shadows: decrypt_shadows,
		}));

		// switch to completed state
		data.state = SessionState::Finished;

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
				if SessionImpl::start_waiting_for_partial_decryption(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data).is_ok() {
					return;
				}
			}
		} else if !is_other_master {
			// disconnected from non-master node on non-master node
			// => this does not affect this session
			return;
		}
		// else: disconnecting from master node means failure

		warn!("{}: decryption session failed because {} connection has timeouted", self.node(), node);

		data.state = SessionState::Failed;
		data.decrypted_secret = Some(Err(Error::NodeDisconnected));
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
				if SessionImpl::start_waiting_for_partial_decryption(self.node(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data).is_ok() {
					return;
				}
			}
		}

		warn!("{}: decryption session failed with timeout", self.node());

		data.state = SessionState::Failed;
		data.decrypted_secret = Some(Err(Error::NodeDisconnected));
		self.completed.notify_all();
	}
}

impl Session for SessionImpl {
	fn wait(&self) -> Result<EncryptedDocumentKeyShadow, Error> {
		let mut data = self.data.lock();
		if !data.decrypted_secret.is_some() {
			self.completed.wait(&mut data);
		}

		data.decrypted_secret.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when decrypted_secret.is_some(); qed")
			.clone()
	}
}

impl DecryptionSessionId {
	/// Create new decryption session Id.
	pub fn new(session_id: SessionId, sub_session_id: Secret) -> Self {
		DecryptionSessionId {
			id: session_id,
			access_key: sub_session_id,
		}
	}
}

impl PartialOrd for DecryptionSessionId {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for DecryptionSessionId {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.id.cmp(&other.id) {
			Ordering::Equal => self.access_key.cmp(&other.access_key),
			r @ _ => r,
		}
	}
}

fn check_encrypted_data(self_node_id: &Public, encrypted_data: &DocumentKeyShare) -> Result<(), Error> {
	use key_server_cluster::generation_session::{check_cluster_nodes, check_threshold};

	// check that common_point and encrypted_point are already set
	if encrypted_data.common_point.is_none() || encrypted_data.encrypted_point.is_none() {
		return Err(Error::NotStartedSessionId);
	}

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)
}


fn do_partial_decryption(node: &NodeId, requestor_public: &Public, is_shadow_decryption: bool, participants: &BTreeSet<NodeId>, access_key: &Secret, encrypted_data: &DocumentKeyShare) -> Result<PartialDecryptionResult, Error> {
	let node_id_number = &encrypted_data.id_numbers[node];
	let node_secret_share = &encrypted_data.secret_share;
	let other_id_numbers = participants.iter()
		.filter(|id| *id != node)
		.map(|id| &encrypted_data.id_numbers[id]);
	let node_shadow = math::compute_node_shadow(node_secret_share, node_id_number, other_id_numbers)?;
	let decrypt_shadow = if is_shadow_decryption { Some(math::generate_random_scalar()?) } else { None };
	let common_point = encrypted_data.common_point.as_ref().expect("checked at the beginning of the session; immutable; qed");
	let (shadow_point, decrypt_shadow) = math::compute_node_shadow_point(access_key, common_point, &node_shadow, decrypt_shadow)?;
	Ok(PartialDecryptionResult {
		shadow_point: shadow_point,
		decrypt_shadow: match decrypt_shadow {
			None => None,
			Some(decrypt_shadow) => Some(encrypt(requestor_public, &DEFAULT_MAC, &**decrypt_shadow)?),
		},
	})
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::BTreeMap;
	use super::super::super::acl_storage::tests::DummyAclStorage;
	use ethkey::{self, Random, Generator, Public, Secret};
	use key_server_cluster::{NodeId, DocumentKeyShare, SessionId, Error, EncryptedDocumentKeyShadow};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::decryption_session::{SessionImpl, SessionParams, SessionState};
	use key_server_cluster::message::{self, Message, DecryptionMessage};
	use key_server_cluster::math;

	const SECRET_PLAIN: &'static str = "d2b57ae7619e070af0af6bc8c703c0cd27814c54d5d6a999cacac0da34ede279ca0d9216e85991029e54e2f0c92ee0bd30237725fa765cbdbfc4529489864c5f";

	fn prepare_decryption_sessions() -> (Vec<Arc<DummyCluster>>, Vec<Arc<DummyAclStorage>>, Vec<SessionImpl>) {
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
			id_numbers: id_numbers.clone().into_iter().collect(),
			secret_share: secret_shares[i].clone(),
			common_point: Some(common_point.clone()),
			encrypted_point: Some(encrypted_point.clone()),
		}).collect();
		let acl_storages: Vec<_> = (0..5).map(|_| Arc::new(DummyAclStorage::default())).collect();
		let clusters: Vec<_> = (0..5).map(|i| {
			let cluster = Arc::new(DummyCluster::new(id_numbers.iter().nth(i).clone().unwrap().0));
			for id_number in &id_numbers {
				cluster.add_node(id_number.0.clone());
			}
			cluster
		}).collect();
		let sessions: Vec<_> = (0..5).map(|i| SessionImpl::new(SessionParams {
			id: session_id.clone(),
			access_key: access_key.clone(),
			self_node_id: id_numbers.iter().nth(i).clone().unwrap().0,
			encrypted_data: encrypted_datas[i].clone(),
			acl_storage: acl_storages[i].clone(),
			cluster: clusters[i].clone()
		}).unwrap()).collect();

		(clusters, acl_storages, sessions)
	}

	fn do_messages_exchange(clusters: &[Arc<DummyCluster>], sessions: &[SessionImpl]) {
		do_messages_exchange_until(clusters, sessions, |_, _, _| false);
	}

	fn do_messages_exchange_until<F>(clusters: &[Arc<DummyCluster>], sessions: &[SessionImpl], mut cond: F) where F: FnMut(&NodeId, &NodeId, &Message) -> bool {
		while let Some((from, to, message)) = clusters.iter().filter_map(|c| c.take_message().map(|(to, msg)| (c.node(), to, msg))).next() {
			let session = &sessions[sessions.iter().position(|s| s.node() == &to).unwrap()];
			if cond(&from, &to, &message) {
				break;
			}

			match message {
				Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(message)) => session.on_consensus_message(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::RequestPartialDecryption(message)) => session.on_partial_decryption_requested(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::PartialDecryption(message)) => session.on_partial_decryption(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::DecryptionSessionCompleted(message)) => session.on_session_completed(from, &message).unwrap(),
				_ => panic!("unexpected"),
			}
		}
	}

	#[test]
	fn constructs_in_cluster_of_single_node() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(self_node_id, Random.generate().unwrap().secret().clone());
		match SessionImpl::new(SessionParams {
			id: SessionId::default(),
			access_key: Random.generate().unwrap().secret().clone(),
			self_node_id: self_node_id.clone(),
			encrypted_data: DocumentKeyShare {
				author: Public::default(),
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}) {
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
			id: SessionId::default(),
			access_key: Random.generate().unwrap().secret().clone(),
			self_node_id: self_node_id.clone(),
			encrypted_data: DocumentKeyShare {
				author: Public::default(),
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}) {
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
			id: SessionId::default(),
			access_key: Random.generate().unwrap().secret().clone(),
			self_node_id: self_node_id.clone(),
			encrypted_data: DocumentKeyShare {
				author: Public::default(),
				threshold: 2,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Some(Random.generate().unwrap().public().clone()),
				encrypted_point: Some(Random.generate().unwrap().public().clone()),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}) {
			Err(Error::InvalidThreshold) => (),
			_ => panic!("unexpected"),
		}
	}

	#[test]
	fn fails_to_initialize_when_already_initialized() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap(), ());
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_initialization_when_already_initialized() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap(), ());
		assert_eq!(sessions[0].on_consensus_message(sessions[1].node().clone(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
				}),
			}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_partial_decrypt_if_requested_by_slave() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_consensus_message(sessions[0].node().clone(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
				}),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[2].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			is_shadow_decryption: false,
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(4).collect(),
		}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_partial_decrypt_if_wrong_number_of_nodes_participating() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_consensus_message(sessions[0].node().clone(), &message::DecryptionConsensusMessage {
				session: SessionId::default().into(),
				sub_session: sessions[0].access_key().clone().into(),
				message: message::ConsensusMessage::InitializeConsensusSession(message::InitializeConsensusSession {
					requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
				}),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[0].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			is_shadow_decryption: false,
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(2).collect(),
		}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_accept_partial_decrypt_if_not_waiting() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].on_partial_decryption(sessions[1].node().clone(), &message::PartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			shadow_point: Random.generate().unwrap().public().clone().into(),
			decrypt_shadow: None,
		}).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_partial_decrypt_twice() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		let mut pd_from = None;
		let mut pd_msg = None;
		do_messages_exchange_until(&clusters, &sessions, |from, _, msg| match msg {
			&Message::Decryption(DecryptionMessage::PartialDecryption(ref msg)) => {
				pd_from = Some(from.clone());
				pd_msg = Some(msg.clone());
				true
			},
			_ => false,
		});

		assert_eq!(sessions[0].on_partial_decryption(pd_from.clone().unwrap(), &pd_msg.clone().unwrap()).unwrap(), ());
		assert_eq!(sessions[0].on_partial_decryption(pd_from.unwrap(), &pd_msg.unwrap()).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn decryption_fails_on_session_timeout() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert!(sessions[0].decrypted_secret().is_none());
		sessions[0].on_session_timeout();
		assert!(sessions[0].decrypted_secret().unwrap().unwrap_err() == Error::NodeDisconnected);
	}

	#[test]
	fn node_is_marked_rejected_when_timed_out_during_initialization_confirmation() {
		let (_, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		// 1 node disconnects => we still can recover secret
		sessions[0].on_node_timeout(sessions[1].node());
		assert!(sessions[0].data.lock().consensus.as_ref().unwrap().core().unwrap().rejected_nodes().contains(sessions[1].node()));
		assert!(sessions[0].data.lock().state == SessionState::EstablishingConsensus);

		// 2 node are disconnected => we can not recover secret
		sessions[0].on_node_timeout(sessions[2].node());
		assert!(sessions[0].data.lock().consensus.as_ref().unwrap().core().is_none());
		assert!(sessions[0].data.lock().state == SessionState::Failed);
	}

	#[test]
	fn session_does_not_fail_if_rejected_node_disconnects() {
		let (clusters, acl_storages, sessions) = prepare_decryption_sessions();
		let key_pair = Random.generate().unwrap();

		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		sessions[0].initialize(ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap(), false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == SessionState::WaitingForPartialDecryption);

		// 1st node disconnects => ignore this
		sessions[0].on_node_timeout(sessions[1].node());
		assert!(sessions[0].data.lock().state == SessionState::WaitingForPartialDecryption);
	}

	#[test]
	fn session_does_not_fail_if_requested_node_disconnects() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == SessionState::WaitingForPartialDecryption);

		// 1 node disconnects => we still can recover secret
		sessions[0].on_node_timeout(sessions[1].node());
		assert!(sessions[0].data.lock().state == SessionState::WaitingForPartialDecryption);

		// 2 node are disconnected => we can not recover secret
		sessions[0].on_node_timeout(sessions[2].node());
		assert!(sessions[0].data.lock().state == SessionState::Failed);
	}

	#[test]
	fn session_does_not_fail_if_node_with_shadow_point_disconnects() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == SessionState::WaitingForPartialDecryption
			&& sessions[0].data.lock().consensus.as_ref().unwrap().job_responses().unwrap().len() == 2);

		// disconnects from the node which has already sent us its own shadow point
		let disconnected = sessions[0].data.lock().
			consensus.as_ref().unwrap().job_responses().unwrap().keys()
			.filter(|n| *n != sessions[0].node())
			.cloned().nth(0).unwrap();
		sessions[0].on_node_timeout(&disconnected);
		assert!(sessions[0].data.lock().state == SessionState::WaitingForPartialDecryption);
	}

	#[test]
	fn session_restarts_if_confirmed_node_disconnects() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == SessionState::WaitingForPartialDecryption);

		// disconnects from the node which has already confirmed its participation
		let disconnected = sessions[0].data.lock().consensus.as_ref().unwrap().job_requests().unwrap().iter().cloned().nth(0).unwrap();
		sessions[0].on_node_timeout(&disconnected);
		assert!(sessions[0].data.lock().state == SessionState::WaitingForPartialDecryption);
		assert!(sessions[0].data.lock().consensus.as_ref().unwrap().core().unwrap().rejected_nodes().contains(&disconnected));
		assert!(!sessions[0].data.lock().consensus.as_ref().unwrap().job_requests().unwrap().contains(&disconnected));
	}

	#[test]
	fn session_does_not_fail_if_non_master_node_disconnects_from_non_master_node() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap(), false).unwrap();

		do_messages_exchange_until(&clusters, &sessions, |_, _, _| sessions[0].state() == SessionState::WaitingForPartialDecryption);

		// disconnects from the node which has already confirmed its participation
		sessions[1].on_node_timeout(sessions[2].node());
		assert!(sessions[0].data.lock().state == SessionState::WaitingForPartialDecryption);
		assert!(sessions[1].data.lock().state == SessionState::WaitingForPartialDecryptionRequest);
	}

	#[test]
	fn complete_dec_session() {
		let (clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature, false).unwrap();

		do_messages_exchange(&clusters, &sessions);

		// now check that:
		// 1) 5 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Finished).count(), 5);
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
		let (clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature, true).unwrap();

		do_messages_exchange(&clusters, &sessions);

		// now check that:
		// 1) 5 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Finished).count(), 5);
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
			.map(|c| Secret::from_slice(&decrypt(key_pair.secret(), &DEFAULT_MAC, &c).unwrap()).unwrap())
			.collect();
		let decrypted_secret = math::decrypt_with_shadow_coefficients(decrypted_secret.decrypted_secret, decrypted_secret.common_point.unwrap(), decrypt_shadows).unwrap();
		assert_eq!(decrypted_secret, SECRET_PLAIN.into());
	}

	#[test]
	fn failed_dec_session() {
		let (clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature, false).unwrap();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 2 of these nodes are disagree
		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		acl_storages[2].prohibit(key_pair.public().clone(), SessionId::default());

		let node3 = sessions[3].node().clone();
		do_messages_exchange_until(&clusters, &sessions, |from, _, _msg| from == &node3);

		// now check that:
		// 1) 3 of 5 sessions are in Failed state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Failed).count(), 3);
		// 2) 2 of 5 sessions are in WaitingForPartialDecryptionRequest state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::WaitingForPartialDecryptionRequest).count(), 2);
		// 3) 0 sessions have decrypted key value
		assert!(sessions.iter().all(|s| s.decrypted_secret().is_none() || s.decrypted_secret().unwrap().is_err()));
	}

	#[test]
	fn complete_dec_session_with_acl_check_failed_on_master() {
		let (clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 1 of these nodes (master) is disagree
		let key_pair = Random.generate().unwrap();
		acl_storages[0].prohibit(key_pair.public().clone(), SessionId::default());

		// now let's try to do a decryption
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature, false).unwrap();

		do_messages_exchange(&clusters, &sessions);

		// now check that:
		// 1) 4 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Finished).count(), 5);
		// 2) 1 session has decrypted key value
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));
		assert_eq!(sessions[0].decrypted_secret().unwrap().unwrap(), EncryptedDocumentKeyShadow {
			decrypted_secret: SECRET_PLAIN.into(),
			common_point: None,
			decrypt_shadows: None,
		});
	}

	#[test]
	fn decryption_session_works_over_network() {
		// TODO
	}
}
