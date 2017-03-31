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
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use ethkey::{self, Secret, Public, Signature};
use key_server_cluster::{Error, AclStorage, DocumentKeyShare, NodeId, SessionId};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::math;
use key_server_cluster::message::{Message, DecryptionMessage, InitializeDecryptionSession, ConfirmDecryptionInitialization,
	RequestPartialDecryption, PartialDecryption, DecryptionSessionError};

/// Decryption session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed. Returns distributely restored secret key.
	fn wait(&self) -> Result<Public, Error>;
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

#[derive(Debug)]
/// Mutable data of encryption (distributed key generation) session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,
	/// Public key of requestor.
	requestor: Option<Public>,

	// === Values, filled during session initialization ===
	/// Nodes, which have been requested for decryption initialization.
	requested_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with reject to initialization request.
	rejected_nodes: BTreeSet<NodeId>,
	/// Nodes, which have responded with confirm to initialization request.
	confirmed_nodes: BTreeSet<NodeId>,
	
	// === Values, filled during partial decryption ===
	/// Shadow points, received from nodes as a response to partial decryption request.
	shadow_points: BTreeMap<NodeId, Public>,

	/// === Values, filled during final decryption ===
	/// Decrypted secret
	decrypted_secret: Option<Result<Public, Error>>,
}

