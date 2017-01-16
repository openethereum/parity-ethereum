// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

extern crate jsonrpc_tcp_server;
extern crate jsonrpc_core;
extern crate jsonrpc_macros;
#[macro_use] extern crate log;
extern crate ethcore_util as util;
extern crate ethcore_ipc as ipc;
extern crate semver;

#[cfg(test)]
extern crate mio;
#[cfg(test)]
extern crate ethcore_devtools as devtools;
#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

mod traits {
	//! Stratum ipc interfaces specification
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/traits.rs"));
}

pub use traits::{
	JobDispatcher, PushWorkHandler, Error, ServiceConfiguration,
	RemoteWorkHandler, RemoteJobDispatcher,
};

use jsonrpc_tcp_server::Server as JsonRpcServer;
use jsonrpc_core::{IoHandler, Params, to_value, Value};
use jsonrpc_macros::IoDelegate;
use std::sync::Arc;

use std::net::SocketAddr;
use std::collections::{HashSet, HashMap};
use util::{H256, Hashable, RwLock, RwLockReadGuard};

type RpcResult = Result<jsonrpc_core::Value, jsonrpc_core::Error>;

struct StratumRpc {
	stratum: RwLock<Option<Arc<Stratum>>>,
}

impl StratumRpc {
	fn subscribe(&self, params: Params) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.subscribe(params)
	}

	fn authorize(&self, params: Params) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.authorize(params)
	}

	fn submit(&self, params: Params) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.submit(params)
	}
}

pub struct Stratum {
	rpc_server: JsonRpcServer<()>,
	/// Subscribed clients
	subscribers: RwLock<Vec<SocketAddr>>,
	/// List of workers supposed to receive job update
	job_que: RwLock<HashSet<SocketAddr>>,
	/// Payload manager
	dispatcher: Arc<JobDispatcher>,
	/// Authorized workers (socket - worker_id)
	workers: Arc<RwLock<HashMap<SocketAddr, String>>>,
	/// Secret if any
	secret: Option<H256>,
	/// Dispatch notify couinter
	notify_counter: RwLock<u32>,
}

const NOTIFY_CONTER_INITIAL: u32 = 16;

impl Stratum {
	pub fn start(
		addr: &SocketAddr,
		dispatcher: Arc<JobDispatcher>,
		secret: Option<H256>,
	) -> Result<Arc<Stratum>, Error> {
		let rpc = Arc::new(StratumRpc {
			stratum: RwLock::new(None),
		});
		let mut delegate = IoDelegate::<StratumRpc>::new(rpc.clone());
		delegate.add_method("miner.subscribe", StratumRpc::subscribe);
		delegate.add_method("miner.authorize", StratumRpc::authorize);
		delegate.add_method("miner.sumbit", StratumRpc::submit);

		let mut handler = IoHandler::default();
		handler.extend_with(delegate);
		let server = JsonRpcServer::new(addr, handler)?;
		let stratum = Arc::new(Stratum {
			rpc_server: server,
			subscribers: RwLock::new(Vec::new()),
			job_que: RwLock::new(HashSet::new()),
			dispatcher: dispatcher,
			workers: Arc::new(RwLock::new(HashMap::new())),
			secret: secret,
			notify_counter: RwLock::new(NOTIFY_CONTER_INITIAL),
		});
		*rpc.stratum.write() = Some(stratum.clone());

		try!(stratum.rpc_server.run_async());

		Ok(stratum)
	}

	fn submit(&self, params: Params) -> std::result::Result<jsonrpc_core::Value, jsonrpc_core::Error> {
		Ok(match params {
			Params::Array(vals) => {
				// first two elements are service messages (worker_id & job_id)
				match self.dispatcher.submit(vals.iter().skip(2)
					.filter_map(|val| match val { &Value::String(ref str) => Some(str.to_owned()), _ => None })
					.collect::<Vec<String>>()) {
						Ok(()) => {
							to_value(true)
						},
						Err(submit_err) => {
							warn!("Error while submitting share: {:?}", submit_err);
							to_value(false)
						}
					}
			},
			_ => {
				trace!(target: "stratum", "Invalid submit work format {:?}", params);
				to_value(false)
			}
		})
	}

