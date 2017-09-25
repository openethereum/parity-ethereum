// TODO: when anys session starts on the node, which doesn't have key share, do not fail session, but delegate it
// TODO: add key share version to db? and never overwrite key share data, but add a new version. Version must be agreed on decryption/signing session start
// TODO: add polynom1 to database. otherwise share add is not possible
// TODO: duplicate NewKeysDissemination messages are sent!!!

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
use std::collections::{BTreeSet, BTreeMap};
use std::collections::btree_map::Entry;
use ethkey::{Public, Secret, Signature};
use parking_lot::Mutex;
use key_server_cluster::{Error, SessionId, NodeId, SessionMeta, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::message::{Message, ShareAddMessage, ShareAddConsensusMessage, ConsensusMessageWithServersMap,
	InitializeConsensusSessionWithServersMap, KeyShareCommon, NewAbsoluteTermShare, NewKeysDissemination, ShareAddError,
	ConfirmConsensusInitialization};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::dummy_job::{DummyJob, DummyJobTransport};
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};

/// Share addition session API.
pub trait Session: Send + Sync + 'static {
}

/// Share addition session transport.
pub trait SessionTransport: Clone + JobTransport<PartialJobRequest=ServersSetChangeAccessRequest, PartialJobResponse=bool> {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error>;
	/// Set all nodes id numbers.
	fn set_id_numbers(&mut self, id_numbers: BTreeMap<NodeId, Secret>);
}

/// Share addition session.
/// Based on "Efficient Multi-Party Digital Signature using Adaptive Secret Sharing for Low-Power Devices in Wireless Networks" paper:
/// http://www.wu.ece.ufl.edu/mypapers/msig.pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for shares addition the message) asks all other nodes to support addition
/// 2) key refreshing distribution (KRD): node generates new random polynom && sends required data to all other nodes
/// 3) key refreshing verification (KRV): node verifies received data
/// 4) node updates its own key share using generated (&& received) data
/// (1) is currently missed from implementation, since this session is only used as wrapped session, when consensus group is already established.
pub struct SessionImpl<T: SessionTransport> {
	/// Session core.
	core: SessionCore<T>,
	/// Session data.
	data: Mutex<SessionData<T>>,
}

/// Immutable session data.
struct SessionCore<T: SessionTransport> {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Share add session id.
	pub sub_session: Secret,
	/// Session-level nonce.
	pub nonce: u64,
	/// Original key share (for old nodes only). TODO: is it possible to read from key_storage
	pub key_share: Option<DocumentKeyShare>,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
}

/// Share add consensus session type.
type ShareAddChangeConsensusSession<T: SessionTransport> = ConsensusSession<ServersSetChangeAccessJob, T, DummyJob, DummyJobTransport>;

/// Mutable session data.
struct SessionData<T: SessionTransport> {
	/// Session state.
	pub state: SessionState,
	/// Consensus session.
	pub consensus_session: Option<ShareAddChangeConsensusSession<T>>,
	/// Consensus result: nodes-specific data.
	pub nodes: Option<BTreeMap<NodeId, NodeData>>,
	/// Sum of old polynom1 and new polynom1.
	pub refreshed_polynom1_sum: Option<Vec<Secret>>,
	/// NewKeyShare: author.
	pub key_share_author: Option<Public>,
	/// NewKeyShare: Common (shared) encryption point.
	pub key_share_common_point: Option<Public>,
	/// NewKeyShare: Encrypted point.
	pub key_share_encrypted_point: Option<Public>,
}

/// Single node data.
#[derive(Debug)]
struct NodeData {
	// === Values, filled during initialization phase ===
	/// Random unique scalar. Persistent.
	pub id_number: Option<Secret>,
	/// Has node confirmed session initialization?
	pub is_initialization_confirmed: bool,
	/// Is this a new node?
	pub is_new_node: bool,

	// === Values, filled during KRD phase ===
	/// Absolute term share, received from this node (for new nodes only).
	pub absolute_term_share: Option<Secret>,
	/// Refreshed secret value, which has been received from this node.
	pub refreshed_secret1: Option<Secret>,
	/// Refreshed Public values, which have been received from this node.
	pub refreshed_publics: Option<Vec<Public>>,
}

/// Session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// State when consensus is establishing.
	ConsensusEstablishing,
	/// Waiting for absolute term share.
	WaitingForAbsoluteTermShare,
	/// Waiting for keys dissemination.
	WaitingForKeysDissemination,
	/// Waiting for public share from every other node.
	WaitingForPublicKeyShare,
	/// Session is completed.
	Finished,
}

