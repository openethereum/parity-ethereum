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
use std::collections::BTreeSet;
use parking_lot::{Mutex, Condvar};
use ethkey::{Public, Signature};
use key_server_cluster::{Error, NodeId, SessionId, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{Message, ShareRemoveMessage, ShareRemoveConsensusMessage, ConsensusMessageWithServersSet,
	ShareRemoveRequest, ShareRemoveConfirm, ShareRemoveError, InitializeConsensusSessionWithServersSet,
	ConfirmConsensusInitialization};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::dummy_job::{DummyJob, DummyJobTransport};
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Share remove session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed.
	fn wait(&self) -> Result<(), Error>;
}

/// Share remove session transport.
pub trait SessionTransport: Clone + JobTransport<PartialJobRequest=ServersSetChangeAccessRequest, PartialJobResponse=bool> {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareRemoveMessage) -> Result<(), Error>;
}

/// Share remove session.
pub struct SessionImpl<T: SessionTransport> {
	/// Session core.
	core: SessionCore<T>,
	/// Session data.
	data: Mutex<SessionData<T>>,
}

/// Immutable session data.
struct SessionCore<T: SessionTransport> {
	/// Session metadata.
	pub meta: ShareChangeSessionMeta,
	/// Session-level nonce.
	pub nonce: u64,
	/// Original key share.
	pub key_share: DocumentKeyShare,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Administrator public key.
	pub admin_public: Option<Public>,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
}

/// Share remove consensus session type.
type ShareRemoveChangeConsensusSession<T> = ConsensusSession<ServersSetChangeAccessJob, T, DummyJob, DummyJobTransport>;

/// Mutable session data.
struct SessionData<T: SessionTransport> {
	/// Session state.
	pub state: SessionState,
	/// Consensus session.
	pub consensus_session: Option<ShareRemoveChangeConsensusSession<T>>,
	/// Shares to remove.
	pub shares_to_remove: Option<BTreeSet<NodeId>>,
	/// Remove confirmations to receive.
	pub remove_confirmations_to_receive: Option<BTreeSet<NodeId>>,
	/// Share remove change result.
	pub result: Option<Result<(), Error>>,
}

/// SessionImpl creation parameters
pub struct SessionParams<T: SessionTransport> {
	/// Session meta.
	pub meta: ShareChangeSessionMeta,
	/// Session nonce.
	pub nonce: u64,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Administrator public key.
	pub admin_public: Option<Public>,
}

/// Share move session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// State when consensus is establishing.
	ConsensusEstablishing,
	/// Waiting for remove confirmation.
	WaitingForRemoveConfirmation,
	/// Session is finished.
	Finished,
}