	fn subscribe(&self, _params: Params) -> std::result::Result<jsonrpc_core::Value, jsonrpc_core::Error> {

		use std::str::FromStr;

		if let Some(context) = self.rpc_server.request_context() {
			self.subscribers.write().push(context.socket_addr);
			self.job_que.write().insert(context.socket_addr);
			trace!(target: "stratum", "Subscription request from {:?}", context.socket_addr);
		}
		Ok(match self.dispatcher.initial() {
			Some(initial) => match jsonrpc_core::Value::from_str(&initial) {
				Ok(val) => val,
				Err(e) => {
					warn!(target: "stratum", "Invalid payload: '{}' ({:?})", &initial, e);
					to_value(&[0u8; 0])
				},
			},
			None => to_value(&[0u8; 0]),
		})
	}

	fn authorize(&self, params: Params) -> RpcResult {
		params.parse::<(String, String)>().map(|(worker_id, secret)|{
			if let Some(valid_secret) = self.secret {
				let hash = secret.sha3();
				if hash != valid_secret {
					return to_value(&false);
				}
			}
			if let Some(context) = self.rpc_server.request_context() {
				self.workers.write().insert(context.socket_addr, worker_id);
				to_value(true)
			}
			else {
				warn!(target: "stratum", "Authorize without valid context received!");
				to_value(false)
			}
		})
	}

	pub fn subscribers(&self) -> RwLockReadGuard<Vec<SocketAddr>> {
		self.subscribers.read()
	}

	pub fn maintain(&self) {
		let mut job_que = self.job_que.write();
		let job_payload = self.dispatcher.job();
		for socket_addr in job_que.drain() {
			job_payload.as_ref().map(
				|json| self.rpc_server.push_message(&socket_addr, json.as_bytes())
			);
		}
	}
}

impl PushWorkHandler for Stratum {
	fn push_work_all(&self, payload: String) -> Result<(), Error> {
		let workers = self.workers.read();
		let next_request_id = {
			let mut counter = self.notify_counter.write();
			if *counter == ::std::u32::MAX { *counter = NOTIFY_CONTER_INITIAL; }
			else { *counter = *counter + 1 }
			*counter
		};

		let workers_msg = format!("{{ \"id\": {}, \"method\": \"mining.notify\", \"params\": {} }}\n", next_request_id, payload);
		trace!(target: "stratum", "pushing work for {} workers (payload: '{}')", workers.len(), &workers_msg);
		for (ref addr, _) in workers.iter() {
			try!(self.rpc_server.push_message(addr, workers_msg.as_bytes()));
		}
		Ok(())
	}

