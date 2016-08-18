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
extern crate ethcore_util as util;

#[cfg(test)]
extern crate mio;

use json_tcp_server::Server as JsonRpcServer;
use jsonrpc_core::{IoHandler, Params, IoDelegate, to_value, from_params};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::net::SocketAddr;
use std::collections::{HashSet, HashMap};
use util::{H256, Hashable};

pub struct Stratum {
	rpc_server: JsonRpcServer,
	handler: Arc<IoHandler>,
	/// Subscribed clients
	subscribers: RwLock<Vec<SocketAddr>>,
	/// List of workers supposed to receive job update
	job_que: RwLock<HashSet<SocketAddr>>,
	/// Payload manager
	payload_manager: Arc<JobPayloadManager>,
	/// Authorized workers (socket - worker_id)
	workers: Arc<RwLock<HashMap<SocketAddr, String>>>,
	/// Secret if any
	secret: Option<H256>,
}

#[cfg(test)]
pub struct VoidManager;

#[cfg(test)]
impl JobPayloadManager for VoidManager { }

pub trait JobPayloadManager: Send + Sync {
	// json for initial client handshake
	fn initial(&self) -> Option<String> { None }
	// json for difficulty dispatch
	fn difficulty(&self) -> Option<String> { None }
	// json for job update given worker_id (payload manager should split job!)
	fn job(&self, _worker_id: &str) -> Option<String> { None }
}

impl Stratum {
	pub fn start(
		addr: &SocketAddr,
		payload_manager: Arc<JobPayloadManager>,
		secret: Option<H256>,
	) -> Result<Arc<Stratum>, json_tcp_server::Error> {
		let handler = Arc::new(IoHandler::new());
		let server = try!(JsonRpcServer::new(addr, &handler));
		let stratum = Arc::new(Stratum {
			rpc_server: server,
			handler: handler,
			subscribers: RwLock::new(Vec::new()),
			job_que: RwLock::new(HashSet::new()),
			payload_manager: payload_manager,
			workers: Arc::new(RwLock::new(HashMap::new())),
			secret: secret,
		});

		let mut delegate = IoDelegate::<Stratum>::new(stratum.clone());
		delegate.add_method("miner.subscribe", Stratum::subscribe);
		delegate.add_method("miner.authorize", Stratum::authorize);
		stratum.handler.add_delegate(delegate);

		try!(stratum.rpc_server.run_async());

		Ok(stratum)
	}

	fn subscribe(&self, _params: Params) -> std::result::Result<jsonrpc_core::Value, jsonrpc_core::Error> {
		use std::str::FromStr;

		if let Some(context) = self.rpc_server.request_context() {
			self.subscribers.write().unwrap().push(context.socket_addr);
			self.job_que.write().unwrap().insert(context.socket_addr);
			trace!(target: "stratum", "Subscription request from {:?}", context.socket_addr);
		}
		Ok(match self.payload_manager.initial() {
			Some(initial) => match jsonrpc_core::Value::from_str(&initial) {
				Ok(val) => val,
				Err(e) => {
					warn!(target: "tcp", "Invalid payload: '{}' ({:?})", &initial, e);
					try!(to_value(&[0u8; 0]))
				},
			},
			None => try!(to_value(&[0u8; 0])),
		})
	}

	fn authorize(&self, params: Params) -> std::result::Result<jsonrpc_core::Value, jsonrpc_core::Error> {
		from_params::<(String, String)>(params).and_then(|(worker_id, secret)|{
			if let Some(valid_secret) = self.secret {
				let hash = secret.sha3();
				if hash != valid_secret {
					return to_value(&false);
				}
			}
			if let Some(context) = self.rpc_server.request_context() {
				self.workers.write().unwrap().insert(context.socket_addr, worker_id);
				to_value(&true)
			}
			else {
				warn!(target: "tcp", "Authorize without valid context received!");
				to_value(&false)
			}
		})
	}

	pub fn subscribers(&self) -> RwLockReadGuard<Vec<SocketAddr>> {
		self.subscribers.read().unwrap()
	}

	pub fn maintain(&self) {
		let mut job_que = self.job_que.write().unwrap();
		let workers = self.workers.read().unwrap();
		for socket_addr in job_que.drain() {
			if let Some(ref worker_id) = workers.get(&socket_addr) {
				let job_payload = self.payload_manager.job(worker_id);
				job_payload.map(
					|json| self.rpc_server.push_message(&socket_addr, json.as_bytes())
				);
			}
			else {
				// anauthorized worker
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	use std::net::SocketAddr;
	use std::sync::Arc;

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
		let stratum = Stratum::start(&SocketAddr::from_str("0.0.0.0:19980").unwrap(), Arc::new(VoidManager), None);
		assert!(stratum.is_ok());
	}

	#[test]
	fn records_subscriber() {
		let addr = SocketAddr::from_str("0.0.0.0:19985").unwrap();
		let stratum = Stratum::start(&addr, Arc::new(VoidManager), None).unwrap();
		let request = r#"{"jsonrpc": "2.0", "method": "miner.subscribe", "params": [], "id": 1}"#;
		dummy_request(&addr, request.as_bytes());
		assert_eq!(1, stratum.subscribers.read().unwrap().len());
	}

	struct DummyManager {
		initial_payload: String
	}

	impl DummyManager {
		fn new() -> Arc<DummyManager> {
			Arc::new(Self::build())
		}

		fn build() -> DummyManager {
			DummyManager { initial_payload: r#"[ "dummy payload" ]"#.to_owned() }
		}

		fn of_initial(mut self, new_initial: &str) -> DummyManager {
			self.initial_payload = new_initial.to_owned();
			self
		}
	}

	impl JobPayloadManager for DummyManager {
		fn initial(&self) -> Option<String> {
			Some(self.initial_payload.clone())
		}
	}

	#[test]
	fn receives_initial_paylaod() {
		let addr = SocketAddr::from_str("0.0.0.0:19975").unwrap();
		Stratum::start(&addr, DummyManager::new(), None).unwrap();
		let request = r#"{"jsonrpc": "2.0", "method": "miner.subscribe", "params": [], "id": 1}"#;

		let response = String::from_utf8(dummy_request(&addr, request.as_bytes())).unwrap();

		assert_eq!(r#"{"jsonrpc":"2.0","result":["dummy payload"],"id":1}"#, response);
	}

	#[test]
	fn can_authorize() {
		let addr = SocketAddr::from_str("0.0.0.0:19970").unwrap();
		let stratum = Stratum::start(
			&addr,
			Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
			None
		).unwrap();

		let request = r#"{"jsonrpc": "2.0", "method": "miner.authorize", "params": ["miner1", ""], "id": 1}"#;

		let response = String::from_utf8(dummy_request(&addr, request.as_bytes())).unwrap();

		assert_eq!(r#"{"jsonrpc":"2.0","result":true,"id":1}"#, response);
		assert_eq!(1, stratum.workers.read().unwrap().len());
	}
}
