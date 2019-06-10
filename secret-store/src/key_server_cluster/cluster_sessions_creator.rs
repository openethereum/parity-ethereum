// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::collections::BTreeMap;
use parking_lot::RwLock;
use ethkey::Public;
use key_server_cluster::{Error, NodeId, SessionId, Requester, AclStorage, KeyStorage, DocumentKeyShare, SessionMeta};
use key_server_cluster::cluster::{Cluster, ClusterConfiguration};
use key_server_cluster::connection_trigger::ServersSetChangeSessionCreatorConnector;
use key_server_cluster::cluster_sessions::{WaitableSession, ClusterSession, SessionIdWithSubSession,
	AdminSession, AdminSessionCreationData};
use key_server_cluster::message::{self, Message, DecryptionMessage, SchnorrSigningMessage, ConsensusMessageOfShareAdd,
	ShareAddMessage, ServersSetChangeMessage, ConsensusMessage, ConsensusMessageWithServersSet, EcdsaSigningMessage};
use key_server_cluster::generation_session::{SessionImpl as GenerationSessionImpl, SessionParams as GenerationSessionParams};
use key_server_cluster::decryption_session::{SessionImpl as DecryptionSessionImpl,
	SessionParams as DecryptionSessionParams};
use key_server_cluster::encryption_session::{SessionImpl as EncryptionSessionImpl, SessionParams as EncryptionSessionParams};
use key_server_cluster::signing_session_ecdsa::{SessionImpl as EcdsaSigningSessionImpl,
	SessionParams as EcdsaSigningSessionParams};
use key_server_cluster::signing_session_schnorr::{SessionImpl as SchnorrSigningSessionImpl,
	SessionParams as SchnorrSigningSessionParams};
use key_server_cluster::share_add_session::{SessionImpl as ShareAddSessionImpl,
	SessionParams as ShareAddSessionParams, IsolatedSessionTransport as ShareAddTransport};
use key_server_cluster::servers_set_change_session::{SessionImpl as ServersSetChangeSessionImpl,
	SessionParams as ServersSetChangeSessionParams};
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSessionImpl,
	SessionParams as KeyVersionNegotiationSessionParams, IsolatedSessionTransport as VersionNegotiationTransport,
	FastestResultComputer as FastestResultKeyVersionsResultComputer};
use key_server_cluster::admin_sessions::ShareChangeSessionMeta;

/// Generic cluster session creator.
pub trait ClusterSessionCreator<S: ClusterSession> {
	/// Get creation data from message.
	fn creation_data_from_message(_message: &Message) -> Result<Option<S::CreationData>, Error> {
		Ok(None)
	}

	/// Prepare error message.
	fn make_error_message(sid: S::Id, nonce: u64, err: Error) -> Message;

