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

use std::io;
use std::time;
use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;
use std::net::{SocketAddr, IpAddr};
use futures::{finished, failed, Future, Stream};
use futures_cpupool::CpuPool;
use parking_lot::{RwLock, Mutex};
use tokio_io::IoFuture;
use tokio_core::reactor::{Handle, Remote, Interval};
use tokio_core::net::{TcpListener, TcpStream};
use ethkey::{Public, KeyPair, Signature, Random, Generator};
use bigint::hash::H256;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage, KeyServerSet, NodeKeyPair};
use key_server_cluster::cluster_sessions::{ClusterSession, AdminSession, ClusterSessions, SessionIdWithSubSession,
	ClusterSessionsContainer, SERVERS_SET_CHANGE_SESSION_ID, create_cluster_view, AdminSessionCreationData, ClusterSessionsListener};
use key_server_cluster::cluster_sessions_creator::{ClusterSessionCreator, IntoSessionId};
use key_server_cluster::message::{self, Message, ClusterMessage};
use key_server_cluster::generation_session::{SessionImpl as GenerationSession};
use key_server_cluster::decryption_session::{SessionImpl as DecryptionSession};
use key_server_cluster::encryption_session::{SessionImpl as EncryptionSession};
use key_server_cluster::signing_session::{SessionImpl as SigningSession};
use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSession,
	IsolatedSessionTransport as KeyVersionNegotiationSessionTransport, ContinueAction};
use key_server_cluster::io::{DeadlineStatus, ReadMessage, SharedTcpStream, read_encrypted_message, WriteMessage, write_encrypted_message};
use key_server_cluster::net::{accept_connection as net_accept_connection, connect as net_connect, Connection as NetConnection};

/// Maintain interval (seconds). Every MAINTAIN_INTERVAL seconds node:
/// 1) checks if connected nodes are responding to KeepAlive messages
/// 2) tries to connect to disconnected nodes
/// 3) checks if enc/dec sessions are time-outed
const MAINTAIN_INTERVAL: u64 = 10;

/// When no messages have been received from node within KEEP_ALIVE_SEND_INTERVAL seconds,
/// we must send KeepAlive message to the node to check if it still responds to messages.
const KEEP_ALIVE_SEND_INTERVAL: u64 = 30;
/// When no messages have been received from node within KEEP_ALIVE_DISCONNECT_INTERVAL seconds,
/// we must treat this node as non-responding && disconnect from it.
const KEEP_ALIVE_DISCONNECT_INTERVAL: u64 = 60;

/// Empty future.
type BoxedEmptyFuture = Box<Future<Item = (), Error = ()> + Send>;

/// Cluster interface for external clients.
pub trait ClusterClient: Send + Sync {
	/// Get cluster state.
	fn cluster_state(&self) -> ClusterState;
	/// Start new generation session.
	fn new_generation_session(&self, session_id: SessionId, author: Public, threshold: usize) -> Result<Arc<GenerationSession>, Error>;
	/// Start new encryption session.
	fn new_encryption_session(&self, session_id: SessionId, requestor_signature: Signature, common_point: Public, encrypted_point: Public) -> Result<Arc<EncryptionSession>, Error>;
	/// Start new decryption session.
	fn new_decryption_session(&self, session_id: SessionId, requestor_signature: Signature, version: Option<H256>, is_shadow_decryption: bool) -> Result<Arc<DecryptionSession>, Error>;
	/// Start new signing session.
	fn new_signing_session(&self, session_id: SessionId, requestor_signature: Signature, version: Option<H256>, message_hash: H256) -> Result<Arc<SigningSession>, Error>;
	/// Start new key version negotiation session.
	fn new_key_version_negotiation_session(&self, session_id: SessionId) -> Result<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>, Error>;
	/// Start new servers set change session.
	fn new_servers_set_change_session(&self, session_id: Option<SessionId>, new_nodes_set: BTreeSet<NodeId>, old_set_signature: Signature, new_set_signature: Signature) -> Result<Arc<AdminSession>, Error>;

	/// Listen for new generation sessions.
	fn add_generation_listener(&self, listener: Arc<ClusterSessionsListener<GenerationSession>>);

	/// Ask node to make 'faulty' generation sessions.
	#[cfg(test)]
	fn make_faulty_generation_sessions(&self);
	/// Get active generation session with given id.
	#[cfg(test)]
	fn generation_session(&self, session_id: &SessionId) -> Option<Arc<GenerationSession>>;
	/// Try connect to disconnected nodes.
	#[cfg(test)]
	fn connect(&self);
	/// Get key storage.
	#[cfg(test)]
	fn key_storage(&self) -> Arc<KeyStorage>;
}

/// Cluster access for single session participant.
pub trait Cluster: Send + Sync {
	/// Broadcast message to all other nodes.
	fn broadcast(&self, message: Message) -> Result<(), Error>;
	/// Send message to given node.
	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error>;
	/// Is connected to given node?
	fn is_connected(&self, node: &NodeId) -> bool;
	/// Get a set of connected nodes.
	fn nodes(&self) -> BTreeSet<NodeId>;
}

/// Cluster initialization parameters.
#[derive(Clone)]
pub struct ClusterConfiguration {
	/// Number of threads reserved by cluster.
	pub threads: usize,
	/// Allow connecting to 'higher' nodes.
	pub allow_connecting_to_higher_nodes: bool,
	/// KeyPair this node holds.
	pub self_key_pair: Arc<NodeKeyPair>,
	/// Interface to listen to.
	pub listen_address: (String, u16),
	/// Cluster nodes set.
	pub key_server_set: Arc<KeyServerSet>,
	/// Reference to key storage
	pub key_storage: Arc<KeyStorage>,
	/// Reference to ACL storage
	pub acl_storage: Arc<AclStorage>,
	/// Administrator public key.
	pub admin_public: Option<Public>,
}

/// Cluster state.
pub struct ClusterState {
	/// Nodes, to which connections are established.
	pub connected: BTreeSet<NodeId>,
}

/// Network cluster implementation.
pub struct ClusterCore {
	/// Handle to the event loop.
	handle: Handle,
	/// Listen address.
	listen_address: SocketAddr,
	/// Cluster data.
	data: Arc<ClusterData>,
}

/// Network cluster client interface implementation.
pub struct ClusterClientImpl {
	/// Cluster data.
	data: Arc<ClusterData>,
}

