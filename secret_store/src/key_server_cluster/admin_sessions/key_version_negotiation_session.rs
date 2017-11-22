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
use ethkey::Secret;
use parking_lot::{Mutex, Condvar};
use key_server_cluster::{Error, SessionId, NodeId, DocumentKeyShare};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::{SessionIdWithSubSession, ClusterSession};
use key_server_cluster::decryption_session::SessionImpl as DecryptionSession;
use key_server_cluster::signing_session::SessionImpl as SigningSession;
use key_server_cluster::message::{Message, KeyVersionNegotiationMessage, RequestKeyVersions, KeyVersions};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

// TODO: optimizations: change sessions so that versions are sent by chunks.
/// Number of versions sent in single message.
const VERSIONS_PER_MESSAGE: usize = 32;

/// Key version negotiation transport.
pub trait SessionTransport {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error>;
}

/// Key version negotiation result computer.
pub trait SessionResultComputer: Send + Sync {
	/// Compute result of session, if possible.
	fn compute_result(&self, threshold: Option<usize>, confirmations: &BTreeSet<NodeId>, versions: &BTreeMap<H256, BTreeSet<NodeId>>) -> Option<Result<(H256, NodeId), Error>>;
}

/// Key discovery session API.
pub struct SessionImpl<T: SessionTransport> {
	/// Session core.
	core: SessionCore<T>,
	/// Session data.
	data: Mutex<SessionData>,
}

/// Action after key version is negotiated.
#[derive(Clone)]
pub enum ContinueAction {
	/// Decryption session + is_shadow_decryption.
	Decrypt(Arc<DecryptionSession>, bool),
	/// Signing session + message hash.
	Sign(Arc<SigningSession>, H256),
}

/// Immutable session data.
struct SessionCore<T: SessionTransport> {
	/// Session meta.
	pub meta: ShareChangeSessionMeta,
	/// Sub-session id.
	pub sub_session: Secret,
	/// Key share.
	pub key_share: Option<DocumentKeyShare>,
	/// Session result computer.
	pub result_computer: Arc<SessionResultComputer>,
	/// Session transport.
	pub transport: T,
	/// Session nonce.
	pub nonce: u64,
	/// SessionImpl completion condvar.
	pub completed: Condvar,
}

/// Mutable session data.
struct SessionData {
	/// Session state.
	pub state: SessionState,
	/// Initialization confirmations.
	pub confirmations: Option<BTreeSet<NodeId>>,
	/// Key threshold.
	pub threshold: Option<usize>,
	/// { Version => Nodes }
	pub versions: Option<BTreeMap<H256, BTreeSet<NodeId>>>,
	/// Session result.
	pub result: Option<Result<(H256, NodeId), Error>>,
	/// Continue action.
	pub continue_with: Option<ContinueAction>,
}

/// SessionImpl creation parameters
pub struct SessionParams<T: SessionTransport> {
	/// Session meta.
	pub meta: ShareChangeSessionMeta,
	/// Sub-session id.
	pub sub_session: Secret,
	/// Key share.
	pub key_share: Option<DocumentKeyShare>,
	/// Session result computer.
	pub result_computer: Arc<SessionResultComputer>,
	/// Session transport to communicate to other cluster nodes.
	pub transport: T,
	/// Session nonce.
	pub nonce: u64,
}

/// Signing session state.
#[derive(Debug, PartialEq)]
enum SessionState {
	/// Waiting for initialization.
	WaitingForInitialization,
	/// Waiting for responses.
	WaitingForResponses,
	/// Session is completed.
	Finished,
}

/// Isolated session transport.
pub struct IsolatedSessionTransport {
	/// Cluster.
	pub cluster: Arc<Cluster>,
	/// Key id.
	pub key_id: SessionId,
	/// Sub session id.
	pub sub_session: Secret,
	/// Session-level nonce.
	pub nonce: u64,
}