/// SessionImpl creation parameters
pub struct SessionParams<T: SessionTransport> {
	/// Session metadata.
	pub meta: SessionMeta,
	/// Sub session identifier.
	pub sub_session: Secret,
	/// Session transport.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Session nonce.
	pub nonce: u64,
}

/// Isolated ShareAdd session transport.
#[derive(Clone)]
pub struct IsolatedSessionTransport {
	/// Key id.
	session: SessionId,
	/// Session id.
	sub_session: Secret,
	/// Session-level nonce.
	nonce: u64,
	/// ID numbers of all participating nodes.
	id_numbers: Option<BTreeMap<NodeId, Secret>>,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new share addition session.
	pub fn new(params: SessionParams<T>) -> Result<Self, Error> {
		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta.clone(),
				sub_session: params.sub_session,
				nonce: params.nonce,
				key_share: params.key_storage.get(&params.meta.id).ok(), // ignore error, it will be checked later
				transport: params.transport,
				key_storage: params.key_storage,
			},
			data: Mutex::new(SessionData {
				consensus_session: None,
				state: SessionState::ConsensusEstablishing,
				nodes: None,
				refreshed_polynom1_sum: None,
				key_share_author: None,
				key_share_common_point: None,
				key_share_encrypted_point: None,
			}),
		})
	}

	/// Set pre-established consensus data.
	pub fn set_consensus_output(&self, old_nodes_set: BTreeSet<NodeId>, mut new_nodes_set: BTreeMap<NodeId, Option<Secret>>) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() || data.nodes.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// check && updatre passed data
		match self.core.key_share.as_ref() {
			Some(key_share) => {
				if old_nodes_set.symmetric_difference(&key_share.id_numbers.keys().cloned().collect()).nth(0).is_some() {
println!("=== 1");
					return Err(Error::InvalidNodesConfiguration);
				}
				for (new_node, new_node_id) in new_nodes_set.iter_mut() {
					if new_node_id.is_none() {
						match key_share.id_numbers.get(new_node) {
							Some(old_node_id) => *new_node_id = Some(old_node_id.clone()),
							None => {
println!("=== 2");
								return Err(Error::InvalidNodesConfiguration) },
						}
					}
				}
			},
			None => {
				if old_nodes_set.contains(&self.core.meta.self_node_id)
					|| !new_nodes_set.contains_key(&self.core.meta.self_node_id) {
println!("=== 3");
					return Err(Error::InvalidNodesConfiguration);
				}
			},
		}

		check_nodes_set(&old_nodes_set, &new_nodes_set)?;
		data.nodes = Some(new_nodes_set.into_iter()
			.map(|(n, nn)| (n, NodeData::new(nn, !old_nodes_set.contains(&n))))
			.collect());

		Ok(())
	}

	/// Initialize share add session on master node.
	pub fn initialize(&self, new_nodes_set: BTreeSet<NodeId>, old_set_signature: Option<Signature>, new_set_signature: Option<Signature>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// if consensus is not yet established => start consensus session
		let is_consensus_pre_established = data.nodes.is_some();
		if !is_consensus_pre_established {
			// prepare set of nodes to add
			let key_share = self.core.key_share.as_ref().ok_or(Error::KeyStorage("key share is not found on master node".into()))?;
			let old_nodes_set: BTreeSet<_> = key_share.id_numbers.keys().cloned().collect();
			let new_nodes_map = new_nodes_set.iter()
				.map(|n| key_share.id_numbers.get(n)
					.cloned()
					.map(Ok)
					.unwrap_or_else(|| math::generate_random_scalar())
					.map(|nn| (n.clone(), Some(nn))))
				.collect::<Result<BTreeMap<_, _>, _>>()?;
			check_nodes_set(&old_nodes_set, &new_nodes_map)?;

			let old_set_signature = old_set_signature.ok_or(Error::InvalidMessage)?;
			let new_set_signature = new_set_signature.ok_or(Error::InvalidMessage)?;
			let mut consensus_transport = self.core.transport.clone();
			consensus_transport.set_id_numbers(new_nodes_map.iter()
				.map(|(k, v)| (k.clone(), v.clone().expect("TODO")))
				.collect());
			let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
				meta: self.core.meta.clone(),
				consensus_executor: ServersSetChangeAccessJob::new_on_master(Public::default(), // TODO: admin key instead of default
					key_share.id_numbers.keys().cloned().collect(),
					key_share.id_numbers.keys().cloned().collect(),
					new_nodes_set.clone(),
					old_set_signature,
					new_set_signature),
				consensus_transport: consensus_transport,
			})?;
			consensus_session.initialize(new_nodes_set.clone())?;
			data.consensus_session = Some(consensus_session);
			data.nodes = Some(new_nodes_map.into_iter()
				.map(|(n, nn)| (n, NodeData::new(nn, !old_nodes_set.contains(&n))))
				.collect());
			return Ok(());
		}

		// otherwise => start sending ShareAdd-specific messages
		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
