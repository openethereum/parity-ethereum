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
use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::net::{SocketAddr, IpAddr};
use futures::{finished, failed, Future, Stream, BoxFuture};
use futures_cpupool::CpuPool;
use parking_lot::RwLock;
use tokio_core::io::IoFuture;
use tokio_core::reactor::{Handle, Remote, Timeout, Interval};
use tokio_core::net::{TcpListener, TcpStream};
use ethkey::KeyPair;
use key_server_cluster::{Error, NodeId, SessionId};
use key_server_cluster::message::Message;
use key_server_cluster::decryption_session::{Session as DecryptionSession, DecryptionSessionId};
use key_server_cluster::encryption_session::Session as EncryptionSession;
use key_server_cluster::io::{DeadlineStatus, ReadMessage, SharedTcpStream, read_message};
use key_server_cluster::net::{accept_connection, connect, Connection as NetConnection};

pub type BoxedEmptyFuture = BoxFuture<(), ()>;

/// Cluster access for single encryption/decryption participant.
pub trait Cluster: Send + Sync {
	/// Broadcast message to all other nodes.
	fn broadcast(&self, message: Message) -> Result<(), Error>;
	/// Send message to given node.
	fn send(&self, to: &NodeId, message: Message) -> Result<(), Error>;
	/// Blacklist node, close connection and remove all pending messages.
	fn blacklist(&self, node: &NodeId);
}

/// Cluster initialization parameters.
pub struct ClusterParams {
	/// Interface to listen to
	pub listener_addr: String,
	/// Port to listen to
	pub listener_port: u16,
	/// Number of threads reserved by cluster.
	pub threads: usize,
	/// Cluster nodes.
	pub cluster_nodes: BTreeMap<NodeId, (String, u16)>,
}

/// Network cluster implementation.
pub struct ClusterImpl {
	/// Event loop handle.
	handle: Handle,
	/// Thread pool.
	pool: CpuPool,
	/// Cluster core.
	core: Arc<ClusterCore>,
	params: ClusterParams,
	/// Cluster configuration.
	config: ClusterConfig,
}

#[derive(Clone)]
pub struct ClusterConfig {
	/// This node id.
	pub self_node_key_pair: KeyPair,
	/// Nodes allowed to connect.
	pub cluster_nodes: BTreeSet<NodeId>,
}

struct ClusterCore {
	/// Event loop handle.
	handle: Remote,
	/// Thread pool.
	pool: CpuPool,
	/// Active connections.
	connections: RwLock<BTreeMap<NodeId, Arc<Connection>>>,
	/// Cluster configuration.
	config: ClusterConfig,
}

struct Connection {
	node_id: NodeId,
	stream: SharedTcpStream,
}

impl ClusterImpl {
	pub fn new(self_node_key_pair: KeyPair, params: ClusterParams, handle: Handle) -> Result<Arc<Self>, Error> {
		let pool = CpuPool::new(params.threads);
		let config = ClusterConfig {
			self_node_key_pair: self_node_key_pair,
			cluster_nodes: params.cluster_nodes.keys().cloned().collect(),
		};
		let core = ClusterCore::new(&params, &config, pool.clone(), handle.remote().clone());

		Ok(Arc::new(ClusterImpl {
			handle: handle,
			pool: pool,
			core: Arc::new(core),
			params: params,
			config: config,
		}))
	}

	/// Run cluster
	pub fn run(&self) -> Result<(), Error> {
		// try to connect to every other peer
		for (_, &(ref peer_addr, peer_port)) in self.params.cluster_nodes.iter().filter(|&(n, _)| n != self.config.self_node_key_pair.public()) {
			let peer_addr = SocketAddr::new(IpAddr::from_str(&peer_addr).unwrap(), peer_port);
			self.connect(peer_addr);
		}

		ClusterCore::autoconnect(self.core.clone(), &self.handle);
		self.listen()?;

		Ok(())
	}

	/// Start listening for incoming connections.
	fn listen(&self) -> Result<(), Error> {
		let server = ClusterCore::listen(self.core.clone(), &self.handle, &self.params.listener_addr, self.params.listener_port, self.config.clone())?;
		self.handle.spawn(server);
		Ok(())
	}

	/// Attempts to connect to the specified node
	pub fn connect(&self, peer_addr: SocketAddr) {
		ClusterCore::connect(self.core.clone(), peer_addr);
	}
}

impl ClusterCore {
	pub fn new(params: &ClusterParams, config: &ClusterConfig, pool: CpuPool, handle: Remote) -> Self {
		ClusterCore {
			handle: handle,
			pool: pool,
			connections: RwLock::new(BTreeMap::new()),
			config: config.clone(),
		}
	}

	/// Start listening for incoming connections.
	pub fn listen(core: Arc<ClusterCore>, handle: &Handle, listen_addr: &str, listen_port: u16, config: ClusterConfig) -> Result<BoxedEmptyFuture, Error> {
		let listen_addr = SocketAddr::new(IpAddr::from_str(&listen_addr).unwrap(), listen_port);
		let server = TcpListener::bind(&listen_addr, &handle)?;
		let server = server.incoming()
			.and_then(move |(stream, socket)| {
				ClusterCore::accept_connection(core.clone(), stream, socket, config.clone());
				Ok(())
			})
			.for_each(|_| Ok(()))
			.then(|_| finished(()))
			.boxed();
		Ok(server)
	}

