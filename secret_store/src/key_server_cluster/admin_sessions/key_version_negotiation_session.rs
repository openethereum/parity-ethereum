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
use key_server_cluster::{Error, SessionId, NodeId, DocumentKeyShare, KeyStorage};
use key_server_cluster::cluster::Cluster;
use key_server_cluster::cluster_sessions::ClusterSession;
use key_server_cluster::message::{Message, KeyVersionNegotiationMessage, RequestKeyVersions, KeyVersions};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

// TODO: when working on scalability, change sessions so that versions are sent by chunks.
/// Number of versions sent in single message.
const VERSIONS_PER_MESSAGE: usize = 32;

/// Key version negotiation session API.
pub trait Session: Send + Sync + 'static {
	/// Wait until session is completed.
	fn wait(&self) -> Result<(H256, NodeId), Error>;
}

/// Key version negotiation transport.
pub trait SessionTransport {
	/// Send message to given node.
	fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error>;
}

/// Key version negotiation result computer.
pub trait SessionResultComputer: Send + Sync {
	/// Compute result of session, if possible.
	fn compute_result(&self, confirmations: &BTreeSet<NodeId>, versions: &BTreeMap<H256, BTreeSet<NodeId>>) -> Option<Result<(H256, NodeId), Error>>;
}

/// Key discovery session API.
pub struct SessionImpl<T: SessionTransport> {
	/// Session core.
	core: SessionCore<T>,
	/// Session data.
	data: Mutex<SessionData>,
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
	/// { Version => Nodes }
	pub versions: Option<BTreeMap<H256, BTreeSet<NodeId>>>,
	/// Session result.
	pub result: Option<Result<(H256, NodeId), Error>>,
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
pub enum SessionState {
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
	cluster: Arc<Cluster>,
	/// Key id.
	key_id: SessionId,
	/// Sub session id.
	sub_session: Secret,
	/// Session-level nonce.
	nonce: u64,
}

/// Fastest session result computer. Computes first possible version that can be recovered on this node.
/// If there's no such version, selects version with the most support.
pub struct FastestResultComputer {
	/// This node id.
	self_node_id: NodeId,
	/// Threshold (if known).
	threshold: Option<usize>,
}


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
				versions: None,
				result: None,
			})
		}
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

		// send requests
		let no_confirmations_required = confirmations.is_empty();
		for connected_node in &confirmations {
			self.core.transport.send(connected_node, KeyVersionNegotiationMessage::RequestKeyVersions(RequestKeyVersions {
				session: self.core.meta.id.clone().into(),
				sub_session: self.core.sub_session.clone().into(),
				session_nonce: self.core.nonce,
			}));
		}

		// update state
		data.state = SessionState::WaitingForResponses;
		data.confirmations = Some(confirmations);
		data.versions = Some(versions);

		// try to complete session
		if received_own_confirmation {
			Self::try_complete(&self.core, &mut *data);
			if no_confirmations_required && data.state != SessionState::Finished {
				return Err(Error::ConsensusUnreachable);
			}
		}

		Ok(())
	}

	/// Process single message.
	pub fn process_message(&self, sender: &NodeId, message: &KeyVersionNegotiationMessage) -> Result<(), Error> {
		match message {
			&KeyVersionNegotiationMessage::RequestKeyVersions(ref message) =>
				self.on_key_versions_request(sender, message),
			&KeyVersionNegotiationMessage::KeyVersions(ref message) =>
				self.on_key_versions(sender, message),
		}
	}

	/// Process key versions request.
	pub fn on_key_versions_request(&self, sender: &NodeId, message: &RequestKeyVersions) -> Result<(), Error> {
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
		if !data.confirmations.as_mut().expect("TODO").remove(sender) {
			return Err(Error::InvalidMessage);
		}

		// remember versions that sender have
		{
			let versions = data.versions.as_mut().expect("TODO");
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
		let confirmations = data.confirmations.as_ref().expect("TODO");
		let versions = data.versions.as_ref().expect("TODO");
		if let Some(result) = core.result_computer.compute_result(confirmations, versions) {
			data.state = SessionState::Finished;
			data.result = Some(result);
			core.completed.notify_all();
		}
	}
}

