// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use key_server_cluster::{Error, NodeId, NodeKeyPair};
use key_server_cluster::cluster::{ServersSetChangeParams, new_servers_set_change_session};
use key_server_cluster::cluster_sessions::{AdminSession};
use key_server_cluster::cluster_connections::{ConnectionProvider, Connection};
use key_server_cluster::cluster_sessions::{ClusterSession, ClusterSessions, ClusterSessionsContainer,
	create_cluster_view};
use key_server_cluster::cluster_sessions_creator::{ClusterSessionCreator, IntoSessionId};
use key_server_cluster::message::{self, Message, ClusterMessage};
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSession,
	IsolatedSessionTransport as KeyVersionNegotiationSessionTransport, ContinueAction};
use key_server_cluster::connection_trigger::ServersSetChangeSessionCreatorConnector;

/// Something that is able to process signals/messages from other nodes.
pub trait MessageProcessor: Send + Sync {
	/// Process disconnect from the remote node.
	fn process_disconnect(&self, node: &NodeId);
	/// Process single message from the connection.
	fn process_connection_message(&self, connection: Arc<Connection>, message: Message);

	/// Start servers set change session. This is typically used by ConnectionManager when
	/// it detects that auto-migration session needs to be started.
	fn start_servers_set_change_session(&self, params: ServersSetChangeParams) -> Result<Arc<AdminSession>, Error>;
	/// Try to continue session after key version negotiation session is completed.
	fn try_continue_session(
		&self,
		session: Option<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>>
	);
	/// Maintain active sessions. Typically called by the ConnectionManager at some intervals.
	/// Should cancel stalled sessions and send keep-alive messages for sessions that support it.
	fn maintain_sessions(&self);
}

/// Bridge between ConnectionManager and ClusterSessions.
pub struct SessionsMessageProcessor {
	self_key_pair: Arc<NodeKeyPair>,
	servers_set_change_creator_connector: Arc<ServersSetChangeSessionCreatorConnector>,
	sessions: Arc<ClusterSessions>,
	connections: Arc<ConnectionProvider>,
}

impl SessionsMessageProcessor {
	/// Create new instance of SessionsMessageProcessor.
	pub fn new(
		self_key_pair: Arc<NodeKeyPair>,
		servers_set_change_creator_connector: Arc<ServersSetChangeSessionCreatorConnector>,
		sessions: Arc<ClusterSessions>,
		connections: Arc<ConnectionProvider>,
	) -> Self {
		SessionsMessageProcessor {
			self_key_pair,
			servers_set_change_creator_connector,
			sessions,
			connections,
		}
	}

	/// Process single session message from connection.
	fn process_message<S: ClusterSession, SC: ClusterSessionCreator<S>>(
		&self,
		sessions: &ClusterSessionsContainer<S, SC>,
		connection: Arc<Connection>,
		mut message: Message,
	) -> Option<Arc<S>>
		where
			Message: IntoSessionId<S::Id>
	{
		// get or create new session, if required
		let mut sender = *connection.node_id();
		let session = self.prepare_session(sessions, &sender, &message);
		// send error if session is not found, or failed to create
		let session = match session {
			Ok(session) => session,
			Err(error) => {
				// this is new session => it is not yet in container
				warn!(target: "secretstore_net",
					"{}: {} session read error '{}' when requested for session from node {}",
					self.self_key_pair.public(), S::type_name(), error, sender);
				if !message.is_error_message() {
					let qed = "session_id only fails for cluster messages;
						only session messages are passed to process_message;
						qed";
					let session_id = message.into_session_id().expect(qed);
					let session_nonce = message.session_nonce().expect(qed);

					connection.send_message(SC::make_error_message(session_id, session_nonce, error));
				}
				return None;
			},
		};

		let session_id = session.id();
		let mut is_queued_message = false;
		loop {
			let message_result = session.on_message(&sender, &message);
			match message_result {
				Ok(_) => {
					// if session is completed => stop
					if session.is_finished() {
						info!(target: "secretstore_net",
							"{}: {} session completed", self.self_key_pair.public(), S::type_name());
						sessions.remove(&session_id);
						return Some(session);
					}

					// try to dequeue message
					match sessions.dequeue_message(&session_id) {
						Some((msg_sender, msg)) => {
							is_queued_message = true;
							sender = msg_sender;
							message = msg;
						},
						None => return Some(session),
					}
				},
				Err(Error::TooEarlyForRequest) => {
					sessions.enqueue_message(&session_id, sender, message, is_queued_message);
					return Some(session);
				},
				Err(err) => {
					warn!(
						target: "secretstore_net",
						"{}: {} session error '{}' when processing message {} from node {}",
						self.self_key_pair.public(),
						S::type_name(),
						err,
						message,
						sender);
					session.on_session_error(self.self_key_pair.public(), err);
					sessions.remove(&session_id);
					return Some(session);
				},
			}
		}
	}

