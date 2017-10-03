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
use parking_lot::{Mutex, Condvar};
use ethkey::{Public, Secret, Signature};
use key_server_cluster::{Error, NodeId, SessionId, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{Message, ShareMoveMessage, ShareMoveConsensusMessage,
	ShareMoveRequest, ShareMove, ShareMoveConfirm, ShareMoveError, ConsensusMessageWithServersMap,
	InitializeConsensusSessionWithServersMap, ConfirmConsensusInitialization};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::dummy_job::{DummyJob, DummyJobTransport};
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Share move session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed.
	fn wait(&self) -> Result<(), Error>;
}

/// Share move session transport.
pub trait SessionTransport: Clone + JobTransport<PartialJobRequest=ServersSetChangeAccessRequest, PartialJobResponse=bool> {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareMoveMessage) -> Result<(), Error>;
	/// Set share destinations.
	fn set_shares_to_move_reversed(&mut self, shares_to_move_reversed: BTreeMap<NodeId, NodeId>);
}

/// Share move session.
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
	/// Original key share (for old nodes only).
	pub key_share: Option<DocumentKeyShare>,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Administrator public key.
	pub admin_public: Option<Public>,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
}

/// Share move consensus session type.
type ShareMoveChangeConsensusSession<T> = ConsensusSession<ServersSetChangeAccessJob, T, DummyJob, DummyJobTransport>;

/// Mutable session data.
struct SessionData<T: SessionTransport> {
	/// Session state.
	pub state: SessionState,
	/// Consensus session.
	pub consensus_session: Option<ShareMoveChangeConsensusSession<T>>,
	/// Shares to move. Keys = new nodes, Values = old nodes.
	pub shares_to_move_reversed: Option<BTreeMap<NodeId, NodeId>>,
	/// Reversed shares to move. Keys = old nodes, Values = new nodes.
	pub shares_to_move: Option<BTreeMap<NodeId, NodeId>>,
	/// Move confirmations to receive.
	pub move_confirmations_to_receive: Option<BTreeSet<NodeId>>,
	/// Received key share (filled on destination nodes only).
	pub received_key_share: Option<DocumentKeyShare>,
	/// Share move change result.
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
	/// Waiting for move confirmation.
	WaitingForMoveConfirmation,
	/// Session is completed.
	Finished,
}

