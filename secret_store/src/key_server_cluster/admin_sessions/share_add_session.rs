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
use bigint::hash::H256;
use ethkey::{Public, Secret, Signature};
use parking_lot::{Mutex, Condvar};
use key_server_cluster::{Error, SessionId, NodeId, DocumentKeyShare, DocumentKeyShareVersion, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::message::{Message, ShareAddMessage, ShareAddConsensusMessage, ConsensusMessageOfShareAdd,
	InitializeConsensusSessionOfShareAdd, KeyShareCommon, NewKeysDissemination, ShareAddError,
	ConfirmConsensusInitialization};
use key_server_cluster::jobs::job_session::JobTransport;
use key_server_cluster::jobs::dummy_job::{DummyJob, DummyJobTransport};
use key_server_cluster::jobs::servers_set_change_access_job::{ServersSetChangeAccessJob, ServersSetChangeAccessRequest};
use key_server_cluster::jobs::consensus_session::{ConsensusSessionParams, ConsensusSessionState, ConsensusSession};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Share addition session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed.
	fn wait(&self) -> Result<(), Error>;
}

/// Share addition session transport.
pub trait SessionTransport: Clone + JobTransport<PartialJobRequest=ServersSetChangeAccessRequest, PartialJobResponse=bool> {
	/// Get all connected nodes. Since ShareAdd session requires all cluster nodes to be connected, this set equals to all known cluster nodes set.
	fn nodes(&self) -> BTreeSet<NodeId>;
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error>;
	/// Set data for master node (sent to slave nodes in consensus session initialization message).
	fn set_master_data(&mut self, consensus_group: BTreeSet<NodeId>, id_numbers: BTreeMap<NodeId, Option<Secret>>);
}

/// Share addition session.
/// Based on "Efficient Multi-Party Digital Signature using Adaptive Secret Sharing for Low-Power Devices in Wireless Networks" paper:
/// http://www.wu.ece.ufl.edu/mypapers/msig.pdf
/// Brief overview:
/// 1) initialization: master node (which has received request for shares addition the message) asks all other nodes to support addition
/// 2) key refreshing distribution (KRD): node generates new random polynom && sends required data to all other nodes
/// 3) key refreshing verification (KRV): node verifies received data
/// 4) node updates its own key share using generated (&& received) data
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

/// Share add consensus session type.
type ShareAddChangeConsensusSession<T> = ConsensusSession<ServersSetChangeAccessJob, T, DummyJob, DummyJobTransport>;

/// Mutable session data.
struct SessionData<T: SessionTransport> {
	/// Session state.
	pub state: SessionState,
	/// Key version to use for decryption.
	pub version: Option<H256>,
	/// Consensus session.
	pub consensus_session: Option<ShareAddChangeConsensusSession<T>>,
	/// NewKeyShare: threshold.
	pub key_share_threshold: Option<usize>,
	/// NewKeyShare: author.
	pub key_share_author: Option<Public>,
	/// NewKeyShare: Common (shared) encryption point.
	pub key_share_common_point: Option<Public>,
	/// NewKeyShare: Encrypted point.
	pub key_share_encrypted_point: Option<Public>,
	/// Nodes id numbers.
	pub id_numbers: Option<BTreeMap<NodeId, Option<Secret>>>,
	/// Secret subshares received from nodes.
	pub secret_subshares: Option<BTreeMap<NodeId, Option<Secret>>>,
	/// Share add change result.
	pub result: Option<Result<(), Error>>,
}

/// Session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// State when consensus is establishing.
	ConsensusEstablishing,
	/// Waiting for keys dissemination.
	WaitingForKeysDissemination,
	/// Session is completed.
	Finished,
}

/// SessionImpl creation parameters
pub struct SessionParams<T: SessionTransport> {
	/// Session metadata.
	pub meta: ShareChangeSessionMeta,
	/// Session transport.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
	/// Administrator public key.
	pub admin_public: Option<Public>,
	/// Session nonce.
	pub nonce: u64,
}

