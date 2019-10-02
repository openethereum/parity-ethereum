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

use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;
use std::io;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::{future, Future, Stream};
use parking_lot::{Mutex, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::timer::{Interval, timeout::Error as TimeoutError};
use tokio_io::IoFuture;
use ethkey::KeyPair;
use parity_runtime::Executor;
use key_server_cluster::{Error, NodeId, ClusterConfiguration, NodeKeyPair};
use key_server_cluster::cluster_connections::{ConnectionProvider, Connection, ConnectionManager};
use key_server_cluster::connection_trigger::{Maintain, ConnectionTrigger};
use key_server_cluster::cluster_message_processor::MessageProcessor;
use key_server_cluster::io::{DeadlineStatus, ReadMessage, SharedTcpStream,
	read_encrypted_message, WriteMessage, write_encrypted_message};
use key_server_cluster::message::{self, ClusterMessage, Message};
use key_server_cluster::net::{accept_connection as io_accept_connection,
	connect as io_connect, Connection as IoConnection};

/// Empty future.
pub type BoxedEmptyFuture = Box<dyn Future<Item = (), Error = ()> + Send>;

/// Maintain interval (seconds). Every MAINTAIN_INTERVAL seconds node:
/// 1) checks if connected nodes are responding to KeepAlive messages
/// 2) tries to connect to disconnected nodes
/// 3) checks if enc/dec sessions are time-outed
const MAINTAIN_INTERVAL: u64 = 10;

/// When no messages have been received from node within KEEP_ALIVE_SEND_INTERVAL seconds,
/// we must send KeepAlive message to the node to check if it still responds to messages.
const KEEP_ALIVE_SEND_INTERVAL: Duration = Duration::from_secs(30);
/// When no messages have been received from node within KEEP_ALIVE_DISCONNECT_INTERVAL seconds,
/// we must treat this node as non-responding && disconnect from it.
const KEEP_ALIVE_DISCONNECT_INTERVAL: Duration = Duration::from_secs(60);

/// Network connection manager configuration.
pub struct NetConnectionsManagerConfig {
	/// Allow connecting to 'higher' nodes.
	pub allow_connecting_to_higher_nodes: bool,
	/// Interface to listen to.
	pub listen_address: (String, u16),
	/// True if we should autostart key servers set change session when servers set changes?
	/// This will only work when servers set is configured using KeyServerSet contract.
	pub auto_migrate_enabled: bool,
}

/// Network connections manager.
pub struct NetConnectionsManager {
	/// Address we're listening for incoming connections.
	listen_address: SocketAddr,
	/// Shared cluster connections data reference.
	data: Arc<NetConnectionsData>,
}

/// Network connections data. Shared among NetConnectionsManager and spawned futures.
struct NetConnectionsData {
	/// Allow connecting to 'higher' nodes.
	allow_connecting_to_higher_nodes: bool,
	/// Reference to tokio task executor.
	executor: Executor,
	/// Key pair of this node.
	self_key_pair: Arc<dyn NodeKeyPair>,
	/// Network messages processor.
	message_processor: Arc<dyn MessageProcessor>,
	/// Connections trigger.
	trigger: Mutex<Box<dyn ConnectionTrigger>>,
	/// Mutable connection data.
	container: Arc<RwLock<NetConnectionsContainer>>,
}

/// Network connections container. This is the only mutable data of NetConnectionsManager.
/// The set of nodes is mutated by the connection trigger and the connections set is also
/// mutated by spawned futures.
pub struct NetConnectionsContainer {
	/// Is this node isolated from cluster?
	pub is_isolated: bool,
	/// Current key servers set.
	pub nodes: BTreeMap<NodeId, SocketAddr>,
	/// Active connections to key servers.
	pub connections: BTreeMap<NodeId, Arc<NetConnection>>,
}

/// Network connection to single key server node.
pub struct NetConnection {
	executor: Executor,
	/// Id of the peer node.
	node_id: NodeId,
	/// Address of the peer node.
	node_address: SocketAddr,
	/// Is this inbound (true) or outbound (false) connection?
	is_inbound: bool,
	/// Key pair that is used to encrypt connection' messages.
	key: KeyPair,
	/// Last message time.
	last_message_time: RwLock<Instant>,
	/// Underlying TCP stream.
	stream: SharedTcpStream,
}

impl NetConnectionsManager {
	/// Create new network connections manager.
	pub fn new(
		executor: Executor,
		message_processor: Arc<dyn MessageProcessor>,
		trigger: Box<dyn ConnectionTrigger>,
		container: Arc<RwLock<NetConnectionsContainer>>,
		config: &ClusterConfiguration,
		net_config: NetConnectionsManagerConfig,
	) -> Result<Self, Error> {
		let listen_address = make_socket_address(
			&net_config.listen_address.0,
			net_config.listen_address.1)?;

		Ok(NetConnectionsManager {
			listen_address,
			data: Arc::new(NetConnectionsData {
				allow_connecting_to_higher_nodes: net_config.allow_connecting_to_higher_nodes,
				executor,
				message_processor,
				self_key_pair: config.self_key_pair.clone(),
				trigger: Mutex::new(trigger),
				container,
			}),
		})
	}

	/// Start listening for connections and schedule connections maintenance.
	pub fn start(&self) -> Result<(), Error> {
		net_listen(&self.listen_address, self.data.clone())?;
		net_schedule_maintain(self.data.clone());
		Ok(())
	}
}

impl ConnectionManager for NetConnectionsManager {
	fn provider(&self) -> Arc<dyn ConnectionProvider> {
		self.data.container.clone()
	}

	fn connect(&self) {
		net_connect_disconnected(self.data.clone());
	}
}

impl ConnectionProvider for RwLock<NetConnectionsContainer> {
	fn connected_nodes(&self) -> Result<BTreeSet<NodeId>, Error> {
		let connections = self.read();
		if connections.is_isolated {
			return Err(Error::NodeDisconnected);
		}

		Ok(connections.connections.keys().cloned().collect())
	}

	fn disconnected_nodes(&self) -> BTreeSet<NodeId> {
		let connections = self.read();
		connections.nodes.keys()
			.filter(|node_id| !connections.connections.contains_key(node_id))
			.cloned()
			.collect()
	}

	fn connection(&self, node: &NodeId) -> Option<Arc<dyn Connection>> {
		match self.read().connections.get(node).cloned() {
			Some(connection) => Some(connection),
			None => None,
		}
	}
}

impl NetConnection {
	/// Create new connection.
	pub fn new(executor: Executor, is_inbound: bool, connection: IoConnection) -> NetConnection {
		NetConnection {
			executor,
			node_id: connection.node_id,
			node_address: connection.address,
			is_inbound: is_inbound,
			stream: connection.stream,
			key: connection.key,
			last_message_time: RwLock::new(Instant::now()),
		}
	}

	/// Get last message time.
	pub fn last_message_time(&self) -> Instant {
		*self.last_message_time.read()
	}

	/// Update last message time
	pub fn set_last_message_time(&self, last_message_time: Instant) {
		*self.last_message_time.write() = last_message_time
	}

	/// Returns future that sends encrypted message over this connection.
	pub fn send_message_future(&self, message: Message) -> WriteMessage<SharedTcpStream> {
		write_encrypted_message(self.stream.clone(), &self.key, message)
	}

	/// Returns future that reads encrypted message from this connection.
	pub fn read_message_future(&self) -> ReadMessage<SharedTcpStream> {
		read_encrypted_message(self.stream.clone(), self.key.clone())
	}
}

impl Connection for NetConnection {
	fn is_inbound(&self) -> bool {
		self.is_inbound
	}

	fn node_id(&self) -> &NodeId {
		&self.node_id
	}

	fn node_address(&self) -> String {
		format!("{}", self.node_address)
	}

	fn send_message(&self, message: Message) {
		execute(&self.executor, self.send_message_future(message).then(|_| Ok(())));
	}
}

impl NetConnectionsData {
	/// Executes closure for each active connection.
	pub fn active_connections(&self) -> Vec<Arc<NetConnection>> {
		self.container.read().connections.values().cloned().collect()
	}

	/// Executes closure for each disconnected node.
	pub fn disconnected_nodes(&self) -> Vec<(NodeId, SocketAddr)> {
		let container = self.container.read();
		container.nodes.iter()
			.filter(|(node_id, _)| !container.connections.contains_key(node_id))
			.map(|(node_id, addr)| (*node_id, *addr))
			.collect()
	}

	/// Try to insert new connection. Returns true if connection has been inserted.
	/// Returns false (and ignores connections) if:
	/// - we do not expect connection from this node
	/// - we are already connected to the node and existing connection 'supersede'
	///   new connection by agreement
	pub fn insert(&self, connection: Arc<NetConnection>) -> bool {
		let node = *connection.node_id();
		let mut container = self.container.write();
		if !container.nodes.contains_key(&node) {
			trace!(target: "secretstore_net", "{}: ignoring unknown connection from {} at {}",
				self.self_key_pair.public(), node, connection.node_address());
			return false;
		}

		if container.connections.contains_key(&node) {
			// we have already connected to the same node
			// the agreement is that node with lower id must establish connection to node with higher id
			if (*self.self_key_pair.public() < node && connection.is_inbound())
				|| (*self.self_key_pair.public() > node && !connection.is_inbound()) {
				return false;
			}
		}

		trace!(target: "secretstore_net",
			"{}: inserting connection to {} at {}. Connected to {} of {} nodes",
			self.self_key_pair.public(), node, connection.node_address(),
			container.connections.len() + 1, container.nodes.len());
		container.connections.insert(node, connection);

		true
	}

	/// Tries to remove connection. Returns true if connection has been removed.
	/// Returns false if we do not know this connection.
	pub fn remove(&self, connection: &NetConnection) -> bool {
		let node_id = *connection.node_id();
		let is_inbound = connection.is_inbound();
		let mut container = self.container.write();
		if let Entry::Occupied(entry) = container.connections.entry(node_id) {
			if entry.get().is_inbound() != is_inbound {
				return false;
			}

			trace!(target: "secretstore_net", "{}: removing connection to {} at {}",
				self.self_key_pair.public(), node_id, entry.get().node_address());
			entry.remove_entry();

			true
		} else {
			false
		}
	}
}

/// Listen incoming connections.
fn net_listen(
	listen_address: &SocketAddr,
	data: Arc<NetConnectionsData>,
) -> Result<(), Error> {
	execute(&data.executor, net_listen_future(listen_address, data.clone())?);
	Ok(())
}

/// Listen incoming connections future.
fn net_listen_future(
	listen_address: &SocketAddr,
	data: Arc<NetConnectionsData>,
) -> Result<BoxedEmptyFuture, Error> {
	Ok(Box::new(TcpListener::bind(listen_address)?
		.incoming()
		.and_then(move |stream| {
			net_accept_connection(data.clone(), stream);
			Ok(())
		})
		.for_each(|_| Ok(()))
		.then(|_| future::ok(()))))
}

/// Accept incoming connection.
fn net_accept_connection(
	data: Arc<NetConnectionsData>,
	stream: TcpStream,
) {
	execute(&data.executor, net_accept_connection_future(data.clone(), stream));
}

/// Accept incoming connection future.
fn net_accept_connection_future(data: Arc<NetConnectionsData>, stream: TcpStream) -> BoxedEmptyFuture {
	Box::new(io_accept_connection(stream, data.self_key_pair.clone())
		.then(move |result| net_process_connection_result(data, None, result))
		.then(|_| future::ok(())))
}

/// Connect to remote node.
fn net_connect(
	data: Arc<NetConnectionsData>,
	remote: SocketAddr,
) {
	execute(&data.executor, net_connect_future(data.clone(), remote));
}

/// Connect to remote node future.
fn net_connect_future(
	data: Arc<NetConnectionsData>,
	remote: SocketAddr,
) -> BoxedEmptyFuture {
	let disconnected_nodes = data.container.disconnected_nodes();
	Box::new(io_connect(&remote, data.self_key_pair.clone(), disconnected_nodes)
		.then(move |result| net_process_connection_result(data, Some(remote), result))
		.then(|_| future::ok(())))
}

/// Process network connection result.
fn net_process_connection_result(
	data: Arc<NetConnectionsData>,
	outbound_addr: Option<SocketAddr>,
	result: Result<DeadlineStatus<Result<IoConnection, Error>>, TimeoutError<io::Error>>,
) -> IoFuture<Result<(), Error>> {
	match result {
		Ok(DeadlineStatus::Meet(Ok(connection))) => {
			let connection = Arc::new(NetConnection::new(data.executor.clone(), outbound_addr.is_none(), connection));
			if data.insert(connection.clone()) {
				let maintain_action = data.trigger.lock().on_connection_established(connection.node_id());
				maintain_connection_trigger(data.clone(), maintain_action);

				return net_process_connection_messages(data, connection);
			}
		},
		Ok(DeadlineStatus::Meet(Err(err))) => {
			warn!(target: "secretstore_net", "{}: protocol error '{}' when establishing {} connection{}",
				data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
				outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
		},
		Ok(DeadlineStatus::Timeout) => {
			warn!(target: "secretstore_net", "{}: timeout when establishing {} connection{}",
				data.self_key_pair.public(), if outbound_addr.is_some() { "outbound" } else { "inbound" },
				outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
		},
		Err(err) => {
			warn!(target: "secretstore_net", "{}: network error '{}' when establishing {} connection{}",
				data.self_key_pair.public(), err, if outbound_addr.is_some() { "outbound" } else { "inbound" },
				outbound_addr.map(|a| format!(" with {}", a)).unwrap_or_default());
		},
	}

	Box::new(future::ok(Ok(())))
}

/// Process connection messages.
fn net_process_connection_messages(
	data: Arc<NetConnectionsData>,
	connection: Arc<NetConnection>,
) -> IoFuture<Result<(), Error>> {
	Box::new(connection
		.read_message_future()
		.then(move |result|
			match result {
				Ok((_, Ok(message))) => {
					connection.set_last_message_time(Instant::now());
					data.message_processor.process_connection_message(connection.clone(), message);
					// continue serving connection
					let process_messages_future = net_process_connection_messages(
						data.clone(), connection).then(|_| Ok(()));
					execute(&data.executor, process_messages_future);
					Box::new(future::ok(Ok(())))
				},
				Ok((_, Err(err))) => {
					warn!(target: "secretstore_net", "{}: protocol error '{}' when reading message from node {}",
						data.self_key_pair.public(), err, connection.node_id());
					// continue serving connection
					let process_messages_future = net_process_connection_messages(
							data.clone(), connection).then(|_| Ok(()));
					execute(&data.executor, process_messages_future);
					Box::new(future::ok(Err(err)))
				},
				Err(err) => {
					let node_id = *connection.node_id();
					warn!(target: "secretstore_net", "{}: network error '{}' when reading message from node {}",
						data.self_key_pair.public(), err, node_id);
					// close connection
					if data.remove(&*connection) {
						let maintain_action = data.trigger.lock().on_connection_closed(&node_id);
						maintain_connection_trigger(data, maintain_action);
					}
					Box::new(future::err(err))
				},
			}
		))
}

/// Schedule connections. maintain.
fn net_schedule_maintain(data: Arc<NetConnectionsData>) {
	let closure_data = data.clone();
	execute(&data.executor, Interval::new_interval(Duration::new(MAINTAIN_INTERVAL, 0))
		.and_then(move |_| Ok(net_maintain(closure_data.clone())))
		.for_each(|_| Ok(()))
		.then(|_| future::ok(())));
}

/// Maintain network connections.
fn net_maintain(data: Arc<NetConnectionsData>) {
	trace!(target: "secretstore_net", "{}: executing maintain procedures", data.self_key_pair.public());

	update_nodes_set(data.clone());
	data.message_processor.maintain_sessions();
	net_keep_alive(data.clone());
	net_connect_disconnected(data);
}

/// Send keep alive messages to remote nodes.
fn net_keep_alive(data: Arc<NetConnectionsData>) {
	let now = Instant::now();
	let active_connections = data.active_connections();
	for connection in active_connections {
		let last_message_diff = now - connection.last_message_time();
		if last_message_diff > KEEP_ALIVE_DISCONNECT_INTERVAL {
			warn!(target: "secretstore_net", "{}: keep alive timeout for node {}",
				data.self_key_pair.public(), connection.node_id());

			let node_id = *connection.node_id();
			if data.remove(&*connection) {
				let maintain_action = data.trigger.lock().on_connection_closed(&node_id);
				maintain_connection_trigger(data.clone(), maintain_action);
			}
			data.message_processor.process_disconnect(&node_id);
		}
		else if last_message_diff > KEEP_ALIVE_SEND_INTERVAL {
			connection.send_message(Message::Cluster(ClusterMessage::KeepAlive(message::KeepAlive {})));
		}
	}
}

/// Connect disconnected nodes.
fn net_connect_disconnected(data: Arc<NetConnectionsData>) {
	let disconnected_nodes = data.disconnected_nodes();
	for (node_id, address) in disconnected_nodes {
		if data.allow_connecting_to_higher_nodes || *data.self_key_pair.public() < node_id {
			net_connect(data.clone(), address);
		}
	}
}

/// Schedule future execution.
fn execute<F: Future<Item = (), Error = ()> + Send + 'static>(executor: &Executor, f: F) {
	if let Err(err) = future::Executor::execute(executor, Box::new(f)) {
		error!("Secret store runtime unable to spawn task. Runtime is shutting down. ({:?})", err);
	}
}

/// Try to update active nodes set from connection trigger.
fn update_nodes_set(data: Arc<NetConnectionsData>) {
	let maintain_action = data.trigger.lock().on_maintain();
	maintain_connection_trigger(data, maintain_action);
}

/// Execute maintain procedures of connections trigger.
fn maintain_connection_trigger(data: Arc<NetConnectionsData>, maintain_action: Option<Maintain>) {
	if maintain_action == Some(Maintain::SessionAndConnections) || maintain_action == Some(Maintain::Session) {
		let session_params = data.trigger.lock().maintain_session();
		if let Some(session_params) = session_params {
			let session = data.message_processor.start_servers_set_change_session(session_params);
			match session {
				Ok(_) => trace!(target: "secretstore_net", "{}: started auto-migrate session",
					data.self_key_pair.public()),
				Err(err) => trace!(target: "secretstore_net", "{}: failed to start auto-migrate session with: {}",
					data.self_key_pair.public(), err),
			}
		}
	}
	if maintain_action == Some(Maintain::SessionAndConnections) || maintain_action == Some(Maintain::Connections) {
		let mut trigger = data.trigger.lock();
		let mut data = data.container.write();
		trigger.maintain_connections(&mut *data);
	}
}

/// Compose SocketAddr from configuration' address and port.
fn make_socket_address(address: &str, port: u16) -> Result<SocketAddr, Error> {
	let ip_address: IpAddr = address.parse().map_err(|_| Error::InvalidNodeAddress)?;
	Ok(SocketAddr::new(ip_address, port))
}