/// Fastest session result computer. Computes first possible version that can be recovered on this node.
/// If there's no such version, selects version with the most support.
pub struct FastestResultComputer {
	/// This node id.
	self_node_id: NodeId,
	/// Threshold (if known).
	threshold: Option<usize>,
}

/// Selects version with most support, waiting for responses from all nodes.
pub struct LargestSupportResultComputer;

impl<T> SessionImpl<T> where T: SessionTransport {
	/// Create new session.
	pub fn new(params: SessionParams<T>) -> Self {
		SessionImpl {
			core: SessionCore {
				meta: params.meta,
				sub_session: params.sub_session,
				key_share: params.key_share,
				result_computer: params.result_computer,
				transport: params.transport,
				nonce: params.nonce,
				completed: Condvar::new(),
			},
			data: Mutex::new(SessionData {
				state: SessionState::WaitingForInitialization,
				confirmations: None,
				threshold: None,
				versions: None,
				result: None,
				continue_with: None,
			})
		}
	}

	/// Return session meta.
	pub fn meta(&self) -> &ShareChangeSessionMeta {
		&self.core.meta
	}

	/// Return key threshold.
	pub fn key_threshold(&self) -> Result<usize, Error> {
		Ok(self.data.lock().threshold.clone().ok_or(Error::InvalidStateForRequest)?)
	}

	/// Return result computer reference.
	pub fn version_holders(&self, version: &H256) -> Result<BTreeSet<NodeId>, Error> {
		Ok(self.data.lock().versions.as_ref().ok_or(Error::InvalidStateForRequest)?
			.get(version).ok_or(Error::KeyStorage("key version not found".into()))?
			.clone())
	}

	/// Set continue action.
	pub fn set_continue_action(&self, action: ContinueAction) {
		self.data.lock().continue_with = Some(action);
	}

	/// Get continue action.
	pub fn continue_action(&self) -> Option<ContinueAction> {
		self.data.lock().continue_with.clone()
	}

	/// Wait for session completion.
	pub fn wait(&self) -> Result<(H256, NodeId), Error> {
		let mut data = self.data.lock();
		if !data.result.is_some() {
			self.core.completed.wait(&mut data);
		}

		data.result.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when result.is_some(); qed")
			.clone()
	}