println!("=== SA: replay protection({} != {})", self.core.nonce, message.session_nonce());
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareAddMessage::ShareAddConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&ShareAddMessage::KeyShareCommon(ref message) =>
				self.on_common_key_share_data(sender, message),
			&ShareAddMessage::NewAbsoluteTermShare(ref message) =>
				self.on_new_absolute_term(sender, message),
			&ShareAddMessage::NewKeysDissemination(ref message) =>
				self.on_new_keys_dissemination(sender, message),
			&ShareAddMessage::ShareAddError(ref message) =>
				self.on_session_error(sender, message),
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ShareAddConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// start slave consensus session if needed
		let mut data = self.data.lock();
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			if data.consensus_session.is_none() {
				match &message.message {
					&ConsensusMessageWithServersMap::InitializeConsensusSession(ref message) => {
						let current_nodes_set = self.core.key_share.as_ref()
							.map(|ks| ks.id_numbers.keys().cloned().collect())
							.unwrap_or_else(|| message.old_nodes_set.clone().into_iter().map(Into::into).collect());
						data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
							meta: self.core.meta.clone(),
							consensus_executor: ServersSetChangeAccessJob::new_on_slave(Public::default(), // TODO: administrator public
								current_nodes_set,
							),
							consensus_transport: self.core.transport.clone(),
						})?);
					},
					_ => return Err(Error::InvalidStateForRequest),
				}
			}
		}

		let (is_establishing_consensus, is_consensus_established, new_nodes_set) = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			let is_establishing_consensus = consensus_session.state() == ConsensusSessionState::EstablishingConsensus;
			let new_nodes_set = match &message.message {
				&ConsensusMessageWithServersMap::InitializeConsensusSession(ref message) => {
					consensus_session.on_consensus_partial_request(sender, ServersSetChangeAccessRequest::from(message))?;
					Some(message.new_nodes_set.iter()
						.map(|(n, nn)| (n.clone().into(), NodeData::new(Some(nn.clone().into()), !message.old_nodes_set.contains(n))))
						.collect())
				},
				&ConsensusMessageWithServersMap::ConfirmConsensusInitialization(ref message) => {
					consensus_session.on_consensus_partial_response(sender, message.is_confirmed)?;
					None
				},
			};

			(
				is_establishing_consensus,
				consensus_session.state() == ConsensusSessionState::ConsensusEstablished,
				new_nodes_set
			)
		};
		if let Some(mut new_nodes_set) = new_nodes_set {
			data.nodes = Some(new_nodes_set);
		}
		if self.core.meta.self_node_id != self.core.meta.master_node_id || !is_establishing_consensus || !is_consensus_established {
			return Ok(());
		}

		Self::on_consensus_established(&self.core, &mut *data)
	}

	/*/// When initialization request is received.
	pub fn on_initialize_session(&self, sender: &NodeId, message: &InitializeShareAddSession) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message from master node only
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}
		// this node must be on final nodes set
		if !message.nodes.contains_key(&self.core.meta.self_node_id.clone().into()) {
			return Err(Error::InvalidMessage);
		}
		// all new nodes must be on final nodes set
		if message.new_nodes.iter().any(|n| !message.nodes.contains_key(n)) {
			return Err(Error::InvalidMessage);
		}
		// this node is either old on both (this && master) nodes, or new on both nodes
		if self.core.key_share.is_some() != !message.new_nodes.contains(&self.core.meta.self_node_id.clone().into()) {
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		data.state = SessionState::WaitingForInitializationConfirm;
		data.nodes = message.nodes.iter()
			.map(|(n, nn)| (
				n.clone().into(),
				NodeData::new(
					nn.clone().into(),
					message.new_nodes.contains(n),
				)
			))
			.collect();

		// confirm initialization
		self.core.transport.send(sender, ShareAddMessage::ConfirmShareAddInitialization(ConfirmShareAddInitialization {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.sub_session.clone().into(),
			session_nonce: self.core.nonce,
		}))
	}

	/// When session initialization confirmation message is received.
	pub fn on_confirm_initialization(&self, sender: &NodeId, message: &ConfirmShareAddInitialization) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// awaiting this message on master node only
		if self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// update node data
		let mut data = self.data.lock();
		{
			let node_data = data.nodes.get_mut(sender).ok_or(Error::InvalidMessage)?;
			if node_data.is_initialization_confirmed {
				return Err(Error::InvalidStateForRequest);
			}

			node_data.is_initialization_confirmed = true;
		}

		// if we haven't received confirmations from all old nodes => wait for more
		if data.nodes.values().any(|nd| !nd.is_initialization_confirmed) {
			return Ok(());
		}

		// all nodes have confirmed initialization => send absolute term shares to new nodes && start keys dissemination
		data.state = SessionState::WaitingForKeysDissemination;
		Self::disseminate_common_share_data(&self.core, &*data)?;
		Self::disseminate_absolute_term_shares(&self.core, &mut *data)?;
		Self::disseminate_keys(&self.core, &mut *data)
	}*/

	/// When common key share data is received by new node.
	pub fn on_common_key_share_data(&self, sender: &NodeId, message: &KeyShareCommon) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// only master can send this message
		if sender != &self.core.meta.master_node_id {
println!("=== 888");
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::ConsensusEstablishing {
			data.state = SessionState::WaitingForAbsoluteTermShare;
		} else if data.state != SessionState::WaitingForAbsoluteTermShare {
			return Err(Error::InvalidStateForRequest);
		}

		{
			// only new nodes are waiting for absolute term share
			let nodes = data.nodes.as_ref()
				.expect("nodes are filled during consensus establishing; WaitingForAbsoluteTermShare starts after consensus is established; qed");
			if !nodes[&self.core.meta.self_node_id].is_new_node {
println!("=== 999");
				return Err(Error::InvalidMessage);
			}

			// we only expect this message once
			if data.key_share_author.is_some() || data.key_share_common_point.is_some() || data.key_share_encrypted_point.is_some() {
				return Err(Error::InvalidStateForRequest);
			}
		}

		data.key_share_author = Some(message.author.clone().into());
		data.key_share_common_point = message.common_point.clone().map(Into::into);
		data.key_share_encrypted_point = message.encrypted_point.clone().map(Into::into);

		Ok(())
	}

	/// When absolute term share is received.
	pub fn on_new_absolute_term(&self, sender: &NodeId, message: &NewAbsoluteTermShare) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::ConsensusEstablishing {
			data.state = SessionState::WaitingForAbsoluteTermShare;
		} else if data.state != SessionState::WaitingForAbsoluteTermShare {
			return Err(Error::InvalidStateForRequest);
		}

		let refreshed_polynom1_sum = {
			// only new nodes are waiting for absolute term share
			let nodes = data.nodes.as_mut()
				.expect("nodes are filled during consensus establishing; WaitingForAbsoluteTermShare starts after consensus is established; qed");
			if !nodes[&self.core.meta.self_node_id].is_new_node {
				return Err(Error::InvalidMessage);
			}

			// update node data
			{
				let node_data = nodes.get_mut(sender).ok_or(Error::InvalidMessage)?;
				if node_data.absolute_term_share.is_some() {
					return Err(Error::InvalidStateForRequest);
				}
				if node_data.id_number.is_some() {
					if node_data.id_number != Some(message.sender_id.clone().into()) {
						return Err(Error::InvalidMessage);
					}
				}

				node_data.id_number = Some(message.sender_id.clone().into());
				node_data.absolute_term_share = Some(message.absolute_term_share.clone().into());
			}

			// if we haven't received shares from all old nodes => wait for more
			if nodes.values().any(|nd| !nd.is_new_node && nd.absolute_term_share.is_none()) {
				return Ok(());
			}

			// all old nodes have sent us its shares => generate/calculate secret polynom
			{
				let absolute_term_shares = nodes.values().filter_map(|nd| nd.absolute_term_share.as_ref());
				generate_refreshed_polynoms_for_new_nodes(absolute_term_shares, self.core.meta.threshold)?
			}
		};
		data.refreshed_polynom1_sum = Some(refreshed_polynom1_sum);

		// now it is time to disseminate keys to all other nodes
		data.state = SessionState::WaitingForKeysDissemination;
		Self::disseminate_keys(&self.core, &mut *data)
	}

	/// When keys dissemination message is received.
	pub fn on_new_keys_dissemination(&self, sender: &NodeId, message: &NewKeysDissemination) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::ConsensusEstablishing {
			data.state = SessionState::WaitingForKeysDissemination;
		} else if data.state == SessionState::WaitingForAbsoluteTermShare {
			return Err(Error::TooEarlyForRequest);
		} else if data.state != SessionState::WaitingForKeysDissemination {
			return Err(Error::InvalidStateForRequest);
		}

		// check message
		if message.refreshed_publics.len() != self.core.meta.threshold + 1 {
			return Err(Error::InvalidMessage);
		}

		// update node data
		let is_new_node = {
			let nodes = data.nodes.as_mut()
				.expect("nodes are filled during consensus establishing; WaitingForKeysDissemination starts after consensus is established; qed");
			{
				let node_data = nodes.get_mut(&sender).ok_or(Error::InvalidMessage)?;
				if node_data.refreshed_secret1.is_some() || node_data.refreshed_publics.is_some() {
					return Err(Error::InvalidStateForRequest);
				}

				node_data.refreshed_secret1 = Some(message.refreshed_secret1.clone().into());
				node_data.refreshed_publics = Some(message.refreshed_publics.iter().cloned().map(Into::into).collect());
			}

			nodes[&self.core.meta.self_node_id].is_new_node
		};

		// receiving this message from master node on old node means that initialization has completed => disseminate our own keys
		if !is_new_node && sender == &self.core.meta.master_node_id {
			Self::disseminate_absolute_term_shares(&self.core, &mut *data)?;
			Self::disseminate_keys(&self.core, &mut *data)?;
		}

		// check if we have received keys from every other node
		if data.nodes.as_ref()
			.expect("nodes are filled during consensus establishing; WaitingForKeysDissemination starts after consensus is established; qed")
			.iter().any(|(node_id, node_data)| node_id != &self.core.meta.self_node_id &&
			(node_data.refreshed_publics.is_none() || node_data.refreshed_secret1.is_none())) {
			return Ok(())
		}

		// verify keys && complete session if keys are ok
		Self::verify_keys(&self.core, &mut *data)?;
		Self::complete_session(&self.core, &mut *data)
	}

	/// When error has occured on another node.
	pub fn on_session_error(&self, sender: &NodeId, message: &ShareAddError) -> Result<(), Error> {
		let mut data = self.data.lock();

		warn!("{}: share add session failed with error: {} from {}", self.core.meta.self_node_id, message.error, sender);

		data.state = SessionState::Finished;

		Ok(())
	}

	/// Start sending ShareAdd-specific messages, when consensus is established.
	fn on_consensus_established(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		let is_old_node = !data.nodes.as_ref()
			.expect("consensus is about nodes set; consensus is established; qed")
			[&core.meta.self_node_id].is_new_node;
		data.state = if is_old_node { SessionState::WaitingForKeysDissemination } else { SessionState::WaitingForAbsoluteTermShare };

		// if on master node, send common shared data to every new node
		let is_master_node = core.meta.self_node_id == core.meta.master_node_id;
		if is_master_node {
			Self::disseminate_common_share_data(core, data)?;
		}

		// if on old node, send absolute term shares to every new node
		if is_old_node {
			Self::disseminate_absolute_term_shares(core, data)?;
		}

		// if on old node, send keys to every node
		if is_old_node {
			Self::disseminate_keys(core, data)?;
		}

		Ok(())
	}

	/// Disseminate absolute term of polynom1 data.
	fn disseminate_absolute_term_shares(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// compute/generate refreshed polynom1
		let old_key_share = core.key_share.as_ref()
			.expect("disseminate_absolute_term_shares is only called on old nodes; key_share is filled in initialization phase on old nodes; qed");
		let nodes = data.nodes.as_ref()
			.expect("nodes are filled during consensus establishing; absolute term shares are sent after consensus is established; qed");
		let num_new_nodes = nodes.values().filter(|nd| nd.is_new_node).count();
		let (absolute_term_shares, refreshed_polynom1_sum) = generate_refreshed_polynoms_for_existing_nodes(
			num_new_nodes, core.meta.threshold, &old_key_share.polynom1)?;
		data.refreshed_polynom1_sum = Some(refreshed_polynom1_sum);

		// send absolute term share to every new node
		let sender_id: &Secret = nodes[&core.meta.self_node_id].id_number.as_ref().expect("TODO");
		for (i, new_node) in nodes.iter().filter(|&(_, nd)| nd.is_new_node).map(|(n, _)| n).enumerate() {
			core.transport.send(new_node, ShareAddMessage::NewAbsoluteTermShare(NewAbsoluteTermShare {
				session: core.meta.id.clone().into(),
				sub_session: core.sub_session.clone().into(),
				session_nonce: core.nonce,
				sender_id: sender_id.clone().into(),
				absolute_term_share: absolute_term_shares[i].clone().into(),
			}))?;
		}

		Ok(())
	}

	/// Send common share data to evey new node.
	fn disseminate_common_share_data(core: &SessionCore<T>, data: &SessionData<T>) -> Result<(), Error> {
		let old_key_share = core.key_share.as_ref()
			.expect("disseminate_common_share_data is only called on master node; key_share is filled in initialization phase on master node; qed");
		let nodes = data.nodes.as_ref()
			.expect("nodes are filled during consensus establishing; common share data sent after consensus is established; qed");
		for new_node in nodes.iter().filter(|&(_, nd)| nd.is_new_node).map(|(n, _)| n) {
			core.transport.send(new_node, ShareAddMessage::KeyShareCommon(KeyShareCommon {
				session: core.meta.id.clone().into(),
				sub_session: core.sub_session.clone().into(),
				session_nonce: core.nonce,
				author: old_key_share.author.clone().into(),
				common_point: old_key_share.common_point.clone().map(Into::into),
				encrypted_point: old_key_share.encrypted_point.clone().map(Into::into),
			}))?;
		}

		Ok(())
	}

	/// Disseminate key refreshing data.
	fn disseminate_keys(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// send required messages
		let refreshed_polynom1_sum = data.refreshed_polynom1_sum.as_ref()
			.expect("disseminate_keys is only called after generating refreshed_polynom1_sum; qed");
		let refreshed_publics = (0..core.meta.threshold+1)
			.map(|i| math::compute_public_share(&refreshed_polynom1_sum[i]))
			.collect::<Result<Vec<_>, _>>()?;

		// send calculated values
		let nodes = data.nodes.as_mut()
			.expect("nodes are filled during consensus establishing; keys are disseminated after consensus is established; qed");
		for (node, node_number) in nodes.iter().filter(|&(n, _)| n != &core.meta.self_node_id).map(|(n, nd)| (n, &nd.id_number)) {
			// also send keys to every other node
			let refreshed_secret1 = math::compute_polynom(refreshed_polynom1_sum, node_number.as_ref().expect("TODO"))?;
			core.transport.send(node, ShareAddMessage::NewKeysDissemination(NewKeysDissemination {
				session: core.meta.id.clone().into(),
				sub_session: core.sub_session.clone().into(),
				session_nonce: core.nonce,
				refreshed_secret1: refreshed_secret1.into(),
				refreshed_publics: refreshed_publics.iter().cloned().map(Into::into).collect(),
			}))?;
		}

		// 'receive' data from self
		let self_node_data = nodes.get_mut(&core.meta.self_node_id)
			.expect("data.nodes contains entry for every session node; this node is a part of the session; qed");
		self_node_data.refreshed_secret1 = Some(math::compute_polynom(refreshed_polynom1_sum, &self_node_data.id_number.as_ref().expect("TODO"))?);
		self_node_data.refreshed_publics = Some(refreshed_publics);

		Ok(())
	}

	/// Verify received keys values.
	fn verify_keys(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		let nodes = data.nodes.as_ref()
			.expect("nodes are filled during consensus establishing; keys are verified after consensus is established; qed");
		let number_id = nodes[&core.meta.self_node_id].id_number.as_ref().expect("TODO");
		for node_data in nodes.iter().filter(|&(n, _)| n != &core.meta.self_node_id).map(|(_, nd)| nd) {
			let refreshed_secret1 = node_data.refreshed_secret1.as_ref().expect("keys received on KRD phase; KRV phase follows KRD phase; qed");
			let refreshed_publics = node_data.refreshed_publics.as_ref().expect("keys received on KRD phase; KRV phase follows KRD phase; qed");
			let is_key_verification_ok = math::refreshed_keys_verification(core.meta.threshold, &number_id, refreshed_secret1, refreshed_publics)?;

			if !is_key_verification_ok {
				// node has sent us incorrect values. In original ECDKG protocol we should have sent complaint here.
				return Err(Error::InvalidMessage);
			}
		}

		Ok(())
	}

	/// Complete session.
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// compose updated key share
		let nodes = data.nodes.as_ref()
			.expect("nodes are filled during consensus establishing; session is completed after consensus is established; qed");
		let refreshed_key_share = DocumentKeyShare {
			// values with the same value as before beginning of the session
			threshold: core.meta.threshold,
			author: core.key_share.as_ref().map(|ks| ks.author.clone())
				.unwrap_or_else(|| data.key_share_author.clone().expect("this is new node; on new nodes this field is filled before KRD; session is completed after KRD; qed")),
			common_point: core.key_share.as_ref().map(|ks| ks.common_point.clone())
				.unwrap_or_else(|| data.key_share_common_point.clone()),
			encrypted_point: core.key_share.as_ref().map(|ks| ks.encrypted_point.clone())
				.unwrap_or_else(|| data.key_share_encrypted_point.clone()),
			// below are updated values
			id_numbers: nodes.iter().map(|(node_id, node_data)| (node_id.clone(), node_data.id_number.as_ref().expect("TODO").clone())).collect(),
			polynom1: data.refreshed_polynom1_sum.clone().expect("this field is filled during KRD; session is completed after KRD; qed"),
			secret_share: math::compute_secret_share(nodes.values()
				.filter_map(|nd| nd.refreshed_secret1.as_ref()))?,
		};

		// save encrypted data to the key storage
		data.state = SessionState::Finished;
		if core.key_share.is_some() {
			core.key_storage.update(core.meta.id.clone(), refreshed_key_share.clone())
		} else {
			core.key_storage.insert(core.meta.id.clone(), refreshed_key_share.clone())
		}.map_err(|e| Error::KeyStorage(e.into()))
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

