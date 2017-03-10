use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use std::collections::BTreeSet;
use futures::{Future, Poll};
use tokio_core::reactor::Handle;
use tokio_core::net::TcpStream;
use ethkey::KeyPair;
use key_server_cluster::{Error, NodeId};
use key_server_cluster::io::{accept_handshake, Handshake, Deadline, deadline};
use key_server_cluster::net::Connection;
use key_server_cluster::cluster::ClusterConfig;

pub fn accept_connection(address: SocketAddr, stream: TcpStream, handle: &Handle, self_key_pair: KeyPair, trusted_nodes: BTreeSet<NodeId>) -> Deadline<AcceptConnection> {
	let accept = AcceptConnection {
		handshake: accept_handshake(stream, self_key_pair, trusted_nodes),
		address: address,
	};

	deadline(Duration::new(5, 0), handle, accept).expect("Failed to create timeout")
}

pub struct AcceptConnection {
	handshake: Handshake<TcpStream>,
	address: SocketAddr,
}

impl Future for AcceptConnection {
	type Item = Result<Connection, Error>;
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (stream, result) = try_ready!(self.handshake.poll());
		let result = match result {
			Ok(result) => result,
			Err(err) => return Ok(Err(err).into()),
		};
		let connection = Connection {
			stream: stream.into(),
			address: self.address,
			node_id: result.node_id,
		};
		Ok(Ok(connection).into())
	}
}