/// Network cluster view. It is a communication channel, required in single session.
pub struct ClusterView {
	core: Arc<Mutex<ClusterViewCore>>,
}

/// Cross-thread shareable cluster data.
pub struct ClusterData {
	/// Cluster configuration.
	pub config: ClusterConfiguration,
	/// Handle to the event loop.
	pub handle: Remote,
	/// Handle to the cpu thread pool.
	pub pool: CpuPool,
	/// KeyPair this node holds.
	pub self_key_pair: Arc<NodeKeyPair>,
	/// Connections data.
	pub connections: ClusterConnections,
	/// Active sessions data.
	pub sessions: ClusterSessions,
}

/// Connections that are forming the cluster.
pub struct ClusterConnections {
	/// Self node id.
	pub self_node_id: NodeId,
	/// All known other key servers.
	pub key_server_set: Arc<KeyServerSet>,
	/// Connections data.
	pub data: RwLock<ClusterConnectionsData>,
}

/// Cluster connections data.
pub struct ClusterConnectionsData {
	/// Active key servers set.
	pub nodes: BTreeMap<Public, SocketAddr>,
	/// Active connections to key servers.
	pub connections: BTreeMap<NodeId, Arc<Connection>>,
}

/// Cluster view core.
struct ClusterViewCore {
	/// Cluster reference.
	cluster: Arc<ClusterData>,
	/// Subset of nodes, required for this session.
	nodes: BTreeSet<NodeId>,
}

/// Connection to single node.
pub struct Connection {
	/// Node id.
	node_id: NodeId,
	/// Node address.
	node_address: SocketAddr,
	/// Is inbound connection?
	is_inbound: bool,
	/// Tcp stream.
	stream: SharedTcpStream,
	/// Connection key.
	key: KeyPair,
	/// Last message time.
	last_message_time: Mutex<time::Instant>,
}

impl ClusterCore {
	pub fn new(handle: Handle, config: ClusterConfiguration) -> Result<Arc<Self>, Error> {
		let listen_address = make_socket_address(&config.listen_address.0, config.listen_address.1)?;
		let connections = ClusterConnections::new(&config)?;
		let sessions = ClusterSessions::new(&config);
		let data = ClusterData::new(&handle, config, connections, sessions);

		Ok(Arc::new(ClusterCore {
			handle: handle,
			listen_address: listen_address,
			data: data,
		}))
	}

	/// Create new client interface.
	pub fn client(&self) -> Arc<ClusterClient> {
		Arc::new(ClusterClientImpl::new(self.data.clone()))
	}

	/// Get cluster configuration.
	#[cfg(test)]
	pub fn config(&self) -> &ClusterConfiguration {
		&self.data.config
	}

	/// Get connection to given node.
	#[cfg(test)]
	pub fn connection(&self, node: &NodeId) -> Option<Arc<Connection>> {
		self.data.connection(node)
	}

	/// Run cluster.
	pub fn run(&self) -> Result<(), Error> {
		self.run_listener()
			.and_then(|_| self.run_connections())?;

		// schedule maintain procedures
		ClusterCore::schedule_maintain(&self.handle, self.data.clone());

		Ok(())
	}

	/// Start listening for incoming connections.
	pub fn run_listener(&self) -> Result<(), Error> {
		// start listeining for incoming connections
		self.handle.spawn(ClusterCore::listen(&self.handle, self.data.clone(), self.listen_address.clone())?);
		Ok(())
	}

	/// Start connecting to other nodes.
	pub fn run_connections(&self) -> Result<(), Error> {
		// try to connect to every other peer
		ClusterCore::connect_disconnected_nodes(self.data.clone());
		Ok(())
	}

	/// Connect to peer.
	fn connect(data: Arc<ClusterData>, node_address: SocketAddr) {
		data.handle.clone().spawn(move |handle| {
			data.pool.clone().spawn(ClusterCore::connect_future(handle, data, node_address))
		})
	}

	/// Connect to socket using given context and handle.
	fn connect_future(handle: &Handle, data: Arc<ClusterData>, node_address: SocketAddr) -> BoxedEmptyFuture {
		let disconnected_nodes = data.connections.disconnected_nodes().keys().cloned().collect();
		Box::new(net_connect(&node_address, handle, data.self_key_pair.clone(), disconnected_nodes)
			.then(move |result| ClusterCore::process_connection_result(data, Some(node_address), result))
			.then(|_| finished(())))
	}

	/// Start listening for incoming connections.
	fn listen(handle: &Handle, data: Arc<ClusterData>, listen_address: SocketAddr) -> Result<BoxedEmptyFuture, Error> {
		Ok(Box::new(TcpListener::bind(&listen_address, &handle)?
			.incoming()
			.and_then(move |(stream, node_address)| {
				ClusterCore::accept_connection(data.clone(), stream, node_address);
				Ok(())
			})
			.for_each(|_| Ok(()))
			.then(|_| finished(()))))
	}

	/// Accept connection.
	fn accept_connection(data: Arc<ClusterData>, stream: TcpStream, node_address: SocketAddr) {
		data.handle.clone().spawn(move |handle| {
			data.pool.clone().spawn(ClusterCore::accept_connection_future(handle, data, stream, node_address))
		})
	}

	/// Accept connection future.
	fn accept_connection_future(handle: &Handle, data: Arc<ClusterData>, stream: TcpStream, node_address: SocketAddr) -> BoxedEmptyFuture {
		Box::new(net_accept_connection(node_address, stream, handle, data.self_key_pair.clone())
			.then(move |result| ClusterCore::process_connection_result(data, None, result))
			.then(|_| finished(())))
	}

	/// Schedule mainatain procedures.
	fn schedule_maintain(handle: &Handle, data: Arc<ClusterData>) {
		let d = data.clone();
		let interval: BoxedEmptyFuture = Box::new(Interval::new(time::Duration::new(MAINTAIN_INTERVAL, 0), handle)
			.expect("failed to create interval")
			.and_then(move |_| Ok(ClusterCore::maintain(data.clone())))
			.for_each(|_| Ok(()))
			.then(|_| finished(())));

		d.spawn(interval);
	}

	/// Execute maintain procedures.
	fn maintain(data: Arc<ClusterData>) {
		trace!(target: "secretstore_net", "{}: executing maintain procedures", data.self_key_pair.public());

		ClusterCore::keep_alive(data.clone());
		ClusterCore::connect_disconnected_nodes(data.clone());
		data.sessions.stop_stalled_sessions();
	}