/// Isolated ShareRemove session transport.
#[derive(Clone)]
pub struct IsolatedSessionTransport {
	/// Key id.
	session: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new share remove session.
	pub fn new(params: SessionParams<T>) -> Result<Self, Error> {
		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				nonce: params.nonce,
				key_share: params.key_storage.get(&params.meta.id).map_err(|e| Error::KeyStorage(e.into()))?,
				transport: params.transport,
				key_storage: params.key_storage,
				admin_public: params.admin_public,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::ConsensusEstablishing,
				consensus_session: None,
				shares_to_remove: None,
				remove_confirmations_to_receive: None,
				result: None,
			}),
		})
	}

	/// Set pre-established consensus data.
	pub fn set_consensus_output(&self, shares_to_remove: BTreeSet<NodeId>) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		check_shares_to_remove(&self.core, &shares_to_remove)?;

		data.remove_confirmations_to_receive = Some(shares_to_remove.clone());
		data.shares_to_remove = Some(shares_to_remove);

		Ok(())
	}

	/// Initialize share remove session on master node.
	pub fn initialize(&self, shares_to_remove: Option<BTreeSet<NodeId>>, old_set_signature: Option<Signature>, new_set_signature: Option<Signature>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		let mut data = self.data.lock();
		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// if consensus is not yet established => start consensus session
		let is_consensus_pre_established = data.shares_to_remove.is_some();
		if !is_consensus_pre_established {
			// TODO: even if node was lost, it is still required for ShareRemove session to complete.
			// It is wrong - if node is not in all_nodes_set, it must be excluded from consensus.
			let shares_to_remove = shares_to_remove.ok_or(Error::InvalidMessage)?;
			check_shares_to_remove(&self.core, &shares_to_remove)?;

			let old_set_signature = old_set_signature.ok_or(Error::InvalidMessage)?;
			let new_set_signature = new_set_signature.ok_or(Error::InvalidMessage)?;
			let all_nodes_set: BTreeSet<_> = self.core.key_share.id_numbers.keys().cloned().collect();
			let new_nodes_set: BTreeSet<_> = all_nodes_set.iter().cloned().filter(|n| !shares_to_remove.contains(&n)).collect();
			let admin_public = self.core.admin_public.clone().ok_or(Error::InvalidMessage)?;

			let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
				meta: self.core.meta.clone().into_consensus_meta(all_nodes_set.len()),
				consensus_executor: ServersSetChangeAccessJob::new_on_master(admin_public,
					all_nodes_set.clone(),
					all_nodes_set.clone(),
					new_nodes_set,
					old_set_signature,
					new_set_signature),
				consensus_transport: self.core.transport.clone(),
			})?;
			consensus_session.initialize(all_nodes_set)?;
			data.consensus_session = Some(consensus_session);
			data.remove_confirmations_to_receive = Some(shares_to_remove.clone());
			data.shares_to_remove = Some(shares_to_remove);
			return Ok(());
		}

		// otherwise => start sending ShareRemove-specific messages
		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareRemoveMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareRemoveMessage::ShareRemoveConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&ShareRemoveMessage::ShareRemoveRequest(ref message) =>
				self.on_share_remove_request(sender, message),
			&ShareRemoveMessage::ShareRemoveConfirm(ref message) =>
				self.on_share_remove_confirmation(sender, message),
			&ShareRemoveMessage::ShareRemoveError(ref message) =>
				self.on_session_error(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ShareRemoveConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// start slave consensus session if needed
		let mut data = self.data.lock();
		if data.consensus_session.is_none() && sender == &self.core.meta.master_node_id {
			match &message.message {
				&ConsensusMessageWithServersSet::InitializeConsensusSession(ref message) => {
					let admin_public = self.core.admin_public.clone().ok_or(Error::InvalidMessage)?;
					let current_nodes_set = self.core.key_share.id_numbers.keys().cloned().collect();
					data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
						meta: self.core.meta.clone().into_consensus_meta(message.old_nodes_set.len()),
						consensus_executor: ServersSetChangeAccessJob::new_on_slave(admin_public, current_nodes_set),
						consensus_transport: self.core.transport.clone(),
					})?);
				},
				_ => return Err(Error::InvalidStateForRequest),
			}
		}

		let (is_establishing_consensus, is_consensus_established, shares_to_remove) = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			let is_establishing_consensus = consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
			let shares_to_remove = match &message.message {
				&ConsensusMessageWithServersSet::InitializeConsensusSession(ref message) => {
					consensus_session.on_consensus_partial_request(sender, ServersSetChangeAccessRequest::from(message))?;
					let shares_to_remove = message.old_nodes_set.difference(&message.new_nodes_set).cloned().map(Into::into).collect::<BTreeSet<_>>();
					check_shares_to_remove(&self.core, &shares_to_remove)?;
					Some(shares_to_remove)
				},
				&ConsensusMessageWithServersSet::ConfirmConsensusInitialization(ref message) => {
					consensus_session.on_consensus_partial_response(sender, message.is_confirmed)?;
					None
				},
			};

			(
				is_establishing_consensus,
				consensus_session.state() == ConsensusSessionState::ConsensusEstablished,
				shares_to_remove
			)
		};

		if let Some(shares_to_remove) = shares_to_remove {
			data.remove_confirmations_to_receive = Some(shares_to_remove.clone());
			data.shares_to_remove = Some(shares_to_remove);
		}
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// When share remove request is received.
	pub fn on_share_remove_request(&self, sender: &NodeId, message: &ShareRemoveRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message from master node only
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state == SessionState::ConsensusEstablishing && data.shares_to_remove.is_some() {
			data.state = SessionState::WaitingForRemoveConfirmation;
		} else if data.state != SessionState::WaitingForRemoveConfirmation {
			return Err(Error::InvalidStateForRequest);
		}
		// only process if we are waiting for this request
		{
			let shares_to_remove = data.shares_to_remove.as_ref()
				.expect("shares_to_remove is filled when consensus is established; we only process share move request after consensus is established; qed");
			if !shares_to_remove.contains(&self.core.meta.self_node_id) {
				return Err(Error::InvalidMessage);
			}
		}

		// remove share
		Self::complete_session(&self.core, &mut *data)
	}

	/// When share is received from destination node.
	pub fn on_share_remove_confirmation(&self, sender: &NodeId, message: &ShareRemoveConfirm) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state == SessionState::ConsensusEstablishing && data.shares_to_remove.is_some() {
			data.state = SessionState::WaitingForRemoveConfirmation;
		} else if data.state != SessionState::WaitingForRemoveConfirmation {
			return Err(Error::InvalidStateForRequest);
		}
		// find share source
		{
			let remove_confirmations_to_receive = data.remove_confirmations_to_receive.as_mut()
				.expect("remove_confirmations_to_receive is filled when consensus is established; we only process share move confirmations after consensus is established; qed");
			if !remove_confirmations_to_receive.remove(sender) {
				return Err(Error::InvalidMessage);
			}

			if !remove_confirmations_to_receive.is_empty() {
				return Ok(());
			}
		}

		Self::complete_session(&self.core, &mut *data)
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &ShareRemoveError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: share remove session failed with error: {} from {}", self.core.meta.self_node_id, message.error, sender);

		data.state = SessionState::Finished;

		Ok(())
	}

	/// Start sending ShareMove-specific messages, when consensus is established.
	fn on_consensus_established(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		data.state = SessionState::WaitingForRemoveConfirmation;

		// send share remove requests to every required node
		Self::disseminate_share_remove_requests(core, data)?;

		{
			let shares_to_remove = data.shares_to_remove.as_ref()
				.expect("shares_to_remove is filled when consensus is established; on_consensus_established is called after consensus is established; qed");
			if !shares_to_remove.contains(&core.meta.self_node_id) {
				// remember remove confirmations to receive
				data.remove_confirmations_to_receive = Some(shares_to_remove.iter().cloned().collect());
				return Ok(());
			}
		}

		// complete session if share is lost
		Self::complete_session(core, data)
	}

	/// Disseminate share remove requests.
	fn disseminate_share_remove_requests(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		let shares_to_remove = data.shares_to_remove.as_ref()
			.expect("shares_to_remove is filled when consensus is established; disseminate_share_remove_requests is called after consensus is established; qed");
		for node in shares_to_remove.iter().filter(|n| **n != core.meta.self_node_id) {
			core.transport.send(node, ShareRemoveMessage::ShareRemoveRequest(ShareRemoveRequest {
				session: core.meta.id.clone().into(),
				session_nonce: core.nonce,
			}))?;
		}

		Ok(())
	}

	/// Complete session on this node.
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		data.state = SessionState::Finished;

		// if we are 'removing' node => remove share from storage
		let shares_to_remove = data.shares_to_remove.as_ref()
			.expect("shares_to_remove is filled when consensus is established; complete_session is called after consensus is established; qed");
		if shares_to_remove.contains(&core.meta.self_node_id) {
			// send confirmation to all other nodes
			let new_nodes_set = core.key_share.id_numbers.keys().filter(|n| !shares_to_remove.contains(n)).collect::<Vec<_>>();
			for node in new_nodes_set.into_iter().filter(|n| **n != core.meta.self_node_id) {
				core.transport.send(&node, ShareRemoveMessage::ShareRemoveConfirm(ShareRemoveConfirm {
					session: core.meta.id.clone().into(),
					session_nonce: core.nonce,
				}))?;
			}

			return core.key_storage.remove(&core.meta.id)
				.map_err(|e| Error::KeyStorage(e.into()));
		}

		// else we need to update key_share.id_numbers.keys()
		let mut key_share = core.key_share.clone();
		for share_to_remove in shares_to_remove {
			key_share.id_numbers.remove(share_to_remove);
		}

		// ... and update key share in storage
		core.key_storage.update(core.meta.id.clone(), key_share)
			.map_err(|e| Error::KeyStorage(e.into()))
	}
}

