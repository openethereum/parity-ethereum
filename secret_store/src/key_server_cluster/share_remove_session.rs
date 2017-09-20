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

use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};
use parking_lot::Mutex;
use ethkey::{Secret, Signature};
use key_server_cluster::{Error, NodeId, SessionMeta, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{ShareRemoveMessage, InitializeShareRemoveSession, ConfirmShareRemoveInitialization,
	ShareRemoveRequest, ShareRemoveConfirm, ShareRemoveError};

/// Share remove session API.
pub trait Session: Send + Sync + 'static {
}

/// Share remove session transport.
pub trait SessionTransport {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareRemoveMessage) -> Result<(), Error>;
}

/// Share remove session.
pub struct SessionImpl<T: SessionTransport> {
	/// Session core.
	core: SessionCore<T>,
	/// Session data.
	data: Mutex<SessionData>,
}

/// Immutable session data.
struct SessionCore<T: SessionTransport> {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Share add session id.
	pub sub_session: Secret,
	/// Session-level nonce.
	pub nonce: u64,
	/// Original key share.
	pub key_share: DocumentKeyShare,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
}

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Initialization confirmations to receive (all nodes set).
	pub init_confirmations_to_receive: BTreeSet<NodeId>,
	/// Remove confirmations to receive.
	pub remove_confirmations_to_receive: BTreeSet<NodeId>,
	/// Shares to remove.
	pub shares_to_remove: BTreeSet<NodeId>,
}

/// SessionImpl creation parameters
pub struct SessionParams<T: SessionTransport> {
	/// Session meta.
	pub meta: SessionMeta,
	/// Sub session identifier.
	pub sub_session: Secret,
	/// Session nonce.
	pub nonce: u64,
	/// Original key share.
	pub key_share: DocumentKeyShare,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
}

/// Share move session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// Waiting for initialization.
	WaitingForInitialization,
	/// Waiting for initialization confirmation.
	WaitingForInitializationConfirm,
	/// Waiting for remove confirmation.
	WaitingForRemoveConfirmation,
	/// Session is finished.
	Finished,
}

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new nested share addition session. Consensus is formed outside.
	pub fn new_nested(params: SessionParams<T>) -> Result<Self, Error> {
		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta,
				sub_session: params.sub_session,
				nonce: params.nonce,
				key_share: params.key_share,
				transport: params.transport,
				key_storage: params.key_storage,
			},
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				init_confirmations_to_receive: BTreeSet::new(),
				remove_confirmations_to_receive: BTreeSet::new(),
				shares_to_remove: BTreeSet::new(),
			}),
		})
	}

	/// Initialize share remove session on master node.
	pub fn initialize(&self, shares_to_remove: BTreeSet<NodeId>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		check_shares_to_remove(&self.core, &shares_to_remove)?;

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
println!("=== 1");
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		data.state = SessionState::WaitingForInitializationConfirm;
		data.shares_to_remove.extend(shares_to_remove.clone());
		data.remove_confirmations_to_receive.extend(shares_to_remove.clone());
		data.remove_confirmations_to_receive.remove(&self.core.meta.self_node_id);
		data.init_confirmations_to_receive.extend(self.core.key_share.id_numbers.keys().cloned());
		data.init_confirmations_to_receive.remove(&self.core.meta.self_node_id);

		// send initialization request to every node
		for node in &data.init_confirmations_to_receive {
			self.core.transport.send(node, ShareRemoveMessage::InitializeShareRemoveSession(InitializeShareRemoveSession {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
				shares_to_remove: shares_to_remove.iter().cloned().map(Into::into).collect(),
			}))?;
		}

		Ok(())
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareRemoveMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareRemoveMessage::InitializeShareRemoveSession(ref message) =>
				self.on_initialize_session(sender, message),
			&ShareRemoveMessage::ConfirmShareRemoveInitialization(ref message) =>
				self.on_confirm_initialization(sender, message),
			&ShareRemoveMessage::ShareRemoveRequest(ref message) =>
				self.on_share_remove_request(sender, message),
			&ShareRemoveMessage::ShareRemoveConfirm(ref message) =>
				self.on_share_remove_confirmation(sender, message),
			&ShareRemoveMessage::ShareRemoveError(ref message) =>
				self.on_session_error(sender, message),
		}
	}

	/// When initialization request is received.
	pub fn on_initialize_session(&self, sender: &NodeId, message: &InitializeShareRemoveSession) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message from master node only
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check shares_to_remove
		let shares_to_remove = message.shares_to_remove.iter().cloned().map(Into::into).collect();
		check_shares_to_remove(&self.core, &shares_to_remove)?;

		// update state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForInitialization {
