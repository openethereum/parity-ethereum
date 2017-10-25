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

use std::time;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use parking_lot::{Mutex, RwLock};
use bigint::hash::H256;
use ethkey::{Public, Secret, Signature};
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage, DocumentKeyShare, EncryptedDocumentKeyShadow, SessionMeta};
use key_server_cluster::cluster::{Cluster, ClusterData, ClusterConfiguration, ClusterView};
use key_server_cluster::cluster_sessions::{ClusterSession, SessionIdWithSubSession, AdminSession};
use key_server_cluster::message::{self, Message, GenerationMessage, EncryptionMessage, DecryptionMessage, SigningMessage,
	ShareAddMessage, ServersSetChangeMessage, KeyVersionNegotiationMessage};
use key_server_cluster::generation_session::{Session as GenerationSession, SessionImpl as GenerationSessionImpl,
	SessionParams as GenerationSessionParams, SessionState as GenerationSessionState};
use key_server_cluster::decryption_session::{Session as DecryptionSession, SessionImpl as DecryptionSessionImpl,
	SessionParams as DecryptionSessionParams};
use key_server_cluster::encryption_session::{Session as EncryptionSession, SessionImpl as EncryptionSessionImpl,
	SessionParams as EncryptionSessionParams, SessionState as EncryptionSessionState};
use key_server_cluster::signing_session::{Session as SigningSession, SessionImpl as SigningSessionImpl,
	SessionParams as SigningSessionParams};
use key_server_cluster::share_add_session::{Session as ShareAddSession, SessionImpl as ShareAddSessionImpl,
	SessionParams as ShareAddSessionParams, IsolatedSessionTransport as ShareAddTransport};
use key_server_cluster::servers_set_change_session::{Session as ServersSetChangeSession, SessionImpl as ServersSetChangeSessionImpl,
	SessionParams as ServersSetChangeSessionParams};
use key_server_cluster::key_version_negotiation_session::{Session as KeyVersionNegotiationSession, SessionImpl as KeyVersionNegotiationSessionImpl,
	SessionParams as KeyVersionNegotiationSessionParams, IsolatedSessionTransport as VersionNegotiationTransport,
	FastestResultComputer as FastestResultKeyVersionsResultComputer, ContinueAction};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Generic cluster session creator.
pub trait ClusterSessionCreator<S: ClusterSession, D> {
	/// Get creation data from message.
	fn creation_data_from_message(message: &Message) -> Result<Option<D>, Error> {
		Ok(None)
	}

	/// Prepare error message.
	fn make_error_message(sid: S::Id, nonce: u64, err: Error) -> Message;

	/// Create cluster session.
	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: S::Id, creation_data: Option<D>) -> Result<Arc<S>, Error>;
}

/// Message with session id.
pub trait IntoSessionId<K> {
	/// Get session id.
	fn into_session_id(&self) -> Result<K, Error>;
}

pub struct SessionCreatorCore {
	/// Self node id.
	self_node_id: NodeId,
	/// Reference to key storage
	key_storage: Arc<KeyStorage>,
	/// Reference to ACL storage
	acl_storage: Arc<AclStorage>,
	/// Always-increasing sessions counter. Is used as session nonce to prevent replay attacks:
	/// 1) during handshake, KeyServers generate new random key to encrypt messages
	/// => there's no way to use messages from previous connections for replay attacks
	/// 2) when session (of any type) is started, master node increases its own session counter and broadcasts it
	/// 3) when slave KeyServer receives session initialization message, it checks that new nonce is larger than previous (from the same master)
	/// => there's no way to use messages from previous sessions for replay attacks
	/// 4) KeyServer checks that each session message contains the same nonce that initialization message
	/// Given that: (A) handshake is secure and (B) session itself is initially replay-protected
	/// => this guarantees that sessions are replay-protected.
	session_counter: AtomicUsize,
	/// Maximal session nonce, received from given connection.
	max_nonce: RwLock<BTreeMap<NodeId, u64>>,
}

impl SessionCreatorCore {
	/// Create new session creator core.
	pub fn new(config: &ClusterConfiguration) -> Self {
		SessionCreatorCore {
			self_node_id: config.self_key_pair.public().clone(),
			acl_storage: config.acl_storage.clone(),
			key_storage: config.key_storage.clone(),
			session_counter: AtomicUsize::new(0),
			max_nonce: RwLock::new(BTreeMap::new()),
		}
	}

