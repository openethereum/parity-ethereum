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

//! Stratum protocol implementation for parity ethereum/bitcoin clients

extern crate jsonrpc_tcp_server;
extern crate jsonrpc_core;
extern crate jsonrpc_macros;
extern crate ethereum_types;
extern crate keccak_hash as hash;
extern crate parking_lot;

#[macro_use] extern crate log;

#[cfg(test)] extern crate tokio_core;
#[cfg(test)] extern crate tokio_io;
#[cfg(test)] extern crate ethcore_logger;

mod traits;

pub use traits::{
	JobDispatcher, PushWorkHandler, Error, ServiceConfiguration,
};

use jsonrpc_tcp_server::{
	Server as JsonRpcServer, ServerBuilder as JsonRpcServerBuilder,
	RequestContext, MetaExtractor, Dispatcher, PushMessageError,
};
use jsonrpc_core::{MetaIoHandler, Params, to_value, Value, Metadata, Compatibility};
use jsonrpc_macros::IoDelegate;
use std::sync::Arc;

use std::net::SocketAddr;
use std::collections::{HashSet, HashMap};
use hash::keccak;
use ethereum_types::H256;
use parking_lot::{RwLock, RwLockReadGuard};

type RpcResult = Result<jsonrpc_core::Value, jsonrpc_core::Error>;

const NOTIFY_COUNTER_INITIAL: u32 = 16;

struct StratumRpc {
	stratum: RwLock<Option<Arc<Stratum>>>,
}

impl StratumRpc {
	fn subscribe(&self, params: Params, meta: SocketMetadata) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.subscribe(params, meta)
	}

	fn authorize(&self, params: Params, meta: SocketMetadata) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.authorize(params, meta)
	}

	fn submit(&self, params: Params, meta: SocketMetadata) -> RpcResult {
		self.stratum.read().as_ref().expect("RPC methods are called after stratum is set.")
			.submit(params, meta)
	}
}

#[derive(Clone)]
pub struct SocketMetadata {
	addr: SocketAddr,
}

impl Default for SocketMetadata {
	fn default() -> Self {
		SocketMetadata { addr: "0.0.0.0:0".parse().unwrap() }
	}
}

impl SocketMetadata {
	pub fn addr(&self) -> &SocketAddr {
		&self.addr
	}
}

impl Metadata for SocketMetadata { }

impl From<SocketAddr> for SocketMetadata {
	fn from(addr: SocketAddr) -> SocketMetadata {
		SocketMetadata { addr: addr }
	}
}

pub struct PeerMetaExtractor;

impl MetaExtractor<SocketMetadata> for PeerMetaExtractor {
	fn extract(&self, context: &RequestContext) -> SocketMetadata {
		context.peer_addr.into()
	}
}

pub struct Stratum {
	rpc_server: Option<JsonRpcServer>,
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
	/// Message dispatcher (tcp/ip service)
	tcp_dispatcher: Dispatcher,
}

impl Drop for Stratum {
	fn drop(&mut self) {
		self.rpc_server.take().map(|server| server.close());
	}
}

impl Stratum {
	pub fn start(
		addr: &SocketAddr,
		dispatcher: Arc<JobDispatcher>,
		secret: Option<H256>,
	) -> Result<Arc<Stratum>, Error> {

		let rpc = Arc::new(StratumRpc {
			stratum: RwLock::new(None),
		});
		let mut delegate = IoDelegate::<StratumRpc, SocketMetadata>::new(rpc.clone());
		delegate.add_method_with_meta("mining.subscribe", StratumRpc::subscribe);
		delegate.add_method_with_meta("mining.authorize", StratumRpc::authorize);
		delegate.add_method_with_meta("mining.submit", StratumRpc::submit);
		let mut handler = MetaIoHandler::<SocketMetadata>::with_compatibility(Compatibility::Both);
		handler.extend_with(delegate);

		let server = JsonRpcServerBuilder::new(handler)
			.session_meta_extractor(PeerMetaExtractor);
		let tcp_dispatcher = server.dispatcher();
		let server = server.start(addr)?;

		let stratum = Arc::new(Stratum {
			tcp_dispatcher: tcp_dispatcher,
			rpc_server: Some(server),
			subscribers: RwLock::new(Vec::new()),
			job_que: RwLock::new(HashSet::new()),
			dispatcher: dispatcher,
			workers: Arc::new(RwLock::new(HashMap::new())),
			secret: secret,
			notify_counter: RwLock::new(NOTIFY_COUNTER_INITIAL),
		});
		*rpc.stratum.write() = Some(stratum.clone());
		Ok(stratum)
	}

