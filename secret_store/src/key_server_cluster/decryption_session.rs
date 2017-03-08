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
use std::sync::Arc;
use parking_lot::Mutex;
use ethkey::{self, Secret, Public, Signature};
use key_server_cluster::{Error, AclStorage, EncryptedData, NodeId, SessionId};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::math;
use key_server_cluster::message::{Message, InitializeDecryptionSession, ConfirmDecryptionInitialization,
	RequestPartialDecryption, PartialDecryption};

/// Distributed decryption session.
/// Based on "ECDKG: A Distributed Key Generation Protocol Based on Elliptic Curve Discrete Logarithm" paper:
/// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.124.4128&rep=rep1&type=pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for decrypting the secret) requests all other nodes to decrypt the secret
/// 2) ACL check: all nodes which have received the request are querying ACL-contract to check if requestor has access to the document
/// 3) partial decryption: every node which has succussfully checked access for the requestor do a partial decryption
/// 4) decryption: master node receives all partial decryptions of the secret and restores the secret
pub struct Session {
	/// Encryption session id.
	id: SessionId,
	/// Decryption session access key.
	access_key: Secret,
	/// Public identifier of this node.
	self_node_id: NodeId,
	/// Encrypted data.
	encrypted_data: EncryptedData,
	/// ACL storate to check access to the resource.
	acl_storage: Arc<AclStorage>,
	/// Cluster which allows this node to send messages to other nodes in the cluster.
	cluster: Arc<Cluster>,
	/// Mutable session data.
	data: Mutex<SessionData>,
}

#[derive(Debug)]
/// Mutable data of encryption (distributed key generation) session.
struct SessionData {
	/// Current state of the session.
	state: SessionState,

	// === Values, filled when session initialization just starts ===
	/// Reference to the node, which has started this session.
	master: Option<NodeId>,

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
	decrypted_secret: Option<Public>,
}

#[derive(Debug)]
struct NodeData {
	/// Node-generated shadow point.
	shadow_point: Option<Public>,
}

#[derive(Debug, Clone, PartialEq)]
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

