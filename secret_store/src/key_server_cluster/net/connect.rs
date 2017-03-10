use std::collections::BTreeSet;
use std::io;
use std::time::Duration;
use std::net::SocketAddr;
use futures::{Future, Poll, Async};
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use ethkey::KeyPair;
use key_server_cluster::{Error, NodeId};
use key_server_cluster::io::{handshake, Handshake, Deadline, deadline};
use key_server_cluster::net::Connection;

pub fn connect(address: &SocketAddr, handle: &Handle, self_key_pair: KeyPair, trusted_nodes: BTreeSet<NodeId>) -> Deadline<Connect> {
	let connect = Connect {
		state: ConnectState::TcpConnect(TcpStream::connect(address, handle)),
		address: address.clone(),
		self_key_pair: self_key_pair,
		trusted_nodes: trusted_nodes,
	};

	deadline(Duration::new(5, 0), handle, connect).expect("Failed to create timeout")
}

enum ConnectState {
	TcpConnect(TcpStreamNew),
	Handshake(Handshake<TcpStream>),
	Connected,
}

pub struct Connect {
	state: ConnectState,
	address: SocketAddr,
	self_key_pair: KeyPair,
	trusted_nodes: BTreeSet<NodeId>,
}

impl Future for Connect {
	type Item = Result<Connection, Error>;
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (next, result) = match self.state {
			ConnectState::TcpConnect(ref mut future) => {
				let stream = try_ready!(future.poll());
				let handshake = handshake(stream, self.self_key_pair.clone(), self.trusted_nodes.clone());
				(ConnectState::Handshake(handshake), Async::NotReady)
			},
			ConnectState::Handshake(ref mut future) => {
				let (stream, result) = try_ready!(future.poll());
				let result = match result {
					Ok(result) => result,
					Err(err) => return Ok(Async::Ready(Err(err))),
				};
				let connection = Connection {
					stream: stream.into(),
					address: self.address,
					node_id: result.node_id,
				};
				(ConnectState::Connected, Async::Ready(Ok(connection)))
			},
			ConnectState::Connected => panic!("poll Connect after it's done"),
		};

		self.state = next;
		match result {
			// by polling again, we register new future
			Async::NotReady => self.poll(),
			result => Ok(result)
		}
	}
}