#[derive(Debug, Clone, PartialEq)]
/// Decryption session data.
pub enum SessionState {
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Master node waits for other nodes to confirm decryption.
	WaitingForInitializationConfirm,
	/// Waiting for partial decrypion request.
	WaitingForPartialDecryptionRequest,
	/// Waiting for partial decryption responses.
	WaitingForPartialDecryption,
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
				requested_nodes: BTreeSet::new(),
				rejected_nodes: BTreeSet::new(),
				confirmed_nodes: BTreeSet::new(),
				shadow_points: BTreeMap::new(),
				decrypted_secret: None,
			})
		})
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	#[cfg(test)]
	/// Get this session access key.
	pub fn access_key(&self) -> &Secret {
		&self.access_key
	}

	#[cfg(test)]
	/// Get current session state.
	pub fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	#[cfg(test)]
	/// Get decrypted secret
	pub fn decrypted_secret(&self) -> Option<Public> {
		self.data.lock().decrypted_secret.clone().and_then(|r| r.ok())
	}

	/// Initialize decryption session.
	pub fn initialize(&self, requestor_signature: Signature) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&requestor_signature, &self.id)?;

		// update state
		data.master = Some(self.node().clone());
		data.state = SessionState::WaitingForInitializationConfirm;
		data.requestor = Some(requestor_public.clone());
		data.requested_nodes.extend(self.encrypted_data.id_numbers.keys().cloned());

		// ..and finally check access on our's own
		let is_requestor_allowed_to_read = self.acl_storage.check(&requestor_public, &self.id).unwrap_or(false);
		process_initialization_response(&self.encrypted_data, &mut *data, self.node(), is_requestor_allowed_to_read)?;

		// check if we have enough nodes to decrypt data
		match data.state {
			// not enough nodes => pass initialization message to all other nodes
			SessionState::WaitingForInitializationConfirm => {
				for node in self.encrypted_data.id_numbers.keys().filter(|n| *n != self.node()) {
					self.cluster.send(node, Message::Decryption(DecryptionMessage::InitializeDecryptionSession(InitializeDecryptionSession {
							session: self.id.clone().into(),
							sub_session: self.access_key.clone().into(),
							requestor_signature: requestor_signature.clone().into(),
						})))?;
				}
			},
			// we can decrypt data on our own
			SessionState::WaitingForPartialDecryption => {
				data.confirmed_nodes.insert(self.node().clone());
				SessionImpl::start_waiting_for_partial_decryption(self.node().clone(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data)?;
				SessionImpl::do_decryption(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
				self.completed.notify_all();
			},
			// we can not decrypt data
			SessionState::Failed => (),
			// cannot reach other states
			_ => unreachable!("process_initialization_response can change state to WaitingForPartialDecryption or Failed; checked that we are in WaitingForInitializationConfirm state above; qed"),
		}

		Ok(())
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&self, sender: NodeId, message: &InitializeDecryptionSession) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// recover requestor signature
		let requestor_public = ethkey::recover(&message.requestor_signature, &self.id)?;

		// check access
		let is_requestor_allowed_to_read = self.acl_storage.check(&requestor_public, &self.id).unwrap_or(false);
		data.state = if is_requestor_allowed_to_read { SessionState::WaitingForPartialDecryptionRequest }
			else { SessionState::Failed };
		data.requestor = Some(requestor_public);

		// respond to master node
		data.master = Some(sender.clone());
		self.cluster.send(&sender, Message::Decryption(DecryptionMessage::ConfirmDecryptionInitialization(ConfirmDecryptionInitialization {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			is_confirmed: is_requestor_allowed_to_read,
		})))
	}

	/// When session initialization confirmation message is reeived.
	pub fn on_confirm_initialization(&self, sender: NodeId, message: &ConfirmDecryptionInitialization) -> Result<(), Error> {
		debug_assert!(self.id == *message.session);
		debug_assert!(self.access_key == *message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitializationConfirm {
			// if there were enough confirmations/rejections before this message
			// we have already moved to the next state
			return Ok(());
		}

		// update state
		process_initialization_response(&self.encrypted_data, &mut *data, &sender, message.is_confirmed)?;

		// check if we have enough nodes to decrypt data
		match data.state {
			// we do not yet have enough nodes for decryption
			SessionState::WaitingForInitializationConfirm => Ok(()),
			// we have enough nodes for decryption
			SessionState::WaitingForPartialDecryption =>
				SessionImpl::start_waiting_for_partial_decryption(self.node().clone(), self.id.clone(), self.access_key.clone(), &self.cluster, &self.encrypted_data, &mut *data),
			// we can not have enough nodes for decryption
			SessionState::Failed => Ok(()),
			// cannot reach other states
			_ => unreachable!("process_initialization_response can change state to WaitingForPartialDecryption or Failed; checked that we are in WaitingForInitializationConfirm state above; qed"),
		}
	}

	/// When partial decryption is requested.
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
		if data.state != SessionState::WaitingForPartialDecryptionRequest {
			return Err(Error::InvalidStateForRequest);
		}

		// calculate shadow point
		let shadow_point = {
			let requestor = data.requestor.as_ref().expect("requestor public is filled during initialization; WaitingForPartialDecryptionRequest follows initialization; qed");
			do_partial_decryption(self.node(), &requestor, &message.nodes.iter().cloned().map(Into::into).collect(), &self.access_key, &self.encrypted_data)?
		};
		self.cluster.send(&sender, Message::Decryption(DecryptionMessage::PartialDecryption(PartialDecryption {
			session: self.id.clone().into(),
			sub_session: self.access_key.clone().into(),
			shadow_point: shadow_point.into(),
		})))?;

		// update sate
		data.state = SessionState::Finished;

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
			return Err(Error::InvalidStateForRequest);
		}

		if !data.confirmed_nodes.remove(&sender) {
			return Err(Error::InvalidStateForRequest);
		}
		data.shadow_points.insert(sender, message.shadow_point.clone().into());

		// check if we have enough shadow points to decrypt the secret
		if data.shadow_points.len() != self.encrypted_data.threshold + 1 {
			return Ok(());
		}

		SessionImpl::do_decryption(self.access_key.clone(), &self.encrypted_data, &mut *data)?;
		self.completed.notify_all();

		Ok(())
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: NodeId, message: &DecryptionSessionError) {
		warn!("{}: decryption session error: {:?} from {}", self.node(), message, sender);
		let mut data = self.data.lock();
		data.state = SessionState::Failed;
		data.decrypted_secret = Some(Err(Error::Io(message.error.clone())));
		self.completed.notify_all();
	}

	/// When session timeout has occured.
	pub fn on_session_timeout(&self, _node: &NodeId) {
		warn!("{}: decryption session timeout", self.node());
		let mut data = self.data.lock();
		// TODO: check that node is a part of decryption process
		data.state = SessionState::Failed;
		data.decrypted_secret = Some(Err(Error::Io("session expired".into())));
		self.completed.notify_all();
	}

	fn start_waiting_for_partial_decryption(self_node_id: NodeId, session_id: SessionId, access_key: Secret, cluster: &Arc<Cluster>, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		let confirmed_nodes: BTreeSet<_> = data.confirmed_nodes.clone();
		for node in data.confirmed_nodes.iter().filter(|n| n != &&self_node_id) {
			cluster.send(node, Message::Decryption(DecryptionMessage::RequestPartialDecryption(RequestPartialDecryption {
				session: session_id.clone().into(),
				sub_session: access_key.clone().into(),
				nodes: confirmed_nodes.iter().cloned().map(Into::into).collect(),
			})))?;
		}

		assert!(data.confirmed_nodes.remove(&self_node_id));

		let shadow_point = {
			let requestor = data.requestor.as_ref().expect("requestor public is filled during initialization; WaitingForPartialDecryption follows initialization; qed");
			do_partial_decryption(&self_node_id, &requestor, &data.confirmed_nodes, &access_key, &encrypted_data)?
		};
		data.shadow_points.insert(self_node_id.clone(), shadow_point);

		Ok(())
	}

	fn do_decryption(access_key: Secret, encrypted_data: &DocumentKeyShare, data: &mut SessionData) -> Result<(), Error> {
		// decrypt the secret using shadow points
		let joint_shadow_point = math::compute_joint_shadow_point(data.shadow_points.values())?;
		let decrypted_secret = math::decrypt_with_joint_shadow(encrypted_data.threshold, &access_key, &encrypted_data.encrypted_point, &joint_shadow_point)?;
		data.decrypted_secret = Some(Ok(decrypted_secret));

		// switch to completed state
		data.state = SessionState::Finished;

		Ok(())
	}
}