impl Session {
	/// Create new decryption session.
	pub fn new(id: SessionId, access_key: Secret, self_node_id: Public, encrypted_data: EncryptedData, acl_storage: Arc<AclStorage>, cluster: Arc<Cluster>) -> Self {
		Session {
			id: id,
			access_key: access_key,
			self_node_id: self_node_id,
			encrypted_data: encrypted_data,
			acl_storage: acl_storage,
			cluster: cluster,
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				master: None,
				requested_nodes: BTreeSet::new(),
				rejected_nodes: BTreeSet::new(),
				confirmed_nodes: BTreeSet::new(),
				shadow_points: BTreeMap::new(),
				decrypted_secret: None,
			})
		}
	}

	/// Get this node Id.
	pub fn node(&self) -> &NodeId {
		&self.self_node_id
	}

	/// Get current session state.
	pub fn state(&self) -> SessionState {
		self.data.lock().state.clone()
	}

	/// Get decrypted secret
	pub fn decrypted_secret(&self) -> Option<Public> {
		self.data.lock().decrypted_secret.clone()
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
		data.requested_nodes.extend(self.encrypted_data.id_numbers.keys().cloned());

		// ..and finally check access on our's own
		let is_requestor_allowed_to_read = self.acl_storage.check(&requestor_public, &self.id).unwrap_or(false);
		process_initialization_response(&self.encrypted_data, &mut *data, self.node(), is_requestor_allowed_to_read)?;

		// if we have checked that

		// check if we have enough nodes to decrypt data
		match data.state {
			// not enough nodes => pass initialization message to all other nodes
			SessionState::WaitingForInitializationConfirm => {
				for node in self.encrypted_data.id_numbers.keys().filter(|n| *n != self.node()) {
					self.cluster.send(node, Message::InitializeDecryptionSession(InitializeDecryptionSession {
							session: self.id.clone(),
							sub_session: self.access_key.clone(),
							requestor_signature: requestor_signature.clone(),
						}))?;
				}
			},
			// we can decrypt data on our own
			SessionState::WaitingForPartialDecryption => unimplemented!(),
			// we can not decrypt data
			SessionState::Failed => unimplemented!(),
			// cannot reach other states
			_ => unreachable!("process_initialization_response can change state to WaitingForPartialDecryption or Failed; checked that we are in WaitingForInitializationConfirm state above; qed"),
		}

		Ok(())
	}

	/// When session initialization message is received.
	pub fn on_initialize_session(&self, sender: NodeId, message: InitializeDecryptionSession) -> Result<(), Error> {
		debug_assert!(self.id == message.session);
		debug_assert!(self.access_key == message.sub_session);
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

		// respond to master node
		data.master = Some(sender.clone());
		self.cluster.send(&sender, Message::ConfirmDecryptionInitialization(ConfirmDecryptionInitialization {
			session: self.id.clone(),
			sub_session: self.access_key.clone(),
			is_confirmed: is_requestor_allowed_to_read,
		}))
	}

	/// When session initialization confirmation message is reeived.
	pub fn on_confirm_initialization(&self, sender: NodeId, message: ConfirmDecryptionInitialization) -> Result<(), Error> {
		debug_assert!(self.id == message.session);
		debug_assert!(self.access_key == message.sub_session);
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
			SessionState::WaitingForPartialDecryption => {
				let confirmed_nodes: BTreeSet<_> = data.confirmed_nodes.clone();
				for node in data.confirmed_nodes.iter().filter(|n| n != &self.node()) {
					self.cluster.send(node, Message::RequestPartialDecryption(RequestPartialDecryption {
						session: self.id.clone(),
						sub_session: self.access_key.clone(),
						nodes: confirmed_nodes.clone(),
					}))?;
				}

				assert!(data.confirmed_nodes.remove(self.node()));

				let shadow_point = do_partial_decryption(self.node(), &data.confirmed_nodes, &self.access_key, &self.encrypted_data)?;
				data.shadow_points.insert(self.node().clone(), shadow_point);

				Ok(())
			},
			// we can not have enough nodes for decryption
			SessionState::Failed => Ok(()),
			// cannot reach other states
			_ => unreachable!("process_initialization_response can change state to WaitingForPartialDecryption or Failed; checked that we are in WaitingForInitializationConfirm state above; qed"),
		}
	}

	/// When partial decryption is requested.
	pub fn on_partial_decryption_requested(&self, sender: NodeId, message: RequestPartialDecryption) -> Result<(), Error> {
		debug_assert!(self.id == message.session);
		debug_assert!(self.access_key == message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.master != Some(sender) {
			return Err(Error::InvalidMessage);
		}
		if data.state != SessionState::WaitingForPartialDecryptionRequest {
			return Err(Error::InvalidStateForRequest);
		}

		// calculate shadow point
		let shadow_point = do_partial_decryption(self.node(), &message.nodes, &self.access_key, &self.encrypted_data)?;
		self.cluster.send(&sender, Message::PartialDecryption(PartialDecryption {
			session: self.id.clone(),
			sub_session: self.access_key.clone(),
			shadow_point: shadow_point,
		}))?;

		// update sate
		data.state = SessionState::Finished;

		Ok(())
	}

	/// When partial decryption is received.
	pub fn on_partial_decryption(&self, sender: NodeId, message: PartialDecryption) -> Result<(), Error> {
		debug_assert!(self.id == message.session);
		debug_assert!(self.access_key == message.sub_session);
		debug_assert!(&sender != self.node());

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForPartialDecryption {
			return Err(Error::InvalidStateForRequest);
		}

		if !data.confirmed_nodes.remove(&sender) {
			return Err(Error::InvalidStateForRequest);
		}
		data.shadow_points.insert(sender, message.shadow_point);

		// check if we have enough shadow points to decrypt the secret
		if data.shadow_points.len() != self.encrypted_data.threshold + 1 {
			return Ok(());
		}

		// decrypt the secret using shadow points
		let joint_shadow_point = math::compute_joint_shadow_point(data.shadow_points.values())?;
		let decrypted_secret = math::decrypt_with_joint_shadow(&self.access_key, &self.encrypted_data.encrypted_point, &joint_shadow_point)?;
		data.decrypted_secret = Some(decrypted_secret);
		data.state = SessionState::Finished;

		Ok(())
	}
}

