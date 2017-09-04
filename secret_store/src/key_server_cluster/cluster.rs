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
use futures::{finished, failed, Future, Stream, BoxFuture};
use futures_cpupool::CpuPool;
use parking_lot::{RwLock, Mutex};
use tokio_io::IoFuture;
use tokio_core::reactor::{Handle, Remote, Interval};
use tokio_core::net::{TcpListener, TcpStream};
use ethkey::{Public, KeyPair, Signature, Random, Generator};
use bigint::hash::H256;
use key_server_cluster::{Error, NodeId, SessionId, AclStorage, KeyStorage, KeyServerSet, NodeKeyPair};
use key_server_cluster::cluster_sessions::{ClusterSession, ClusterSessions, GenerationSessionWrapper, EncryptionSessionWrapper,
	DecryptionSessionWrapper, SigningSessionWrapper};
use key_server_cluster::message::{self, Message, ClusterMessage, GenerationMessage, EncryptionMessage, DecryptionMessage,
	SigningMessage, ConsensusMessage};
use key_server_cluster::generation_session::{Session as GenerationSession, SessionState as GenerationSessionState};
#[cfg(test)]
use key_server_cluster::generation_session::SessionImpl as GenerationSessionImpl;
use key_server_cluster::decryption_session::{Session as DecryptionSession, DecryptionSessionId};
use key_server_cluster::encryption_session::{Session as EncryptionSession, SessionState as EncryptionSessionState};
use key_server_cluster::signing_session::{Session as SigningSession, SigningSessionId};
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

/// Encryption sesion timeout interval. It works
/// Empty future.
type BoxedEmptyFuture = BoxFuture<(), ()>;

/// Cluster interface for external clients.
pub trait ClusterClient: Send + Sync {
	/// Get cluster state.
	fn cluster_state(&self) -> ClusterState;
	/// Start new generation session.
	fn new_generation_session(&self, session_id: SessionId, author: Public, threshold: usize) -> Result<Arc<GenerationSession>, Error>;
	/// Start new encryption session.
	fn new_encryption_session(&self, session_id: SessionId, requestor_signature: Signature, common_point: Public, encrypted_point: Public) -> Result<Arc<EncryptionSession>, Error>;
	/// Start new decryption session.
	fn new_decryption_session(&self, session_id: SessionId, requestor_signature: Signature, is_shadow_decryption: bool) -> Result<Arc<DecryptionSession>, Error>;
	/// Start new signing session.
	fn new_signing_session(&self, session_id: SessionId, requestor_signature: Signature, message_hash: H256) -> Result<Arc<SigningSession>, Error>;

	#[cfg(test)]
	/// Ask node to make 'faulty' generation sessions.
	fn make_faulty_generation_sessions(&self);
	#[cfg(test)]
	/// Get active generation session with given id.
	fn generation_session(&self, session_id: &SessionId) -> Option<Arc<GenerationSessionImpl>>;
	#[cfg(test)]
	/// Try connect to disconnected nodes.
	fn connect(&self);
}

/// Cluster access for single encryption/decryption/signing participant.
pub trait Cluster: Send + Sync {
	/// Broadcast message to all other nodes.
	fn broadcast(&self, message: Message) -> Result<(), Error>;
	/// Send message to given node.
	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error>;
}