	/// Get or insert new session.
	fn prepare_session<S: ClusterSession, SC: ClusterSessionCreator<S>>(
		&self,
		sessions: &ClusterSessionsContainer<S, SC>,
		sender: &NodeId,
		message: &Message
	) -> Result<Arc<S>, Error>
		where
			Message: IntoSessionId<S::Id>
	{
		fn requires_all_connections(message: &Message) -> bool {
			match *message {
				Message::Generation(_) => true,
				Message::ShareAdd(_) => true,
				Message::ServersSetChange(_) => true,
				_ => false,
			}
		}

		// get or create new session, if required
		let session_id = message.into_session_id()
			.expect("into_session_id fails for cluster messages only;
				only session messages are passed to prepare_session;
				qed");
		let is_initialization_message = message.is_initialization_message();
		let is_delegation_message = message.is_delegation_message();
		match is_initialization_message || is_delegation_message {
			false => sessions.get(&session_id, true).ok_or(Error::NoActiveSessionWithId),
			true => {
				let creation_data = SC::creation_data_from_message(&message)?;
				let master = if is_initialization_message {
					*sender
				} else {
					*self.self_key_pair.public()
				};
				let cluster = create_cluster_view(
					self.self_key_pair.clone(),
					self.connections.clone(),
					requires_all_connections(&message))?;

				let nonce = Some(message.session_nonce().ok_or(Error::InvalidMessage)?);
				let exclusive = message.is_exclusive_session_message();
				sessions.insert(cluster, master, session_id, nonce, exclusive, creation_data).map(|s| s.session)
			},
		}
	}

	/// Process single cluster message from the connection.
	fn process_cluster_message(&self, connection: Arc<Connection>, message: ClusterMessage) {
		match message {
			ClusterMessage::KeepAlive(_) => {
				let msg = Message::Cluster(ClusterMessage::KeepAliveResponse(message::KeepAliveResponse {
					session_id: None,
				}));
				connection.send_message(msg)
			},
			ClusterMessage::KeepAliveResponse(msg) => if let Some(session_id) = msg.session_id {
				self.sessions.on_session_keep_alive(connection.node_id(), session_id.into());
			},
			_ => warn!(target: "secretstore_net", "{}: received unexpected message {} from node {} at {}",
				self.self_key_pair.public(), message, connection.node_id(), connection.node_address()),
		}
	}
}

impl MessageProcessor for SessionsMessageProcessor {
	fn process_disconnect(&self, node: &NodeId) {
		self.sessions.on_connection_timeout(node);
	}

