// TODO: when anys session starts on the node, which doesn't have key share, do not fail session, but delegate it
// TODO: add key share version to db? and never overwrite key share data, but add a new version. Version must be agreed on decryption/signing session start
// TODO: add polynom1 to database. otherwise share add is not possible

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
use key_server_cluster::{Error, NodeId, SessionMeta, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::math;
use key_server_cluster::message::{Message, ShareAddMessage, InitializeShareAddSession, ConfirmShareAddInitialization,
	KeyShareCommon, NewAbsoluteTermShare, NewKeysDissemination, ShareAddError};

/// Share addition session API.
pub trait Session: Send + Sync + 'static {
}

/// Share addition session transport.
pub trait SessionTransport {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: ShareAddMessage) -> Result<(), Error>;
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
	/// Original key share (for old nodes only).
	pub key_share: Option<DocumentKeyShare>,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Key storage.
	pub key_storage: Arc<KeyStorage>,
}

/// Mutable session data.
#[derive(Debug)]
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Nodes-specific data.
	pub nodes: BTreeMap<NodeId, NodeData>,
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
	pub id_number: Secret,
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
	/// Every node starts in this state.
	WaitingForInitialization,
	/// Waiting for initialization confirmation.
	WaitingForInitializationConfirm,
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
	/// Key share (some on existing nodes only).
	pub key_share: Option<DocumentKeyShare>,
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
				nodes: BTreeMap::new(),
				refreshed_polynom1_sum: None,
				key_share_author: None,
				key_share_common_point: None,
				key_share_encrypted_point: None,
			}),
		})
	}

	/// Initialize share add session on master node.
	pub fn initialize(&self, nodes_to_add: BTreeSet<NodeId>) -> Result<(), Error> {
		debug_assert_eq!(self.core.meta.self_node_id, self.core.meta.master_node_id);

		// all nodes_to_add must be new nodes for the session
		let old_key_share = self.core.key_share.as_ref()
			.expect("initialize is called on master node; master node owns its own key share; qed");
		if nodes_to_add.iter().any(|n| old_key_share.id_numbers.contains_key(n)) {
			return Err(Error::InvalidNodesConfiguration);
		}

		let mut data = self.data.lock();

		// check state
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		let nodes_to_add = nodes_to_add.into_iter()
			.map(|n| math::generate_random_scalar().map(|nn| (n, nn)))
			.collect::<Result<BTreeMap<_, _>, _>>()?;
		data.state = SessionState::WaitingForInitializationConfirm;
		data.nodes = old_key_share.id_numbers.iter().chain(nodes_to_add.iter())
			.map(|(n, nn)| (n.clone(), NodeData::new(nn.clone(), nodes_to_add.contains_key(n))))
			.collect();
		data.nodes.get_mut(&self.core.meta.self_node_id)
			.expect("initialize is called on master node; master node owns its own key share; all key shares are in data.nodes; qed")
			.is_initialization_confirmed = true;

		// send initialization request to every node
		for node in data.nodes.keys().filter(|n| **n != self.core.meta.self_node_id) {
			self.core.transport.send(node, ShareAddMessage::InitializeShareAddSession(InitializeShareAddSession {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
				threshold: self.core.meta.threshold,
				nodes: data.nodes.iter().map(|(n, nd)| (n.clone().into(), nd.id_number.clone().into())).collect(),
				new_nodes: nodes_to_add.keys().cloned().map(Into::into).collect(),
			}))?;
		}

		Ok(())
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &ShareAddMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&ShareAddMessage::InitializeShareAddSession(ref message) =>
				self.on_initialize_session(sender, message),
			&ShareAddMessage::ConfirmShareAddInitialization(ref message) =>
				self.on_confirm_initialization(sender, message),
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

	/// When initialization request is received.
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
	}

	/// When common key share data is received by new node.
	pub fn on_common_key_share_data(&self, sender: &NodeId, message: &KeyShareCommon) -> Result<(), Error> {
		debug_assert!(self.core.meta.id == *message.session);
		debug_assert!(self.core.sub_session == *message.sub_session);
		debug_assert!(sender != &self.core.meta.self_node_id);

		// only master can send this message
		if sender != &self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		let mut data = self.data.lock();

		// check state
		if data.state == SessionState::WaitingForInitializationConfirm {
			data.state = SessionState::WaitingForAbsoluteTermShare;
		} else if data.state != SessionState::WaitingForAbsoluteTermShare {
			return Err(Error::InvalidStateForRequest);
		}

		// only new nodes are waiting for absolute term share
		if !data.nodes[&self.core.meta.self_node_id].is_new_node {
			return Err(Error::InvalidMessage);
		}

		// we only expect this message once
		if data.key_share_author.is_some() || data.key_share_common_point.is_some() || data.key_share_encrypted_point.is_some() {
			return Err(Error::InvalidStateForRequest);
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
		if data.state == SessionState::WaitingForInitializationConfirm {
			data.state = SessionState::WaitingForAbsoluteTermShare;
		} else if data.state != SessionState::WaitingForAbsoluteTermShare {
			return Err(Error::InvalidStateForRequest);
		}

		// only new nodes are waiting for absolute term share
		if !data.nodes[&self.core.meta.self_node_id].is_new_node {
			return Err(Error::InvalidMessage);
		}

		// update node data
		{
			let node_data = data.nodes.get_mut(sender).ok_or(Error::InvalidMessage)?;
			if node_data.absolute_term_share.is_some() {
				return Err(Error::InvalidStateForRequest);
			}

			node_data.absolute_term_share = Some(message.absolute_term_share.clone().into());
		}

		// if we haven't received shares from all old nodes => wait for more
		if data.nodes.values().any(|nd| !nd.is_new_node && nd.absolute_term_share.is_none()) {
			return Ok(());
		}

		// all old nodes have sent us its shares => generate/calculate secret polynom
		let refreshed_polynom1_sum = {
			let absolute_term_shares = data.nodes.values().filter_map(|nd| nd.absolute_term_share.as_ref());
			generate_refreshed_polynoms_for_new_nodes(absolute_term_shares, self.core.meta.threshold)?
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
		if data.state == SessionState::WaitingForInitializationConfirm {
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
		{
			let node_data = data.nodes.get_mut(&sender).ok_or(Error::InvalidMessage)?;
			if node_data.refreshed_secret1.is_some() || node_data.refreshed_publics.is_some() {
				return Err(Error::InvalidStateForRequest);
			}

			node_data.refreshed_secret1 = Some(message.refreshed_secret1.clone().into());
			node_data.refreshed_publics = Some(message.refreshed_publics.iter().cloned().map(Into::into).collect());
		}

		// receiving this message from master node on old node means that initialization has completed => disseminate our own keys
		if !data.nodes[&self.core.meta.self_node_id].is_new_node && sender == &self.core.meta.master_node_id {
			Self::disseminate_absolute_term_shares(&self.core, &mut *data)?;
			Self::disseminate_keys(&self.core, &mut *data)?;
		}

		// check if we have received keys from every other node
		if data.nodes.iter().any(|(node_id, node_data)| node_id != &self.core.meta.self_node_id &&
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

	/// Disseminate absolute term of polynom1 data.
	fn disseminate_absolute_term_shares(core: &SessionCore<T>, data: &mut SessionData) -> Result<(), Error> {
		// compute/generate refreshed polynom1
		let old_key_share = core.key_share.as_ref()
			.expect("disseminate_absolute_term_shares is only called on old nodes; key_share is filled in initialization phase on old nodes; qed");
		let num_new_nodes = data.nodes.values().filter(|nd| nd.is_new_node).count();
		let (absolute_term_shares, refreshed_polynom1_sum) = generate_refreshed_polynoms_for_existing_nodes(
			num_new_nodes, core.meta.threshold, &old_key_share.polynom1)?;
		data.refreshed_polynom1_sum = Some(refreshed_polynom1_sum);

		// send absolute term share to every new node
		for (i, new_node) in data.nodes.iter().filter(|&(_, nd)| nd.is_new_node).map(|(n, _)| n).enumerate() {
			core.transport.send(new_node, ShareAddMessage::NewAbsoluteTermShare(NewAbsoluteTermShare {
				session: core.meta.id.clone().into(),
				sub_session: core.sub_session.clone().into(),
				session_nonce: core.nonce,
				absolute_term_share: absolute_term_shares[i].clone().into(),
			}))?;
		}

		Ok(())
	}

	/// Send common share data to evey new node.
	fn disseminate_common_share_data(core: &SessionCore<T>, data: &SessionData) -> Result<(), Error> {
		let old_key_share = core.key_share.as_ref()
			.expect("disseminate_common_share_data is only called on master node; key_share is filled in initialization phase on master node; qed");
		for new_node in data.nodes.iter().filter(|&(_, nd)| nd.is_new_node).map(|(n, _)| n) {
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
	fn disseminate_keys(core: &SessionCore<T>, data: &mut SessionData) -> Result<(), Error> {
		// send required messages
		let refreshed_polynom1_sum = data.refreshed_polynom1_sum.as_ref()
			.expect("disseminate_keys is only called after generating refreshed_polynom1_sum; qed");
println!("=== threshold = {} refreshed_polynom1_sum = {:?}", core.meta.threshold, refreshed_polynom1_sum);
		let refreshed_publics = (0..core.meta.threshold+1)
			.map(|i| math::compute_public_share(&refreshed_polynom1_sum[i]))
			.collect::<Result<Vec<_>, _>>()?;

		// send calculated values
		for (node, node_number) in data.nodes.iter().filter(|&(n, _)| n != &core.meta.self_node_id).map(|(n, nd)| (n, &nd.id_number)) {
			// also send keys to every other node
			let refreshed_secret1 = math::compute_polynom(refreshed_polynom1_sum, node_number)?;
			core.transport.send(node, ShareAddMessage::NewKeysDissemination(NewKeysDissemination {
				session: core.meta.id.clone().into(),
				sub_session: core.sub_session.clone().into(),
				session_nonce: core.nonce,
				refreshed_secret1: refreshed_secret1.into(),
				refreshed_publics: refreshed_publics.iter().cloned().map(Into::into).collect(),
			}))?;
		}

		Ok(())
	}

	/// Verify received keys values.
	fn verify_keys(core: &SessionCore<T>, data: &mut SessionData) -> Result<(), Error> {
		let number_id = &data.nodes[&core.meta.self_node_id].id_number;
		for node_data in data.nodes.iter().filter(|&(n, _)| n != &core.meta.self_node_id).map(|(_, nd)| nd) {
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
	fn complete_session(core: &SessionCore<T>, data: &mut SessionData) -> Result<(), Error> {
		// compose updated key share
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
			id_numbers: data.nodes.iter().map(|(node_id, node_data)| (node_id.clone(), node_data.id_number.clone())).collect(),
			polynom1: data.refreshed_polynom1_sum.clone().expect("this field is filled during KRD; session is completed after KRD; qed"),
			secret_share: math::compute_secret_share(data.nodes.values()
				.filter_map(|nd| nd.refreshed_secret1.as_ref()))?,
		};

		// save encrypted data to the key storage
		data.state = SessionState::Finished;
		core.key_storage.insert(core.meta.id.clone(), refreshed_key_share.clone())
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

impl NodeData {
	/// Create new node data.
	pub fn new(id_number: Secret, is_new_node: bool) -> Self {
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
	use key_server_cluster::share_change_session::ShareAddTransport;
	use super::{SessionImpl, SessionParams, SessionTransport};

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl<ShareAddTransport>,
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
				let session = SessionImpl::new_nested(SessionParams {
					meta: meta,
					sub_session: sub_session.clone(),
					transport: ShareAddTransport::new(session_id.clone(), 1, cluster.clone()),
					key_storage: nd.key_storage.clone(),
					nonce: 1,
					key_share: Some(key_storage.get(&key_id).unwrap()),
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
				let session = SessionImpl::new_nested(SessionParams {
					meta: meta,
					sub_session: sub_session.clone(),
					transport: ShareAddTransport::new(session_id.clone(), 1, cluster.clone()),
					key_storage: key_storage.clone(),
					nonce: 1,
					key_share: None,
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
println!("=== {:?}", message);
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
					Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeShareAddMessage(ref message)) =>
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
	fn share_add_session_works() {
		// initial 2-of-3 session
		let gml = generate_key(1, 3);
		let gml_nodes: BTreeSet<_> = gml.nodes.keys().cloned().collect();
		let key_id = gml.session_id.clone();
		let master = gml.nodes.keys().cloned().nth(0).unwrap();
		let share0 = gml.nodes.values().nth(0).unwrap().key_storage.get(&key_id).unwrap();
		let share1 = gml.nodes.values().nth(1).unwrap().key_storage.get(&key_id).unwrap();
		let share2 = gml.nodes.values().nth(2).unwrap().key_storage.get(&key_id).unwrap();
		let joint_secret = math::compute_joint_secret([share0.polynom1[0].clone(), share1.polynom1[0].clone(), share2.polynom1[0].clone()].iter()).unwrap();
		let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();

		// insert 1 node so that it becames 2-of-4 session
		let mut ml = MessageLoop::new(gml, 1, 1);
		ml.nodes[&master].session.initialize(ml.nodes.keys().cloned().filter(|n| !gml_nodes.contains(n)).collect());
		ml.run();

		// try to recover secret for every possible combination of nodes && check that secret is the same
		let document_secret_plain = math::generate_random_point().unwrap();
		for n1 in 0..4 {
			for n2 in n1+1..4 {
				let share1 = ml.nodes.values().nth(n1).unwrap().key_storage.get(&key_id).unwrap();
				let share2 = ml.nodes.values().nth(n2).unwrap().key_storage.get(&key_id).unwrap();
				let id_number1 = share1.id_numbers[ml.nodes.keys().nth(n1).unwrap()].clone();
				let id_number2 = share1.id_numbers[ml.nodes.keys().nth(n2).unwrap()].clone();

				// now encrypt and decrypt data
				let (document_secret_decrypted, document_secret_decrypted_test) =
					math::tests::do_encryption_and_decryption(1,
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