	/// When node has teimtouted.
	pub fn on_connection_timeout(&self, node_id: &NodeId) {
		self.max_nonce.write().remove(node_id);
	}

	/// Check or generate new session nonce.
	fn check_session_nonce(&self, master: &NodeId, nonce: Option<u64>) -> Result<u64, Error> {
		// if we're master node of the session, then nonce should be generated
		// if we're slave node of the session, then nonce should be passed from outside
		// TODO: not true for delegation
		// debug_assert!((master == &self.self_node_id) == nonce.is_none());

		match nonce {
			Some(nonce) => match nonce > *self.max_nonce.write().entry(master.clone()).or_insert(0) {
				true => Ok(nonce),
				false => Err(Error::ReplayProtection),
			},
			None => Ok(self.session_counter.fetch_add(1, Ordering::Relaxed) as u64 + 1),
		}
	}

	/// Read key share && remove disconnected nodes.
	fn read_key_share(&self, key_id: &SessionId, cluster: &Arc<Cluster>) -> Result<Option<DocumentKeyShare>, Error> {
		let mut encrypted_data = self.key_storage.get(key_id).map_err(|e| Error::KeyStorage(e.into()))?;

		// some of nodes, which were encrypting secret may be down
		// => do not use these in session
		//let disconnected_nodes: BTreeSet<_> = encrypted_data.id_numbers.keys().cloned().collect();
		//for disconnected_node in disconnected_nodes.difference(&cluster.nodes()) {
		//	encrypted_data.id_numbers.remove(&disconnected_node);
		//}
		Ok(encrypted_data)
	}
}

/// Generation session creator.
pub struct GenerationSessionCreator {
	/// True if generation sessions must fail.
	pub make_faulty_generation_sessions: AtomicBool,
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl GenerationSessionCreator {
	#[cfg(test)]
	pub fn make_faulty_generation_sessions(&self) {
		self.make_faulty_generation_sessions.store(true, Ordering::Relaxed);
	}
}

impl ClusterSessionCreator<GenerationSessionImpl, ()> for GenerationSessionCreator {
	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::Generation(message::GenerationMessage::SessionError(message::SessionError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionId, _creation_data: Option<()>) -> Result<Arc<GenerationSessionImpl>, Error> {
		// check that there's no finished encryption session with the same id
		if self.core.key_storage.contains(&id) {
			return Err(Error::DuplicateSessionId);
		}

		let nonce = self.core.check_session_nonce(&master, nonce)?;
		Ok(GenerationSessionImpl::new(GenerationSessionParams {
			id: id.clone(),
			self_node_id: self.core.self_node_id.clone(),
			key_storage: Some(self.core.key_storage.clone()),
			cluster: cluster,
			nonce: Some(nonce),
		}))
		.map(|session| {
			if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
				session.simulate_faulty_behaviour();
			}
			session
		})
		.map(Arc::new)
	}
}

/// Encryption session creator.
pub struct EncryptionSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<EncryptionSessionImpl, ()> for EncryptionSessionCreator {
	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::Encryption(message::EncryptionMessage::EncryptionSessionError(message::EncryptionSessionError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionId, _creation_data: Option<()>) -> Result<Arc<EncryptionSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id, &cluster)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		Ok(Arc::new(EncryptionSessionImpl::new(EncryptionSessionParams {
			id: id,
			self_node_id: self.core.self_node_id.clone(),
			encrypted_data: encrypted_data,
			key_storage: self.core.key_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		})?))
	}
}

/// Decryption session creator.
pub struct DecryptionSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<DecryptionSessionImpl, Signature> for DecryptionSessionCreator {
	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::Decryption(message::DecryptionMessage::DecryptionSessionError(message::DecryptionSessionError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionIdWithSubSession, requester_signature: Option<Signature>) -> Result<Arc<DecryptionSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id, &cluster)?;
		//let nonce = if master == self.core.self_node_id { None } else { nonce }; // TODO
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		Ok(Arc::new(DecryptionSessionImpl::new(DecryptionSessionParams {
			meta: SessionMeta {
				id: id.id,
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.as_ref().map(|ks| ks.threshold).unwrap_or_default(),
			},
			access_key: id.access_key,
			key_share: encrypted_data,
			acl_storage: self.core.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester_signature)?))
	}
}

