// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Stratum protocol implementation for parity ethereum/bitcoin clients

extern crate json_tcp_server;
extern crate jsonrpc_core;
#[macro_use] extern crate log;

#[cfg(test)]
extern crate mio;

use json_tcp_server::Server as JsonRpcServer;
use jsonrpc_core::{IoHandler, Params, IoDelegate, to_value};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::net::SocketAddr;

pub struct Stratum {
	rpc_server: JsonRpcServer,
	handler: Arc<IoHandler>,
	subscribers: RwLock<Vec<SocketAddr>>,
}

impl Stratum {
	pub fn start(addr: &SocketAddr) -> Result<Arc<Stratum>, json_tcp_server::Error> {
		let handler = Arc::new(IoHandler::new());
		let server = try!(JsonRpcServer::new(addr, &handler));
		let stratum = Arc::new(Stratum {
			rpc_server: server,
			handler: handler,
			subscribers: RwLock::new(Vec::new()),
		});

		let mut delegate = IoDelegate::<Stratum>::new(stratum.clone());
		delegate.add_method("miner.subscribe", Stratum::subscribe);
		stratum.handler.add_delegate(delegate);

		try!(stratum.rpc_server.run_async());

		Ok(stratum)
	}

	fn subscribe(&self, _params: Params) -> std::result::Result<jsonrpc_core::Value, jsonrpc_core::Error> {
		if let Some(context) = self.rpc_server.request_context() {
			self.subscribers.write().unwrap().push(context.socket_addr);
			trace!(target: "stratum", "Subscription request from {:?}", context.socket_addr);
		}
		Ok(try!(to_value(&0)))
	}

	pub fn subscribers(&self) -> RwLockReadGuard<Vec<SocketAddr>> {
		self.subscribers.read().unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	use std::net::SocketAddr;

	pub fn dummy_request(addr: &SocketAddr, buf: &[u8]) -> Vec<u8> {
		use std::io::{Read, Write};
		use mio::*;
		use mio::tcp::*;

		let mut poll = Poll::new().unwrap();
		let mut sock = TcpStream::connect(addr).unwrap();
		poll.register(&sock, Token(0), EventSet::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
		poll.poll(Some(500)).unwrap();
		sock.write_all(buf).unwrap();
		poll.reregister(&sock, Token(0), EventSet::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
		poll.poll(Some(500)).unwrap();

		let mut buf = Vec::new();
		sock.read_to_end(&mut buf).unwrap_or_else(|_| { 0 });
		buf
	}

	#[test]
	fn can_be_started() {
		let stratum = Stratum::start(&SocketAddr::from_str("0.0.0.0:19980").unwrap());
		assert!(stratum.is_ok());
	}

	#[test]
	fn records_subscriber() {
		let addr = SocketAddr::from_str("0.0.0.0:19985").unwrap();
		let stratum = Stratum::start(&addr).unwrap();
		let request = r#"{"jsonrpc": "2.0", "method": "miner.subscribe", "params": [], "id": 1}"#;
		dummy_request(&addr, request.as_bytes());
		assert_eq!(1, stratum.subscribers.read().unwrap().len());
	}
}