	/// Initialize session.
	pub fn initialize(&self, connected_nodes: BTreeSet<NodeId>) -> Result<(), Error> {
		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// update state
		let mut confirmations = connected_nodes;
		let mut versions: BTreeMap<H256, BTreeSet<NodeId>> = BTreeMap::new();
		let received_own_confirmation = confirmations.remove(&self.core.meta.self_node_id);
		if received_own_confirmation {
			if let Some(key_share) = self.core.key_share.as_ref() {
				for version in &key_share.versions {
					versions.entry(version.hash.clone())
						.or_insert_with(Default::default)
						.insert(self.core.meta.self_node_id.clone());
				}
			}
		}

		// update state
		let no_confirmations_required = confirmations.is_empty();
		data.state = SessionState::WaitingForResponses;
		data.confirmations = Some(confirmations);
		data.versions = Some(versions);

		// try to complete session
		Self::try_complete(&self.core, &mut *data);
		if no_confirmations_required && data.state != SessionState::Finished {
			return Err(Error::ConsensusUnreachable);
		} else if data.state == SessionState::Finished {
			return Ok(());
		}

		// send requests
		let confirmations = data.confirmations.as_ref().expect("dilled couple of lines above; qed");
		for connected_node in confirmations {
			self.core.transport.send(connected_node, KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
			}))?;
		}

		Ok(())
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &KeyVersionNegotiationMessage) -> Result<(), Error> {
		if self.core.nonce != message.session_nonce() {
			return Err(Error::ReplayProtection);
		}

		match message {
			&KeyVersionNegotiationMessage::RequestKeyVersions(ref message) =>
				self.on_key_versions_request(sender, message),
			&KeyVersionNegotiationMessage::KeyVersions(ref message) =>
				self.on_key_versions(sender, message),
			&KeyVersionNegotiationMessage::KeyVersionsError(ref message) => {
				self.on_session_error(sender, Error::Io(message.error.clone()));
				Ok(())
			},
		}
	}

	/// Process key versions request.
	pub fn on_key_versions_request(&self, sender: &NodeId, _message: &RequestKeyVersions) -> Result<(), Error> {
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check message
		if *sender != self.core.meta.master_node_id {
			return Err(Error::InvalidMessage);
		}

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForInitialization {
			return Err(Error::InvalidStateForRequest);
		}

		// send response
		self.core.transport.send(sender, KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: self.core.meta.id.clone().into(),
			sub_session: self.core.sub_session.clone().into(),
			session_nonce: self.core.nonce,
			threshold: self.core.key_share.as_ref().map(|key_share| key_share.threshold),
			versions: self.core.key_share.as_ref().map(|key_share|
				key_share.versions.iter().rev()
					.filter(|v| v.id_numbers.contains_key(sender))
					.chain(key_share.versions.iter().rev().filter(|v| !v.id_numbers.contains_key(sender)))
					.map(|v| v.hash.clone().into())
					.take(VERSIONS_PER_MESSAGE)
					.collect())
				.unwrap_or_else(|| Default::default())
		}))?;

		// update state
		data.state = SessionState::Finished;

		Ok(())
	}

	/// Process key versions response.
	pub fn on_key_versions(&self, sender: &NodeId, message: &KeyVersions) -> Result<(), Error> {
		debug_assert!(sender != &self.core.meta.self_node_id);

		// check state
		let mut data = self.data.lock();
		if data.state != SessionState::WaitingForResponses && data.state != SessionState::Finished {
			return Err(Error::InvalidStateForRequest);
		}
		let reason = "this field is filled on master node when initializing; this is initialized master node; qed";
		if !data.confirmations.as_mut().expect(reason).remove(sender) {
			return Err(Error::InvalidMessage);
		}

		// remember versions that sender have
		{
			match message.threshold.clone() {
				Some(threshold) if data.threshold.is_none() => {
					data.threshold = Some(threshold);
				},
				Some(threshold) if data.threshold.as_ref() == Some(&threshold) => (),
				Some(_) => return Err(Error::InvalidMessage),
				None if message.versions.is_empty() => (),
				None => return Err(Error::InvalidMessage),
			}

			let versions = data.versions.as_mut().expect(reason);
			for version in &message.versions {
				versions.entry(version.clone().into())
					.or_insert_with(Default::default)
					.insert(sender.clone());
			}
		}

		// try to compute result
		if data.state != SessionState::Finished {
			Self::try_complete(&self.core, &mut *data);
		}

		Ok(())
	}

	/// Try to complete result && finish session.
	fn try_complete(core: &SessionCore<T>, data: &mut SessionData) {
		let reason = "this field is filled on master node when initializing; try_complete is only called on initialized master node; qed";
		let confirmations = data.confirmations.as_ref().expect(reason);
		let versions = data.versions.as_ref().expect(reason);
		if let Some(result) = core.result_computer.compute_result(data.threshold.clone(), confirmations, versions) {
			data.state = SessionState::Finished;
			data.result = Some(result);
			core.completed.notify_all();
		}
	}
}

impl<T> ClusterSession for SessionImpl<T> where T: SessionTransport {
	type Id = SessionIdWithSubSession;