	fn process_connection_message(&self, connection: Arc<Connection>, message: Message) {
		trace!(target: "secretstore_net", "{}: received message {} from {}",
			self.self_key_pair.public(), message, connection.node_id());

		// error is ignored as we only process errors on session level
		match message {
			Message::Generation(message) => self
				.process_message(&self.sessions.generation_sessions, connection, Message::Generation(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Encryption(message) => self
				.process_message(&self.sessions.encryption_sessions, connection, Message::Encryption(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Decryption(message) => self
				.process_message(&self.sessions.decryption_sessions, connection, Message::Decryption(message))
				.map(|_| ()).unwrap_or_default(),
			Message::SchnorrSigning(message) => self
				.process_message(&self.sessions.schnorr_signing_sessions, connection, Message::SchnorrSigning(message))
				.map(|_| ()).unwrap_or_default(),
			Message::EcdsaSigning(message) => self
				.process_message(&self.sessions.ecdsa_signing_sessions, connection, Message::EcdsaSigning(message))
				.map(|_| ()).unwrap_or_default(),
			Message::ServersSetChange(message) => {
				let message = Message::ServersSetChange(message);
				let is_initialization_message = message.is_initialization_message();
				let session = self.process_message(&self.sessions.admin_sessions, connection, message);
				if is_initialization_message {
					if let Some(session) = session {
						self.servers_set_change_creator_connector
							.set_key_servers_set_change_session(session.clone());
					}
				}
			},
			Message::KeyVersionNegotiation(message) => {
				let session = self.process_message(
					&self.sessions.negotiation_sessions, connection, Message::KeyVersionNegotiation(message));
				self.try_continue_session(session);
			},
			Message::ShareAdd(message) => self.process_message(
				&self.sessions.admin_sessions, connection, Message::ShareAdd(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Cluster(message) => self.process_cluster_message(connection, message),
		}
	}

	fn try_continue_session(
		&self,
		session: Option<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>>
	) {
		if let Some(session) = session {
			let meta = session.meta();
			let is_master_node = meta.self_node_id == meta.master_node_id;
			if is_master_node && session.is_finished() {
				self.sessions.negotiation_sessions.remove(&session.id());
				match session.result() {
					Some(Ok(Some((version, master)))) => match session.take_continue_action() {
						Some(ContinueAction::Decrypt(
							session, origin, is_shadow_decryption, is_broadcast_decryption
						)) => {
							let initialization_error = if self.self_key_pair.public() == &master {
								session.initialize(
									origin, version, is_shadow_decryption, is_broadcast_decryption)
							} else {
								session.delegate(
									master, origin, version, is_shadow_decryption, is_broadcast_decryption)
							};

							if let Err(error) = initialization_error {
								session.on_session_error(&meta.self_node_id, error);
								self.sessions.decryption_sessions.remove(&session.id());
							}
						},
						Some(ContinueAction::SchnorrSign(session, message_hash)) => {
							let initialization_error = if self.self_key_pair.public() == &master {
								session.initialize(version, message_hash)
							} else {
								session.delegate(master, version, message_hash)
							};

							if let Err(error) = initialization_error {
								session.on_session_error(&meta.self_node_id, error);
								self.sessions.schnorr_signing_sessions.remove(&session.id());
							}
						},
						Some(ContinueAction::EcdsaSign(session, message_hash)) => {
							let initialization_error = if self.self_key_pair.public() == &master {
								session.initialize(version, message_hash)
							} else {
								session.delegate(master, version, message_hash)
							};

							if let Err(error) = initialization_error {
								session.on_session_error(&meta.self_node_id, error);
								self.sessions.ecdsa_signing_sessions.remove(&session.id());
							}
						},
						None => (),
					},
					Some(Err(error)) => match session.take_continue_action() {
						Some(ContinueAction::Decrypt(session, _, _, _)) => {
							session.on_session_error(&meta.self_node_id, error);
							self.sessions.decryption_sessions.remove(&session.id());
						},
						Some(ContinueAction::SchnorrSign(session, _)) => {
							session.on_session_error(&meta.self_node_id, error);
							self.sessions.schnorr_signing_sessions.remove(&session.id());
						},
						Some(ContinueAction::EcdsaSign(session, _)) => {
							session.on_session_error(&meta.self_node_id, error);
							self.sessions.ecdsa_signing_sessions.remove(&session.id());
						},
						None => (),
					},
					None | Some(Ok(None)) => unreachable!("is_master_node; session is finished;
						negotiation version always finished with result on master;
						qed"),
				}
			}
		}
	}

	fn maintain_sessions(&self) {
		self.sessions.stop_stalled_sessions();
		self.sessions.sessions_keep_alive();
	}

	fn start_servers_set_change_session(&self, params: ServersSetChangeParams) -> Result<Arc<AdminSession>, Error> {
		new_servers_set_change_session(
			self.self_key_pair.clone(),
			&*self.sessions,
			self.connections.clone(),
			self.servers_set_change_creator_connector.clone(),
			params,
		).map(|s| s.session)
	}
}