impl<T> Session for SessionImpl<T> where T: SessionTransport + Send + Sync + 'static {
	fn wait(&self) -> Result<(), Error> {
		let mut data = self.data.lock();
		if !data.result.is_some() {
			self.core.completed.wait(&mut data);
		}

		data.result.clone()
			.expect("checked above or waited for completed; completed is only signaled when result.is_some(); qed")
	}
}

impl<T> ClusterSession for SessionImpl<T> where T: SessionTransport {
	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		warn!("{}: share remove session failed with timeout", self.core.meta.self_node_id);

		data.state = SessionState::Finished;
		data.result = Some(Err(Error::NodeDisconnected));
		self.core.completed.notify_all();
	}

	fn on_node_timeout(&self, node: &NodeId) {
		let mut data = self.data.lock();

		warn!("{}: share remove session failed because {} connection has timeouted", self.core.meta.self_node_id, node);

		data.state = SessionState::Finished;
		data.result = Some(Err(Error::NodeDisconnected));
		self.core.completed.notify_all();
	}
}

impl IsolatedSessionTransport {
	pub fn new(session_id: SessionId, nonce: u64, cluster: Arc<Cluster>) -> Self {
		IsolatedSessionTransport {
			session: session_id,
			nonce: nonce,
			cluster: cluster,
		}
	}
}