	fn type_name() -> &'static str {
		"version negotiation"
	}

	fn id(&self) -> SessionIdWithSubSession {
		SessionIdWithSubSession::new(self.core.meta.id.clone(), self.core.sub_session.clone())
	}

	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		if data.confirmations.is_some() {
			data.confirmations.as_mut().expect("checked a line above; qed").clear();
			Self::try_complete(&self.core, &mut *data);
			if data.state != SessionState::Finished {
				warn!("{}: key version negotiation session failed with timeout", self.core.meta.self_node_id);

				data.result = Some(Err(Error::ConsensusUnreachable));
				self.core.completed.notify_all();
			}
		}
	}

	fn on_node_timeout(&self, node: &NodeId) {
		self.on_session_error(node, Error::NodeDisconnected)
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		let mut data = self.data.lock();

		if data.confirmations.is_some() {
			let is_waiting_for_confirmation = data.confirmations.as_mut().expect("checked a line above; qed").remove(node);
			if is_waiting_for_confirmation {
				Self::try_complete(&self.core, &mut *data);
				if data.state != SessionState::Finished {
					warn!("{}: key version negotiation session failed because {} connection has timeouted", self.core.meta.self_node_id, node);

					data.state = SessionState::Finished;
					data.result = Some(Err(error));
					self.core.completed.notify_all();
				}
			}
		}
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		match *message {
			Message::KeyVersionNegotiation(ref message) => self.process_message(sender, message),
			_ => unreachable!("cluster checks message to be correct before passing; qed"),
		}
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::KeyVersionNegotiation(message))
	}
}

impl FastestResultComputer {
	pub fn new(self_node_id: NodeId, key_share: Option<&DocumentKeyShare>) -> Self {
		let threshold = key_share.map(|ks| ks.threshold);
		FastestResultComputer {
			self_node_id: self_node_id,
			threshold: threshold,
		}
	}}

impl SessionResultComputer for FastestResultComputer {
	fn compute_result(&self, threshold: Option<usize>, confirmations: &BTreeSet<NodeId>, versions: &BTreeMap<H256, BTreeSet<NodeId>>) -> Option<Result<(H256, NodeId), Error>> {
		match self.threshold.or(threshold) {
			// if we have key share on this node
			Some(threshold) => {
				// select version this node have, with enough participants
				let has_key_share = self.threshold.is_some();
				let version = versions.iter().find(|&(_, ref n)| !has_key_share || n.contains(&self.self_node_id) && n.len() >= threshold + 1);
				// if there's no such version, wait for more confirmations
				match version {
					Some((version, nodes)) => Some(Ok((version.clone(), if has_key_share { self.self_node_id.clone() } else { nodes.iter().cloned().nth(0)
						.expect("version is only inserted when there's at least one owner; qed") }))),
					None if !confirmations.is_empty() => None,
					// otherwise - try to find any version
					None => Some(versions.iter()
						.find(|&(_, ref n)| n.len() >= threshold + 1)
						.map(|(version, nodes)| Ok((version.clone(), nodes.iter().cloned().nth(0)
							.expect("version is only inserted when there's at least one owner; qed"))))
						.unwrap_or(Err(Error::ConsensusUnreachable))),
				}
			},
			// if we do not have share, then wait for all confirmations
			None if !confirmations.is_empty() => None,
			// ...and select version with largest support
			None => Some(versions.iter()
				.max_by_key(|&(_, ref n)| n.len())
				.map(|(version, nodes)| Ok((version.clone(), nodes.iter().cloned().nth(0)
					.expect("version is only inserted when there's at least one owner; qed"))))
				.unwrap_or(Err(Error::ConsensusUnreachable))),
		}
	}
}