	/// Called for every incomming mesage.
	fn process_connection_messages(data: Arc<ClusterData>, connection: Arc<Connection>) -> IoFuture<Result<(), Error>> {
		Box::new(connection
			.read_message()
			.then(move |result|
				match result {
					Ok((_, Ok(message))) => {
						ClusterCore::process_connection_message(data.clone(), connection.clone(), message);
						// continue serving connection
						data.spawn(ClusterCore::process_connection_messages(data.clone(), connection));
						Box::new(finished(Ok(())))
					},
					Ok((_, Err(err))) => {
						warn!(target: "secretstore_net", "{}: protocol error '{}' when reading message from node {}", data.self_key_pair.public(), err, connection.node_id());
						// continue serving connection
						data.spawn(ClusterCore::process_connection_messages(data.clone(), connection));
						Box::new(finished(Err(err)))
					},
					Err(err) => {
						warn!(target: "secretstore_net", "{}: network error '{}' when reading message from node {}", data.self_key_pair.public(), err, connection.node_id());
						// close connection
						data.connections.remove(connection.node_id(), connection.is_inbound());
						Box::new(failed(err))
					},
				}
			))
	}

	/// Send keepalive messages to every othe node.
	fn keep_alive(data: Arc<ClusterData>) {
		data.sessions.sessions_keep_alive();
		for connection in data.connections.active_connections() {
			let last_message_diff = time::Instant::now() - connection.last_message_time();
			if last_message_diff > time::Duration::from_secs(KEEP_ALIVE_DISCONNECT_INTERVAL) {
				data.connections.remove(connection.node_id(), connection.is_inbound());
				data.sessions.on_connection_timeout(connection.node_id());
			}
			else if last_message_diff > time::Duration::from_secs(KEEP_ALIVE_SEND_INTERVAL) {
				data.spawn(connection.send_message(Message::Cluster(ClusterMessage::KeepAlive(message::KeepAlive {}))));
			}
		}
	}

	/// Try to connect to every disconnected node.
	fn connect_disconnected_nodes(data: Arc<ClusterData>) {
		// do not update nodes set if any admin session is active
		// this could happen, but will possibly lead to admin session error
		// => should be performed later
		if data.sessions.admin_sessions.is_empty() {
			data.connections.update_nodes_set();
		}

		for (node_id, node_address) in data.connections.disconnected_nodes() {
			if data.config.allow_connecting_to_higher_nodes || data.self_key_pair.public() < &node_id {
				ClusterCore::connect(data.clone(), node_address);
			}
		}
	}