/// Isolated ShareMove session transport.
#[derive(Clone)]
pub struct IsolatedSessionTransport {
	/// Key id.
	session: SessionId,
	/// Session-level nonce.
	nonce: u64,
	/// Shares to move between. Keys = new nodes, values = old nodes.
	shares_to_move_reversed: Option<BTreeMap<NodeId, NodeId>>,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new share move session.
	pub fn new(params: SessionParams<T>) -> Result<Self, Error> {
		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				nonce: params.nonce,
				key_share: params.key_storage.get(&params.meta.id).ok(), // ignore error, it will be checked later
				transport: params.transport,
				key_storage: params.key_storage,
				admin_public: params.admin_public,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::ConsensusEstablishing,
				consensus_session: None,
				shares_to_move_reversed: None,
				shares_to_move: None,
				move_confirmations_to_receive: None,
				received_key_share: None,
				result: None,
			}),
		})
	}

	/// Set pre-established consensus data.
	pub fn set_consensus_output(&self, shares_to_move_reversed: BTreeMap<NodeId, NodeId>) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		let old_id_numbers = self.core.key_share.as_ref().map(|ks| &ks.id_numbers);
		check_shares_to_move(&self.core.meta.self_node_id, &shares_to_move_reversed, old_id_numbers)?;

		data.move_confirmations_to_receive = Some(shares_to_move_reversed.keys().cloned().collect());
		data.shares_to_move = Some(shares_to_move_reversed.iter().map(|(k, v)| (v.clone(), k.clone())).collect());
		data.shares_to_move_reversed = Some(shares_to_move_reversed);

		Ok(())
	}

	/// Initialize share add session on master node.
	pub fn initialize(&self, shares_to_move_reversed: Option<BTreeMap<NodeId, NodeId>>, old_set_signature: Option<Signature>, new_set_signature: Option<Signature>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// if consensus is not yet established => start consensus session
		let is_consensus_pre_established = data.shares_to_move.is_some();
		if !is_consensus_pre_established {
			let shares_to_move_reversed = shares_to_move_reversed.ok_or(Error::InvalidMessage)?;
			let key_share = self.core.key_share.as_ref().ok_or(Error::KeyStorage("key share is not found on master node".into()))?;
			check_shares_to_move(&self.core.meta.self_node_id, &shares_to_move_reversed, Some(&key_share.id_numbers))?;

			let old_set_signature = old_set_signature.ok_or(Error::InvalidMessage)?;
			let new_set_signature = new_set_signature.ok_or(Error::InvalidMessage)?;
			let admin_public = self.core.admin_public.clone().ok_or(Error::InvalidMessage)?;
			let old_nodes_set: BTreeSet<_> = key_share.id_numbers.keys().cloned().collect();
			let mut all_nodes_set = old_nodes_set.clone();
			let mut new_nodes_set = all_nodes_set.clone();
			for (target, source) in &shares_to_move_reversed {
				new_nodes_set.remove(source);
				new_nodes_set.insert(target.clone());
				all_nodes_set.insert(target.clone());
			}
			let mut consensus_transport = self.core.transport.clone();
			consensus_transport.set_shares_to_move_reversed(shares_to_move_reversed.clone());

			let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
				meta: self.core.meta.clone().into_consensus_meta(all_nodes_set.len())?,
				consensus_executor: ServersSetChangeAccessJob::new_on_master(admin_public,
					old_nodes_set.clone(),
					old_nodes_set.clone(),
					new_nodes_set,
					old_set_signature,
					new_set_signature),
				consensus_transport: consensus_transport,
			})?;
			consensus_session.initialize(all_nodes_set)?;
			data.consensus_session = Some(consensus_session);
			data.move_confirmations_to_receive = Some(shares_to_move_reversed.keys().cloned().collect());
			data.shares_to_move = Some(shares_to_move_reversed.iter().map(|(k, v)| (v.clone(), k.clone())).collect());
			data.shares_to_move_reversed = Some(shares_to_move_reversed);
			return Ok(());
		}

		// otherwise => start sending ShareMove-specific messages
		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareMoveMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareMoveMessage::ShareMoveConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&ShareMoveMessage::ShareMoveRequest(ref message) =>
				self.on_share_move_request(sender, message),
			&ShareMoveMessage::ShareMove(ref message) =>
				self.on_share_move(sender, message),
			&ShareMoveMessage::ShareMoveConfirm(ref message) =>
				self.on_share_move_confirmation(sender, message),
			&ShareMoveMessage::ShareMoveError(ref message) =>
				self.on_session_error(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ShareMoveConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// start slave consensus session if needed
		let mut data = self.data.lock();
		if data.consensus_session.is_none() && sender == &self.core.meta.master_node_id {
			match &message.message {
				&ConsensusMessageWithServersMap::InitializeConsensusSession(ref message) => {
					let admin_public = self.core.admin_public.clone().ok_or(Error::InvalidMessage)?;
					let current_nodes_set = self.core.key_share.as_ref()
						.map(|ks| ks.id_numbers.keys().cloned().collect())
						.unwrap_or_else(|| message.old_nodes_set.clone().into_iter().map(Into::into).collect());
					let all_nodes_set_len = message.new_nodes_set.keys().chain(message.old_nodes_set.iter()).collect::<BTreeSet<_>>().len();
					data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
						meta: self.core.meta.clone().into_consensus_meta(all_nodes_set_len)?,
						consensus_executor: ServersSetChangeAccessJob::new_on_slave(admin_public, current_nodes_set),
						consensus_transport: self.core.transport.clone(),
					})?);
				},
				_ => return Err(Error::InvalidStateForRequest),
			}
		}

		let (is_establishing_consensus, is_consensus_established, shares_to_move_reversed) = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			let is_establishing_consensus = consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
			let shares_to_move_reversed = match &message.message {
				&ConsensusMessageWithServersMap::InitializeConsensusSession(ref message) => {
					consensus_session.on_consensus_partial_request(sender, ServersSetChangeAccessRequest::from(message))?;
					let shares_to_move_reversed = message.new_nodes_set.iter()
						.filter(|&(old, new)| old != new)
						.map(|(old, new)| (old.clone().into(), new.clone().into()))
						.collect::<BTreeMap<NodeId, NodeId>>();
					check_shares_to_move(&self.core.meta.self_node_id, &shares_to_move_reversed, self.core.key_share.as_ref().map(|ks| &ks.id_numbers))?;
					Some(shares_to_move_reversed)
				},
				&ConsensusMessageWithServersMap::ConfirmConsensusInitialization(ref message) => {
					consensus_session.on_consensus_partial_response(sender, message.is_confirmed)?;
					None
				},
			};

			(
				is_establishing_consensus,
				consensus_session.state() == ConsensusSessionState::ConsensusEstablished,
				shares_to_move_reversed
			)
		};

		if let Some(shares_to_move_reversed) = shares_to_move_reversed {
			data.move_confirmations_to_receive = Some(shares_to_move_reversed.keys().cloned().collect());
			data.shares_to_move = Some(shares_to_move_reversed.iter().map(|(k, v)| (v.clone(), k.clone())).collect());
			data.shares_to_move_reversed = Some(shares_to_move_reversed);
		}
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// When share move request is received.
	pub fn on_share_move_request(&self, sender: &NodeId, message: &ShareMoveRequest) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message from master node only
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state == SessionState::ConsensusEstablishing && data.shares_to_move.is_some() {
			data.state = SessionState::WaitingForMoveConfirmation;
		} else if data.state != SessionState::WaitingForMoveConfirmation {
			return Err(Error::InvalidStateForRequest);
		}

		// move share
		{
			let shares_to_move = data.shares_to_move.as_ref()
				.expect("shares_to_move are filled during consensus establishing; share move requests are processed after this; qed");
			if let Some(share_destination) = shares_to_move.get(&self.core.meta.self_node_id) {
				Self::move_share(&self.core, share_destination)?;
			} else {
				return Err(Error::InvalidMessage);
			}
		}

		// and complete session
		Self::complete_session(&self.core, &mut *data)
	}

	/// When moving share is received.
	pub fn on_share_move(&self, sender: &NodeId, message: &ShareMove) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state == SessionState::ConsensusEstablishing && data.shares_to_move.is_some() {
			data.state = SessionState::WaitingForMoveConfirmation;
		} else if data.state != SessionState::WaitingForMoveConfirmation {
			return Err(Error::InvalidStateForRequest);
		}

		// check that we are expecting this share
		if data.shares_to_move_reversed.as_ref()
			.expect("shares_to_move are filled during consensus establishing; share moves are processed after this; qed")
			.get(&self.core.meta.self_node_id) != Some(sender) {
			return Err(Error::InvalidMessage);
		}

		// update state
		let is_last_confirmation = {
			let move_confirmations_to_receive = data.move_confirmations_to_receive.as_mut()
				.expect("move_confirmations_to_receive are filled during consensus establishing; share moves are processed after this; qed");
				move_confirmations_to_receive.remove(&self.core.meta.self_node_id);
				move_confirmations_to_receive.is_empty()
		};
		data.received_key_share = Some(DocumentKeyShare {
			author: message.author.clone().into(),
			threshold: message.threshold,
			id_numbers: message.id_numbers.iter().map(|(k, v)| (k.clone().into(), v.clone().into())).collect(),
			polynom1: message.polynom1.iter().cloned().map(Into::into).collect(),
			secret_share: message.secret_share.clone().into(),
			common_point: message.common_point.clone().map(Into::into),
			encrypted_point: message.encrypted_point.clone().map(Into::into),
		});

		// send confirmation to all other nodes
		{
			let shares_to_move = data.shares_to_move.as_ref()
				.expect("shares_to_move are filled during consensus establishing; share moves are processed after this; qed");
			let new_nodes_set: BTreeSet<_> = shares_to_move.values().cloned()
				.chain(message.id_numbers.keys().filter(|n| !shares_to_move.contains_key(n)).cloned().map(Into::into))
				.collect();

			for node in new_nodes_set.into_iter().filter(|n| n != &self.core.meta.self_node_id) {
				self.core.transport.send(&node, ShareMoveMessage::ShareMoveConfirm(ShareMoveConfirm {
					session: self.core.meta.id.clone().into(),
					session_nonce: self.core.nonce,
				}))?;
			}
		}

		// complete session if this was last share
		if is_last_confirmation {
			Self::complete_session(&self.core, &mut *data)?;
		}

		Ok(())
	}

	/// When share is received from destination node.
	pub fn on_share_move_confirmation(&self, sender: &NodeId, message: &ShareMoveConfirm) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state == SessionState::ConsensusEstablishing && data.shares_to_move.is_some() {
			data.state = SessionState::WaitingForMoveConfirmation;
		} else if data.state != SessionState::WaitingForMoveConfirmation {
			return Err(Error::InvalidStateForRequest);
		}

		// find share source
		{
			let mut move_confirmations_to_receive = data.move_confirmations_to_receive.as_mut()
				.expect("move_confirmations_to_receive are filled during consensus establishing; move confirmations are processed after this; qed");
			if !move_confirmations_to_receive.remove(sender) {
				return Err(Error::InvalidMessage);
			}
			
			if !move_confirmations_to_receive.is_empty() {
				return Ok(());
			}
		}

		Self::complete_session(&self.core, &mut *data)
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &ShareMoveError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: share move session failed with error: {} from {}", self.core.meta.self_node_id, message.error, sender);

		data.state = SessionState::Finished;

		Ok(())
	}

	/// Start sending ShareMove-specific messages, when consensus is established.
	fn on_consensus_established(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		data.state = SessionState::WaitingForMoveConfirmation;

		// send share move requests to every required node
		Self::disseminate_share_move_requests(core, data)?;

		{
			let shares_to_move = data.shares_to_move.as_ref()
				.expect("shares_to_move are filled during consensus establishing; this method is called after consensus established; qed");
			if let Some(share_destination) = shares_to_move.get(&core.meta.self_node_id) {
				// move share
				Self::move_share(core, share_destination)?;
			} else {
				// remember move confirmations to receive
				data.move_confirmations_to_receive = Some(shares_to_move.values().cloned().collect());
				return Ok(());
			}
		}

		// complete session if share is lost
		Self::complete_session(core, data)
	}

	/// Disseminate share move requests.
	fn disseminate_share_move_requests(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		let shares_to_move = data.shares_to_move.as_ref()
			.expect("shares_to_move are filled during consensus establishing; this method is called after consensus established; qed");
		for share_source in shares_to_move.keys().filter(|n| **n != core.meta.self_node_id) {
			core.transport.send(share_source, ShareMoveMessage::ShareMoveRequest(ShareMoveRequest {
				session: core.meta.id.clone().into(),
				session_nonce: core.nonce,
			}))?;
		}

		Ok(())
	}

	/// Send share move message.
	fn move_share(core: &SessionCore<T>, share_destination: &NodeId) -> Result<(), Error> {
		let key_share = core.key_share.as_ref()
			.expect("move_share is called on nodes from shares_to_move.values(); all 'values' nodes have shares; qed");
		core.transport.send(share_destination, ShareMoveMessage::ShareMove(ShareMove {
			session: core.meta.id.clone().into(),
			session_nonce: core.nonce,
			author: key_share.author.clone().into(),
			threshold: key_share.threshold,
			id_numbers: key_share.id_numbers.iter().map(|(k, v)| (k.clone().into(), v.clone().into())).collect(),
			polynom1: key_share.polynom1.iter().cloned().map(Into::into).collect(),
			secret_share: key_share.secret_share.clone().into(),
			common_point: key_share.common_point.clone().map(Into::into),
			encrypted_point: key_share.encrypted_point.clone().map(Into::into),
		}))
	}

	/// Complete session on this node.
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		data.state = SessionState::Finished;

		// if we are source node => remove share from storage
		let shares_to_move = data.shares_to_move.as_ref()
			.expect("shares_to_move are filled during consensus establishing; this method is called after consensus established; qed");
		if shares_to_move.contains_key(&core.meta.self_node_id) {
			return core.key_storage.remove(&core.meta.id)
				.map_err(|e| Error::KeyStorage(e.into()));
		}

		// else we need to update key_share.id_numbers.keys()
		let is_old_node = data.received_key_share.is_none();
		let mut key_share = data.received_key_share.take()
			.unwrap_or_else(|| core.key_share.as_ref()
				.expect("on target nodes received_key_share is non-empty; on old nodes key_share is not empty; qed")
				.clone());
		for (source_node, target_node) in shares_to_move {
			let id_number = key_share.id_numbers.remove(source_node)
				.expect("source_node is old node; there's entry in id_numbers for each old node; qed");
			key_share.id_numbers.insert(target_node.clone(), id_number);
		}

		// ... and update key share in storage
		if is_old_node {
			core.key_storage.update(core.meta.id.clone(), key_share)
		} else {
			core.key_storage.insert(core.meta.id.clone(), key_share)
		}.map_err(|e| Error::KeyStorage(e.into()))
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

		warn!("{}: share move session failed with timeout", self.core.meta.self_node_id);

		data.state = SessionState::Finished;
		data.result = Some(Err(Error::NodeDisconnected));
		self.core.completed.notify_all();
	}

	fn on_node_timeout(&self, node: &NodeId) {
		let mut data = self.data.lock();

		warn!("{}: share move session failed because {} connection has timeouted", self.core.meta.self_node_id, node);

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
			shares_to_move_reversed: None,
		}
	}
}