impl SessionResultComputer for LargestSupportResultComputer {
	fn compute_result(&self, _threshold: Option<usize>, confirmations: &BTreeSet<NodeId>, versions: &BTreeMap<H256, BTreeSet<NodeId>>) -> Option<Result<(H256, NodeId), Error>> {
		if !confirmations.is_empty() {
			return None;
		}

		versions.iter()
			.max_by_key(|&(_, ref n)| n.len())
			.map(|(version, nodes)| Ok((version.clone(), nodes.iter().cloned().nth(0)
				.expect("version is only inserted when there's at least one owner; qed"))))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::collections::{VecDeque, BTreeMap, BTreeSet};
	use key_server_cluster::{NodeId, SessionId, Error, KeyStorage, DummyKeyStorage, DocumentKeyShare, DocumentKeyShareVersion};
	use key_server_cluster::math;
	use key_server_cluster::cluster::Cluster;
	use key_server_cluster::cluster::tests::DummyCluster;
	use key_server_cluster::admin_sessions::ShareChangeSessionMeta;
	use key_server_cluster::message::{Message, KeyVersionNegotiationMessage, RequestKeyVersions, KeyVersions};
	use super::{SessionImpl, SessionTransport, SessionParams, FastestResultComputer, SessionState};

	struct DummyTransport {
		cluster: Arc<DummyCluster>,
	}

	impl SessionTransport for DummyTransport {
		fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error> {
			self.cluster.send(node, Message::KeyVersionNegotiation(message))
		}
	}

	struct Node {
		pub cluster: Arc<DummyCluster>,
		pub key_storage: Arc<DummyKeyStorage>,
		pub session: SessionImpl<DummyTransport>,
	}

	struct MessageLoop {
		pub session_id: SessionId,
		pub nodes: BTreeMap<NodeId, Node>,
		pub queue: VecDeque<(NodeId, NodeId, Message)>,
	}

	impl MessageLoop {
		pub fn prepare_nodes(nodes_num: usize) -> BTreeMap<NodeId, Arc<DummyKeyStorage>> {
			(0..nodes_num).map(|_| (math::generate_random_point().unwrap(),
				Arc::new(DummyKeyStorage::default()))).collect()
		}

		pub fn empty(nodes_num: usize) -> Self {
			Self::new(Self::prepare_nodes(nodes_num))
		}

		pub fn new(nodes: BTreeMap<NodeId, Arc<DummyKeyStorage>>) -> Self {
			let master_node_id = nodes.keys().cloned().nth(0).unwrap();
			let sub_sesion = math::generate_random_scalar().unwrap();
			let all_nodes_ids: BTreeSet<_> = nodes.keys().cloned().collect();
			MessageLoop {
				session_id: Default::default(),
				nodes: nodes.iter().map(|(node_id, key_storage)| {
					let cluster = Arc::new(DummyCluster::new(node_id.clone()));
					cluster.add_nodes(all_nodes_ids.iter().cloned());
					(node_id.clone(), Node {
						cluster: cluster.clone(),
						key_storage: key_storage.clone(),
						session: SessionImpl::new(SessionParams {
							meta: ShareChangeSessionMeta {
								id: Default::default(),
								self_node_id: node_id.clone(),
								master_node_id: master_node_id.clone(),
							},
							sub_session: sub_sesion.clone(),
							key_share: key_storage.get(&Default::default()).unwrap(),
							result_computer: Arc::new(FastestResultComputer::new(
								node_id.clone(),
								key_storage.get(&Default::default()).unwrap().as_ref(),
							)),
							transport: DummyTransport {
								cluster: cluster,
							},
							nonce: 0,
						}),
					})
				}).collect(),
				queue: VecDeque::new(),
			}
		}

		pub fn node_id(&self, idx: usize) -> &NodeId {
			self.nodes.keys().nth(idx).unwrap()
		}

		pub fn session(&self, idx: usize) -> &SessionImpl<DummyTransport> {
			&self.nodes.values().nth(idx).unwrap().session
		}
	}

	#[test]
	fn negotiation_fails_if_initialized_twice() {
		let ml = MessageLoop::empty(1);
		assert_eq!(ml.session(0).initialize(BTreeSet::new()), Ok(()));
		assert_eq!(ml.session(0).initialize(BTreeSet::new()), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn negotiation_fails_if_message_contains_wrong_nonce() {
		let ml = MessageLoop::empty(2);
		assert_eq!(ml.session(1).process_message(ml.node_id(0), &KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 100,
		})), Err(Error::ReplayProtection));
	}

	#[test]
	fn negotiation_fails_if_versions_request_received_from_non_master() {
		let ml = MessageLoop::empty(3);
		assert_eq!(ml.session(2).process_message(ml.node_id(1), &KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
		})), Err(Error::InvalidMessage));
	}

	#[test]
	fn negotiation_fails_if_versions_request_received_twice() {
		let ml = MessageLoop::empty(2);
		assert_eq!(ml.session(1).process_message(ml.node_id(0), &KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
		})), Ok(()));
		assert_eq!(ml.session(1).process_message(ml.node_id(0), &KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
		})), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn negotiation_fails_if_versions_received_before_initialization() {
		let ml = MessageLoop::empty(2);
		assert_eq!(ml.session(1).process_message(ml.node_id(0), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: Some(10),
			versions: Vec::new(),
		})), Err(Error::InvalidStateForRequest));
	}

	#[test]
	fn negotiation_does_not_fails_if_versions_received_after_completion() {
		let ml = MessageLoop::empty(3);
		ml.session(0).initialize(ml.nodes.keys().cloned().collect()).unwrap();
		assert_eq!(ml.session(0).data.lock().state, SessionState::WaitingForResponses);

		let version_id = (*math::generate_random_scalar().unwrap()).clone();
		assert_eq!(ml.session(0).process_message(ml.node_id(1), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: Some(0),
			versions: vec![version_id.clone().into()]
		})), Ok(()));
		assert_eq!(ml.session(0).data.lock().state, SessionState::Finished);

		assert_eq!(ml.session(0).process_message(ml.node_id(2), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: Some(0),
			versions: vec![version_id.clone().into()]
		})), Ok(()));
		assert_eq!(ml.session(0).data.lock().state, SessionState::Finished);
	}

	#[test]
	fn negotiation_fails_if_wrong_threshold_sent() {
		let ml = MessageLoop::empty(3);
		ml.session(0).initialize(ml.nodes.keys().cloned().collect()).unwrap();

		let version_id = (*math::generate_random_scalar().unwrap()).clone();
		assert_eq!(ml.session(0).process_message(ml.node_id(1), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: Some(1),
			versions: vec![version_id.clone().into()]
		})), Ok(()));
		assert_eq!(ml.session(0).process_message(ml.node_id(2), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: Some(2),
			versions: vec![version_id.clone().into()]
		})), Err(Error::InvalidMessage));
	}

	#[test]
	fn negotiation_fails_if_threshold_empty_when_versions_are_not_empty() {
		let ml = MessageLoop::empty(2);
		ml.session(0).initialize(ml.nodes.keys().cloned().collect()).unwrap();

		let version_id = (*math::generate_random_scalar().unwrap()).clone();
		assert_eq!(ml.session(0).process_message(ml.node_id(1), &KeyVersionNegotiationMessage::KeyVersions(KeyVersions {
			session: Default::default(),
			sub_session: math::generate_random_scalar().unwrap().into(),
			session_nonce: 0,
			threshold: None,
			versions: vec![version_id.clone().into()]
		})), Err(Error::InvalidMessage));
	}

	#[test]
	fn fast_negotiation_does_not_completes_instantly_when_enough_share_owners_are_connected() {
		let nodes = MessageLoop::prepare_nodes(2);
		let version_id = (*math::generate_random_scalar().unwrap()).clone();
		nodes.values().nth(0).unwrap().insert(Default::default(), DocumentKeyShare {
			author: Default::default(),
			threshold: 1,
			common_point: None,
			encrypted_point: None,
			versions: vec![DocumentKeyShareVersion {
				hash: version_id,
				id_numbers: vec![(nodes.keys().cloned().nth(0).unwrap(), math::generate_random_scalar().unwrap())].into_iter().collect(),
				secret_share: math::generate_random_scalar().unwrap(),
			}],
		}).unwrap();
		let ml = MessageLoop::new(nodes);
		ml.session(0).initialize(ml.nodes.keys().cloned().collect()).unwrap();
		// we can't be sure that node has given key version because previous ShareAdd session could fail
		assert!(ml.session(0).data.lock().state != SessionState::Finished);
	}
}