println!("=== 2");
			return Err(Error::InvalidStateForRequest);
		}
		data.state = SessionState::WaitingForRemoveConfirmation;
		data.shares_to_remove.extend(shares_to_remove);
		let remove_confirmations_to_receive: Vec<_> = data.shares_to_remove.iter().cloned().collect();
		data.remove_confirmations_to_receive.extend(remove_confirmations_to_receive);
		data.remove_confirmations_to_receive.remove(&self.core.meta.self_node_id);

		// confirm initialization
		self.core.transport.send(sender, ShareRemoveMessage::ConfirmShareRemoveInitialization(ConfirmShareRemoveInitialization {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.sub_session.clone().into(),
			session_nonce: self.core.nonce,
		}))?;

		Ok(())
	}

	/// When session initialization confirmation message is received.
	pub fn on_confirm_initialization(&self, sender: &NodeId, message: &ConfirmShareRemoveInitialization) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message on master node only
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForInitializationConfirm {
println!("=== 3");
			return Err(Error::InvalidStateForRequest);
		}
		// do not expect double confirmations
		if !data.init_confirmations_to_receive.remove(sender) {
			return Err(Error::InvalidMessage);
		}
		// if not all init confirmations are received => return
		if !data.init_confirmations_to_receive.is_empty() {
			return Ok(());
		}

		// update state
		data.state = SessionState::WaitingForRemoveConfirmation;
		// send share remove requests
		for share_source in data.shares_to_remove.iter().filter(|n| **n != self.core.meta.self_node_id) {
			self.core.transport.send(share_source, ShareRemoveMessage::ShareRemoveRequest(ShareRemoveRequest {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
			}))?;
		}
		// TODO: check if session is completed

		Ok(())
	}

	/// When share remove request is received.
	pub fn on_share_remove_request(&self, sender: &NodeId, message: &ShareRemoveRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message from master node only
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForRemoveConfirmation {
println!("=== 4");
			return Err(Error::InvalidStateForRequest);
		}
		// only process if we are waiting for this request
		if !data.shares_to_remove.contains(&self.core.meta.self_node_id) {
			return Err(Error::InvalidMessage);
		}
		// send confirmation to all other nodes
		let all_nodes_set = self.core.key_share.id_numbers.keys().filter(|n| !data.shares_to_remove.contains(n)).collect::<Vec<_>>();
		for node in all_nodes_set.into_iter().filter(|n| **n != self.core.meta.self_node_id) {
			self.core.transport.send(&node, ShareRemoveMessage::ShareRemoveConfirm(ShareRemoveConfirm {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
			}))?;
		}
		// remove share
		Self::complete_session(&self.core, &mut *data)
	}

	/// When share is received from destination node.
	pub fn on_share_remove_confirmation(&self, sender: &NodeId, message: &ShareRemoveConfirm) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForRemoveConfirmation {
println!("=== 5: {:?}", data.state);
			return Err(Error::InvalidStateForRequest);
		}
		// find share source
		if !data.remove_confirmations_to_receive.remove(sender) {
			return Err(Error::InvalidMessage);
		}
		if data.remove_confirmations_to_receive.is_empty() {
			Self::complete_session(&self.core, &mut *data)?;
		}

		Ok(())
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &ShareRemoveError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: share remove session failed with error: {} from {}", self.core.meta.self_node_id, message.error, sender);

		data.state = SessionState::Finished;

		Ok(())
	}

	/// Complete session on this node.
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData) -> Result<(), Error> {
		// if we are 'removing' node => remove share from storage
		if data.shares_to_remove.contains(&core.meta.self_node_id) {
			return core.key_storage.remove(&core.meta.id)
				.map_err(|e| Error::KeyStorage(e.into()));
		}

		// else we need to update key_share.id_numbers.keys()
		let mut key_share = core.key_share.clone();
		for share_to_remove in &data.shares_to_remove {
			key_share.id_numbers.remove(share_to_remove);
		}
		// TODO: case when if all nodes are removed

		// ... and update key share in storage
		core.key_storage.update(core.meta.id.clone(), key_share)
			.map_err(|e| Error::KeyStorage(e.into()))
	}
}

impl<T> ClusterSession for SessionImpl<T> where T: SessionTransport {
	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		unimplemented!()
	}

	fn on_node_timeout(&self, _node_id: &NodeId) {
		unimplemented!()
	}
}