impl<T> Session for SessionImpl<T> where T: SessionTransport + Send + Sync + 'static {
	fn wait(&self) -> Result<(H256, NodeId), Error> {
		let mut data = self.data.lock();
		if !data.result.is_some() {
			self.core.completed.wait(&mut data);
		}

		data.result.as_ref()
			.expect("checked above or waited for completed; completed is only signaled when result.is_some(); qed")
			.clone()
	}
}

impl<T> ClusterSession for SessionImpl<T> where T: SessionTransport {
	fn is_finished(&self) -> bool {
		self.data.lock().state == SessionState::Finished
	}

	fn on_session_timeout(&self) {
		let mut data = self.data.lock();

		if data.confirmations.is_some() {
			data.confirmations.as_mut().expect("TODO").clear();
			Self::try_complete(&self.core, &mut *data);
			if data.state != SessionState::Finished {
				warn!("{}: key version negotiation session failed with timeout", self.core.meta.self_node_id);

				data.result = Some(Err(Error::ConsensusUnreachable));
				self.core.completed.notify_all();
			}
		}
	}

	fn on_node_timeout(&self, node: &NodeId) {
		let mut data = self.data.lock();

		if data.confirmations.is_some() {
			let is_waiting_for_confirmation = data.confirmations.as_mut().expect("TODO").remove(node);
			if is_waiting_for_confirmation {
				Self::try_complete(&self.core, &mut *data);
				if data.state != SessionState::Finished {
					warn!("{}: key version negotiation session failed because {} connection has timeouted", self.core.meta.self_node_id, node);

					data.state = SessionState::Finished;
					data.result = Some(Err(Error::NodeDisconnected));
					self.core.completed.notify_all();
				}
			}
		}
	}

	fn on_session_error(&self, node: &NodeId, error: Error) {
		unimplemented!()
	}

	fn on_message(&self, sender: &NodeId, message: &Message) -> Result<(), Error> {
		unimplemented!()
	}
}

impl SessionTransport for IsolatedSessionTransport {
	fn send(&self, node: &NodeId, message: KeyVersionNegotiationMessage) -> Result<(), Error> {
		self.cluster.send(node, Message::KeyVersionNegotiation(message))
	}
}

impl SessionResultComputer for FastestResultComputer {
	fn compute_result(&self, confirmations: &BTreeSet<NodeId>, versions: &BTreeMap<H256, BTreeSet<NodeId>>) -> Option<Result<(H256, NodeId), Error>> {
		match self.threshold {
			// if we have key share on this node
			Some(threshold) => {
				// select version this node have, with enough participants
				let version = versions.iter().find(|&(_, ref n)| n.contains(&self.self_node_id) && n.len() >= threshold + 1);
				// if there's no such version, wait for more confirmations
				match version {
					Some((version, nodes)) => Some(Ok((version.clone(), self.self_node_id.clone()))),
					None if !confirmations.is_empty() => None,
					// otherwise - try to find any version
					None => Some(versions.iter()
						.find(|&(_, ref n)| n.len() >= threshold + 1)
						.map(|(version, nodes)| Ok((version.clone(), nodes.iter().cloned().nth(0).expect("TODO"))))
						.unwrap_or(Err(Error::ConsensusUnreachable))),
				}
			},
			// if we do not have share, then wait for all confirmations
			None if !confirmations.is_empty() => None,
			// ...and select version with largest support
			None => Some(versions.iter()
				.max_by_key(|&(_, ref n)| n.len())
				.map(|(version, nodes)| Ok((version.clone(), nodes.iter().cloned().nth(0).expect("TODO"))))
				.unwrap_or(Err(Error::ConsensusUnreachable))),
		}
	}
}