	/// Process connection future result.
	fn process_connection_result(data: Arc<ClusterData>, outbound_addr: Option<SocketAddr>, result: Result<DeadlineStatus<Result<NetConnection, Error>>, io::Error>) -> IoFuture<Result<(), Error>> {
		match result {
			Ok(DeadlineStatus::Meet(Ok(connection))) => {
				let connection = Connection::new(outbound_addr.is_none(), connection);
				if data.connections.insert(connection.clone()) {
					ClusterCore::process_connection_messages(data.clone(), connection)
				} else {
					Box::new(finished(Ok(())))
				}
			},
			Ok(DeadlineStatus::Meet(Err(err))) => {
				warn!(target: "secretstore_net", "{}: protocol error '{}' when establishing {} connection{}",
					data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				Box::new(finished(Ok(())))
			},
			Ok(DeadlineStatus::Timeout) => {
				warn!(target: "secretstore_net", "{}: timeout when establishing {} connection{}",
					data.self_key_pair.public(), if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				Box::new(finished(Ok(())))
			},
			Err(err) => {
				warn!(target: "secretstore_net", "{}: network error '{}' when establishing {} connection{}",
					data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				Box::new(finished(Ok(())))
			},
		}
	}

	/// Process single message from the connection.
	fn process_connection_message(data: Arc<ClusterData>, connection: Arc<Connection>, message: Message) {
		connection.set_last_message_time(time::Instant::now());
		trace!(target: "secretstore_net", "{}: received message {} from {}", data.self_key_pair.public(), message, connection.node_id());
		// error is ignored as we only process errors on session level
		match message {
			Message::Generation(message) => Self::process_message(&data, &data.sessions.generation_sessions, connection, Message::Generation(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Encryption(message) => Self::process_message(&data, &data.sessions.encryption_sessions, connection, Message::Encryption(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Decryption(message) => Self::process_message(&data, &data.sessions.decryption_sessions, connection, Message::Decryption(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Signing(message) => Self::process_message(&data, &data.sessions.signing_sessions, connection, Message::Signing(message))
				.map(|_| ()).unwrap_or_default(),
			Message::ServersSetChange(message) => Self::process_message(&data, &data.sessions.admin_sessions, connection, Message::ServersSetChange(message))
				.map(|_| ()).unwrap_or_default(),
			Message::KeyVersionNegotiation(message) => {
				let session = Self::process_message(&data, &data.sessions.negotiation_sessions, connection, Message::KeyVersionNegotiation(message));
				Self::try_continue_session(&data, session);
			},
			Message::ShareAdd(message) => Self::process_message(&data, &data.sessions.admin_sessions, connection, Message::ShareAdd(message))
				.map(|_| ()).unwrap_or_default(),
			Message::Cluster(message) => ClusterCore::process_cluster_message(data, connection, message),
		}
	}

	/// Try to contnue session.
	fn try_continue_session(data: &Arc<ClusterData>, session: Option<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>>) {
		if let Some(session) = session {
			let meta = session.meta();
			let is_master_node = meta.self_node_id == meta.master_node_id;
			if is_master_node && session.is_finished() {
				data.sessions.negotiation_sessions.remove(&session.id());
				match session.wait() {
					Ok((version, master)) => match session.continue_action() {
						Some(ContinueAction::Decrypt(session, is_shadow_decryption)) => {
							let initialization_error = if data.self_key_pair.public() == &master {
								session.initialize(version, is_shadow_decryption)
							} else {
								session.delegate(master, version, is_shadow_decryption)
							};

							if let Err(error) = initialization_error {
								session.on_session_error(&meta.self_node_id, error);
								data.sessions.decryption_sessions.remove(&session.id());
							}
						},
						Some(ContinueAction::Sign(session, message_hash)) => {
							let initialization_error = if data.self_key_pair.public() == &master {
								session.initialize(version, message_hash)
							} else {
								session.delegate(master, version, message_hash)
							};

							if let Err(error) = initialization_error {
								session.on_session_error(&meta.self_node_id, error);
								data.sessions.signing_sessions.remove(&session.id());
							}
						},
						None => (),
					},
					Err(error) => match session.continue_action() {
						Some(ContinueAction::Decrypt(session, _)) => {
							data.sessions.decryption_sessions.remove(&session.id());
							session.on_session_error(&meta.self_node_id, error);
						},
						Some(ContinueAction::Sign(session, _)) => {
							data.sessions.signing_sessions.remove(&session.id());
							session.on_session_error(&meta.self_node_id, error);
						},
						None => (),
					},
				}
			}
		}
	}

	/// Get or insert new session.
	fn prepare_session<S: ClusterSession, SC: ClusterSessionCreator<S, D>, D>(data: &Arc<ClusterData>, sessions: &ClusterSessionsContainer<S, SC, D>, sender: &NodeId, message: &Message) -> Result<Arc<S>, Error>
		where Message: IntoSessionId<S::Id> {
		fn requires_all_connections(message: &Message) -> bool {
			match *message {
				Message::Generation(_) => true,
				Message::ShareAdd(_) => true,
				Message::ServersSetChange(_) => true,
				_ => false,
			}
		}

		// get or create new session, if required
		let session_id = message.into_session_id().expect("into_session_id fails for cluster messages only; only session messages are passed to prepare_session; qed");
		let is_initialization_message = message.is_initialization_message();
		let is_delegation_message = message.is_delegation_message();
		match is_initialization_message || is_delegation_message {
			false => sessions.get(&session_id, true).ok_or(Error::InvalidSessionId),
			true => {
				let creation_data = SC::creation_data_from_message(&message)?;
				let master = if is_initialization_message { sender.clone() } else { data.self_key_pair.public().clone() };
				let cluster = create_cluster_view(data, requires_all_connections(&message))?;
				sessions.insert(cluster, master, session_id, Some(message.session_nonce().ok_or(Error::InvalidMessage)?), message.is_exclusive_session_message(), creation_data)
			},
		}
	}

	/// Process single session message from connection.
	fn process_message<S: ClusterSession, SC: ClusterSessionCreator<S, D>, D>(data: &Arc<ClusterData>, sessions: &ClusterSessionsContainer<S, SC, D>, connection: Arc<Connection>, mut message: Message) -> Option<Arc<S>>
		where Message: IntoSessionId<S::Id> {

		// get or create new session, if required
		let mut sender = connection.node_id().clone();
		let session = Self::prepare_session(data, sessions, &sender, &message);
		// send error if session is not found, or failed to create
		let session = match session {
			Ok(session) => session,
			Err(error) => {
				// this is new session => it is not yet in container
				warn!(target: "secretstore_net", "{}: {} session read error '{}' when requested for session from node {}",
					data.self_key_pair.public(), S::type_name(), error, sender);
				if message.is_initialization_message() {
					let session_id = message.into_session_id().expect("session_id only fails for cluster messages; only session messages are passed to process_message; qed");
					let session_nonce = message.session_nonce().expect("session_nonce only fails for cluster messages; only session messages are passed to process_message; qed");
					data.spawn(connection.send_message(SC::make_error_message(session_id, session_nonce, error)));
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
						info!(target: "secretstore_net", "{}: {} session completed", data.self_key_pair.public(), S::type_name());
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
					warn!(target: "secretstore_net", "{}: {} session error '{}' when processing message {} from node {}",
						data.self_key_pair.public(),
						S::type_name(),
						err,
						message,
						sender);
					session.on_session_error(data.self_key_pair.public(), err);
					sessions.remove(&session_id);
					return Some(session);
				},
			}
		}
	}

	/// Process single cluster message from the connection.
	fn process_cluster_message(data: Arc<ClusterData>, connection: Arc<Connection>, message: ClusterMessage) {
		match message {
			ClusterMessage::KeepAlive(_) => data.spawn(connection.send_message(Message::Cluster(ClusterMessage::KeepAliveResponse(message::KeepAliveResponse {
				session_id: None,
			})))),
			ClusterMessage::KeepAliveResponse(msg) => if let Some(session_id) = msg.session_id {
				data.sessions.on_session_keep_alive(connection.node_id(), session_id.into());
			},
			_ => warn!(target: "secretstore_net", "{}: received unexpected message {} from node {} at {}", data.self_key_pair.public(), message, connection.node_id(), connection.node_address()),
		}
	}
}

impl ClusterConnections {
	pub fn new(config: &ClusterConfiguration) -> Result<Self, Error> {
		let mut nodes = config.key_server_set.get();
		nodes.remove(config.self_key_pair.public());

		Ok(ClusterConnections {
			self_node_id: config.self_key_pair.public().clone(),
			key_server_set: config.key_server_set.clone(),
			data: RwLock::new(ClusterConnectionsData {
				nodes: nodes,
				connections: BTreeMap::new(),
			}),
		})
	}

	pub fn cluster_state(&self) -> ClusterState {
		ClusterState {
			connected: self.data.read().connections.keys().cloned().collect(),
		}
	}

	pub fn get(&self, node: &NodeId) -> Option<Arc<Connection>> {
		self.data.read().connections.get(node).cloned()
	}

	pub fn insert(&self, connection: Arc<Connection>) -> bool {
		let mut data = self.data.write();
		if !data.nodes.contains_key(connection.node_id()) {
			// incoming connections are checked here
			trace!(target: "secretstore_net", "{}: ignoring unknown connection from {} at {}", self.self_node_id, connection.node_id(), connection.node_address());
			debug_assert!(connection.is_inbound());
			return false;
		}
		if data.connections.contains_key(connection.node_id()) {
			// we have already connected to the same node
			// the agreement is that node with lower id must establish connection to node with higher id
			if (&self.self_node_id < connection.node_id() && connection.is_inbound())
				|| (&self.self_node_id > connection.node_id() && !connection.is_inbound()) {
				return false;
			}
		}

		trace!(target: "secretstore_net", "{}: inserting connection to {} at {}. Connected to {} of {} nodes",
			self.self_node_id, connection.node_id(), connection.node_address(), data.connections.len() + 1, data.nodes.len());
		data.connections.insert(connection.node_id().clone(), connection);
		true
	}

	pub fn remove(&self, node: &NodeId, is_inbound: bool) {
		let mut data = self.data.write();
		if let Entry::Occupied(entry) = data.connections.entry(node.clone()) {
			if entry.get().is_inbound() != is_inbound {
				return;
			}

			trace!(target: "secretstore_net", "{}: removing connection to {} at {}", self.self_node_id, entry.get().node_id(), entry.get().node_address());
			entry.remove_entry();
		}
	}

	pub fn connected_nodes(&self) -> BTreeSet<NodeId> {
		self.data.read().connections.keys().cloned().collect()
	}

	pub fn active_connections(&self)-> Vec<Arc<Connection>> {
		self.data.read().connections.values().cloned().collect()
	}

	pub fn disconnected_nodes(&self) -> BTreeMap<NodeId, SocketAddr> {
		let data = self.data.read();
		data.nodes.iter()
			.filter(|&(node_id, _)| !data.connections.contains_key(node_id))
			.map(|(node_id, node_address)| (node_id.clone(), node_address.clone()))
			.collect()
	}

	pub fn update_nodes_set(&self) {
		let mut data = self.data.write();
		let mut new_nodes = self.key_server_set.get();
		// we do not need to connect to self
		// + we do not need to try to connect to any other node if we are not the part of a cluster
		if new_nodes.remove(&self.self_node_id).is_none() {
			new_nodes.clear();
		}

		let mut num_added_nodes = 0;
		let mut num_removed_nodes = 0;
		let mut num_changed_nodes = 0;

		for obsolete_node in data.nodes.keys().cloned().collect::<Vec<_>>() {
			if !new_nodes.contains_key(&obsolete_node) {
				if let Entry::Occupied(entry) = data.connections.entry(obsolete_node) {
					trace!(target: "secretstore_net", "{}: removing connection to {} at {}", self.self_node_id, entry.get().node_id(), entry.get().node_address());
					entry.remove();
				}

				data.nodes.remove(&obsolete_node);
				num_removed_nodes += 1;
			}
		}

		for (new_node_public, new_node_addr) in new_nodes {
			match data.nodes.insert(new_node_public, new_node_addr) {
				None => num_added_nodes += 1,
				Some(old_node_addr) => if new_node_addr != old_node_addr {
					if let Entry::Occupied(entry) = data.connections.entry(new_node_public) {
						trace!(target: "secretstore_net", "{}: removing connection to {} at {}", self.self_node_id, entry.get().node_id(), entry.get().node_address());
						entry.remove();
					}
					num_changed_nodes += 1;
				},
			}
		}

		if num_added_nodes != 0 || num_removed_nodes != 0 || num_changed_nodes != 0 {
			trace!(target: "secretstore_net", "{}: updated nodes set: removed {}, added {}, changed {}. Connected to {} of {} nodes",
				self.self_node_id, num_removed_nodes, num_added_nodes, num_changed_nodes, data.connections.len(), data.nodes.len());
		}
	}
}

impl ClusterData {
	pub fn new(handle: &Handle, config: ClusterConfiguration, connections: ClusterConnections, sessions: ClusterSessions) -> Arc<Self> {
		Arc::new(ClusterData {
			handle: handle.remote().clone(),
			pool: CpuPool::new(config.threads),
			self_key_pair: config.self_key_pair.clone(),
			connections: connections,
			sessions: sessions,
			config: config,
		})
	}

	/// Get connection to given node.
	pub fn connection(&self, node: &NodeId) -> Option<Arc<Connection>> {
		self.connections.get(node)
	}

	/// Spawns a future using thread pool and schedules execution of it with event loop handle.
	pub fn spawn<F>(&self, f: F) where F: Future + Send + 'static, F::Item: Send + 'static, F::Error: Send + 'static {
		let pool_work = self.pool.spawn(f);
		self.handle.spawn(move |_handle| {
			pool_work.then(|_| finished(()))
		})
	}
}

impl Connection {
	pub fn new(is_inbound: bool, connection: NetConnection) -> Arc<Connection> {
		Arc::new(Connection {
			node_id: connection.node_id,
			node_address: connection.address,
			is_inbound: is_inbound,
			stream: connection.stream,
			key: connection.key,
			last_message_time: Mutex::new(time::Instant::now()),
		})
	}

	pub fn is_inbound(&self) -> bool {
		self.is_inbound
	}

	pub fn node_id(&self) -> &NodeId {
		&self.node_id
	}

	pub fn last_message_time(&self) -> time::Instant {
		*self.last_message_time.lock()
	}

	pub fn set_last_message_time(&self, last_message_time: time::Instant) {
		*self.last_message_time.lock() = last_message_time;
	}

	pub fn node_address(&self) -> &SocketAddr {
		&self.node_address
	}

	pub fn send_message(&self, message: Message) -> WriteMessage<SharedTcpStream> {
		write_encrypted_message(self.stream.clone(), &self.key, message)
	}

	pub fn read_message(&self) -> ReadMessage<SharedTcpStream> {
		read_encrypted_message(self.stream.clone(), self.key.clone())
	}
}

impl ClusterView {
	pub fn new(cluster: Arc<ClusterData>, nodes: BTreeSet<NodeId>) -> Self {
		ClusterView {
			core: Arc::new(Mutex::new(ClusterViewCore {
				cluster: cluster,
				nodes: nodes,
			})),
		}
	}
}

impl Cluster for ClusterView {
	fn broadcast(&self, message: Message) -> Result<(), Error> {
		let core = self.core.lock();
		for node in core.nodes.iter().filter(|n| *n != core.cluster.self_key_pair.public()) {
			trace!(target: "secretstore_net", "{}: sent message {} to {}", core.cluster.self_key_pair.public(), message, node);
			let connection = core.cluster.connection(node).ok_or(Error::NodeDisconnected)?;
			core.cluster.spawn(connection.send_message(message.clone()))
		}
		Ok(())
	}

	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error> {
		let core = self.core.lock();
		trace!(target: "secretstore_net", "{}: sent message {} to {}", core.cluster.self_key_pair.public(), message, to);
		let connection = core.cluster.connection(to).ok_or(Error::NodeDisconnected)?;
		core.cluster.spawn(connection.send_message(message));
		Ok(())
	}

	fn is_connected(&self, node: &NodeId) -> bool {
		self.core.lock().nodes.contains(node)
	}

	fn nodes(&self) -> BTreeSet<NodeId> {
		self.core.lock().nodes.clone()
	}
}

impl ClusterClientImpl {
	pub fn new(data: Arc<ClusterData>) -> Self {
		ClusterClientImpl {
			data: data,
		}
	}

	fn create_key_version_negotiation_session(&self, session_id: SessionId) -> Result<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let access_key = Random.generate()?.secret().clone();
		let session_id = SessionIdWithSubSession::new(session_id, access_key);
		let cluster = create_cluster_view(&self.data, false)?;
		let session = self.data.sessions.negotiation_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id.clone(), None, false, None)?;
		match session.initialize(connected_nodes) {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.negotiation_sessions.remove(&session.id());
				Err(error)
			}
		}
	}
}

impl ClusterClient for ClusterClientImpl {
	fn cluster_state(&self) -> ClusterState {
		self.data.connections.cluster_state()
	}

	fn new_generation_session(&self, session_id: SessionId, author: Public, threshold: usize) -> Result<Arc<GenerationSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let cluster = create_cluster_view(&self.data, true)?;
		let session = self.data.sessions.generation_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id, None, false, None)?;
		match session.initialize(author, threshold, connected_nodes) {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.generation_sessions.remove(&session.id());
				Err(error)
			},
		}
	}

	fn new_encryption_session(&self, session_id: SessionId, requestor_signature: Signature, common_point: Public, encrypted_point: Public) -> Result<Arc<EncryptionSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let cluster = create_cluster_view(&self.data, true)?;
		let session = self.data.sessions.encryption_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id, None, false, None)?;
		match session.initialize(requestor_signature, common_point, encrypted_point) {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.encryption_sessions.remove(&session.id());
				Err(error)
			},
		}
	}

	fn new_decryption_session(&self, session_id: SessionId, requestor_signature: Signature, version: Option<H256>, is_shadow_decryption: bool) -> Result<Arc<DecryptionSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let access_key = Random.generate()?.secret().clone();
		let session_id = SessionIdWithSubSession::new(session_id, access_key);
		let cluster = create_cluster_view(&self.data, false)?;
		let session = self.data.sessions.decryption_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id.clone(), None, false, Some(requestor_signature))?;

		let initialization_result = match version {
			Some(version) => session.initialize(version, is_shadow_decryption),
			None => {
				self.create_key_version_negotiation_session(session_id.id.clone())
					.map(|version_session| {
						version_session.set_continue_action(ContinueAction::Decrypt(session.clone(), is_shadow_decryption));
						ClusterCore::try_continue_session(&self.data, Some(version_session));
					})
			},
		};

		match initialization_result {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.decryption_sessions.remove(&session.id());
				Err(error)
			},
		}
	}

	fn new_signing_session(&self, session_id: SessionId, requestor_signature: Signature, version: Option<H256>, message_hash: H256) -> Result<Arc<SigningSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let access_key = Random.generate()?.secret().clone();
		let session_id = SessionIdWithSubSession::new(session_id, access_key);
		let cluster = create_cluster_view(&self.data, false)?;
		let session = self.data.sessions.signing_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id.clone(), None, false, Some(requestor_signature))?;

		let initialization_result = match version {
			Some(version) => session.initialize(version, message_hash),
			None => {
				self.create_key_version_negotiation_session(session_id.id.clone())
					.map(|version_session| {
						version_session.set_continue_action(ContinueAction::Sign(session.clone(), message_hash));
						ClusterCore::try_continue_session(&self.data, Some(version_session));
					})
			},
		};

		match initialization_result {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.signing_sessions.remove(&session.id());
				Err(error)
			},
		}
	}

	fn new_key_version_negotiation_session(&self, session_id: SessionId) -> Result<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>, Error> {
		let session = self.create_key_version_negotiation_session(session_id)?;
		Ok(session)
	}

	fn new_servers_set_change_session(&self, session_id: Option<SessionId>, new_nodes_set: BTreeSet<NodeId>, old_set_signature: Signature, new_set_signature: Signature) -> Result<Arc<AdminSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let session_id = match session_id {
			Some(session_id) if session_id == *SERVERS_SET_CHANGE_SESSION_ID => session_id,
			Some(_) => return Err(Error::InvalidMessage),
			None => *SERVERS_SET_CHANGE_SESSION_ID,
		};

		let cluster = create_cluster_view(&self.data, true)?;
		let session = self.data.sessions.admin_sessions.insert(cluster, self.data.self_key_pair.public().clone(), session_id, None, true, Some(AdminSessionCreationData::ServersSetChange))?;
		let initialization_result = session.as_servers_set_change().expect("servers set change session is created; qed")
			.initialize(new_nodes_set, old_set_signature, new_set_signature);

		match initialization_result {
			Ok(()) => Ok(session),
			Err(error) => {
				self.data.sessions.admin_sessions.remove(&session.id());
				Err(error)
			},
		}
	}

	fn add_generation_listener(&self, listener: Arc<ClusterSessionsListener<GenerationSession>>) {
		self.data.sessions.generation_sessions.add_listener(listener);
	}

	#[cfg(test)]
	fn connect(&self) {
		ClusterCore::connect_disconnected_nodes(self.data.clone());
	}

	#[cfg(test)]
	fn make_faulty_generation_sessions(&self) {
		self.data.sessions.make_faulty_generation_sessions();
	}

	#[cfg(test)]
	fn generation_session(&self, session_id: &SessionId) -> Option<Arc<GenerationSession>> {
		self.data.sessions.generation_sessions.get(session_id, false)
	}

	#[cfg(test)]
	fn key_storage(&self) -> Arc<KeyStorage> {
		self.data.config.key_storage.clone()
	}
}