fn check_shares_to_remove<T: SessionTransport>(core: &SessionCore<T>, shares_to_remove: &BTreeSet<NodeId>) -> Result<(), Error> {
	// shares to remove must not be empty
	if shares_to_remove.is_empty() {
		return Err(Error::InvalidMessage);
	}

	// all shares_to_remove nodes must be old nodes of the session
	if shares_to_remove.iter().any(|n| !core.key_share.id_numbers.contains_key(n)) {
		return Err(Error::InvalidNodesConfiguration);
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, KeyPair, sign};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage, SessionMeta};
	use key_server_cluster::cluster::Cluster;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::MessageLoop as GenerationMessageLoop;
	use key_server_cluster::math;
	use key_server_cluster::message::{Message, ServersSetChangeMessage, ShareAddMessage};
	use key_server_cluster::servers_set_change_session::tests::generate_key;
	use key_server_cluster::share_change_session::ShareChangeTransport;
	use super::{SessionImpl, SessionParams, SessionTransport};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl<ShareChangeTransport>,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	impl MessageLoop {
		pub fn new(gml: GenerationMessageLoop, threshold: usize) -> Self {
			let key_id = gml.session_id.clone();
			let session_id = SessionId::default();
			let sub_session = Random.generate().unwrap().secret().clone();
			let mut nodes = BTreeMap::new();
			let master_node_id = gml.nodes.keys().cloned().nth(0).unwrap();
			let meta = SessionMeta {
				self_node_id: master_node_id.clone(),
				master_node_id: master_node_id.clone(),
				id: session_id.clone(),
				threshold: threshold,
			};
 
			for (n, nd) in &gml.nodes {
				let cluster = nd.cluster.clone();
				let key_storage = nd.key_storage.clone();
				let mut meta = meta.clone();
				meta.self_node_id = n.clone();
				let session = SessionImpl::new_nested(SessionParams {
					meta: meta,
					sub_session: sub_session.clone(),
					transport: ShareChangeTransport::new(session_id.clone(), 1, cluster.clone()),
					key_storage: nd.key_storage.clone(),
					nonce: 1,
					key_share: key_storage.get(&key_id).unwrap(),
				}).unwrap();
				nodes.insert(n.clone(), Node {
					cluster: cluster,
					key_storage: key_storage,
					session: session,
				});
			}

			MessageLoop {
				session_id: session_id,
				nodes: nodes,
				queue: Default::default(),
			}
		}

		pub fn run(&mut self) {
			while let Some((from, to, message)) = self.take_message() {
println!("=== {} -> {}: {}", from, to, message);
				self.process_message((from, to, message)).unwrap();
			}
		}

		pub fn take_message(&mut self) -> Option<(NodeId, NodeId, Message)> {
			self.nodes.values()
				.filter_map(|n| n.cluster.take_message().map(|m| (n.session.core.meta.self_node_id.clone(), m.0, m.1)))
				.nth(0)
				.or_else(|| self.queue.pop_front())
		}

		pub fn process_message(&mut self, msg: (NodeId, NodeId, Message)) -> Result<(), Error> {
			match {
				match msg.2 {
					Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareRemoveMessage(ref message)) =>
						self.nodes[&msg.1].session.process_message(&msg.0, &message.message),
					_ => unreachable!("only servers set change messages are expected"),
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
	fn node_removed_using_share_remove() {
		// initial 2-of-3 session
		let (t, n) = (1, 3);
		let gml = generate_key(t, n);
		let gml_nodes: BTreeSet<_> = gml.nodes.keys().cloned().collect();
		let key_id = gml.session_id.clone();
		let master = gml.nodes.keys().cloned().nth(0).unwrap();
		let nodes_to_remove: BTreeSet<_> = gml.nodes.keys().cloned().skip(1).take(1).collect();
		let joint_secret = math::compute_joint_secret(gml.nodes.values()
			.map(|nd| nd.key_storage.get(&key_id).unwrap().polynom1[0].clone())
			.collect::<Vec<_>>()
			.iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// add 1 node && move share
		let mut ml = MessageLoop::new(gml, t);
		let new_nodes_set: BTreeSet<_> = ml.nodes.keys().cloned().filter(|n| !gml_nodes.contains(n)).collect();
		ml.nodes[&master].session.initialize(nodes_to_remove.clone());
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..n-1 {
			for n2 in n1+1..n-1 {
				let node1 = ml.nodes.keys().nth(n1).unwrap();
				let node2 = ml.nodes.keys().nth(n2).unwrap();
				if nodes_to_remove.contains(node1) {
					assert!(ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}
				if nodes_to_remove.contains(node2) {
					assert!(ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).is_err());
					continue;
				}

				let share1 = ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).unwrap();
				let share2 = ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).unwrap();
				let id_number1 = share1.id_numbers[ml.nodes.keys().nth(n1).unwrap()].clone();
				let id_number2 = share1.id_numbers[ml.nodes.keys().nth(n2).unwrap()].clone();

				// now encrypt and decrypt data
				let (document_secret_decrypted, document_secret_decrypted_test) =
					math::tests::do_encryption_and_decryption(t,
						joint_key_pair.public(),
						&[id_number1, id_number2],
						&[share1.secret_share, share2.secret_share],
						Some(&joint_secret),
						document_secret_plain.clone());

				assert_eq!(document_secret_plain, document_secret_decrypted_test);
				assert_eq!(document_secret_plain, document_secret_decrypted);
			}
		}
	}
}