	fn push_work(&self, payloads: Vec<String>) -> Result<(), Error>  {
		if !payloads.len() > 0 {
			return Err(Error::NoWork);
		}
		let workers = self.workers.read();
		let addrs = workers.keys().collect::<Vec<&SocketAddr>>();
		if !workers.len() > 0 {
			return Err(Error::NoWorkers);
		}
		let mut que = payloads;
		let mut addr_index = 0;
		while que.len() > 0 {
			let next_worker = addrs[addr_index];
			let mut next_payload = que.drain(0..1);
			self.rpc_server.push_message(
					next_worker,
					next_payload.nth(0).expect("drained successfully of 0..1, so 0-th element should exist").as_bytes()
				)?;
			addr_index = addr_index + 1;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	use std::net::SocketAddr;
	use std::sync::{Arc, RwLock};
	use std::thread;

	pub struct VoidManager;

	impl JobDispatcher for VoidManager {
		fn submit(&self, _payload: Vec<String>) -> Result<(), Error> {
			Ok(())
		}
	}

	lazy_static! {
		static ref LOG_DUMMY: bool = {
			use log::LogLevelFilter;
			use env_logger::LogBuilder;
			use std::env;

			let mut builder = LogBuilder::new();
			builder.filter(None, LogLevelFilter::Info);

			if let Ok(log) = env::var("RUST_LOG") {
				builder.parse(&log);
			}

			if let Ok(_) = builder.init() {
				println!("logger initialized");
			}
			true
		};
	}

	/// Intialize log with default settings
	#[cfg(test)]
	fn init_log() {
		let _ = *LOG_DUMMY;
	}

	pub fn dummy_request(addr: &SocketAddr, buf: &[u8]) -> Vec<u8> {
		use std::io::{Read, Write};
		use mio::*;
		use mio::tcp::*;

		let mut poll = Poll::new().unwrap();
		let mut sock = TcpStream::connect(addr).unwrap();
		poll.register(&sock, Token(0), EventSet::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
		poll.poll(Some(50)).unwrap();
		sock.write_all(buf).unwrap();
		poll.reregister(&sock, Token(0), EventSet::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
		poll.poll(Some(50)).unwrap();

		let mut buf = Vec::new();
		sock.read_to_end(&mut buf).unwrap_or_else(|_| { 0 });
		buf
	}

	pub fn dummy_async_waiter(addr: &SocketAddr, initial: Vec<String>, result: Arc<RwLock<Vec<String>>>) -> ::devtools::StopGuard {
		use std::io::{Read, Write};
		use mio::*;
		use mio::tcp::*;
		use std::sync::atomic::Ordering;

		let stop_guard = ::devtools::StopGuard::new();
		let collector = result.clone();
		let thread_stop = stop_guard.share();
		let socket_addr = addr.clone();
		thread::spawn(move || {
			let mut poll = Poll::new().unwrap();
			let mut sock = TcpStream::connect(&socket_addr).unwrap();
			poll.register(&sock, Token(0), EventSet::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();

			for initial_req in initial {
				poll.poll(Some(120)).unwrap();
				sock.write_all(initial_req.as_bytes()).unwrap();
				poll.reregister(&sock, Token(0), EventSet::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
				poll.poll(Some(120)).unwrap();

				let mut buf = Vec::new();
				sock.read_to_end(&mut buf).unwrap_or_else(|_| { 0 });
				collector.write().unwrap().push(String::from_utf8(buf).unwrap());
				poll.reregister(&sock, Token(0), EventSet::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
			}

			while !thread_stop.load(Ordering::Relaxed) {
				poll.reregister(&sock, Token(0), EventSet::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
				poll.poll(Some(120)).unwrap();

				let mut buf = Vec::new();
				sock.read_to_end(&mut buf).unwrap_or_else(|_| { 0 });
				if buf.len() > 0 {
					collector.write().unwrap().push(String::from_utf8(buf).unwrap());
				}
			}
		});

		stop_guard
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
		let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 1}"#;
		dummy_request(&addr, request.as_bytes());
		assert_eq!(1, stratum.subscribers.read().len());
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

	impl JobDispatcher for DummyManager {
		fn initial(&self) -> Option<String> {
			Some(self.initial_payload.clone())
		}

		fn submit(&self, _payload: Vec<String>) -> Result<(), Error> {
			Ok(())
		}
	}

	#[test]
	fn receives_initial_paylaod() {
		let addr = SocketAddr::from_str("0.0.0.0:19975").unwrap();
		Stratum::start(&addr, DummyManager::new(), None).unwrap();
		let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 2}"#;

		let response = String::from_utf8(dummy_request(&addr, request.as_bytes())).unwrap();

		assert_eq!(r#"{"jsonrpc":"2.0","result":["dummy payload"],"id":2}"#, response);
	}

	#[test]
	fn can_authorize() {
		let addr = SocketAddr::from_str("0.0.0.0:19970").unwrap();
		let stratum = Stratum::start(
			&addr,
			Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
			None
		).unwrap();

		let request = r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#;

		let response = String::from_utf8(dummy_request(&addr, request.as_bytes())).unwrap();

		assert_eq!(r#"{"jsonrpc":"2.0","result":true,"id":1}"#, response);
		assert_eq!(1, stratum.workers.read().len());
	}

	#[test]
	fn can_push_work() {
		init_log();

		let addr = SocketAddr::from_str("0.0.0.0:19965").unwrap();
		let stratum = Stratum::start(
			&addr,
			Arc::new(DummyManager::build().of_initial(r#"["dummy push request payload"]"#)),
			None
		).unwrap();

		let result = Arc::new(RwLock::new(Vec::<String>::new()));
		let _stop = dummy_async_waiter(
			&addr,
			vec![
				r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#.to_owned(),
			],
			result.clone(),
		);
		::std::thread::park_timeout(::std::time::Duration::from_millis(150));

		stratum.push_work_all(r#"{ "00040008", "100500" }"#.to_owned()).unwrap();
		::std::thread::park_timeout(::std::time::Duration::from_millis(150));

		assert_eq!(2, result.read().unwrap().len());
	}
}