	/// Create cluster session.
	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: S::Id,
		creation_data: Option<S::CreationData>,
	) -> Result<WaitableSession<S>, Error>;
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
		match nonce {
			Some(nonce) => match nonce > *self.max_nonce.write().entry(master.clone()).or_insert(0) {
				true => Ok(nonce),
				false => Err(Error::ReplayProtection),
			},
			None => Ok(self.session_counter.fetch_add(1, Ordering::Relaxed) as u64 + 1),
		}
	}

	/// Read key share && remove disconnected nodes.
	fn read_key_share(&self, key_id: &SessionId) -> Result<Option<DocumentKeyShare>, Error> {
		self.key_storage.get(key_id)
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

impl ClusterSessionCreator<GenerationSessionImpl> for GenerationSessionCreator {
	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::Generation(message::GenerationMessage::SessionError(message::SessionError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionId,
		_creation_data: Option<()>,
	) -> Result<WaitableSession<GenerationSessionImpl>, Error> {
		// check that there's no finished encryption session with the same id
		if self.core.key_storage.contains(&id) {
			return Err(Error::ServerKeyAlreadyGenerated);
		}

		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let (session, oneshot) = GenerationSessionImpl::new(GenerationSessionParams {
			id: id.clone(),
			self_node_id: self.core.self_node_id.clone(),
			key_storage: Some(self.core.key_storage.clone()),
			cluster: cluster,
			nonce: Some(nonce),
		});

		if self.make_faulty_generation_sessions.load(Ordering::Relaxed) {
			session.simulate_faulty_behaviour();
		}

		Ok(WaitableSession::new(session, oneshot))
	}
}

/// Encryption session creator.
pub struct EncryptionSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<EncryptionSessionImpl> for EncryptionSessionCreator {
	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::Encryption(message::EncryptionMessage::EncryptionSessionError(message::EncryptionSessionError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionId,
		_creation_data: Option<()>,
	) -> Result<WaitableSession<EncryptionSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let (session, oneshot) = EncryptionSessionImpl::new(EncryptionSessionParams {
			id: id,
			self_node_id: self.core.self_node_id.clone(),
			encrypted_data: encrypted_data,
			key_storage: self.core.key_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		})?;

		Ok(WaitableSession::new(session, oneshot))
	}
}

/// Decryption session creator.
pub struct DecryptionSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<DecryptionSessionImpl> for DecryptionSessionCreator {
	fn creation_data_from_message(message: &Message) -> Result<Option<Requester>, Error> {
		match *message {
			Message::Decryption(DecryptionMessage::DecryptionConsensusMessage(ref message)) => match &message.message {
				&ConsensusMessage::InitializeConsensusSession(ref message) => Ok(Some(message.requester.clone().into())),
				_ => Err(Error::InvalidMessage),
			},
			Message::Decryption(DecryptionMessage::DecryptionSessionDelegation(ref message)) => Ok(Some(message.requester.clone().into())),
			_ => Err(Error::InvalidMessage),
		}
	}

	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::Decryption(message::DecryptionMessage::DecryptionSessionError(message::DecryptionSessionError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionIdWithSubSession,
		requester: Option<Requester>,
	) -> Result<WaitableSession<DecryptionSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let (session, oneshot) = DecryptionSessionImpl::new(DecryptionSessionParams {
			meta: SessionMeta {
				id: id.id,
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.as_ref().map(|ks| ks.threshold).unwrap_or_default(),
				configured_nodes_count: cluster.configured_nodes_count(),
				connected_nodes_count: cluster.connected_nodes_count(),
			},
			access_key: id.access_key,
			key_share: encrypted_data,
			acl_storage: self.core.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester)?;

		Ok(WaitableSession::new(session, oneshot))
	}
}

/// Schnorr signing session creator.
pub struct SchnorrSigningSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<SchnorrSigningSessionImpl> for SchnorrSigningSessionCreator {
	fn creation_data_from_message(message: &Message) -> Result<Option<Requester>, Error> {
		match *message {
			Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningConsensusMessage(ref message)) => match &message.message {
				&ConsensusMessage::InitializeConsensusSession(ref message) => Ok(Some(message.requester.clone().into())),
				_ => Err(Error::InvalidMessage),
			},
			Message::SchnorrSigning(SchnorrSigningMessage::SchnorrSigningSessionDelegation(ref message)) => Ok(Some(message.requester.clone().into())),
			_ => Err(Error::InvalidMessage),
		}
	}

	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::SchnorrSigning(message::SchnorrSigningMessage::SchnorrSigningSessionError(message::SchnorrSigningSessionError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionIdWithSubSession,
		requester: Option<Requester>,
	) -> Result<WaitableSession<SchnorrSigningSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let (session, oneshot) = SchnorrSigningSessionImpl::new(SchnorrSigningSessionParams {
			meta: SessionMeta {
				id: id.id,
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.as_ref().map(|ks| ks.threshold).unwrap_or_default(),
				configured_nodes_count: cluster.configured_nodes_count(),
				connected_nodes_count: cluster.connected_nodes_count(),
			},
			access_key: id.access_key,
			key_share: encrypted_data,
			acl_storage: self.core.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester)?;
		Ok(WaitableSession::new(session, oneshot))
	}
}

/// ECDSA signing session creator.
pub struct EcdsaSigningSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<EcdsaSigningSessionImpl> for EcdsaSigningSessionCreator {
	fn creation_data_from_message(message: &Message) -> Result<Option<Requester>, Error> {
		match *message {
			Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningConsensusMessage(ref message)) => match &message.message {
				&ConsensusMessage::InitializeConsensusSession(ref message) => Ok(Some(message.requester.clone().into())),
				_ => Err(Error::InvalidMessage),
			},
			Message::EcdsaSigning(EcdsaSigningMessage::EcdsaSigningSessionDelegation(ref message)) => Ok(Some(message.requester.clone().into())),
			_ => Err(Error::InvalidMessage),
		}
	}

	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::EcdsaSigning(message::EcdsaSigningMessage::EcdsaSigningSessionError(message::EcdsaSigningSessionError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(&self, cluster: Arc<Cluster>, master: NodeId, nonce: Option<u64>, id: SessionIdWithSubSession, requester: Option<Requester>) -> Result<WaitableSession<EcdsaSigningSessionImpl>, Error> {
		let encrypted_data = self.core.read_key_share(&id.id)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let (session, oneshot) = EcdsaSigningSessionImpl::new(EcdsaSigningSessionParams {
			meta: SessionMeta {
				id: id.id,
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				threshold: encrypted_data.as_ref().map(|ks| ks.threshold).unwrap_or_default(),
				configured_nodes_count: cluster.configured_nodes_count(),
				connected_nodes_count: cluster.connected_nodes_count(),
			},
			access_key: id.access_key,
			key_share: encrypted_data,
			acl_storage: self.core.acl_storage.clone(),
			cluster: cluster,
			nonce: nonce,
		}, requester)?;

		Ok(WaitableSession::new(session, oneshot))
	}
}

/// Key version negotiation session creator.
pub struct KeyVersionNegotiationSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
}

impl ClusterSessionCreator<KeyVersionNegotiationSessionImpl<VersionNegotiationTransport>> for KeyVersionNegotiationSessionCreator {
	fn make_error_message(sid: SessionIdWithSubSession, nonce: u64, err: Error) -> Message {
		message::Message::KeyVersionNegotiation(message::KeyVersionNegotiationMessage::KeyVersionsError(message::KeyVersionsError {
			session: sid.id.into(),
			sub_session: sid.access_key.into(),
			session_nonce: nonce,
			error: err.into(),
			// we don't care about continue action here. it only matters when we're completing the session with confirmed
			// fatal error from result computer
			continue_with: None,
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionIdWithSubSession,
		_creation_data: Option<()>,
	) -> Result<WaitableSession<KeyVersionNegotiationSessionImpl<VersionNegotiationTransport>>, Error> {
		let configured_nodes_count = cluster.configured_nodes_count();
		let connected_nodes_count = cluster.connected_nodes_count();
		let encrypted_data = self.core.read_key_share(&id.id)?;
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		let computer = Arc::new(FastestResultKeyVersionsResultComputer::new(self.core.self_node_id.clone(), encrypted_data.as_ref(),
			configured_nodes_count, configured_nodes_count));
		let (session, oneshot) = KeyVersionNegotiationSessionImpl::new(KeyVersionNegotiationSessionParams {
			meta: ShareChangeSessionMeta {
				id: id.id.clone(),
				self_node_id: self.core.self_node_id.clone(),
				master_node_id: master,
				configured_nodes_count: configured_nodes_count,
				connected_nodes_count: connected_nodes_count,
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
		});
		Ok(WaitableSession::new(session, oneshot))
	}
}

/// Administrative session creator.
pub struct AdminSessionCreator {
	/// Creator core.
	pub core: Arc<SessionCreatorCore>,
	/// Administrator public.
	pub admin_public: Option<Public>,
	/// Servers set change sessions creator connector.
	pub servers_set_change_session_creator_connector: Arc<ServersSetChangeSessionCreatorConnector>,
}

impl ClusterSessionCreator<AdminSession> for AdminSessionCreator {
	fn creation_data_from_message(message: &Message) -> Result<Option<AdminSessionCreationData>, Error> {
		match *message {
			Message::ServersSetChange(ServersSetChangeMessage::ServersSetChangeConsensusMessage(ref message)) => match &message.message {
				&ConsensusMessageWithServersSet::InitializeConsensusSession(ref message) => Ok(Some(AdminSessionCreationData::ServersSetChange(
					message.migration_id.clone().map(Into::into),
					message.new_nodes_set.clone().into_iter().map(Into::into).collect()
				))),
				_ => Err(Error::InvalidMessage),
			},
			Message::ShareAdd(ShareAddMessage::ShareAddConsensusMessage(ref message)) => match &message.message {
				&ConsensusMessageOfShareAdd::InitializeConsensusSession(ref message) => Ok(Some(AdminSessionCreationData::ShareAdd(message.version.clone().into()))),
				_ => Err(Error::InvalidMessage),
			},
			_ => Err(Error::InvalidMessage),
		}
	}

	fn make_error_message(sid: SessionId, nonce: u64, err: Error) -> Message {
		message::Message::ServersSetChange(message::ServersSetChangeMessage::ServersSetChangeError(message::ServersSetChangeError {
			session: sid.into(),
			session_nonce: nonce,
			error: err.into(),
		}))
	}

	fn create(
		&self,
		cluster: Arc<Cluster>,
		master: NodeId,
		nonce: Option<u64>,
		id: SessionId,
		creation_data: Option<AdminSessionCreationData>,
	) -> Result<WaitableSession<AdminSession>, Error> {
		let nonce = self.core.check_session_nonce(&master, nonce)?;
		match creation_data {
			Some(AdminSessionCreationData::ShareAdd(version)) => {
				let (session, oneshot) = ShareAddSessionImpl::new(ShareAddSessionParams {
					meta: ShareChangeSessionMeta {
						id: id.clone(),
						self_node_id: self.core.self_node_id.clone(),
						master_node_id: master,
						configured_nodes_count: cluster.configured_nodes_count(),
						connected_nodes_count: cluster.connected_nodes_count(),
					},
					transport: ShareAddTransport::new(id.clone(), Some(version), nonce, cluster),
					key_storage: self.core.key_storage.clone(),
					nonce: nonce,
					admin_public: Some(self.admin_public.clone().ok_or(Error::AccessDenied)?),
				})?;
				Ok(WaitableSession::new(AdminSession::ShareAdd(session), oneshot))
			},
			Some(AdminSessionCreationData::ServersSetChange(migration_id, new_nodes_set)) => {
				let admin_public = self.servers_set_change_session_creator_connector.admin_public(migration_id.as_ref(), new_nodes_set)
					.map_err(|_| Error::AccessDenied)?;

				let (session, oneshot) = ServersSetChangeSessionImpl::new(ServersSetChangeSessionParams {
					meta: ShareChangeSessionMeta {
						id: id.clone(),
						self_node_id: self.core.self_node_id.clone(),
						master_node_id: master,
						configured_nodes_count: cluster.configured_nodes_count(),
						connected_nodes_count: cluster.connected_nodes_count(),
					},
					cluster: cluster.clone(),
					key_storage: self.core.key_storage.clone(),
					nonce: nonce,
					all_nodes_set: cluster.nodes(),
					admin_public: admin_public,
					migration_id: migration_id,
				})?;
				Ok(WaitableSession::new(AdminSession::ServersSetChange(session), oneshot))
			},
			None => unreachable!("expected to call with non-empty creation data; qed"),
		}
	}
}

impl IntoSessionId<SessionId> for Message {
	fn into_session_id(&self) -> Result<SessionId, Error> {
		match *self {
			Message::Generation(ref message) => Ok(message.session_id().clone()),
			Message::Encryption(ref message) => Ok(message.session_id().clone()),
			Message::Decryption(_) => Err(Error::InvalidMessage),
			Message::SchnorrSigning(_) => Err(Error::InvalidMessage),
			Message::EcdsaSigning(_) => Err(Error::InvalidMessage),
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
			Message::SchnorrSigning(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::EcdsaSigning(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::ServersSetChange(_) => Err(Error::InvalidMessage),
			Message::ShareAdd(_) => Err(Error::InvalidMessage),
			Message::KeyVersionNegotiation(ref message) => Ok(SessionIdWithSubSession::new(message.session_id().clone(), message.sub_session_id().clone())),
			Message::Cluster(_) => Err(Error::InvalidMessage),
		}
	}
}