fn make_socket_address(address: &str, port: u16) -> Result<SocketAddr, Error> {
	let ip_address: IpAddr = address.parse().map_err(|_| Error::InvalidNodeAddress)?;
	Ok(SocketAddr::new(ip_address, port))
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::time;
	use std::collections::{BTreeSet, VecDeque};
	use parking_lot::Mutex;
	use tokio_core::reactor::Core;
	use bigint::hash::H256;
	use ethkey::{Random, Generator, Public, Signature, sign};
	use key_server_cluster::{NodeId, SessionId, Error, DummyAclStorage, DummyKeyStorage, MapKeyServerSet, PlainNodeKeyPair, KeyStorage};
	use key_server_cluster::message::Message;
	use key_server_cluster::cluster::{Cluster, ClusterCore, ClusterConfiguration, ClusterClient, ClusterState};
	use key_server_cluster::cluster_sessions::{ClusterSession, AdminSession, ClusterSessionsListener};
	use key_server_cluster::generation_session::{SessionImpl as GenerationSession, SessionState as GenerationSessionState};
	use key_server_cluster::decryption_session::{SessionImpl as DecryptionSession};
	use key_server_cluster::encryption_session::{SessionImpl as EncryptionSession};
	use key_server_cluster::signing_session::{SessionImpl as SigningSession};
	use key_server_cluster::key_version_negotiation_session::{SessionImpl as KeyVersionNegotiationSession,
		IsolatedSessionTransport as KeyVersionNegotiationSessionTransport};

	#[derive(Default)]
	pub struct DummyClusterClient;

	#[derive(Debug)]
	pub struct DummyCluster {
		id: NodeId,
		data: Mutex<DummyClusterData>,
	}

	#[derive(Debug, Default)]
	struct DummyClusterData {
		nodes: BTreeSet<NodeId>,
		messages: VecDeque<(NodeId, Message)>,
	}

	impl ClusterClient for DummyClusterClient {
		fn cluster_state(&self) -> ClusterState { unimplemented!() }
		fn new_generation_session(&self, _session_id: SessionId, _author: Public, _threshold: usize) -> Result<Arc<GenerationSession>, Error> { unimplemented!() }
		fn new_encryption_session(&self, _session_id: SessionId, _requestor_signature: Signature, _common_point: Public, _encrypted_point: Public) -> Result<Arc<EncryptionSession>, Error> { unimplemented!() }
		fn new_decryption_session(&self, _session_id: SessionId, _requestor_signature: Signature, _version: Option<H256>, _is_shadow_decryption: bool) -> Result<Arc<DecryptionSession>, Error> { unimplemented!() }
		fn new_signing_session(&self, _session_id: SessionId, _requestor_signature: Signature, _version: Option<H256>, _message_hash: H256) -> Result<Arc<SigningSession>, Error> { unimplemented!() }
		fn new_key_version_negotiation_session(&self, _session_id: SessionId) -> Result<Arc<KeyVersionNegotiationSession<KeyVersionNegotiationSessionTransport>>, Error> { unimplemented!() }
		fn new_servers_set_change_session(&self, _session_id: Option<SessionId>, _new_nodes_set: BTreeSet<NodeId>, _old_set_signature: Signature, _new_set_signature: Signature) -> Result<Arc<AdminSession>, Error> { unimplemented!() }

		fn add_generation_listener(&self, _listener: Arc<ClusterSessionsListener<GenerationSession>>) {}

		fn make_faulty_generation_sessions(&self) { unimplemented!() }
		fn generation_session(&self, _session_id: &SessionId) -> Option<Arc<GenerationSession>> { unimplemented!() }
		fn connect(&self) { unimplemented!() }
		fn key_storage(&self) -> Arc<KeyStorage> { unimplemented!() }
	}

	impl DummyCluster {
		pub fn new(id: NodeId) -> Self {
			DummyCluster {
				id: id,
				data: Mutex::new(DummyClusterData::default())
			}
		}

		pub fn node(&self) -> NodeId {
			self.id.clone()
		}

		pub fn add_node(&self, node: NodeId) {
			self.data.lock().nodes.insert(node);
		}

		pub fn add_nodes<I: Iterator<Item=NodeId>>(&self, nodes: I) {
			self.data.lock().nodes.extend(nodes)
		}

		pub fn remove_node(&self, node: &NodeId) {
			self.data.lock().nodes.remove(node);
		}

		pub fn take_message(&self) -> Option<(NodeId, Message)> {
			self.data.lock().messages.pop_front()
		}
	}

	impl Cluster for DummyCluster {
		fn broadcast(&self, message: Message) -> Result<(), Error> {
			let mut data = self.data.lock();
			let all_nodes: Vec<_> = data.nodes.iter().cloned().filter(|n| n != &self.id).collect();
			for node in all_nodes {
				data.messages.push_back((node, message.clone()));
			}
			Ok(())
		}

		fn send(&self, to: &NodeId, message: Message) -> Result<(), Error> {
			debug_assert!(&self.id != to);
			self.data.lock().messages.push_back((to.clone(), message));
			Ok(())
		}

		fn is_connected(&self, node: &NodeId) -> bool {
			let data = self.data.lock();
			&self.id == node || data.nodes.contains(node)
		}

		fn nodes(&self) -> BTreeSet<NodeId> {
			self.data.lock().nodes.iter().cloned().collect()
		}
	}

	pub fn loop_until<F>(core: &mut Core, timeout: time::Duration, predicate: F) where F: Fn() -> bool {
		let start = time::Instant::now();
		loop {
			core.turn(Some(time::Duration::from_millis(1)));
			if predicate() {
				break;
			}

			if time::Instant::now() - start > timeout {
				panic!("no result in {:?}", timeout);
			}
		}
	}

	pub fn all_connections_established(cluster: &Arc<ClusterCore>) -> bool {
		cluster.config().key_server_set.get().keys()
			.filter(|p| *p != cluster.config().self_key_pair.public())
			.all(|p| cluster.connection(p).is_some())
	}

	pub fn make_clusters(core: &Core, ports_begin: u16, num_nodes: usize) -> Vec<Arc<ClusterCore>> {
		let key_pairs: Vec<_> = (0..num_nodes).map(|_| Random.generate().unwrap()).collect();
		let cluster_params: Vec<_> = (0..num_nodes).map(|i| ClusterConfiguration {
			threads: 1,
			self_key_pair: Arc::new(PlainNodeKeyPair::new(key_pairs[i].clone())),
			listen_address: ("127.0.0.1".to_owned(), ports_begin + i as u16),
			key_server_set: Arc::new(MapKeyServerSet::new(key_pairs.iter().enumerate()
				.map(|(j, kp)| (kp.public().clone(), format!("127.0.0.1:{}", ports_begin + j as u16).parse().unwrap()))
				.collect())),
			allow_connecting_to_higher_nodes: false,
			key_storage: Arc::new(DummyKeyStorage::default()),
			acl_storage: Arc::new(DummyAclStorage::default()),
			admin_public: None,
		}).collect();
		let clusters: Vec<_> = cluster_params.into_iter().enumerate()
			.map(|(_, params)| ClusterCore::new(core.handle(), params).unwrap())
			.collect();

		clusters
	}

	pub fn run_clusters(clusters: &[Arc<ClusterCore>]) {
		for cluster in clusters {
			cluster.run_listener().unwrap();
		}
		for cluster in clusters {
			cluster.run_connections().unwrap();
		}
	}

	#[test]
	fn cluster_connects_to_other_nodes() {
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6010, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));
	}

	#[test]
	fn cluster_wont_start_generation_session_if_not_fully_connected() {
		let core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6013, 3);
		clusters[0].run().unwrap();
		match clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1) {
			Err(Error::NodeDisconnected) => (),
			Err(e) => panic!("unexpected error {:?}", e),
			_ => panic!("unexpected success"),
		}
	}

	#[test]
	fn error_in_generation_session_broadcasted_to_all_other_nodes() {
		//::logger::init_log();
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6016, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// ask one of nodes to produce faulty generation sessions
		clusters[1].client().make_faulty_generation_sessions();

		// start && wait for generation session to fail
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some()
			&& clusters[0].client().generation_session(&SessionId::default()).is_none());
		assert!(session.joint_public_and_secret().unwrap().is_err());

		// check that faulty session is either removed from all nodes, or nonexistent (already removed)
		for i in 1..3 {
			if let Some(session) = clusters[i].client().generation_session(&SessionId::default()) {
				// wait for both session completion && session removal (session completion event is fired
				// before session is removed from its own container by cluster)
				loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some()
					&& clusters[i].client().generation_session(&SessionId::default()).is_none());
				assert!(session.joint_public_and_secret().unwrap().is_err());
			}
		}
	}

	#[test]
	fn generation_session_completion_signalled_if_failed_on_master() {
		//::logger::init_log();
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6025, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// ask one of nodes to produce faulty generation sessions
		clusters[0].client().make_faulty_generation_sessions();

		// start && wait for generation session to fail
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some()
			&& clusters[0].client().generation_session(&SessionId::default()).is_none());
		assert!(session.joint_public_and_secret().unwrap().is_err());

		// check that faulty session is either removed from all nodes, or nonexistent (already removed)
		for i in 1..3 {
			if let Some(session) = clusters[i].client().generation_session(&SessionId::default()) {
				// wait for both session completion && session removal (session completion event is fired
				// before session is removed from its own container by cluster)
				loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some()
					&& clusters[i].client().generation_session(&SessionId::default()).is_none());
				assert!(session.joint_public_and_secret().unwrap().is_err());
			}
		}
	}

	#[test]
	fn generation_session_is_removed_when_succeeded() {
		//::logger::init_log();
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6019, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// start && wait for generation session to complete
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || (session.state() == GenerationSessionState::Finished
			|| session.state() == GenerationSessionState::Failed)
			&& clusters[0].client().generation_session(&SessionId::default()).is_none());
		assert!(session.joint_public_and_secret().unwrap().is_ok());

		// check that session is either removed from all nodes, or nonexistent (already removed)
		for i in 1..3 {
			if let Some(session) = clusters[i].client().generation_session(&SessionId::default()) {
				loop_until(&mut core, time::Duration::from_millis(300), || (session.state() == GenerationSessionState::Finished
					|| session.state() == GenerationSessionState::Failed)
					&& clusters[i].client().generation_session(&SessionId::default()).is_none());
				assert!(session.joint_public_and_secret().unwrap().is_err());
			}
		}
	}

	#[test]
	fn sessions_are_removed_when_initialization_fails() {
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6022, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// generation session
		{
			// try to start generation session => fail in initialization
			assert_eq!(clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 100).map(|_| ()),
				Err(Error::InvalidThreshold));

			// try to start generation session => fails in initialization
			assert_eq!(clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 100).map(|_| ()),
				Err(Error::InvalidThreshold));
		
			assert!(clusters[0].data.sessions.generation_sessions.is_empty());
		}

		// decryption session
		{
			// try to start decryption session => fails in initialization
			assert_eq!(clusters[0].client().new_decryption_session(Default::default(), Default::default(), Some(Default::default()), false).map(|_| ()),
				Err(Error::InvalidMessage));

			// try to start generation session => fails in initialization
			assert_eq!(clusters[0].client().new_decryption_session(Default::default(), Default::default(), Some(Default::default()), false).map(|_| ()),
				Err(Error::InvalidMessage));

			assert!(clusters[0].data.sessions.decryption_sessions.is_empty());
			assert!(clusters[0].data.sessions.negotiation_sessions.is_empty());
		}
	}

	#[test]
	fn signing_session_completes_if_node_does_not_have_a_share() {
		//::logger::init_log();
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6028, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// start && wait for generation session to complete
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || (session.state() == GenerationSessionState::Finished
			|| session.state() == GenerationSessionState::Failed)
			&& clusters[0].client().generation_session(&SessionId::default()).is_none());
		assert!(session.joint_public_and_secret().unwrap().is_ok());

		// now remove share from node2
		assert!((0..3).all(|i| clusters[i].data.sessions.generation_sessions.is_empty()));
		clusters[2].data.config.key_storage.remove(&Default::default()).unwrap();

		// and try to sign message with generated key
		let signature = sign(Random.generate().unwrap().secret(), &Default::default()).unwrap();
		let session0 = clusters[0].client().new_signing_session(Default::default(), signature, None, Default::default()).unwrap();
		let session = clusters[0].data.sessions.signing_sessions.first().unwrap();

		loop_until(&mut core, time::Duration::from_millis(300), || session.is_finished() && (0..3).all(|i|
			clusters[i].data.sessions.signing_sessions.is_empty()));
		session0.wait().unwrap();

		// and try to sign message with generated key using node that has no key share
		let signature = sign(Random.generate().unwrap().secret(), &Default::default()).unwrap();
		let session2 = clusters[2].client().new_signing_session(Default::default(), signature, None, Default::default()).unwrap();
		let session = clusters[2].data.sessions.signing_sessions.first().unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.is_finished()  && (0..3).all(|i|
			clusters[i].data.sessions.signing_sessions.is_empty()));
		session2.wait().unwrap();

		// now remove share from node1
		clusters[1].data.config.key_storage.remove(&Default::default()).unwrap();

		// and try to sign message with generated key
		let signature = sign(Random.generate().unwrap().secret(), &Default::default()).unwrap();
		let session1 = clusters[0].client().new_signing_session(Default::default(), signature, None, Default::default()).unwrap();
		let session = clusters[0].data.sessions.signing_sessions.first().unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.is_finished());
		session1.wait().unwrap_err();
	}
}