impl NodeData {
	/// Create new node data.
	pub fn new(id_number: Option<Secret>, is_new_node: bool) -> Self {
		NodeData {
			id_number: id_number,
			is_initialization_confirmed: false,
			is_new_node: is_new_node,
			absolute_term_share: None,
			refreshed_secret1: None,
			refreshed_publics: None,
		}
	}
}

impl JobTransport for IsolatedSessionTransport {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;

	fn send_partial_request(&self, node: &NodeId, request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		let id_numbers = self.id_numbers.as_ref().expect("TODO");
		self.cluster.send(node, Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ShareAddConsensusMessage {
			session: self.session.clone().into(),
			sub_session: self.sub_session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersMap::InitializeConsensusSession(InitializeConsensusSessionWithServersMap {
				old_nodes_set: request.old_servers_set.into_iter().map(Into::into).collect(),
				new_nodes_set: request.new_servers_set.into_iter().map(|n| (n.into(), id_numbers[&n].clone().into())).collect(),
				old_set_signature: request.old_set_signature.into(),
				new_set_signature: request.new_set_signature.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ShareAddConsensusMessage {
			session: self.session.clone().into(),
			sub_session: self.sub_session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageWithServersMap::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			}),
		})))
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn set_id_numbers(&mut self, id_numbers: BTreeMap<NodeId, Secret>) {
		self.id_numbers = Some(id_numbers);
	}

	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareAdd(message))
	}
}