impl Session for SessionImpl {
	fn wait(&self) -> Result<Public, Error> {
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
	use key_server_cluster::encryption_session::{check_cluster_nodes, check_threshold};

	let nodes = encrypted_data.id_numbers.keys().cloned().collect();
	check_cluster_nodes(self_node_id, &nodes)?;
	check_threshold(encrypted_data.threshold, &nodes)?;

	Ok(())
}

fn process_initialization_response(encrypted_data: &DocumentKeyShare, data: &mut SessionData, node: &NodeId, check_result: bool) -> Result<(), Error> {
	if !data.requested_nodes.remove(node) {
		return Err(Error::InvalidMessage);
	}

	match check_result {
		true => {
			data.confirmed_nodes.insert(node.clone());

			// check if we have enough nodes to do a decryption?
			if data.confirmed_nodes.len() == encrypted_data.threshold + 1 {
				data.state = SessionState::WaitingForPartialDecryption;
			}
		},
		false => {
			data.rejected_nodes.insert(node.clone());

			// check if we still can receive enough confirmations to do a decryption?
			if encrypted_data.id_numbers.len() - data.rejected_nodes.len() < encrypted_data.threshold + 1 {
				data.state = SessionState::Failed;
			}
		},
	}

	Ok(())
}

fn do_partial_decryption(node: &NodeId, _requestor_public: &Public, participants: &BTreeSet<NodeId>, access_key: &Secret, encrypted_data: &DocumentKeyShare) -> Result<Public, Error> {
	let node_id_number = &encrypted_data.id_numbers[node];
	let node_secret_share = &encrypted_data.secret_share;
	let other_id_numbers = participants.iter()
		.filter(|id| *id != node)
		.map(|id| &encrypted_data.id_numbers[id]);
	// TODO: commutative encryption using _requestor_public
	let node_shadow = math::compute_node_shadow(node_id_number, node_secret_share, other_id_numbers)?;
	math::compute_node_shadow_point(access_key, &encrypted_data.common_point, &node_shadow)
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::BTreeMap;
	use super::super::super::acl_storage::DummyAclStorage;
	use ethkey::{self, Random, Generator, Public, Secret};
	use key_server_cluster::{NodeId, DocumentKeyShare, SessionId, Error};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::decryption_session::{SessionImpl, SessionParams, SessionState};
	use key_server_cluster::message::{self, Message, DecryptionMessage};

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
			threshold: 3,
			id_numbers: id_numbers.clone().into_iter().collect(),
			secret_share: secret_shares[i].clone(),
			common_point: common_point.clone(),
			encrypted_point: encrypted_point.clone(),
		}).collect();
		let acl_storages: Vec<_> = (0..5).map(|_| Arc::new(DummyAclStorage::default())).collect();
		let clusters: Vec<_> = (0..5).map(|i| Arc::new(DummyCluster::new(id_numbers.iter().nth(i).clone().unwrap().0))).collect();
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
				Message::Decryption(DecryptionMessage::InitializeDecryptionSession(message)) => session.on_initialize_session(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::ConfirmDecryptionInitialization(message)) => session.on_confirm_initialization(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::RequestPartialDecryption(message)) => session.on_partial_decryption_requested(from, &message).unwrap(),
				Message::Decryption(DecryptionMessage::PartialDecryption(message)) => session.on_partial_decryption(from, &message).unwrap(),
				_ => panic!("unexpected"),
			}
		}
	}

	#[test]
	fn fails_to_construct_in_cluster_of_single_node() {
		let mut nodes = BTreeMap::new();
		let self_node_id = Random.generate().unwrap().public().clone();
		nodes.insert(self_node_id, Random.generate().unwrap().secret().clone());
		match SessionImpl::new(SessionParams {
			id: SessionId::default(),
			access_key: Random.generate().unwrap().secret().clone(),
			self_node_id: self_node_id.clone(),
			encrypted_data: DocumentKeyShare {
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Random.generate().unwrap().public().clone(),
				encrypted_point: Random.generate().unwrap().public().clone(),
			},
			acl_storage: Arc::new(DummyAclStorage::default()),
			cluster: Arc::new(DummyCluster::new(self_node_id.clone())),
		}) {
			Err(Error::InvalidNodesCount) => (),
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
				threshold: 0,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Random.generate().unwrap().public().clone(),
				encrypted_point: Random.generate().unwrap().public().clone(),
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
				threshold: 2,
				id_numbers: nodes,
				secret_share: Random.generate().unwrap().secret().clone(),
				common_point: Random.generate().unwrap().public().clone(),
				encrypted_point: Random.generate().unwrap().public().clone(),
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
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap()).unwrap(), ());
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap()).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_initialization_when_already_initialized() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap()).unwrap(), ());
		assert_eq!(sessions[0].on_initialize_session(sessions[1].node().clone(), &message::InitializeDecryptionSession {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
		}).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_partial_decrypt_if_not_waiting() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_initialize_session(sessions[0].node().clone(), &message::InitializeDecryptionSession {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[0].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(4).collect(),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[0].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(4).collect(),
		}).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_partial_decrypt_if_requested_by_slave() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_initialize_session(sessions[0].node().clone(), &message::InitializeDecryptionSession {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[2].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			nodes: sessions.iter().map(|s| s.node().clone().into()).take(4).collect(),
		}).unwrap_err(), Error::InvalidMessage);
	}

	#[test]
	fn fails_to_partial_decrypt_if_wrong_number_of_nodes_participating() {
		let (_, _, sessions) = prepare_decryption_sessions();
		assert_eq!(sessions[1].on_initialize_session(sessions[0].node().clone(), &message::InitializeDecryptionSession {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
			requestor_signature: ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap().into(),
		}).unwrap(), ());
		assert_eq!(sessions[1].on_partial_decryption_requested(sessions[0].node().clone(), &message::RequestPartialDecryption {
			session: SessionId::default().into(),
			sub_session: sessions[0].access_key().clone().into(),
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
		}).unwrap_err(), Error::InvalidStateForRequest);
	}

	#[test]
	fn fails_to_accept_partial_decrypt_twice() {
		let (clusters, _, sessions) = prepare_decryption_sessions();
		sessions[0].initialize(ethkey::sign(Random.generate().unwrap().secret(), &SessionId::default()).unwrap()).unwrap();

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
	fn complete_dec_session() {
		let (clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature).unwrap();

		do_messages_exchange(&clusters, &sessions);

		// now check that:
		// 1) 4 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Finished).count(), 4);
		// 2) 1 session is in WaitingForPartialDecryptionRequest state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::WaitingForPartialDecryptionRequest).count(), 1);
		// 3) 1 session has decrypted key value
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));
		assert_eq!(sessions[0].decrypted_secret(), Some(SECRET_PLAIN.into()));
	}

	#[test]
	fn failed_dec_session() {
		let (clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature).unwrap();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 2 of these nodes are disagree
		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		acl_storages[2].prohibit(key_pair.public().clone(), SessionId::default());

		do_messages_exchange(&clusters, &sessions);

		// now check that:
		// 1) 3 of 5 sessions are in Failed state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Failed).count(), 3);
		// 2) 2 of 5 sessions are in WaitingForPartialDecryptionRequest state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::WaitingForPartialDecryptionRequest).count(), 2);
		// 3) 0 sessions have decrypted key value
		assert!(sessions.iter().all(|s| s.decrypted_secret().is_none()));
	}

	#[test]
	fn decryption_session_works_over_network() {
		// TODO
	}
}