/// Isolated ShareAdd session transport.
#[derive(Clone)]
pub struct IsolatedSessionTransport {
	/// Key id.
	session: SessionId,
	/// Key version.
	version: Option<H256>,
	/// Session-level nonce.
	nonce: u64,
	/// Consensus group.
	consensus_group: Option<BTreeSet<NodeId>>,
	/// Id numbers of all new nodes.
	id_numbers: Option<BTreeMap<NodeId, Option<Secret>>>,
	/// Cluster.
	cluster: Arc<Cluster>,
}

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new share addition session.
	pub fn new(params: SessionParams<T>) -> Result<Self, Error> {
		let key_share = params.key_storage
			.get(&params.meta.id)
			.map_err(|e| Error::KeyStorage(e.into()))?;

		Ok(SessionImpl {
			core: SessionCore {
				meta: params.meta,
				nonce: params.nonce,
				key_share: key_share,
				transport: params.transport,
				key_storage: params.key_storage,
				admin_public: params.admin_public,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::ConsensusEstablishing,
				version: None,
				consensus_session: None,
				key_share_threshold: None,
				key_share_author: None,
				key_share_common_point: None,
				key_share_encrypted_point: None,
				id_numbers: None,
				secret_subshares: None,
				result: None,
			}),
		})
	}

	/// Set pre-established consensus data.
	pub fn set_consensus_output(&self, version: &H256, consensus_group: BTreeSet<NodeId>, mut new_nodes_map: BTreeMap<NodeId, Option<Secret>>) -> Result<(), Error> {
		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() || data.id_numbers.is_some() || data.secret_subshares.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// key share version is required on ShareAdd master node
		if let Some(key_share) = self.core.key_share.as_ref() {
			if let Ok(key_version) = key_share.version(version) {
				for (node, id_number) in &key_version.id_numbers {
					{
						let external_id_number = new_nodes_map.get(node);
						match external_id_number {
							Some(&Some(ref external_id_number)) => {
								if external_id_number == id_number {
									continue;
								}
								return Err(Error::ConsensusUnreachable);
							},
							Some(&None) => (),
							None => return Err(Error::ConsensusUnreachable),
						}
					}

					new_nodes_map.insert(node.clone(), Some(id_number.clone()));
				}

				// check that all id_numbers are filled
				if new_nodes_map.values().any(Option::is_none) {
					return Err(Error::ConsensusUnreachable);
				}
			}
		}

		// check passed consensus data
		Self::check_nodes_map(&self.core, version, &consensus_group, &new_nodes_map)?;

		// update data
		data.version = Some(version.clone());
		data.id_numbers = Some(new_nodes_map);
		data.secret_subshares = Some(consensus_group.into_iter()
			.map(|n| (n, None))
			.collect());

		Ok(())
	}

	/// Initialize share add session on master node.
	pub fn initialize(&self, version: Option<H256>, new_nodes_set: Option<BTreeSet<NodeId>>, old_set_signature: Option<Signature>, new_set_signature: Option<Signature>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.consensus_session.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// if consensus is pre-established => start sending ShareAdd-specific messages
		let is_consensus_pre_established = data.id_numbers.is_some();
		if is_consensus_pre_established {
			return Self::on_consensus_established(&self.core, &mut *data);
		}

		// else => prepare to start consensus session
		// require all initialization params for consensus session
		let version = version.ok_or(Error::InvalidMessage)?;
		let old_set_signature = old_set_signature.ok_or(Error::InvalidMessage)?;
		let new_set_signature = new_set_signature.ok_or(Error::InvalidMessage)?;
		let new_nodes_set = new_nodes_set.ok_or(Error::InvalidMessage)?;
		let admin_public = self.core.admin_public.as_ref().cloned().ok_or(Error::ConsensusUnreachable)?;

		// key share version is required on ShareAdd master node
		let key_share = self.core.key_share.as_ref().ok_or(Error::KeyStorage("key share is not found on master node".into()))?;
		let key_version = key_share.version(&version).map_err(|e| Error::KeyStorage(e.into()))?;

		// old nodes set is all non-isolated owners of version holders
		let non_isolated_nodes = self.core.transport.nodes();
		let old_nodes_set: BTreeSet<_> = key_version.id_numbers.keys()
			.filter(|n| non_isolated_nodes.contains(n))
			.cloned()
			.collect();

		// new nodes map contains previous id_numbers for old nodes && random number for new nodes
		let mut new_nodes_map = BTreeMap::new();
		for new_node in new_nodes_set.into_iter().filter(|n| non_isolated_nodes.contains(n)) {
			new_nodes_map.insert(new_node, match key_version.id_numbers.get(&new_node) {
				Some(old_id_number) => Some(old_id_number.clone()),
				None => Some(math::generate_random_scalar()?),
			});
		}

		// let's select consensus group
		let consensus_group: BTreeSet<_> = ::std::iter::once(self.core.meta.self_node_id.clone())
			.chain(old_nodes_set.iter()
				.filter(|n| **n != self.core.meta.self_node_id && non_isolated_nodes.contains(*n))
				.take(key_share.threshold)
				.cloned())
			.collect();

		// now check nodes map
		Self::check_nodes_map(&self.core, &version, &consensus_group, &new_nodes_map)?;

		// prepare consensus session transport
		let mut consensus_transport = self.core.transport.clone();
		consensus_transport.set_master_data(consensus_group.clone(), new_nodes_map.clone());

		// create && initialize consensus session
		let mut consensus_session = ConsensusSession::new(ConsensusSessionParams {
			meta: self.core.meta.clone().into_consensus_meta(new_nodes_map.len())?,
			consensus_executor: ServersSetChangeAccessJob::new_on_master(admin_public,
				old_nodes_set.clone(),
				new_nodes_map.keys().cloned().collect(),
				old_set_signature,
				new_set_signature),
			consensus_transport: consensus_transport,
		})?;
		consensus_session.initialize(new_nodes_map.keys().cloned().collect())?;

		// update data
		data.version = Some(version);
		data.consensus_session = Some(consensus_session);
		data.id_numbers = Some(new_nodes_map);
		data.secret_subshares = Some(consensus_group.into_iter().map(|n| (n, None)).collect());

		Ok(())
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareAddMessage::ShareAddConsensusMessage(ref message) =>
				self.on_consensus_message(sender, message),
			&ShareAddMessage::KeyShareCommon(ref message) =>
				self.on_common_key_share_data(sender, message),
			&ShareAddMessage::NewKeysDissemination(ref message) =>
				self.on_new_keys_dissemination(sender, message),
			&ShareAddMessage::ShareAddError(ref message) => {
				self.on_session_error(sender, Error::Io(message.error.clone().into()));
				Ok(())
			},
		}
	}

	/// When consensus-related message is received.
	pub fn on_consensus_message(&self, sender: &NodeId, message: &ShareAddConsensusMessage) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// start slave consensus session if needed
		let mut data = self.data.lock();
		match &message.message {
			&ConsensusMessageOfShareAdd::InitializeConsensusSession(ref message)
				if data.consensus_session.is_none() && sender == &self.core.meta.master_node_id => {
					let admin_public = self.core.admin_public.as_ref().cloned().ok_or(Error::ConsensusUnreachable)?;
					data.consensus_session = Some(ConsensusSession::new(ConsensusSessionParams {
						meta: self.core.meta.clone().into_consensus_meta(message.new_nodes_map.len())?,
						consensus_executor: ServersSetChangeAccessJob::new_on_slave(admin_public),
						consensus_transport: self.core.transport.clone(),
					})?);
				},
			_ => (),
		};

		// process consensus message
		let (is_establishing_consensus, is_consensus_established, version, new_nodes_map, consensus_group) = {
			let consensus_session = data.consensus_session.as_mut().ok_or(Error::InvalidMessage)?;
			let is_establishing_consensus = consensus_session.state() == ConsensusSessionState::EstablishingConsensus;

			let (version, new_nodes_map, consensus_group) = match &message.message {
				&ConsensusMessageOfShareAdd::InitializeConsensusSession(ref message) => {
					consensus_session.on_consensus_partial_request(sender, ServersSetChangeAccessRequest::from(message))?;

					let version = message.version.clone().into();
					let consensus_group = message.consensus_group.iter().cloned().map(Into::into).collect();
					let new_nodes_map: BTreeMap<_, _> = message.new_nodes_map.iter()
						.map(|(n, nn)| (n.clone().into(), Some(nn.clone().into())))
						.collect();

					// check that all id_numbers are filled
					if new_nodes_map.values().any(Option::is_none) {
						return Err(Error::ConsensusUnreachable);
					}

					// check old set of nodes
					Self::check_nodes_map(&self.core, &version, &consensus_group, &new_nodes_map)?;

					(Some(version), Some(new_nodes_map), Some(consensus_group))
				},
				&ConsensusMessageOfShareAdd::ConfirmConsensusInitialization(ref message) => {
					consensus_session.on_consensus_partial_response(sender, message.is_confirmed)?;
					(None, None, None)
				},
			};

			(
				is_establishing_consensus,
				consensus_session.state() == ConsensusSessionState::ConsensusEstablished,
				version,
				new_nodes_map,
				consensus_group,
			)
		};

		// update data
		if let Some(version) = version {
			data.version = Some(version);
		}
		if let Some(new_nodes_map) = new_nodes_map {
			data.id_numbers = Some(new_nodes_map);
		}
		if let Some(consensus_group) = consensus_group {
			data.secret_subshares = Some(consensus_group.into_iter().map(|n| (n, None)).collect());
		}

		// if consensus is stablished, proceed
		if !is_establishing_consensus || !is_consensus_established || self.core.meta.self_node_id != self.core.meta.master_node_id {
			return Ok(());
		}

		Self::on_consensus_established(&self.core, &mut *data)
	}

	/// When common key share data is received by new node.
	pub fn on_common_key_share_data(&self, sender: &NodeId, message: &KeyShareCommon) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// only master can send this message
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::ConsensusEstablishing || data.id_numbers.is_none() {
			return Ok(());
		}

		// we only expect this message once
		if data.key_share_threshold.is_some() || data.key_share_author.is_some() || data.key_share_common_point.is_some() || data.key_share_encrypted_point.is_some() {
			return Err(Error::InvalidStateForRequest);
		}

		// check if we actually waiting for this message
		{
			let version = data.version.as_ref().ok_or(Error::InvalidStateForRequest)?;
			let key_version = self.core.key_share.as_ref().and_then(|ks| ks.version(version).ok());
			if key_version.is_some() {
				return Ok(());
			}
		}

		// update data
		data.state = SessionState::WaitingForKeysDissemination;
		data.key_share_threshold = Some(message.threshold);
		data.key_share_author = Some(message.author.clone().into());
		data.key_share_common_point = message.common_point.clone().map(Into::into);
		data.key_share_encrypted_point = message.encrypted_point.clone().map(Into::into);

		let mut id_numbers = data.id_numbers.as_mut()
			.expect("common key share data is expected after initialization; id_numers are filled during initialization; qed");
		for (node, id_number) in &message.id_numbers {
			let id_number: Secret = id_number.clone().into();
			{
				let local_id_number = id_numbers.get(&node.clone().into());
				match local_id_number {
					Some(&Some(ref local_id_number)) => {
						if *local_id_number == id_number {
							continue;
						}

						return Err(Error::ConsensusUnreachable);
					},
					Some(&None) => (),
					None => continue, // can happen for isolated nodes
				}
			}

			id_numbers.insert(node.clone().into(), Some(id_number));
		}

		Ok(())
	}

	/// When keys dissemination message is received.
	pub fn on_new_keys_dissemination(&self, sender: &NodeId, message: &NewKeysDissemination) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::ConsensusEstablishing && data.secret_subshares.is_some() {
			data.state = SessionState::WaitingForKeysDissemination;
		} else if data.state != SessionState::WaitingForKeysDissemination {
			return Err(Error::InvalidStateForRequest);
		}

		// update data
		let explanation = "secret_subshares is filled during initialization; keys are disseminated after initialization; qed";
		{
			match data.secret_subshares.as_ref().expect(explanation).get(sender) {
				None => return Err(Error::InvalidMessage),
				Some(&Some(_)) => return Err(Error::InvalidMessage),
				Some(&None) => (),
			};

			let secret_subshare = Self::compute_secret_subshare(&self.core, &mut *data, sender, &message.secret_subshare.clone().into())?;
			*data.secret_subshares.as_mut().expect(explanation)
				.get_mut(sender)
				.expect("checked couple of lines above; qed") = Some(secret_subshare);
		}

		// if we have received subshare from master node, it means that we should start dissemination
		if sender == &self.core.meta.master_node_id {
			Self::on_consensus_established(&self.core, &mut *data)?;
		}

		// check if shares from all nodes are received
		if data.secret_subshares.as_ref().expect(explanation).values().any(|v| v.is_none()) {
			return Ok(())
		}

		// TODO: find a way to verificate keys
		Self::complete_session(&self.core, &mut *data)
	}

	/// Check nodes map.
	fn check_nodes_map(core: &SessionCore<T>, version: &H256, consensus_group: &BTreeSet<NodeId>, new_nodes_map: &BTreeMap<NodeId, Option<Secret>>) -> Result<(), Error> {
		// check if this node has given version
		let has_this_version = match core.key_share.as_ref() {
			Some(key_share) => key_share.version(version).is_ok(),
			None => false,
		};

		// check && update passed data
		match has_this_version {
			true => {
				// check if version exists
				let explanation = "has_this_version is true; it is true if we have given version of the key; qed";
				let key_share = core.key_share.as_ref().expect(explanation);
				let key_version = key_share.version(version).expect(explanation);

				// there must be exactly thresold + 1 nodes in consensus group
				if consensus_group.len() != key_share.threshold + 1 {
					return Err(Error::ConsensusUnreachable);
				}

				// every non-isolated node must be a part of new_nodes_set
				let non_isolated_nodes = core.transport.nodes();
				if key_version.id_numbers.keys().any(|n| non_isolated_nodes.contains(n) && !new_nodes_map.contains_key(n)) {
					return Err(Error::ConsensusUnreachable);
				}

				// there must be at least one new node in new_nodes_map
				if key_version.id_numbers.len() >= new_nodes_map.len() {
					return Err(Error::ConsensusUnreachable);
				}
			},
			false => {
				// if we do not have a share, we should not be a part of consenus group
				// but we must be on new nodes set, since this is a ShareAdd session
				if consensus_group.contains(&core.meta.self_node_id) ||
					!new_nodes_map.contains_key(&core.meta.self_node_id) {
					return Err(Error::ConsensusUnreachable);
				}
			},
		}

		// master node must always be a part of consensus group
		if !consensus_group.contains(&core.meta.master_node_id) {
			return Err(Error::ConsensusUnreachable);
		}

		// master node must always be a part of new_nodes_map
		if !new_nodes_map.contains_key(&core.meta.master_node_id) {
			return Err(Error::ConsensusUnreachable);
		}

		Ok(())
	}

	/// Start sending ShareAdd-specific messages, when consensus is established.
	fn on_consensus_established(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// update state
		data.state = SessionState::WaitingForKeysDissemination;

		// if we're not a part of consensus group, wait for secret subshares
		let explanation = "secret_subshares is a result of consensus job; consensus is established; qed";
		let is_consensus_group_node = data.secret_subshares.as_ref().expect(explanation).contains_key(&core.meta.self_node_id);
		if !is_consensus_group_node {
			return Ok(());
		}

		// else if master => send shared data to every new node
		if core.meta.self_node_id == core.meta.master_node_id {
			Self::disseminate_common_share_data(core, data)?;
		}

		// ...and then disseminate keys
		Self::disseminate_keys(core, data)?;

		// ..and check if session could be completed
		if data.secret_subshares.as_ref().expect(explanation).values().any(|v| v.is_none()) {
			return Ok(())
		}

		// TODO: find a way to verificate keys
		Self::complete_session(core, data)
	}

	/// Send common share data to evey new node.
	fn disseminate_common_share_data(core: &SessionCore<T>, data: &SessionData<T>) -> Result<(), Error> {
		let explanation = "disseminate_common_share_data is only called on master node; master node has specified version of the key; qed";
		let old_key_share = core.key_share.as_ref().expect(explanation);
		let old_key_version = old_key_share.version(data.version.as_ref().expect(explanation)).expect(explanation);
		let consensus_group = data.secret_subshares.as_ref()
			.expect("disseminate_common_share_data is only called on master node; consensus group is created during initialization on master node; qed");
		let nodes = data.id_numbers.as_ref()
			.expect("nodes are filled during consensus establishing; common share data sent after consensus is established; qed")
			.keys()
			.filter(|n| !consensus_group.contains_key(n));
		for new_node in nodes {
			core.transport.send(new_node, ShareAddMessage::KeyShareCommon(KeyShareCommon {
				session: core.meta.id.clone().into(),
				session_nonce: core.nonce,
				threshold: old_key_share.threshold,
				author: old_key_share.author.clone().into(),
				common_point: old_key_share.common_point.clone().map(Into::into),
				encrypted_point: old_key_share.encrypted_point.clone().map(Into::into),
				id_numbers: old_key_version.id_numbers.iter().map(|(k, v)| (k.clone().into(), v.clone().into())).collect(),
			}))?;
		}

		Ok(())
	}

	/// Disseminate key refreshing data.
	fn disseminate_keys(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// generate random polynom with secret share as absolute term
		let explanation = "disseminate_keys is only called on consensus group nodes; consensus group nodes have specified version of the key; qed";
		let key_share = core.key_share.as_ref().expect(explanation);
		let key_version = key_share.version(data.version.as_ref().expect(explanation)).expect(explanation);
		let mut secret_share_polynom = math::generate_random_polynom(key_share.threshold)?;
		secret_share_polynom[0] = key_version.secret_share.clone();

		// calculate secret subshare for every new node (including this node)
		let explanation = "disseminate_keys is called after initialization has completed; this field is filled during initialization; qed";
		for (new_node, new_node_number) in data.id_numbers.as_ref().expect(explanation).iter() {
			let new_node_number = new_node_number.as_ref().ok_or(Error::InvalidMessage)?;
			let secret_subshare = math::compute_polynom(&secret_share_polynom, new_node_number)?;
			if new_node != &core.meta.self_node_id {
				core.transport.send(new_node, ShareAddMessage::NewKeysDissemination(NewKeysDissemination {
					session: core.meta.id.clone().into(),
					session_nonce: core.nonce,
					secret_subshare: secret_subshare.into(),
				}))?;
			} else {
				let secret_subshare = Self::compute_secret_subshare(core, data, new_node, &secret_subshare)?;
				*data.secret_subshares.as_mut().expect(explanation)
					.get_mut(&core.meta.self_node_id)
					.expect("disseminate_keys is only calle on consensus group nodes; there's entry for every consensus node in secret_subshares; qed")
						= Some(secret_subshare);
			}
		}

		Ok(())
	}

	/// Compute secret subshare from passed secret value.
	fn compute_secret_subshare(core: &SessionCore<T>, data: &SessionData<T>, sender: &NodeId, secret_value: &Secret) -> Result<Secret, Error> {
		let explanation = "this field is a result of consensus job; compute_secret_subshare is called after consensus is established";
		let id_numbers = data.id_numbers.as_ref().expect(explanation);
		let secret_subshares = data.secret_subshares.as_ref().expect(explanation);
		let threshold = core.key_share.as_ref().map(|ks| ks.threshold)
			.unwrap_or_else(|| *data.key_share_threshold.as_ref()
				.expect("computation occurs after receiving key share threshold if not having one already; qed"));

		let explanation = "id_numbers are checked to have Some value for every consensus group node when consensus is establishe; qed";
		let sender_id_number = id_numbers[sender].as_ref().expect(explanation);
		let other_id_numbers = secret_subshares.keys().filter(|k| *k != sender).map(|n| id_numbers[n].as_ref().expect(explanation));
		math::compute_secret_subshare(threshold, secret_value, sender_id_number, other_id_numbers)
	}

	/// Complete session.
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData<T>) -> Result<(), Error> {
		// if already completed, do nothing
		if data.state == SessionState::Finished {
			return Ok(());
		}

		// compose updated key share
		let explanation = "this field is a result of consensus job; complete_session is called after consensus is established";
		let id_numbers = data.id_numbers.as_ref().expect(explanation);
		let secret_subshares = data.secret_subshares.as_ref()
			.expect("nodes are filled during consensus establishing; session is completed after consensus is established; qed");
		let secret_share = math::compute_secret_share(secret_subshares.values().map(|ss| ss.as_ref()
			.expect("complete_session is only called when subshares from all nodes are received; qed")))?;

		let refreshed_key_version = DocumentKeyShareVersion::new(id_numbers.clone().into_iter().map(|(k, v)| (k.clone(),
			v.expect("id_numbers are checked to have Some value for every consensus group node when consensus is establishe; qed"))).collect(),
			secret_share);
		let mut refreshed_key_share = core.key_share.as_ref().cloned().unwrap_or_else(|| DocumentKeyShare {
			author: data.key_share_author.clone()
				.expect("this is new node; on new nodes this field is filled before KRD; session is completed after KRD; qed"),
			threshold: data.key_share_threshold.clone()
				.expect("this is new node; on new nodes this field is filled before KRD; session is completed after KRD; qed"),
			common_point: data.key_share_common_point.clone(),
			encrypted_point: data.key_share_encrypted_point.clone(),
			versions: Vec::new(),
		});
		refreshed_key_share.versions.push(refreshed_key_version);

		// save encrypted data to the key storage
		data.state = SessionState::Finished;
		if core.key_share.is_some() {
			core.key_storage.update(core.meta.id.clone(), refreshed_key_share.clone())
		} else {
			core.key_storage.insert(core.meta.id.clone(), refreshed_key_share.clone())
		}.map_err(|e| Error::KeyStorage(e.into()))?;

		// signal session completion
		data.state = SessionState::Finished;
		data.result = Some(Ok(()));
		core.completed.notify_all();

		Ok(())
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
	type Id = SessionId;

	fn type_name() -> &'static str {
		"share add"
	}

	fn id(&self) -> SessionId {
		self.core.meta.id.clone()
	}

	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		self.on_session_error(&self.core.meta.self_node_id, Error::NodeDisconnected)
	}

	fn on_node_timeout(&self, node: &NodeId) {
		self.on_session_error(node, Error::NodeDisconnected)
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		// error in generation session is considered fatal
		// => broadcast error if error occured on this node
		if *node == self.core.meta.self_node_id {
			for node in self.core.transport.nodes() {
				// do not bother processing send error, as we already processing error
				let _ = self.core.transport.send(&node, ShareAddMessage::ShareAddError(ShareAddError {
					session: self.core.meta.id.clone().into(),
					session_nonce: self.core.nonce,
					error: error.clone().into(),
				}));
			}
		}

		let mut data = self.data.lock();

		warn!("{}: share add session failed: {} on {}", self.core.meta.self_node_id, error, node);

		data.state = SessionState::Finished;
		data.result = Some(Err(error));
		self.core.completed.notify_all();
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		match *message {
			Message::ShareAdd(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl IsolatedSessionTransport {
	pub fn new(session_id: SessionId, version: Option<H256>, nonce: u64, cluster: Arc<Cluster>) -> Self {
		IsolatedSessionTransport {
			session: session_id,
			version: version,
			nonce: nonce,
			cluster: cluster,
			id_numbers: None,
			consensus_group: None,
		}
	}
}

impl JobTransport for IsolatedSessionTransport {
	type PartialJobRequest = ServersSetChangeAccessRequest;
	type PartialJobResponse = bool;

	fn send_partial_request(&self, node: &NodeId, request: ServersSetChangeAccessRequest) -> Result<(), Error> {
		let explanation = "partial requests are sent from master node only; on master node this field is filled during creation; qed";
		let id_numbers = self.id_numbers.as_ref().expect(explanation);

		self.cluster.send(node, Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ShareAddConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageOfShareAdd::InitializeConsensusSession(InitializeConsensusSessionOfShareAdd {
				version: self.version.clone().expect(explanation).into(),
				consensus_group: self.consensus_group.as_ref().expect(explanation).iter().cloned().map(Into::into).collect(),
				old_nodes_set: request.old_servers_set.into_iter().map(Into::into).collect(),
				new_nodes_map: request.new_servers_set.into_iter()
					.filter_map(|n| id_numbers.get(&n)
						.map(|id| (n.into(), id.clone()
							.expect("partial requests are sent from master node only after consensus is established;
								on master id_numbers are initialized with Some id_number for every consensus group node; qed").into())))
					.collect(),
				old_set_signature: request.old_set_signature.into(),
				new_set_signature: request.new_set_signature.into(),
			}),
		})))
	}

	fn send_partial_response(&self, node: &NodeId, response: bool) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ShareAddConsensusMessage {
			session: self.session.clone().into(),
			session_nonce: self.nonce,
			message: ConsensusMessageOfShareAdd::ConfirmConsensusInitialization(ConfirmConsensusInitialization {
				is_confirmed: response,
			}),
		})))
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn nodes(&self) -> BTreeSet<NodeId> {
		self.cluster.nodes()
	}

	fn set_master_data(&mut self, consensus_group: BTreeSet<NodeId>, id_numbers: BTreeMap<NodeId, Option<Secret>>) {
		self.consensus_group = Some(consensus_group);
		self.id_numbers = Some(id_numbers);
	}

	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::ShareAdd(message))
	}
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use ethkey::{Random, Generator, Public, KeyPair, Signature, sign};
	use bigint::hash::H256;
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage};
	use key_server_cluster::cluster::Cluster;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::cluster_sessions::ClusterSession;
	use key_server_cluster::generation_session::tests::{Node as GenerationNode, generate_nodes_ids};
	use key_server_cluster::math;
	use key_server_cluster::message::Message;
	use key_server_cluster::servers_set_change_session::tests::generate_key;
	use key_server_cluster::jobs::servers_set_change_access_job::ordered_nodes_hash;
	use key_server_cluster::admin_sessions::ShareChangeSessionMeta;
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
		pub version: H256,
	}

	fn create_session(mut meta: ShareChangeSessionMeta, admin_public: Public, self_node_id: NodeId, cluster: Arc<Cluster>, key_storage: Arc<KeyStorage>) -> SessionImpl<IsolatedSessionTransport> {
		let session_id = meta.id.clone();
		meta.self_node_id = self_node_id;
		let key_version = key_storage.get(&session_id).unwrap().map(|ks| ks.versions.iter().last().unwrap().hash.clone());

		SessionImpl::new(SessionParams {
			meta: meta.clone(),
			transport: IsolatedSessionTransport::new(session_id, key_version, 1, cluster),
			key_storage: key_storage,
			admin_public: Some(admin_public),
			nonce: 1,
		}).unwrap()
	}

	fn create_node(meta: ShareChangeSessionMeta, admin_public: Public, node: GenerationNode, added_nodes: &BTreeSet<NodeId>) -> Node {
		node.cluster.add_nodes(added_nodes.iter().cloned());
		Node {
			cluster: node.cluster.clone(),
			key_storage: node.key_storage.clone(),
			session: create_session(meta, admin_public, node.session.node().clone(), node.cluster, node.key_storage),
		}
	}

	/// This only works for schemes where threshold = 1
	pub fn check_secret_is_preserved(joint_key_pair: KeyPair, nodes: BTreeMap<NodeId, Arc<DummyKeyStorage>>) {
		let n = nodes.len();
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..n {
			for n2 in n1+1..n {
				let share1 = nodes.values().nth(n1).unwrap().get(&SessionId::default()).unwrap();
				let share2 = nodes.values().nth(n2).unwrap().get(&SessionId::default()).unwrap();
				let id_number1 = share1.as_ref().unwrap().last_version().unwrap().id_numbers[nodes.keys().nth(n1).unwrap()].clone();
				let id_number2 = share1.as_ref().unwrap().last_version().unwrap().id_numbers[nodes.keys().nth(n2).unwrap()].clone();

				// now encrypt and decrypt data
				let (document_secret_decrypted, document_secret_decrypted_test) =
					math::tests::do_encryption_and_decryption(1,
						joint_key_pair.public(),
						&[id_number1, id_number2],
						&[share1.unwrap().last_version().unwrap().secret_share.clone(),
							share2.unwrap().last_version().unwrap().secret_share.clone()],
						Some(joint_key_pair.secret()),
						document_secret_plain.clone());

				assert_eq!(document_secret_plain, document_secret_decrypted_test);
				assert_eq!(document_secret_plain, document_secret_decrypted);
			}
		}
	}

	impl MessageLoop {
		pub fn new(t: usize, master_node_id: NodeId, old_nodes_set: BTreeSet<NodeId>, new_nodes_set: BTreeSet<NodeId>) -> Self {
			// generate admin key pair
			let admin_key_pair = Random.generate().unwrap();
			let admin_public = admin_key_pair.public().clone();

			// run initial generation session
			let gml = generate_key(t, old_nodes_set.clone());

			// compute original secret key
			let version = gml.nodes.values().nth(0).unwrap().key_storage.get(&Default::default()).unwrap().unwrap().versions[0].hash.clone();
			let original_key_pair = gml.compute_key_pair(t);

			// prepare sessions on all nodes
			let meta = ShareChangeSessionMeta {
				id: SessionId::default(),
				self_node_id: NodeId::default(),
				master_node_id: master_node_id,
			};
			let new_nodes = new_nodes_set.iter()
				.filter(|n| !old_nodes_set.contains(&n))
				.map(|new_node_id| {
					let new_node_cluster = Arc::new(DummyCluster::new(new_node_id.clone()));
					let new_node_key_storage = Arc::new(DummyKeyStorage::default());
					let new_node_session = create_session(meta.clone(), admin_public.clone(), new_node_id.clone(), new_node_cluster.clone(), new_node_key_storage.clone());
					new_node_cluster.add_nodes(new_nodes_set.iter().cloned());
					Node {
						cluster: new_node_cluster,
						key_storage: new_node_key_storage,
						session: new_node_session,
					}
				});
			let old_nodes = gml.nodes.into_iter().map(|gn| create_node(meta.clone(), admin_public.clone(), gn.1, &new_nodes_set));
			let nodes = old_nodes.chain(new_nodes).map(|n| (n.session.core.meta.self_node_id.clone(), n)).collect();

			let old_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&old_nodes_set)).unwrap();
			let new_set_signature = sign(admin_key_pair.secret(), &ordered_nodes_hash(&new_nodes_set)).unwrap();
			MessageLoop {
				admin_key_pair: admin_key_pair,
				original_key_pair: original_key_pair,
				version: version,
				old_nodes_set: old_nodes_set.clone(),
				new_nodes_set: new_nodes_set.clone(),
				old_set_signature: old_set_signature,
				new_set_signature: new_set_signature,
				nodes: nodes,
				queue: Default::default(),
			}
		}

		pub fn new_additional(master_node_id: NodeId, ml: MessageLoop, new_nodes_set: BTreeSet<NodeId>) -> Self {
			let version = ml.nodes.values().nth(0).unwrap().key_storage.get(&Default::default()).unwrap().unwrap().versions.last().unwrap().hash.clone();

			// prepare sessions on all nodes
			let meta = ShareChangeSessionMeta {
				id: SessionId::default(),
				self_node_id: NodeId::default(),
				master_node_id: master_node_id,
			};
			let old_nodes_set = ml.nodes.keys().cloned().collect();
			let nodes = ml.nodes.iter()
				.map(|(n, nd)| {
					let node_cluster = nd.cluster.clone();
					let node_key_storage = nd.key_storage.clone();
					let node_session = create_session(meta.clone(), ml.admin_key_pair.public().clone(), n.clone(), node_cluster.clone(), node_key_storage.clone());
					node_cluster.add_nodes(new_nodes_set.iter().cloned());
					(n.clone(), Node {
						cluster: node_cluster,
						key_storage: node_key_storage,
						session: node_session,
					})
				}).chain(new_nodes_set.difference(&old_nodes_set).map(|n| {
					let new_node_cluster = Arc::new(DummyCluster::new(n.clone()));
					let new_node_key_storage = Arc::new(DummyKeyStorage::default());
					let new_node_session = create_session(meta.clone(), ml.admin_key_pair.public().clone(), n.clone(), new_node_cluster.clone(), new_node_key_storage.clone());
					new_node_cluster.add_nodes(new_nodes_set.iter().cloned());
					(n.clone(), Node {
						cluster: new_node_cluster,
						key_storage: new_node_key_storage,
						session: new_node_session,
					})
				})).collect();

			let old_set_signature = sign(ml.admin_key_pair.secret(), &ordered_nodes_hash(&old_nodes_set)).unwrap();
			let new_set_signature = sign(ml.admin_key_pair.secret(), &ordered_nodes_hash(&new_nodes_set)).unwrap();
			MessageLoop {
				admin_key_pair: ml.admin_key_pair,
				original_key_pair: ml.original_key_pair,
				version: version,
				old_nodes_set: old_nodes_set.clone(),
				new_nodes_set: new_nodes_set.clone(),
				old_set_signature: old_set_signature,
				new_set_signature: new_set_signature,
				nodes: nodes,
				queue: Default::default(),
			}
		}

		pub fn update_signature(&mut self) {
			self.old_set_signature = sign(self.admin_key_pair.secret(), &ordered_nodes_hash(&self.old_nodes_set)).unwrap();
			self.new_set_signature = sign(self.admin_key_pair.secret(), &ordered_nodes_hash(&self.new_nodes_set)).unwrap();
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
				Message::ShareAdd(ref message) =>
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
	fn node_add_fails_if_nodes_removed() {
		let old_nodes_set = generate_nodes_ids(3);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let node_to_remove_id = old_nodes_set.iter().cloned().nth(1).unwrap();
		let mut new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(1)).collect();
		new_nodes_set.remove(&node_to_remove_id);
		let ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())
		).unwrap_err(), Error::ConsensusUnreachable);
	}

	#[test]
	fn node_add_fails_if_no_nodes_added() {
		let old_nodes_set = generate_nodes_ids(3);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let new_nodes_set = old_nodes_set.clone();
		let ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())
		).unwrap_err(), Error::ConsensusUnreachable);
	}

	#[test]
	fn node_add_fails_if_started_on_adding_node() {
		let old_nodes_set = generate_nodes_ids(3);
		let nodes_to_add_set = generate_nodes_ids(1);
		let master_node_id = nodes_to_add_set.iter().cloned().nth(0).unwrap();
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(nodes_to_add_set.into_iter()).collect();
		let ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())
		).unwrap_err(), Error::KeyStorage("key share is not found on master node".into()));
	}

	#[test]
	fn node_add_fails_if_initialized_twice() {
		let old_nodes_set = generate_nodes_ids(3);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(1)).collect();
		let ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set.clone()),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())
		), Ok(()));
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())
		), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn node_add_fails_if_started_without_signatures() {
		let old_nodes_set = generate_nodes_ids(3);
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(1)).collect();
		let ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());
		assert_eq!(ml.nodes[&master_node_id].session.initialize(None, None, None, None), Err(Error::InvalidMessage));
	}

	#[test]
	fn nodes_added_using_share_add() {
		let test_cases = vec![(3, 1), (3, 3)];
		for (n, nodes_to_add) in test_cases {
			// generate key && prepare ShareAdd sessions
			let old_nodes_set = generate_nodes_ids(n);
			let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(nodes_to_add)).collect();
			let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
			let mut ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());

			// initialize session on master node && run to completion
			ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
				Some(ml.old_set_signature.clone()),
				Some(ml.new_set_signature.clone())).unwrap();
			ml.run();

			// check that session has completed on all nodes
			assert!(ml.nodes.values().all(|n| n.session.is_finished()));
			
			// check that secret is still the same as before adding the share
			check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes.iter().map(|(k, v)| (k.clone(), v.key_storage.clone())).collect());
		}
	}

	#[test]
	fn nodes_added_using_share_add_with_isolated_nodes() {
		let (n, nodes_to_add) = (3, 3);

		// generate key && prepare ShareAdd sessions
		let old_nodes_set = generate_nodes_ids(n);
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(nodes_to_add)).collect();
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let isolated_node_id = old_nodes_set.iter().cloned().nth(1).unwrap();
		let mut ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set, new_nodes_set.clone());

		// now let's isolate 1 of 3 nodes owning key share
		ml.nodes.remove(&isolated_node_id);
		ml.old_nodes_set.remove(&isolated_node_id);
		ml.new_nodes_set.remove(&isolated_node_id);
		for (_, node) in ml.nodes.iter_mut() {
			node.cluster.remove_node(&isolated_node_id);
		}
		ml.update_signature();

		// initialize session on master node && run to completion
		ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())).unwrap();
		ml.run();

		// check that session has completed on all nodes
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));
		
		// check that secret is still the same as before adding the share
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes
			.iter()
			.map(|(k, v)| (k.clone(), v.key_storage.clone()))
			.collect());
	}

	#[test]
	fn nodes_add_to_the_node_with_obsolete_version() {
		let (n, nodes_to_add) = (3, 3);

		// generate key (2-of-3) && prepare ShareAdd sessions
		let old_nodes_set = generate_nodes_ids(n);
		let newest_nodes_set = generate_nodes_ids(nodes_to_add);
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(newest_nodes_set.clone()).collect();
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let isolated_node_id = old_nodes_set.iter().cloned().nth(1).unwrap();
		let oldest_nodes_set: BTreeSet<_> = old_nodes_set.iter().filter(|n| **n != isolated_node_id).cloned().collect();
		let mut ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set.clone(), new_nodes_set.clone());
		let isolated_key_storage = ml.nodes[&isolated_node_id].key_storage.clone();

		// now let's isolate 1 of 3 nodes owning key share
		ml.nodes.remove(&isolated_node_id);
		ml.old_nodes_set.remove(&isolated_node_id);
		ml.new_nodes_set.remove(&isolated_node_id);
		for (_, node) in ml.nodes.iter_mut() {
			node.cluster.remove_node(&isolated_node_id);
		}
		ml.update_signature();

		// initialize session on master node && run to completion (2-of-5)
		ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())).unwrap();
		ml.run();

		// now let's add back old node so that key becames 2-of-6
		let new_nodes_set: BTreeSet<_> = ml.nodes.keys().cloned().chain(::std::iter::once(isolated_node_id.clone())).collect();
		let mut ml = MessageLoop::new_additional(master_node_id.clone(), ml, new_nodes_set.clone());
		ml.nodes.get_mut(&isolated_node_id).unwrap().key_storage = isolated_key_storage.clone();
		ml.nodes.get_mut(&isolated_node_id).unwrap().session.core.key_share = isolated_key_storage.get(&Default::default()).unwrap();
		ml.nodes.get_mut(&isolated_node_id).unwrap().session.core.key_storage = isolated_key_storage;

		// initialize session on master node && run to completion (2-of65)
		ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())).unwrap();
		ml.run();

		// check that session has completed on all nodes
		assert!(ml.nodes.values().all(|n| n.session.is_finished()));

		// check that secret is still the same as before adding the share
		check_secret_is_preserved(ml.original_key_pair.clone(), ml.nodes
			.iter()
			.map(|(k, v)| (k.clone(), v.key_storage.clone()))
			.collect());
		
		// check that all oldest nodes have versions A, B, C
		// isolated node has version A, C
		// new nodes have versions B, C
		let oldest_key_share = ml.nodes[oldest_nodes_set.iter().nth(0).unwrap()].key_storage.get(&Default::default()).unwrap().unwrap();
		debug_assert_eq!(oldest_key_share.versions.len(), 3);
		let version_a = oldest_key_share.versions[0].hash.clone();
		let version_b = oldest_key_share.versions[1].hash.clone();
		let version_c = oldest_key_share.versions[2].hash.clone();
		debug_assert!(version_a != version_b && version_b != version_c);

		debug_assert!(oldest_nodes_set.iter().all(|n| vec![version_a.clone(), version_b.clone(), version_c.clone()] ==
			ml.nodes[n].key_storage.get(&Default::default()).unwrap().unwrap().versions.iter().map(|v| v.hash.clone()).collect::<Vec<_>>()));
		debug_assert!(::std::iter::once(&isolated_node_id).all(|n| vec![version_a.clone(), version_c.clone()] ==
			ml.nodes[n].key_storage.get(&Default::default()).unwrap().unwrap().versions.iter().map(|v| v.hash.clone()).collect::<Vec<_>>()));
		debug_assert!(newest_nodes_set.iter().all(|n| vec![version_b.clone(), version_c.clone()] ==
			ml.nodes[n].key_storage.get(&Default::default()).unwrap().unwrap().versions.iter().map(|v| v.hash.clone()).collect::<Vec<_>>()));
	}

	#[test]
	fn nodes_add_fails_when_not_enough_share_owners_are_connected() {
		let (n, nodes_to_add) = (3, 3);

		// generate key (2-of-3) && prepare ShareAdd sessions
		let old_nodes_set = generate_nodes_ids(n);
		let new_nodes_set: BTreeSet<_> = old_nodes_set.clone().into_iter().chain(generate_nodes_ids(nodes_to_add)).collect();
		let master_node_id = old_nodes_set.iter().cloned().nth(0).unwrap();
		let isolated_node_id1 = old_nodes_set.iter().cloned().nth(1).unwrap();
		let isolated_node_id2 = old_nodes_set.iter().cloned().nth(2).unwrap();
		let mut ml = MessageLoop::new(1, master_node_id.clone(), old_nodes_set.clone(), new_nodes_set.clone());

		// now let's isolate 2 of 3 nodes owning key share
		ml.nodes.remove(&isolated_node_id1);
		ml.nodes.remove(&isolated_node_id2);
		ml.old_nodes_set.remove(&isolated_node_id1);
		ml.new_nodes_set.remove(&isolated_node_id1);
		ml.old_nodes_set.remove(&isolated_node_id2);
		ml.new_nodes_set.remove(&isolated_node_id2);
		for (_, node) in ml.nodes.iter_mut() {
			node.cluster.remove_node(&isolated_node_id1);
			node.cluster.remove_node(&isolated_node_id2);
		}
		ml.update_signature();

		// initialize session on master node && run to completion (2-of-5)
		assert_eq!(ml.nodes[&master_node_id].session.initialize(Some(ml.version), Some(new_nodes_set),
			Some(ml.old_set_signature.clone()),
			Some(ml.new_set_signature.clone())).map(|_| ()), Err(Error::ConsensusUnreachable));
	}
}