fn process_initialization_response(encrypted_data: &EncryptedData, data: &mut SessionData, node: &NodeId, check_result: bool) -> Result<(), Error> {
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

fn do_partial_decryption(node: &NodeId, participants: &BTreeSet<NodeId>, access_key: &Secret, encrypted_data: &EncryptedData) -> Result<Public, Error> {
	let node_id_number = &encrypted_data.id_numbers[node];
	let node_secret_share = &encrypted_data.secret_share;
	let other_id_numbers = participants.iter()
		.filter(|id| *id != node)
		.map(|id| &encrypted_data.id_numbers[id]);
	let node_shadow = math::compute_node_shadow(node_id_number, node_secret_share, other_id_numbers)?;
	math::compute_node_shadow_point(access_key, &encrypted_data.common_point, &node_shadow)
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use std::collections::BTreeMap;
	use super::super::super::acl_storage::DummyAclStorage;
	use ethkey::{self, Random, Generator, Public, Secret};
	use key_server_cluster::{NodeId, EncryptedData, SessionId};
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::decryption_session::{Session, SessionState};
	use key_server_cluster::message::Message;

	fn prepare_decryption_sessions() -> (BTreeMap<NodeId, Secret>, Vec<Arc<DummyCluster>>, Vec<Arc<DummyAclStorage>>, Vec<Session>) {
		// prepare encrypted data + cluster configuration for scheme 4-of-5
		let session_id = SessionId::default();
		let access_key = Random.generate().unwrap().secret().clone();
		let secret_shares = vec![
			Secret::from_str("d286f976206b5a8f9e8edaa4b6fb2ba9b0991e3b0abc594d760a31a7c1d12377").unwrap(),
			Secret::from_str("6799f4d07694034822e2f1c9252b51c1481267ffc4a099945821b6b62b7f552a").unwrap(),
			Secret::from_str("cdb9e94cac754f6b35cf74da55286bd68a51ab818780e2acf7c6c4d38f29ee52").unwrap(),
			Secret::from_str("432c77dec10cfb73a2f3616af33eb776d56b0305413721bb6dd664418cc4a15f").unwrap(),
			Secret::from_str("cb12c2739ae40976583c633b2cc67efff6228ea3b39b3285cc748156944a97e2").unwrap(),
		];
		let mut id_numbers: BTreeMap<NodeId, Secret> = BTreeMap::new();
		id_numbers.insert("b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".into(),
			Secret::from_str("906d4a7c4995c2aad0bf09465c412cf7c5876341091f341fee290a59dee66bd9").unwrap());
		id_numbers.insert("1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb".into(),
			Secret::from_str("5903b3263804b5be6accaec93ae9dd0d3010c929d138a775d6edc1c069f52710").unwrap());
		id_numbers.insert("99e82b163b062d55a64085bacfd407bb55f194ba5fb7a1af9c34b84435455520f1372e0e650a4f91aed0058cb823f62146ccb5599c8d13372c300dea866b69fc".into(),
			Secret::from_str("f6cb9f0dab4cc8582d0a67252cdff78220de54ef74afa97bb446333728f45633").unwrap());
		id_numbers.insert("7e05df9dd077ec21ed4bc45c9fe9e0a43d65fa4be540630de615ced5e95cf5c3003035eb713317237d7667feeeb64335525158f5f7411f67aca9645169ea554c".into(),
			Secret::from_str("64c5b7b2d8aa305f430392e808ffab209c5832fe1f69ed5cc536d1e1d7b29bd1").unwrap());
		id_numbers.insert("321977760d1d8e15b047a309e4c7fe6f355c10bb5a06c68472b676926427f69f229024fa2692c10da167d14cdc77eb95d0fce68af0a0f704f0d3db36baa83bb2".into(),
			Secret::from_str("dce41867d9d7fa6f21e5998c08c129ea4837023d0e51e22904309a43107c8831").unwrap());
		let common_point: Public = "633f7c443e4bd0ec55fe55e787ae5c7da6064c18de3220375711684a1a1571c2af81fd43da74c86890bc76ff1386dbaeaaa306333cb1d16bb269bf15c2b30af9".into();
		let encrypted_point: Public = "3262349e215ca29834587997b5932afddefcb85928aa2e3a302f6f7a27ecf3375c51acff5e3b13bb949ef3eecca71abc6b82d26687393eabca55019fd2d5758f".into();
		let encrypted_datas: Vec<_> = (0..5).map(|i| EncryptedData {
			threshold: 3,
			id_numbers: id_numbers.clone(),
			secret_share: secret_shares[i].clone(),
			common_point: common_point.clone(),
			encrypted_point: encrypted_point.clone(),
		}).collect();
		let acl_storages: Vec<_> = (0..5).map(|_| Arc::new(DummyAclStorage::default())).collect();
		let clusters: Vec<_> = (0..5).map(|i| Arc::new(DummyCluster::new(id_numbers.keys().nth(i).cloned().unwrap()))).collect();
		let sessions: Vec<_> = (0..5).map(|i| Session::new(session_id.clone(),
			access_key.clone(),
			id_numbers.keys().nth(i).cloned().unwrap(),
			encrypted_datas[i].clone(),
			acl_storages[i].clone(),
			clusters[i].clone())).collect();

		(id_numbers, clusters, acl_storages, sessions)
	}

	fn do_messages_exchange(id_numbers: &BTreeMap<NodeId, Secret>, clusters: &[Arc<DummyCluster>], sessions: &[Session]) {
		while let Some((from, to, message)) = clusters.iter().filter_map(|c| c.take_message().map(|(to, msg)| (c.node(), to, msg))).next() {
			let session = &sessions[id_numbers.keys().position(|k| k == &to).unwrap()];
			match message {
				Message::InitializeDecryptionSession(message) => session.on_initialize_session(from, message).unwrap(),
				Message::ConfirmDecryptionInitialization(message) => session.on_confirm_initialization(from, message).unwrap(),
				Message::RequestPartialDecryption(message) => session.on_partial_decryption_requested(from, message).unwrap(),
				Message::PartialDecryption(message) => session.on_partial_decryption(from, message).unwrap(),
				_ => panic!("unexpected"),
			}
		}
	}

	#[test]
	fn complete_dec_session() {
		let (id_numbers, clusters, _, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature).unwrap();

		do_messages_exchange(&id_numbers, &clusters, &sessions);

		// now check that:
		// 1) 4 of 5 sessions are in Finished state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Finished).count(), 4);
		// 2) 1 session is in WaitingForPartialDecryptionRequest state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::WaitingForPartialDecryptionRequest).count(), 1);
		// 3) 1 session has decrypted key value
		assert!(sessions[0].decrypted_secret().is_some());
		assert!(sessions.iter().skip(1).all(|s| s.decrypted_secret().is_none()));
	}

	#[test]
	fn failed_dec_session() {
		let (id_numbers, clusters, acl_storages, sessions) = prepare_decryption_sessions();

		// now let's try to do a decryption
		let key_pair = Random.generate().unwrap();
		let signature = ethkey::sign(key_pair.secret(), &SessionId::default()).unwrap();
		sessions[0].initialize(signature).unwrap();

		// we need 4 out of 5 nodes to agree to do a decryption
		// let's say that 2 of these nodes are disagree
		acl_storages[1].prohibit(key_pair.public().clone(), SessionId::default());
		acl_storages[2].prohibit(key_pair.public().clone(), SessionId::default());

		do_messages_exchange(&id_numbers, &clusters, &sessions);

		// now check that:
		// 1) 3 of 5 sessions are in Failed state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::Failed).count(), 3);
		// 2) 2 of 5 sessions are in WaitingForPartialDecryptionRequest state
		assert_eq!(sessions.iter().filter(|s| s.state() == SessionState::WaitingForPartialDecryptionRequest).count(), 2);
		// 3) 0 sessions have decrypted key value
		assert!(sessions.iter().all(|s| s.decrypted_secret().is_none()));
	}
}