impl JobTransport for IsolatedSessionTransport {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;

	fn send_partial_request(&self, node: &NodeId, request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareRemove(ShareRemoveMessage::ShareRemoveConsensusMessage(ShareRemoveConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersSet::InitializeConsensusSession(InitializeConsensusSessionWithServersSet {
				old_nodes_set: request.old_servers_set.into_iter().map(Into::into).collect(),
				new_nodes_set: request.new_servers_set.into_iter().map(Into::into).collect(),
				old_set_signature: request.old_set_signature.into(),
				new_set_signature: request.new_set_signature.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareRemove(ShareRemoveMessage::ShareRemoveConsensusMessage(ShareRemoveConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersSet::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			}),
		})))
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn send(&self, node: &NodeId, message: ShareRemoveMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareRemove(message))
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

	// do not allow removing more shares than possible
	let nodes_left = core.key_share.id_numbers.len() - shares_to_remove.len();
	if core.key_share.threshold + 1 > nodes_left {
		return Err(Error::InvalidNodesConfiguration);
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, Signature, KeyPair, sign};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage};
	use key_server_cluster::cluster::Cluster;
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::generation_session::tests::{Node as GenerationNode, generate_nodes_ids};
	use key_server_cluster::math;
	use key_server_cluster::message::Message;
	use key_server_cluster::servers_set_change_session::tests::generate_key;
	use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
	use key_server_cluster::admin_sessions::ShareChangeSessionMeta;
	use key_server_cluster::admin_sessions::share_add_session::tests::check_secret_is_preserved;
	use super::{SessionImpl, SessionParams, IsolatedSessionTransport};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl<IsolatedSessionTransport>,
	}

	struct MessageLoop {
		pub admin_key_pair: KeyPair,
		pub original_key_pair: KeyPair,
		pub old_nodes_set: BTreeSet<NodeId>,
		pub new_nodes_set: BTreeSet<NodeId>,
		pub old_set_signature: Signature,
		pub new_set_signature: Signature,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	fn create_session(mut meta: ShareChangeSessionMeta, admin_public: Public, self_node_id: NodeId, cluster: Arc<Cluster>, key_storage: Arc<KeyStorage>) -> SessionImpl<IsolatedSessionTransport> {
		let session_id = meta.id.clone();
		meta.self_node_id = self_node_id;
		SessionImpl::new(SessionParams {
			meta: meta.clone(),
			transport: IsolatedSessionTransport::new(session_id, 1, cluster),
			key_storage: key_storage,
			admin_public: Some(admin_public),
			nonce: 1,
		}).unwrap()
	}

	fn create_node(meta: ShareChangeSessionMeta, admin_public: Public, node: GenerationNode) -> Node {
		Node {
			cluster: node.cluster.clone(),
			key_storage: node.key_storage.clone(),
			session: create_session(meta, admin_public, node.session.node().clone(), node.cluster, node.key_storage),
		}
	}

	impl MessageLoop {
		pub fn new(t: usize, master_node_id: NodeId, old_nodes_set: BTreeSet<NodeId>, shares_to_remove: BTreeSet<NodeId>) -> Self {
			// generate admin key pair
			let admin_key_pair = Random.generate().unwrap();
			let admin_public = admin_key_pair.public().clone();

			// run initial generation session
			let gml = generate_key(t, old_nodes_set.clone());
			let original_secret = math::compute_joint_secret(gml.nodes.values()
				.map(|nd| nd.key_storage.get(&SessionId::default()).unwrap().polynom1[0].clone())
				.collect::<Vec<_>>()
				.iter()).unwrap();
			let original_key_pair = KeyPair::from_secret(original_secret).unwrap();

			// prepare sessions on all nodes
			let meta = ShareChangeSessionMeta {
				id: SessionId::default(),
				self_node_id: NodeId::default(),
				master_node_id: master_node_id,
			};
			let new_nodes_set: BTreeSet<_> = old_nodes_set.iter()
				.filter(|n| !shares_to_remove.contains(n))
				.cloned()
				.collect();
			let nodes = gml.nodes.into_iter().map(|gn| create_node(meta.clone(), admin_public.clone(), gn.1));
			let nodes = nodes.map(|n| (n.session.core.meta.self_node_id.clone(), n)).collect();

			let old_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&old_nodes_set)).unwrap();
			let new_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&new_nodes_set)).unwrap();
			MessageLoop {
				admin_key_pair: admin_key_pair,
				original_key_pair: original_key_pair,
				old_nodes_set: old_nodes_set.clone(),
				new_nodes_set: new_nodes_set.clone(),
				old_set_signature: old_set_signature,
				new_set_signature: new_set_signature,
				nodes: nodes,
				queue: Default::default(),
			}
		}

		pub fn run(&mut self) {
			while let Some((from, to, message)) = self.take_message() {
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
			match { match msg.2 {
				Message::ShareRemove(ref message) =>
					self.nodes[&msg.1].session.process_message(&msg.0, message),
				_ => unreachable!("only servers set change messages are expected"),
			} } {
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
	fn remove_session_fails_if_no_nodes_are_removed() {
		let (t, n) = (1, 3);
		let old_nodes_set = generate_nodes_ids(n);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let nodes_to_remove = BTreeSet::new();
		let ml = MessageLoop::new(t, master_node_id.clone(), old_nodes_set, nodes_to_remove.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(nodes_to_remove.clone()),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())), Err(Error::InvalidMessage));
	}

	#[test]
	fn remove_session_fails_if_foreign_nodes_are_removed() {
		let (t, n) = (1, 3);
		let old_nodes_set = generate_nodes_ids(n);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let nodes_to_remove: BTreeSet<_> = vec![math::generate_random_point().unwrap()].into_iter().collect();
		let ml = MessageLoop::new(t, master_node_id.clone(), old_nodes_set, nodes_to_remove.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(nodes_to_remove.clone()),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())), Err(Error::InvalidNodesConfiguration));
	}

	#[test]
	fn remove_session_fails_if_too_many_nodes_are_removed() {
		let (t, n) = (1, 3);
		let old_nodes_set = generate_nodes_ids(n);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let nodes_to_remove: BTreeSet<_> = old_nodes_set.iter().cloned().take(2).collect();
		let ml = MessageLoop::new(t, master_node_id.clone(), old_nodes_set, nodes_to_remove.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(nodes_to_remove.clone()),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())), Err(Error::InvalidNodesConfiguration));
	}

	#[test]
	fn nodes_removed_using_share_remove_from_master_node() {
		let t = 1;
		let test_cases = vec![(3, 1), (5, 3)];
		for (n, nodes_to_remove) in test_cases {
			// generate key && prepare ShareMove sessions
			let old_nodes_set = generate_nodes_ids(n);
			let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
			let nodes_to_remove: BTreeSet<_> = old_nodes_set.iter().cloned().take(nodes_to_remove).collect();
			let mut ml = MessageLoop::new(t, master_node_id.clone(), old_nodes_set, nodes_to_remove.clone());

			// initialize session on master node && run to completion
			ml.nodes[&master_node_id].session.initialize(Some(nodes_to_remove.clone()),
				Some(ml.old_set_signature.clone()),
				Some(ml.new_set_signature.clone())).unwrap();
			ml.run();

			// check that session has completed on all nodes
			assert!(ml.nodes.values().all(|n| n.session.is_finished()));
			
			// check that secret is still the same as before adding the share
			check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
				.filter(|&(k, _)| !nodes_to_remove.contains(k))
				.map(|(k, v)| (k.clone(), v.key_storage.clone()))
				.collect());
		}
	}

	#[test]
	fn nodes_removed_using_share_remove_from_non_master_node() {
		let t = 1;
		let test_cases = vec![(3, 1), (5, 3)];
		for (n, nodes_to_remove) in test_cases {
			// generate key && prepare ShareMove sessions
			let old_nodes_set = generate_nodes_ids(n);
			let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
			let nodes_to_remove: BTreeSet<_> = old_nodes_set.iter().cloned().skip(1).take(nodes_to_remove).collect();
			let mut ml = MessageLoop::new(t, master_node_id.clone(), old_nodes_set, nodes_to_remove.clone());

			// initialize session on master node && run to completion
			ml.nodes[&master_node_id].session.initialize(Some(nodes_to_remove.clone()),
				Some(ml.old_set_signature.clone()),
				Some(ml.new_set_signature.clone())).unwrap();
			ml.run();

			// check that session has completed on all nodes
			assert!(ml.nodes.values().all(|n| n.session.is_finished()));
			
			// check that secret is still the same as before adding the share
			check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
				.filter(|&(k, _)| !nodes_to_remove.contains(k))
				.map(|(k, v)| (k.clone(), v.key_storage.clone()))
				.collect());
		}
	}
}