#[derive(Clone)]
/// Cluster initialization parameters.
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
	config: ClusterConfiguration,
	/// Handle to the event loop.
	handle: Remote,
	/// Handle to the cpu thread pool.
	pool: CpuPool,
	/// KeyPair this node holds.
	self_key_pair: Arc<NodeKeyPair>,
	/// Connections data.
	connections: ClusterConnections,
	/// Active sessions data.
	sessions: ClusterSessions,
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

	#[cfg(test)]
	/// Get cluster configuration.
	pub fn config(&self) -> &ClusterConfiguration {
		&self.data.config
	}

	#[cfg(test)]
	/// Get connection to given node.
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
		net_connect(&node_address, handle, data.self_key_pair.clone(), disconnected_nodes)
			.then(move |result| ClusterCore::process_connection_result(data, Some(node_address), result))
			.then(|_| finished(()))
			.boxed()
	}

	/// Start listening for incoming connections.
	fn listen(handle: &Handle, data: Arc<ClusterData>, listen_address: SocketAddr) -> Result<BoxedEmptyFuture, Error> {
		Ok(TcpListener::bind(&listen_address, &handle)?
			.incoming()
			.and_then(move |(stream, node_address)| {
				ClusterCore::accept_connection(data.clone(), stream, node_address);
				Ok(())
			})
			.for_each(|_| Ok(()))
			.then(|_| finished(()))
			.boxed())
	}

	/// Accept connection.
	fn accept_connection(data: Arc<ClusterData>, stream: TcpStream, node_address: SocketAddr) {
		data.handle.clone().spawn(move |handle| {
			data.pool.clone().spawn(ClusterCore::accept_connection_future(handle, data, stream, node_address))
		})
	}

	/// Accept connection future.
	fn accept_connection_future(handle: &Handle, data: Arc<ClusterData>, stream: TcpStream, node_address: SocketAddr) -> BoxedEmptyFuture {
		net_accept_connection(node_address, stream, handle, data.self_key_pair.clone())
			.then(move |result| ClusterCore::process_connection_result(data, None, result))
			.then(|_| finished(()))
			.boxed()
	}

	/// Schedule mainatain procedures.
	fn schedule_maintain(handle: &Handle, data: Arc<ClusterData>) {
		let d = data.clone();
		let interval: BoxedEmptyFuture = Interval::new(time::Duration::new(MAINTAIN_INTERVAL, 0), handle)
			.expect("failed to create interval")
			.and_then(move |_| Ok(ClusterCore::maintain(data.clone())))
			.for_each(|_| Ok(()))
			.then(|_| finished(()))
			.boxed();

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
		connection
			.read_message()
			.then(move |result|
				match result {
					Ok((_, Ok(message))) => {
						ClusterCore::process_connection_message(data.clone(), connection.clone(), message);
						// continue serving connection
						data.spawn(ClusterCore::process_connection_messages(data.clone(), connection));
						finished(Ok(())).boxed()
					},
					Ok((_, Err(err))) => {
						warn!(target: "secretstore_net", "{}: protocol error {} when reading message from node {}", data.self_key_pair.public(), err, connection.node_id());
						// continue serving connection
						data.spawn(ClusterCore::process_connection_messages(data.clone(), connection));
						finished(Err(err)).boxed()
					},
					Err(err) => {
						warn!(target: "secretstore_net", "{}: network error {} when reading message from node {}", data.self_key_pair.public(), err, connection.node_id());
						// close connection
						data.connections.remove(connection.node_id(), connection.is_inbound());
						failed(err).boxed()
					},
				}
			).boxed()
	}

	/// Send keepalive messages to every othe node.
	fn keep_alive(data: Arc<ClusterData>) {
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
		data.connections.update_nodes_set();
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
					finished(Ok(())).boxed()
				}
			},
			Ok(DeadlineStatus::Meet(Err(err))) => {
				warn!(target: "secretstore_net", "{}: protocol error {} when establishing {} connection{}",
					data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				finished(Ok(())).boxed()
			},
			Ok(DeadlineStatus::Timeout) => {
				warn!(target: "secretstore_net", "{}: timeout when establishing {} connection{}",
					data.self_key_pair.public(), if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				finished(Ok(())).boxed()
			},
			Err(err) => {
				warn!(target: "secretstore_net", "{}: network error {} when establishing {} connection{}",
					data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
					outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
				finished(Ok(())).boxed()
			},
		}
	}

	/// Process single message from the connection.
	fn process_connection_message(data: Arc<ClusterData>, connection: Arc<Connection>, message: Message) {
		connection.set_last_message_time(time::Instant::now());
		trace!(target: "secretstore_net", "{}: received message {} from {}", data.self_key_pair.public(), message, connection.node_id());
		match message {
			Message::Generation(message) => ClusterCore::process_generation_message(data, connection, message),
			Message::Encryption(message) => ClusterCore::process_encryption_message(data, connection, message),
			Message::Decryption(message) => ClusterCore::process_decryption_message(data, connection, message),
			Message::Signing(message) => ClusterCore::process_signing_message(data, connection, message),
			Message::Cluster(message) => ClusterCore::process_cluster_message(data, connection, message),
		}
	}

	/// Process single generation message from the connection.
	fn process_generation_message(data: Arc<ClusterData>, connection: Arc<Connection>, mut message: GenerationMessage) {
		let session_id = message.session_id().clone();
		let mut sender = connection.node_id().clone();
		let session = match message {
			GenerationMessage::InitializeSession(_) => {
				let mut connected_nodes = data.connections.connected_nodes();
				connected_nodes.insert(data.self_key_pair.public().clone());

				let cluster = Arc::new(ClusterView::new(data.clone(), connected_nodes));
				data.sessions.new_generation_session(sender.clone(), session_id.clone(), cluster)
			},
			_ => {
				data.sessions.generation_sessions.get(&session_id)
					.ok_or(Error::InvalidSessionId)
			},
		};

		let mut is_queued_message = false;
		loop {
			match session.clone().and_then(|session| session.process_message(&sender, &message)) {
				Ok(_) => {
					// if session is completed => stop
					let session = session.clone().expect("session.method() call finished with success; session exists; qed");
					let session_state = session.state();
					if session_state == GenerationSessionState::Finished {
						info!(target: "secretstore_net", "{}: generation session completed", data.self_key_pair.public());
					}
					if session_state == GenerationSessionState::Finished || session_state == GenerationSessionState::Failed {
						data.sessions.generation_sessions.remove(&session_id);
						break;
					}

					// try to dequeue message
					match data.sessions.generation_sessions.dequeue_message(&session_id) {
						Some((msg_sender, msg)) => {
							is_queued_message = true;
							sender = msg_sender;
							message = msg;
						},
						None => break,
					}
				},
				Err(Error::TooEarlyForRequest) => {
					data.sessions.generation_sessions.enqueue_message(&session_id, sender, message, is_queued_message);
					break;
				},
				Err(err) => {
					warn!(target: "secretstore_net", "{}: generation session error {} when processing message {} from node {}", data.self_key_pair.public(), err, message, sender);
					data.sessions.respond_with_generation_error(&session_id, message::SessionError {
						session: session_id.clone().into(),
						error: format!("{:?}", err),
					});
					if err != Error::InvalidSessionId {
						data.sessions.generation_sessions.remove(&session_id);
					}
					break;
				},
			}
		}
	}

	/// Process single encryption message from the connection.
	fn process_encryption_message(data: Arc<ClusterData>, connection: Arc<Connection>, mut message: EncryptionMessage) {
		let session_id = message.session_id().clone();
		let mut sender = connection.node_id().clone();
		let session = match message {
			EncryptionMessage::InitializeEncryptionSession(_) => {
				let mut connected_nodes = data.connections.connected_nodes();
				connected_nodes.insert(data.self_key_pair.public().clone());

				let cluster = Arc::new(ClusterView::new(data.clone(), connected_nodes));
				data.sessions.new_encryption_session(sender.clone(), session_id.clone(), cluster)
			},
			_ => {
				data.sessions.encryption_sessions.get(&session_id)
					.ok_or(Error::InvalidSessionId)
			},
		};

		let mut is_queued_message = false;
		loop {
			match session.clone().and_then(|session| match message {
				EncryptionMessage::InitializeEncryptionSession(ref message) =>
					session.on_initialize_session(sender.clone(), message),
				EncryptionMessage::ConfirmEncryptionInitialization(ref message) =>
					session.on_confirm_initialization(sender.clone(), message),
				EncryptionMessage::EncryptionSessionError(ref message) =>
					session.on_session_error(sender.clone(), message),
			}) {
				Ok(_) => {
					// if session is completed => stop
					let session = session.clone().expect("session.method() call finished with success; session exists; qed");
					let session_state = session.state();
					if session_state == EncryptionSessionState::Finished {
						info!(target: "secretstore_net", "{}: encryption session completed", data.self_key_pair.public());
					}
					if session_state == EncryptionSessionState::Finished || session_state == EncryptionSessionState::Failed {
						data.sessions.encryption_sessions.remove(&session_id);
						break;
					}

					// try to dequeue message
					match data.sessions.encryption_sessions.dequeue_message(&session_id) {
						Some((msg_sender, msg)) => {
							is_queued_message = true;
							sender = msg_sender;
							message = msg;
						},
						None => break,
					}
				},
				Err(Error::TooEarlyForRequest) => {
					data.sessions.encryption_sessions.enqueue_message(&session_id, sender, message, is_queued_message);
					break;
				},
				Err(err) => {
					warn!(target: "secretstore_net", "{}: encryption session error {} when processing message {} from node {}", data.self_key_pair.public(), err, message, sender);
					data.sessions.respond_with_encryption_error(&session_id, message::EncryptionSessionError {
						session: session_id.clone().into(),
						error: format!("{:?}", err),
					});
					if err != Error::InvalidSessionId {
						data.sessions.encryption_sessions.remove(&session_id);
					}
					break;
				},
			}
		}
	}

	/// Process single decryption message from the connection.
	fn process_decryption_message(data: Arc<ClusterData>, connection: Arc<Connection>, mut message: DecryptionMessage) {
		let session_id = message.session_id().clone();
		let sub_session_id = message.sub_session_id().clone();
		let decryption_session_id = DecryptionSessionId::new(session_id.clone(), sub_session_id.clone());
		let mut sender = connection.node_id().clone();
		let session = match message {
			DecryptionMessage::DecryptionConsensusMessage(ref message) if match message.message {
				ConsensusMessage::InitializeConsensusSession(_) => true,
				_ => false,
			} => {
				let mut connected_nodes = data.connections.connected_nodes();
				connected_nodes.insert(data.self_key_pair.public().clone());

				let cluster = Arc::new(ClusterView::new(data.clone(), connected_nodes));
				data.sessions.new_decryption_session(sender.clone(), session_id.clone(), sub_session_id.clone(), cluster, None)
			},
			_ => {
				data.sessions.decryption_sessions.get(&decryption_session_id)
					.ok_or(Error::InvalidSessionId)
			},
		};

		loop {
			match session.clone().and_then(|session| session.process_message(&sender, &message)) {
				Ok(_) => {
					// if session is completed => stop
					let session = session.clone().expect("session.method() call finished with success; session exists; qed");
					if session.is_finished() {
						info!(target: "secretstore_net", "{}: decryption session completed", data.self_key_pair.public());
						data.sessions.decryption_sessions.remove(&decryption_session_id);
						break;
					}

					// try to dequeue message
					match data.sessions.decryption_sessions.dequeue_message(&decryption_session_id) {
						Some((msg_sender, msg)) => {
							sender = msg_sender;
							message = msg;
						},
						None => break,
					}
				},
				Err(err) => {
					warn!(target: "secretstore_net", "{}: decryption session error {} when processing message {} from node {}", data.self_key_pair.public(), err, message, sender);
					data.sessions.respond_with_decryption_error(&session_id, &sub_session_id, &sender, message::DecryptionSessionError {
						session: session_id.clone().into(),
						sub_session: sub_session_id.clone().into(),
						error: format!("{:?}", err),
					});
					if err != Error::InvalidSessionId {
						data.sessions.decryption_sessions.remove(&decryption_session_id);
					}
					break;
				},
			}
		}
	}

	/// Process singlesigning message from the connection.
	fn process_signing_message(data: Arc<ClusterData>, connection: Arc<Connection>, mut message: SigningMessage) {
		let session_id = message.session_id().clone();
		let sub_session_id = message.sub_session_id().clone();
		let signing_session_id = SigningSessionId::new(session_id.clone(), sub_session_id.clone());
		let mut sender = connection.node_id().clone();
		let session = match message {
			SigningMessage::SigningConsensusMessage(ref message) if match message.message {
				ConsensusMessage::InitializeConsensusSession(_) => true,
				_ => false,
			} => {
				let mut connected_nodes = data.connections.connected_nodes();
				connected_nodes.insert(data.self_key_pair.public().clone());

				let cluster = Arc::new(ClusterView::new(data.clone(), connected_nodes));
				data.sessions.new_signing_session(sender.clone(), session_id.clone(), sub_session_id.clone(), cluster, None)
			},
			_ => {
				data.sessions.signing_sessions.get(&signing_session_id)
					.ok_or(Error::InvalidSessionId)
			},
		};

		let mut is_queued_message = false;
		loop {
			match session.clone().and_then(|session| session.process_message(&sender, &message)) {
				Ok(_) => {
					// if session is completed => stop
					let session = session.clone().expect("session.method() call finished with success; session exists; qed");
					if session.is_finished() {
						info!(target: "secretstore_net", "{}: signing session completed", data.self_key_pair.public());
						data.sessions.signing_sessions.remove(&signing_session_id);
						break;
					}

					// try to dequeue message
					match data.sessions.signing_sessions.dequeue_message(&signing_session_id) {
						Some((msg_sender, msg)) => {
							is_queued_message = true;
							sender = msg_sender;
							message = msg;
						},
						None => break,
					}
				},
				Err(Error::TooEarlyForRequest) => {
					data.sessions.signing_sessions.enqueue_message(&signing_session_id, sender, message, is_queued_message);
					break;
				},
				Err(err) => {
					warn!(target: "secretstore_net", "{}: signing session error {} when processing message {} from node {}", data.self_key_pair.public(), err, message, sender);
					data.sessions.respond_with_signing_error(&session_id, &sub_session_id, &sender, message::SigningSessionError {
						session: session_id.clone().into(),
						sub_session: sub_session_id.clone().into(),
						error: format!("{:?}", err),
					});
					if err != Error::InvalidSessionId {
						data.sessions.signing_sessions.remove(&signing_session_id);
					}
					break;
				},
			}
		}
	}

	/// Process single cluster message from the connection.
	fn process_cluster_message(data: Arc<ClusterData>, connection: Arc<Connection>, message: ClusterMessage) {
		match message {
			ClusterMessage::KeepAlive(_) => data.spawn(connection.send_message(Message::Cluster(ClusterMessage::KeepAliveResponse(message::KeepAliveResponse {})))),
			ClusterMessage::KeepAliveResponse(_) => (),
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

	/// Get sessions reference.
	pub fn sessions(&self) -> &ClusterSessions {
		&self.sessions
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

	pub fn is_connected(&self, node: &NodeId) -> bool {
		self.core.lock().nodes.contains(node)
	}

	pub fn nodes(&self) -> BTreeSet<NodeId> {
		self.core.lock().nodes.clone()
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
}

impl ClusterClientImpl {
	pub fn new(data: Arc<ClusterData>) -> Self {
		ClusterClientImpl {
			data: data,
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

		let cluster = Arc::new(ClusterView::new(self.data.clone(), connected_nodes.clone()));
		let session = self.data.sessions.new_generation_session(self.data.self_key_pair.public().clone(), session_id, cluster)?;
		session.initialize(author, threshold, connected_nodes)?;
		Ok(GenerationSessionWrapper::new(Arc::downgrade(&self.data), session_id, session))
	}

	fn new_encryption_session(&self, session_id: SessionId, requestor_signature: Signature, common_point: Public, encrypted_point: Public) -> Result<Arc<EncryptionSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let cluster = Arc::new(ClusterView::new(self.data.clone(), connected_nodes.clone()));
		let session = self.data.sessions.new_encryption_session(self.data.self_key_pair.public().clone(), session_id, cluster)?;
		session.initialize(requestor_signature, common_point, encrypted_point)?;
		Ok(EncryptionSessionWrapper::new(Arc::downgrade(&self.data), session_id, session))
	}

	fn new_decryption_session(&self, session_id: SessionId, requestor_signature: Signature, is_shadow_decryption: bool) -> Result<Arc<DecryptionSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let access_key = Random.generate()?.secret().clone();
		let cluster = Arc::new(ClusterView::new(self.data.clone(), connected_nodes.clone()));
		let session = self.data.sessions.new_decryption_session(self.data.self_key_pair.public().clone(), session_id, access_key.clone(), cluster, Some(requestor_signature))?;
		session.initialize(is_shadow_decryption)?;
		Ok(DecryptionSessionWrapper::new(Arc::downgrade(&self.data), DecryptionSessionId::new(session_id, access_key), session))
	}

	fn new_signing_session(&self, session_id: SessionId, requestor_signature: Signature, message_hash: H256) -> Result<Arc<SigningSession>, Error> {
		let mut connected_nodes = self.data.connections.connected_nodes();
		connected_nodes.insert(self.data.self_key_pair.public().clone());

		let access_key = Random.generate()?.secret().clone();
		let cluster = Arc::new(ClusterView::new(self.data.clone(), connected_nodes.clone()));
		let session = self.data.sessions.new_signing_session(self.data.self_key_pair.public().clone(), session_id, access_key.clone(), cluster, Some(requestor_signature))?;
		session.initialize(message_hash)?;
		Ok(SigningSessionWrapper::new(Arc::downgrade(&self.data), SigningSessionId::new(session_id, access_key), session))
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
	fn generation_session(&self, session_id: &SessionId) -> Option<Arc<GenerationSessionImpl>> {
		self.data.sessions.generation_sessions.get(session_id)
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
	use std::collections::VecDeque;
	use parking_lot::Mutex;
	use tokio_core::reactor::Core;
	use ethkey::{Random, Generator, Public};
	use key_server_cluster::{NodeId, SessionId, Error, DummyAclStorage, DummyKeyStorage, MapKeyServerSet, PlainNodeKeyPair};
	use key_server_cluster::message::Message;
	use key_server_cluster::cluster::{Cluster, ClusterCore, ClusterConfiguration};
	use key_server_cluster::generation_session::{Session as GenerationSession, SessionState as GenerationSessionState};

	#[derive(Debug)]
	pub struct DummyCluster {
		id: NodeId,
		data: Mutex<DummyClusterData>,
	}

	#[derive(Debug, Default)]
	struct DummyClusterData {
		nodes: Vec<NodeId>,
		messages: VecDeque<(NodeId, Message)>,
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
			self.data.lock().nodes.push(node);
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
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6016, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// ask one of nodes to produce faulty generation sessions
		clusters[1].client().make_faulty_generation_sessions();

		// start && wait for generation session to fail
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some());
		assert!(session.joint_public_and_secret().unwrap().is_err());

		// check that faulty session is either removed from all nodes, or nonexistent (already removed)
		assert!(clusters[0].client().generation_session(&SessionId::default()).is_none());
		for i in 1..3 {
			if let Some(session) = clusters[i].client().generation_session(&SessionId::default()) {
				loop_until(&mut core, time::Duration::from_millis(300), || session.joint_public_and_secret().is_some());
				assert!(session.joint_public_and_secret().unwrap().is_err());
				assert!(clusters[i].client().generation_session(&SessionId::default()).is_none());
			}
		}
	}

	#[test]
	fn generation_session_is_removed_when_succeeded() {
		let mut core = Core::new().unwrap();
		let clusters = make_clusters(&core, 6019, 3);
		run_clusters(&clusters);
		loop_until(&mut core, time::Duration::from_millis(300), || clusters.iter().all(all_connections_established));

		// start && wait for generation session to complete
		let session = clusters[0].client().new_generation_session(SessionId::default(), Public::default(), 1).unwrap();
		loop_until(&mut core, time::Duration::from_millis(300), || session.state() == GenerationSessionState::Finished);
		assert!(session.joint_public_and_secret().unwrap().is_ok());

		// check that session is either removed from all nodes, or nonexistent (already removed)
		assert!(clusters[0].client().generation_session(&SessionId::default()).is_none());
		for i in 1..3 {
			if let Some(session) = clusters[i].client().generation_session(&SessionId::default()) {
				loop_until(&mut core, time::Duration::from_millis(300), || session.state() == GenerationSessionState::Finished);
				assert!(session.joint_public_and_secret().unwrap().is_err());
				assert!(clusters[i].client().generation_session(&SessionId::default()).is_none());
			}
		}
	}
}