fn check_nodes_set(old_nodes_set: &BTreeSet<NodeId>, new_nodes_set: &BTreeMap<NodeId, Option<Secret>>) -> Result<(), Error> {
	// it is impossible to remove nodes using share add session
	if old_nodes_set.iter().any(|n| !new_nodes_set.contains_key(n)) {
println!("=== 4");
		return Err(Error::InvalidNodesConfiguration);
	}
	// it is impossible to not to add any nodes using share add session
	if new_nodes_set.len() == old_nodes_set.len() {
println!("=== 5");
		return Err(Error::InvalidNodesConfiguration);
	}

	Ok(())
}

fn generate_refreshed_polynoms_for_existing_nodes(new_nodes: usize, threshold: usize, existing_polynom1: &Vec<Secret>) -> Result<(Vec<Secret>, Vec<Secret>), Error> {
	// TODO: could add secrets instead of polynoms
	let refreshed_polynoms1 = (0..new_nodes).map(|_| math::generate_random_polynom(threshold)).collect::<Result<Vec<_>, _>>()?;
	let mut refreshed_polynom1_sum = existing_polynom1.clone();
	for refreshed_polynom1 in &refreshed_polynoms1 {
		refreshed_polynom1_sum = math::add_polynoms(&refreshed_polynom1_sum, refreshed_polynom1, false)?;
	}

	Ok((
		refreshed_polynoms1.into_iter().map(|p| p[0].clone()).collect(),
		refreshed_polynom1_sum,
	))
}