	/// Accept connection.
	pub fn accept_connection(core: Arc<ClusterCore>, stream: TcpStream, socket: SocketAddr, config: ClusterConfig) {
		core.handle.clone().spawn(move |handle| {
			core.pool.clone().spawn(ClusterCore::accept_connection_future(core, stream, socket, handle, config))
		})
	}

	/// Accept connection future.
	pub fn accept_connection_future(core: Arc<ClusterCore>, stream: TcpStream, socket: SocketAddr, handle: &Handle, config: ClusterConfig) -> BoxedEmptyFuture {
		// TODO pass only disconnected cluster_nodes
		accept_connection(socket, stream, handle, config.self_node_key_pair.clone(), config.cluster_nodes.clone()).then(move |result| {
			match result {
				Ok(DeadlineStatus::Meet(Ok(connection))) => {
					let connection = Connection::new(connection);
					core.connections.write().insert(connection.node_id().clone(), connection.clone());
					ClusterCore::on_message(core.clone(), connection)
				},
				Ok(DeadlineStatus::Meet(Err(err))) => {
					finished(Ok(())).boxed()
				},
				Ok(DeadlineStatus::Timeout) => {
					finished(Ok(())).boxed()
				},
				Err(_) => {
					// network error
					finished(Ok(())).boxed()
				},
			}
		})
		.then(|_| finished(()))
		.boxed()
	}

	/// Called on incomming mesage.
	pub fn on_message(core: Arc<ClusterCore>, connection: Arc<Connection>) -> IoFuture<Result<(), Error>> {
		connection.read_message().then(move |result| {
			match result {
				Ok((_, Ok(message))) => {
					match core.process_message(message) {
						Ok(_) => {
							let on_message = ClusterCore::on_message(core.clone(), connection);
							core.spawn(on_message);
							finished(Ok(())).boxed()
						},
						Err(err) => {
							// protocol error
							core.close_connection_with_error(connection.node_id().clone(), &err);
							finished(Err(err)).boxed()
						}
					}
				},
				Ok((_, Err(err))) => {
					// protocol error
					core.close_connection_with_error(connection.node_id().clone(), &err);
					finished(Err(err)).boxed()
				},
				Err(err) => {
					// network error
					core.close_connection_with_error(connection.node_id().clone(), &Error::Io(format!("{}", err)));
					failed(err).boxed()
				}
			}
		}).boxed()
	}

	/// Connect to socket using given context.
	pub fn connect(core: Arc<ClusterCore>, socket: SocketAddr) {
		core.handle.clone().spawn(move |handle| {
			core.pool.clone().spawn(ClusterCore::connect_future(core, socket, handle))
		})
	}

	/// Connect to socket using given context and handle.
	fn connect_future(core: Arc<ClusterCore>, socket: SocketAddr, handle: &Handle) -> BoxedEmptyFuture {
		let connection = connect(&socket, handle, core.config.self_node_key_pair.clone(), core.config.cluster_nodes.clone());
		connection.then(move |result| {
			match result {
				Ok(DeadlineStatus::Meet(Ok(connection))) => {
					let connection = Connection::new(connection);
					core.connections.write().insert(connection.node_id().clone(), connection.clone());
					ClusterCore::on_message(core.clone(), connection)
				},
				Ok(DeadlineStatus::Meet(Err(err))) => {
					finished(Ok(())).boxed()
				},
				Ok(DeadlineStatus::Timeout) => {
					finished(Ok(())).boxed()
				},
				Err(_) => {
					// network error
					finished(Ok(())).boxed()
				},
			}
		})
		.then(|_| finished(()))
		.boxed()
	}

	pub fn autoconnect(core: Arc<ClusterCore>, handle: &Handle) {
		let c = core.clone();
		let interval: BoxedEmptyFuture = Interval::new(time::Duration::new(10, 0), handle).expect("Failed to create interval")
			.and_then(move |_| {
				// TODO: connect
				Ok(())
			})
			.for_each(|_| Ok(()))
			.then(|_| finished(()))
			.boxed();
		c.spawn(interval);
	}

	fn process_message(&self, message: Message) -> Result<(), Error> {
		unimplemented!()
	}

	fn close_connection_with_error(&self, node_id: NodeId, err: &Error) {
		unimplemented!()
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
	pub fn new(connection: NetConnection) -> Arc<Connection> {
		Arc::new(Connection {
			stream: connection.stream,
			node_id: connection.node_id,
		})
	}

	pub fn node_id(&self) -> &NodeId {
		&self.node_id
	}

	pub fn read_message(&self) -> ReadMessage<SharedTcpStream> {
		read_message(self.stream.clone())
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::VecDeque;
	use parking_lot::Mutex;
	use key_server_cluster::{NodeId, Error};
	use key_server_cluster::message::Message;
	use key_server_cluster::cluster::Cluster;

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

		fn blacklist(&self, _node: &NodeId) {
		}
	}
}