	fn update_peers(&self) {
		if let Some(job) = self.dispatcher.job() {
			if let Err(e) = self.push_work_all(job) {
				warn!("Failed to update some of the peers: {:?}", e);
			}
		}
	}

	fn submit(&self, params: Params, _meta: SocketMetadata) -> RpcResult {
		Ok(match params {
			Params::Array(vals) => {
				// first two elements are service messages (worker_id & job_id)
				match self.dispatcher.submit(vals.iter().skip(2)
					.filter_map(|val| match val { &Value::String(ref str) => Some(str.to_owned()), _ => None })
					.collect::<Vec<String>>()) {
						Ok(()) => {
							self.update_peers();
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
		}.expect("Only true/false is returned and it's always serializable; qed"))
	}

	fn subscribe(&self, _params: Params, meta: SocketMetadata) -> RpcResult {
		use std::str::FromStr;

		self.subscribers.write().push(meta.addr().clone());
		self.job_que.write().insert(meta.addr().clone());
		trace!(target: "stratum", "Subscription request from {:?}", meta.addr());

		Ok(match self.dispatcher.initial() {
			Some(initial) => match jsonrpc_core::Value::from_str(&initial) {
				Ok(val) => Ok(val),
				Err(e) => {
					warn!(target: "stratum", "Invalid payload: '{}' ({:?})", &initial, e);
					to_value(&[0u8; 0])
				},
			},
			None => to_value(&[0u8; 0]),
		}.expect("Empty slices are serializable; qed"))
	}

	fn authorize(&self, params: Params, meta: SocketMetadata) -> RpcResult {
		params.parse::<(String, String)>().map(|(worker_id, secret)|{
			if let Some(valid_secret) = self.secret {
				let hash = keccak(secret);
				if hash != valid_secret {
					return to_value(&false);
				}
			}
			trace!(target: "stratum", "New worker #{} registered", worker_id);
			self.workers.write().insert(meta.addr().clone(), worker_id);
			to_value(true)
		}).map(|v| v.expect("Only true/false is returned and it's always serializable; qed"))
	}

	pub fn subscribers(&self) -> RwLockReadGuard<Vec<SocketAddr>> {
		self.subscribers.read()
	}

	pub fn maintain(&self) {
		let mut job_que = self.job_que.write();
		let job_payload = self.dispatcher.job();
		for socket_addr in job_que.drain() {
			job_payload.as_ref().map(
				|json| self.tcp_dispatcher.push_message(&socket_addr, json.to_owned())
			);
		}
	}
}

impl PushWorkHandler for Stratum {
	fn push_work_all(&self, payload: String) -> Result<(), Error> {
		let hup_peers = {
			let workers = self.workers.read();
			let next_request_id = {
				let mut counter = self.notify_counter.write();
				if *counter == ::std::u32::MAX { *counter = NOTIFY_COUNTER_INITIAL; }
				else { *counter = *counter + 1 }
				*counter
			};

			let mut hup_peers = HashSet::with_capacity(0); // most of the cases won't be needed, hence avoid allocation
			let workers_msg = format!("{{ \"id\": {}, \"method\": \"mining.notify\", \"params\": {} }}", next_request_id, payload);
			trace!(target: "stratum", "pushing work for {} workers (payload: '{}')", workers.len(), &workers_msg);
			for (ref addr, _) in workers.iter() {
				trace!(target: "stratum", "pusing work to {}", addr);
				match self.tcp_dispatcher.push_message(addr, workers_msg.clone()) {
					Err(PushMessageError::NoSuchPeer) => {
						trace!(target: "stratum", "Worker no longer connected: {}", &addr);
						hup_peers.insert(*addr.clone());
					},
					Err(e) => {
						warn!(target: "stratum", "Unexpected transport error: {:?}", e);
					},
					Ok(_) => { },
				}
			}
			hup_peers
		};

		if !hup_peers.is_empty() {
			let mut workers = self.workers.write();
			for hup_peer in hup_peers { workers.remove(&hup_peer); }
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
			self.tcp_dispatcher.push_message(
					next_worker,
					next_payload.nth(0).expect("drained successfully of 0..1, so 0-th element should exist")
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
	use std::sync::Arc;

	use tokio_core::reactor::{Core, Timeout};
	use tokio_core::net::TcpStream;
	use tokio_io::io;
	use jsonrpc_core::futures::{Future, future};

	use ethcore_logger::init_log;

	pub struct VoidManager;

	impl JobDispatcher for VoidManager {
		fn submit(&self, _payload: Vec<String>) -> Result<(), Error> {
			Ok(())
		}
	}

	fn dummy_request(addr: &SocketAddr, data: &str) -> Vec<u8> {
		let mut core = Core::new().expect("Tokio Core should be created with no errors");
		let mut buffer = vec![0u8; 2048];

		let mut data_vec = data.as_bytes().to_vec();
		data_vec.extend(b"\n");

		let stream = TcpStream::connect(addr, &core.handle())
			.and_then(|stream| {
				io::write_all(stream, &data_vec)
			})
			.and_then(|(stream, _)| {
				io::read(stream, &mut buffer)
			})
			.and_then(|(_, read_buf, len)| {
				future::ok(read_buf[0..len].to_vec())
			});
			let result = core.run(stream).expect("Core should run with no errors");

			result
	}

	#[test]
	fn can_be_started() {
		let stratum = Stratum::start(&SocketAddr::from_str("127.0.0.1:19980").unwrap(), Arc::new(VoidManager), None);
		assert!(stratum.is_ok());
	}

	#[test]
	fn records_subscriber() {
		init_log();

		let addr = SocketAddr::from_str("127.0.0.1:19985").unwrap();
		let stratum = Stratum::start(&addr, Arc::new(VoidManager), None).unwrap();
		let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 1}"#;
		dummy_request(&addr, request);
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

	fn terminated_str(origin: &'static str) -> String {
		let mut s = String::new();
		s.push_str(origin);
		s.push_str("\n");
		s
	}

	#[test]
	fn receives_initial_paylaod() {
		let addr = SocketAddr::from_str("127.0.0.1:19975").unwrap();
		Stratum::start(&addr, DummyManager::new(), None).expect("There should be no error starting stratum");
		let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 2}"#;

		let response = String::from_utf8(dummy_request(&addr, request)).unwrap();

		assert_eq!(terminated_str(r#"{"jsonrpc":"2.0","result":["dummy payload"],"id":2}"#), response);
	}

	#[test]
	fn can_authorize() {
		let addr = SocketAddr::from_str("127.0.0.1:19970").unwrap();
		let stratum = Stratum::start(
			&addr,
			Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
			None
		).expect("There should be no error starting stratum");

		let request = r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#;
		let response = String::from_utf8(dummy_request(&addr, request)).unwrap();

		assert_eq!(terminated_str(r#"{"jsonrpc":"2.0","result":true,"id":1}"#), response);
		assert_eq!(1, stratum.workers.read().len());
	}

	#[test]
	fn can_push_work() {
		init_log();

		let addr = SocketAddr::from_str("127.0.0.1:19995").unwrap();
		let stratum = Stratum::start(
			&addr,
			Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
			None
		).expect("There should be no error starting stratum");

		let mut auth_request =
			r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#
			.as_bytes()
			.to_vec();
		auth_request.extend(b"\n");

		let mut core = Core::new().expect("Tokio Core should be created with no errors");
		let timeout1 = Timeout::new(::std::time::Duration::from_millis(100), &core.handle())
			.expect("There should be a timeout produced in message test");
		let timeout2 = Timeout::new(::std::time::Duration::from_millis(100), &core.handle())
			.expect("There should be a timeout produced in message test");
		let mut buffer = vec![0u8; 2048];
		let mut buffer2 = vec![0u8; 2048];
		let stream = TcpStream::connect(&addr, &core.handle())
			.and_then(|stream| {
				io::write_all(stream, &auth_request)
			})
			.and_then(|(stream, _)| {
				io::read(stream, &mut buffer)
			})
			.and_then(|(stream, _, _)| {
				trace!(target: "stratum", "Received authorization confirmation");
				timeout1.join(future::ok(stream))
			})
			.and_then(|(_, stream)| {
				trace!(target: "stratum", "Pusing work to peers");
				stratum.push_work_all(r#"{ "00040008", "100500" }"#.to_owned())
					.expect("Pushing work should produce no errors");
				timeout2.join(future::ok(stream))
			})
			.and_then(|(_, stream)| {
				trace!(target: "stratum", "Ready to read work from server");
				io::read(stream, &mut buffer2)
			})
			.and_then(|(_, read_buf, len)| {
				trace!(target: "stratum", "Received work from server");
				future::ok(read_buf[0..len].to_vec())
			});
		let response = String::from_utf8(
			core.run(stream).expect("Core should run with no errors")
		).expect("Response should be utf-8");

		assert_eq!(
			"{ \"id\": 17, \"method\": \"mining.notify\", \"params\": { \"00040008\", \"100500\" } }\n",
			response);
	}
}