fn generate_refreshed_polynoms_for_new_nodes<'a, I>(absolute_term_shares: I, threshold: usize) -> Result<Vec<Secret>, Error> where I: Iterator<Item=&'a Secret> {
	let mut new_polynom1 = math::generate_random_polynom(threshold)?;
	let new_polynom_absolute_term = math::compute_additional_polynom1_absolute_term(absolute_term_shares)?;
	new_polynom1[0] = new_polynom_absolute_term;
	Ok(new_polynom1)
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
	use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
	use super::{SessionImpl, SessionParams, SessionTransport, IsolatedSessionTransport};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl<IsolatedSessionTransport>,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	impl MessageLoop {
		pub fn new(gml: GenerationMessageLoop, threshold: usize, num_new_nodes: usize) -> Self {
			let mut new_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().collect();
			let new_nodes_ids: BTreeSet<_> = (0..num_new_nodes).map(|_| Random.generate().unwrap().public().clone()).collect();
			new_nodes_set.extend(new_nodes_ids.iter().cloned());

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
				let session = SessionImpl::new(SessionParams {
					meta: meta.clone(),
					sub_session: sub_session.clone(),
					transport: IsolatedSessionTransport {
						session: meta.id.clone(),
						sub_session: sub_session.clone(),
						nonce: 1,
						id_numbers: None,
						cluster: cluster.clone(),
					},
					key_storage: nd.key_storage.clone(),
					nonce: 1,
				}).unwrap();
				nodes.insert(n.clone(), Node {
					cluster: cluster,
					key_storage: key_storage,
					session: session,
				});
			}
			for new_node_id in new_nodes_ids {
				let cluster = Arc::new(DummyCluster::new(new_node_id.clone()));
				let key_storage = Arc::new(DummyKeyStorage::default());
				let mut meta = meta.clone();
				meta.self_node_id = new_node_id;
				let session = SessionImpl::new(SessionParams {
					meta: meta.clone(),
					sub_session: sub_session.clone(),
					transport: IsolatedSessionTransport {
						session: meta.id.clone(),
						sub_session: sub_session.clone(),
						nonce: 1,
						id_numbers: None,
						cluster: cluster.clone(),
					},
					key_storage: key_storage.clone(),
					nonce: 1,
				}).unwrap();
				nodes.insert(new_node_id, Node {
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
					Message::ShareAdd(ref message) =>
						self.nodes[&msg.1].session.process_message(&msg.0, message),
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
	fn node_added_using_share_add() {
		// initial 2-of-3 session
		let (t, n) = (1, 3);
		let gml = generate_key(t, n);
		let gml_nodes: BTreeSet<_> = gml.nodes.keys().cloned().collect();
		let key_id = gml.session_id.clone();
		let master = gml.nodes.keys().cloned().nth(0).unwrap();
		let joint_secret = math::compute_joint_secret(gml.nodes.values()
			.map(|nd| nd.key_storage.get(&key_id).unwrap().polynom1[0].clone())
			.collect::<Vec<_>>()
			.iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// insert 1 node so that it becames 2-of-4 session
		let old_nodes_set: BTreeSet<_> = gml.nodes.keys().cloned().collect();
		let old_set_signature = sign(&joint_secret, &ordered_nodes_hash(&old_nodes_set)).unwrap();
		let mut ml = MessageLoop::new(gml, t, 1);
		let nodes_to_add: BTreeSet<_> = ml.nodes.keys().cloned().filter(|n| !gml_nodes.contains(n)).collect();
		assert_eq!(nodes_to_add.len(), 1);
		let new_nodes_set: BTreeSet<_> = old_nodes_set.into_iter().chain(nodes_to_add.into_iter()).collect();
		let new_set_signature = sign(&joint_secret, &ordered_nodes_hash(&new_nodes_set)).unwrap();
		ml.nodes[&master].session.initialize(new_nodes_set, Some(old_set_signature), Some(new_set_signature)).unwrap();
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..n+1 {
			for n2 in n1+1..n+1 {
				let share1 = ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).unwrap();
				let share2 = ml. nodes.values().nth(n2).unwrap().key_storage.get(&key_id).unwrap();
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