/// Signing session creator.
pub struct SigningSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<SigningSessionImpl, Signature> for SigningSessionCreator {
	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::Signing(message::SigningMessage::SigningSessionError(message::SigningSessionError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionIdWithSubSession, requester_signature: Option<Signature>) -> Result<Arc<SigningSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id, &cluster)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		Ok(Arc::new(SigningSessionImpl::new(SigningSessionParams {
			meta: SessionMeta {
				id: id.id,
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.as_ref().map(|ks| ks.threshold).unwrap_or_default(),
			},
			access_key: id.access_key,
			key_share: encrypted_data,
			acl_storage: self.core.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester_signature)?))
	}
}

/// Key version negotiation session creator.
pub struct KeyVersionNegotiationSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<KeyVersionNegotiationSessionImpl<VersionNegotiationTransport>, ()> for KeyVersionNegotiationSessionCreator {
	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::KeyVersionNegotiation(message::KeyVersionNegotiationMessage::KeyVersionsError(message::KeyVersionsError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionIdWithSubSession, _creation_data: Option<()>) -> Result<Arc<KeyVersionNegotiationSessionImpl<VersionNegotiationTransport>>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id, &cluster)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let computer = Arc::new(FastestResultKeyVersionsResultComputer::new(self.core.self_node_id.clone(), encrypted_data.as_ref(), &cluster.nodes()));
		Ok(Arc::new(KeyVersionNegotiationSessionImpl::new(KeyVersionNegotiationSessionParams {
			meta: ShareChangeSessionMeta {
				id: id.id.clone(),
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
			},
			sub_session: id.access_key.clone(),
			key_share: encrypted_data,
			result_computer: computer,
			transport: VersionNegotiationTransport {
				cluster: cluster,
				key_id: id.id,
				sub_session: id.access_key.clone(),
				nonce: nonce,
			},
			nonce: nonce,
		})))
	}
}

/// Administrative session creator.
pub struct AdminSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
	/// Administrator public.
	pub admin_public: Option<Public>,
}

impl ClusterSessionCreator<AdminSession, ()> for AdminSessionCreator {
	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::ServersSetChange(message::ServersSetChangeMessage::ServersSetChangeError(message::ServersSetChangeError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionId, _creation_data: Option<()>) -> Result<Arc<AdminSession>, Error> {
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let admin_public = self.admin_public.clone().ok_or(Error::AccessDenied)?;
		Ok(Arc::new(AdminSession::ServersSetChange(ServersSetChangeSessionImpl::new(ServersSetChangeSessionParams {
			meta: ShareChangeSessionMeta {
				id: id.clone(),
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
			},
			cluster: cluster.clone(),
			key_storage: self.core.key_storage.clone(),
			nonce: nonce,
			all_nodes_set: cluster.nodes(),
			admin_public: admin_public,
		})?)))
	}
}

impl IntoSessionId<SessionId> for Message {
	fn into_session_id(&self) -> Result<SessionId, Error> {
		match *self {
			Message::Generation(ref message) => Ok(message.session_id().clone()),
			Message::Encryption(ref message) => Ok(message.session_id().clone()),
			Message::Decryption(_) => Err(Error::InvalidMessage),
			Message::Signing(_) => Err(Error::InvalidMessage),
			Message::ServersSetChange(ref message) => Ok(message.session_id().clone()),
			Message::ShareAdd(ref message) => Ok(message.session_id().clone()),
			Message::KeyVersionNegotiation(_) => Err(Error::InvalidMessage),
			Message::Cluster(_) => Err(Error::InvalidMessage),
		}
	}
}

impl IntoSessionId<SessionIdWithSubSession> for Message {
	fn into_session_id(&self) -> Result<SessionIdWithSubSession, Error> {
		match *self {
			Message::Generation(_) => Err(Error::InvalidMessage),
			Message::Encryption(_) => Err(Error::InvalidMessage),
			Message::Decryption(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::Signing(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::ServersSetChange(_) => Err(Error::InvalidMessage),
			Message::ShareAdd(_) => Err(Error::InvalidMessage),
			Message::KeyVersionNegotiation(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::Cluster(_) => Err(Error::InvalidMessage),
		}
	}
}