impl JobTransport for IsolatedSessionTransport {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;

	fn send_partial_request(&self, node: &NodeId, request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		let shares_to_move_reversed = self.shares_to_move_reversed.as_ref()
			.expect("partial requests are sent from master node only; on master node shares_to_move_reversed are filled during creation; qed");
		self.cluster.send(node, Message::ShareMove(ShareMoveMessage::ShareMoveConsensusMessage(ShareMoveConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersMap::InitializeConsensusSession(InitializeConsensusSessionWithServersMap {
				old_nodes_set: request.old_servers_set.into_iter().map(Into::into).collect(),
				new_nodes_set: request.new_servers_set.into_iter().map(|n| (n.into(),
					shares_to_move_reversed.get(&n).cloned().unwrap_or_else(|| n.clone()).into())).collect(),
				old_set_signature: request.old_set_signature.into(),
				new_set_signature: request.new_set_signature.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareMove(ShareMoveMessage::ShareMoveConsensusMessage(ShareMoveConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersMap::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			}),
		})))
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn set_shares_to_move_reversed(&mut self, shares_to_move_reversed: BTreeMap<NodeId, NodeId>) {
		self.shares_to_move_reversed = Some(shares_to_move_reversed);
	}

	fn send(&self, node: &NodeId, message: ShareMoveMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareMove(message))
	}
}

fn check_shares_to_move(self_node_id: &NodeId, shares_to_move_reversed: &BTreeMap<NodeId, NodeId>, id_numbers: Option<&BTreeMap<NodeId, Secret>>) -> Result<(), Error> {
	// shares to move must not be empty
	if shares_to_move_reversed.is_empty() {
		return Err(Error::InvalidMessage);
	}

	if let Some(id_numbers) = id_numbers {
		// all values in share_to_move_reversed must be old nodes of the session
		if shares_to_move_reversed.values().any(|n| !id_numbers.contains_key(n)) {
			return Err(Error::InvalidNodesConfiguration);
		}
		// all keys in share_to_move_reversed must be new nodes for the session
		if shares_to_move_reversed.keys().any(|n| id_numbers.contains_key(n)) {
			return Err(Error::InvalidNodesConfiguration);
		}
	} else {
		// this node must NOT in values of share_to_move_reversed
		if shares_to_move_reversed.values().any(|n| n == self_node_id) {
			return Err(Error::InvalidMessage);
		}
		// this node must be in keys of share_to_move_reversed
		if !shares_to_move_reversed.contains_key(self_node_id) {
			return Err(Error::InvalidMessage);
		}
	}

	// all values of the shares_to_move must be distinct
	if shares_to_move_reversed.values().collect::<BTreeSet<_>>().len() != shares_to_move_reversed.len() {
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
		pub fn new(t: usize, master_node_id: NodeId, old_nodes_set: BTreeSet<NodeId>, shares_to_move: BTreeMap<NodeId, NodeId>) -> Self {
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
				.filter(|n| !shares_to_move.values().any(|n2| *n == n2))
				.cloned()
				.chain(shares_to_move.keys().cloned())
				.collect();
			let new_nodes = new_nodes_set.iter()
				.filter(|n| !old_nodes_set.contains(&n))
				.map(|new_node_id| {
					let new_node_cluster = Arc::new(DummyCluster::new(new_node_id.clone()));
					let new_node_key_storage = Arc::new(DummyKeyStorage::default());
					let new_node_session = create_session(meta.clone(), admin_public.clone(), new_node_id.clone(), new_node_cluster.clone(), new_node_key_storage.clone());
					Node {
						cluster: new_node_cluster,
						key_storage: new_node_key_storage,
						session: new_node_session,
					}
				});
			let old_nodes = gml.nodes.into_iter().map(|gn| create_node(meta.clone(), admin_public.clone(), gn.1));
			let nodes = old_nodes.chain(new_nodes).map(|n| (n.session.core.meta.self_node_id.clone(), n)).collect();

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
				Message::ShareMove(ref message) =>
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
	fn nodes_moved_using_share_move_from_master_node() {
		let test_cases = vec![(3, 1), (3, 3)];
		for (n, nodes_to_add) in test_cases {
			// generate key && prepare ShareAdd sessions
			let old_nodes_set = generate_nodes_ids(n);
			let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
			let nodes_to_add = generate_nodes_ids(nodes_to_add);
			let mut shares_to_move = BTreeMap::new();
			for (source, target) in old_nodes_set.iter().zip(nodes_to_add.iter()) {
				shares_to_move.insert(target.clone(), source.clone());
			}
			let mut ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, shares_to_move.clone());

			// initialize session on master node && run to completion
			ml.nodes[&master_node_id].session.initialize(Some(shares_to_move.clone()),
				Some(ml.old_set_signature.clone()),
				Some(ml.new_set_signature.clone())).unwrap();
			ml.run();

			// check that session has completed on all nodes
			assert!(ml.nodes.values().all(|n| n.session.is_finished()));
			
			// check that secret is still the same as before adding the share
			check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter()
				.filter(|&(k, _)| !shares_to_move.values().any(|v| v == k))
				.map(|(k, v)| (k.clone(), v.key_storage.clone()))
				.collect());
		}
	}
}
