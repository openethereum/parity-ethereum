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

//! IPC over nanomsg transport

extern crate ethcore_ipc as ipc;
extern crate nanomsg;
#[macro_use] extern crate log;
extern crate jsonrpc_core;
use jsonrpc_core::IoHandler;

pub use ipc::{WithSocket, IpcInterface, IpcConfig};

use std::sync::*;
use std::sync::atomic::*;
use nanomsg::{Socket, Protocol, Error, Endpoint, PollRequest, PollFd, PollInOut};
use std::ops::Deref;

const POLL_TIMEOUT: isize = 100;

/// Generic worker to handle service (binded) sockets
pub struct Worker<S> where S: IpcInterface<S> {
	service: Arc<S>,
	sockets: Vec<(Socket, Endpoint)>,
	polls: Vec<PollFd>,
	buf: Vec<u8>,
}

/// struct for guarding `_endpoint` (so that it wont drop)
/// derefs to client `S`
pub struct GuardedSocket<S> where S: WithSocket<Socket> {
	client: Arc<S>,
	_endpoint: Endpoint,
}

impl<S> Deref for GuardedSocket<S> where S: WithSocket<Socket> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.client
    }
}

/// Spawns client <`S`> over specified address
/// creates socket and connects endpoint to it
/// for duplex (paired) connections with the service
pub fn init_duplex_client<S>(socket_addr: &str) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	let mut socket = try!(Socket::new(Protocol::Pair).map_err(|e| {
		warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
		SocketError::DuplexLink
	}));

	let endpoint = try!(socket.connect(socket_addr).map_err(|e| {
		warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", socket_addr, e);
		SocketError::DuplexLink
	}));

	Ok(GuardedSocket {
		client: Arc::new(S::init(socket)),
		_endpoint: endpoint,
	})
}

/// Spawns client <`S`> over specified address
/// creates socket and connects endpoint to it
/// for request-reply connections to the service
pub fn init_client<S>(socket_addr: &str) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	let mut socket = try!(Socket::new(Protocol::Req).map_err(|e| {
		warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
		SocketError::RequestLink
	}));

	let endpoint = try!(socket.connect(socket_addr).map_err(|e| {
		warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", socket_addr, e);
		SocketError::RequestLink
	}));

	Ok(GuardedSocket {
		client: Arc::new(S::init(socket)),
		_endpoint: endpoint,
	})
}

/// Error occured while establising socket or endpoint
#[derive(Debug)]
pub enum SocketError {
	/// Error establising duplex (paired) socket and/or endpoint
	DuplexLink,
	/// Error establising duplex (paired) socket and/or endpoint
	RequestLink,
}

impl<S> Worker<S> where S: IpcInterface<S> {
	/// New worker over specified `service`
	pub fn new(service: &Arc<S>) -> Worker<S> {
		Worker::<S> {
			service: service.clone(),
			sockets: Vec::new(),
			polls: Vec::new(),
			buf: Vec::new(),
		}
	}

	/// Polls all sockets, reads and dispatches method invocations
	pub fn poll(&mut self) {
		let mut request = PollRequest::new(&mut self.polls[..]);
 		let _result_guard = Socket::poll(&mut request, POLL_TIMEOUT);

		for (fd_index, fd) in request.get_fds().iter().enumerate() {
			if fd.can_read() {
				let (ref mut socket, _) = self.sockets[fd_index];
				unsafe { self.buf.set_len(0); }
				match socket.nb_read_to_end(&mut self.buf) {
					Ok(method_sign_len) => {
						if method_sign_len >= 2 {

							// method_num
							let method_num = self.buf[0] as u16 * 256 + self.buf[1] as u16;
							// payload
							let payload = &self.buf[2..];

							// dispatching for ipc interface
							let result = self.service.dispatch_buf(method_num, payload);

							if let Err(e) = socket.nb_write(&result) {
								warn!(target: "ipc", "Failed to write response: {:?}", e);
							}
						}
						else {
							warn!(target: "ipc", "Failed to read method signature from socket: unexpected message length({})", method_sign_len);
						}
					},
					Err(Error::TryAgain) => {
					},
					Err(x) => {
						warn!(target: "ipc", "Error polling connections {:?}", x);
						panic!();
					}
				}
			}
		}
	}

	/// Stores nanomsg poll request for reuse
	fn rebuild_poll_request(&mut self) {
		self.polls = self.sockets.iter()
			.map(|&(ref socket, _)| socket.new_pollfd(PollInOut::In))
			.collect::<Vec<PollFd>>();
	}

	/// Add exclusive socket for paired client
	/// Only one connection over this address is allowed
	pub fn add_duplex(&mut self, addr: &str) -> Result<(), SocketError>  {
		let mut socket = try!(Socket::new(Protocol::Pair).map_err(|e| {
			warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
			SocketError::DuplexLink
		}));

		let endpoint = try!(socket.bind(addr).map_err(|e| {
			warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", addr, e);
			SocketError::DuplexLink
		}));

		self.sockets.push((socket, endpoint));

		self.rebuild_poll_request();

		Ok(())
	}

	/// Add generic socket for request-reply style communications
	/// with multiple clients
	pub fn add_reqrep(&mut self, addr: &str) -> Result<(), SocketError>  {
		let mut socket = try!(Socket::new(Protocol::Rep).map_err(|e| {
			warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
			SocketError::DuplexLink
		}));

		let endpoint = try!(socket.bind(addr).map_err(|e| {
			warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", addr, e);
			SocketError::DuplexLink
		}));

		self.sockets.push((socket, endpoint));

		self.rebuild_poll_request();

		Ok(())
	}
}

/// Error in handling JSON RPC request
pub enum IoHandlerError {
	BadRequest,
	HandlerError,
}

/// Worker to handle JSON RPC requests
pub struct IoHandlerWorker {
	handler: Arc<IoHandler>,
	socket: Socket,
	_endpoint: Endpoint,
	poll: Vec<PollFd>,
	buf: Vec<u8>,
}

/// IPC server for json-rpc handler (single thread)
pub struct IoHandlerServer {
	is_stopping: Arc<AtomicBool>,
	is_stopped: Arc<AtomicBool>,
	handler: Arc<IoHandler>,
	socket_addr: String,
}

impl IoHandlerServer {
	/// New IPC server for JSON RPC `handler` and ipc socket address `socket_addr`
	pub fn new(handler: &Arc<IoHandler>, socket_addr: &str) -> IoHandlerServer {
		IoHandlerServer {
			handler: handler.clone(),
			is_stopping: Arc::new(AtomicBool::new(false)),
			is_stopped: Arc::new(AtomicBool::new(true)),
			socket_addr: socket_addr.to_owned(),
		}
	}

	/// IPC Server starts (non-blocking, in seprate thread)
	pub fn start(&self) -> Result<(), SocketError> {
		let mut worker = try!(IoHandlerWorker::new(&self.handler, &self.socket_addr));
		self.is_stopping.store(false, Ordering::Relaxed);
		let worker_is_stopping = self.is_stopping.clone();
		let worker_is_stopped = self.is_stopped.clone();

		::std::thread::spawn(move || {
			worker_is_stopped.store(false, Ordering::Relaxed);
			while !worker_is_stopping.load(Ordering::Relaxed) {
				worker.poll()
			}
			worker_is_stopped.store(true, Ordering::Relaxed);
		});

		Ok(())
	}

	/// IPC server stop (func will wait until effective stop)
	pub fn stop(&self) {
		self.is_stopping.store(true, Ordering::Relaxed);
		while !self.is_stopped.load(Ordering::Relaxed) {
			std::thread::sleep(std::time::Duration::from_millis(50));
		}
	}
}

impl Drop for IoHandlerServer {
	fn drop(&mut self) {
		self.stop()
	}
}

impl IoHandlerWorker {
	pub fn new(handler: &Arc<IoHandler>, socket_addr: &str) -> Result<IoHandlerWorker, SocketError> {
		let mut socket = try!(Socket::new(Protocol::Rep).map_err(|e| {
			warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
			SocketError::RequestLink
		}));

		let endpoint = try!(socket.bind(socket_addr).map_err(|e| {
			warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", socket_addr, e);
			SocketError::RequestLink
		}));

		let poll = vec![socket.new_pollfd(PollInOut::In)];

		Ok(IoHandlerWorker {
			handler: handler.clone(),
			socket: socket,
			_endpoint: endpoint,
			poll: poll,
			buf: Vec::with_capacity(1024),
		})
	}

	pub fn poll(&mut self) {
		let mut request = PollRequest::new(&mut self.poll[..]);
 		let _result_guard = Socket::poll(&mut request, POLL_TIMEOUT);
		let fd = request.get_fds()[0]; 	// guaranteed to exist and be the only one
										// because contains only immutable socket field as a member
		if !fd.can_read() {
			return;
		}

		unsafe { self.buf.set_len(0); }
		match self.socket.nb_read_to_end(&mut self.buf) {
			Ok(0) => {
				warn!(target: "ipc", "RPC empty message received");
				return;
			},
			Ok(_) => {
				let rpc_msg = match String::from_utf8(self.buf.clone()) {
					Ok(val) => val,
					Err(e) => {
						warn!(target: "ipc", "RPC decoding error (utf-8): {:?}", e);
						return;
					}
				};
				let response: Option<String> = self.handler.handle_request(&rpc_msg);
				if let Some(response_str) = response {
					let response_bytes = response_str.into_bytes();
					if let Err(e) = self.socket.nb_write(&response_bytes) {
						warn!(target: "ipc", "Failed to write response: {:?}", e);
					}
				}
			},
			Err(Error::TryAgain) => {
				// no data
			},
			Err(x) => {
				warn!(target: "ipc", "Error polling connections {:?}", x);
				panic!("IPC RPC fatal error");
			},
		}
	}

}

#[cfg(test)]
mod service_tests {

	use super::{Worker, IoHandlerServer};
	use ipc::*;
	use std::io::{Read, Write};
	use std::sync::{Arc, RwLock};
	use nanomsg::{Socket, Protocol, Endpoint};
	use jsonrpc_core;
	use jsonrpc_core::{IoHandler, Value, Params, MethodCommand};

	struct TestInvoke {
		method_num: u16,
		params: Vec<u8>,
	}

	struct DummyService {
		methods_stack: RwLock<Vec<TestInvoke>>,
	}

	impl DummyService {
		fn new() -> DummyService {
			DummyService { methods_stack: RwLock::new(Vec::new()) }
		}
	}

	impl IpcInterface<DummyService> for DummyService {
		fn dispatch<R>(&self, _r: &mut R) -> Vec<u8> where R: Read {
			vec![]
		}
		fn dispatch_buf(&self, method_num: u16, buf: &[u8]) -> Vec<u8> {
			self.methods_stack.write().unwrap().push(
				TestInvoke {
					method_num: method_num,
					params: buf.to_vec(),
				});
			vec![]
		}
	}

	impl IpcConfig for DummyService {}

	fn dummy_write(addr: &str, buf: &[u8]) -> (Socket, Endpoint) {
		let mut socket = Socket::new(Protocol::Pair).unwrap();
		let endpoint = socket.connect(addr).unwrap();
		socket.write(buf).unwrap();
		(socket, endpoint)
	}

	fn dummy_request(addr: &str, buf: &[u8]) -> Vec<u8> {
		let mut socket = Socket::new(Protocol::Req).unwrap();
		let _endpoint = socket.connect(addr).unwrap();
		socket.write(buf).unwrap();
		let mut buf = Vec::new();
		socket.read_to_end(&mut buf).unwrap();
		buf
	}

	#[test]
	fn can_create_worker() {
		let worker = Worker::<DummyService>::new(&Arc::new(DummyService::new()));
		assert_eq!(0, worker.sockets.len());
	}

	#[test]
	fn can_add_duplex_socket_to_worker() {
		let mut worker = Worker::<DummyService>::new(&Arc::new(DummyService::new()));
		worker.add_duplex("ipc:///tmp/parity-test10.ipc").unwrap();
		assert_eq!(1, worker.sockets.len());
	}

	#[test]
	fn worker_can_poll_empty() {
		let service = Arc::new(DummyService::new());
		let mut worker = Worker::<DummyService>::new(&service);
		worker.add_duplex("ipc:///tmp/parity-test20.ipc").unwrap();
		worker.poll();
		assert_eq!(0, service.methods_stack.read().unwrap().len());
	}

	#[test]
	fn worker_can_poll() {
		let url = "ipc:///tmp/parity-test30.ipc";

		let mut worker = Worker::<DummyService>::new(&Arc::new(DummyService::new()));
		worker.add_duplex(url).unwrap();

		let (_socket, _endpoint) = dummy_write(url, &vec![0, 0, 7, 7, 6, 6]);
		worker.poll();

		assert_eq!(1, worker.service.methods_stack.read().unwrap().len());
		assert_eq!(0, worker.service.methods_stack.read().unwrap()[0].method_num);
		assert_eq!([7, 7, 6, 6], worker.service.methods_stack.read().unwrap()[0].params[..]);
	}

	#[test]
	fn worker_can_poll_long() {
		let url = "ipc:///tmp/parity-test40.ipc";

		let mut worker = Worker::<DummyService>::new(&Arc::new(DummyService::new()));
		worker.add_duplex(url).unwrap();

		let message = [0u8; 1024*1024];

		let (_socket, _endpoint) = dummy_write(url, &message);
		worker.poll();

		assert_eq!(1, worker.service.methods_stack.read().unwrap().len());
		assert_eq!(0, worker.service.methods_stack.read().unwrap()[0].method_num);
		assert_eq!(vec![0u8; 1024*1024-2], worker.service.methods_stack.read().unwrap()[0].params);
	}

	#[test]
	fn test_jsonrpc_handler() {
		let url = "ipc:///tmp/parity-test50.ipc";

		struct SayHello;
		impl MethodCommand for SayHello {
			fn execute(&self, _params: Params) -> Result<Value, jsonrpc_core::Error> {
				Ok(Value::String("hello".to_string()))
			}
		}

		let io = Arc::new(IoHandler::new());
		io.add_method("say_hello", SayHello);

		let request = r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
		let response = r#"{"jsonrpc":"2.0","result":"hello","id":1}"#;

		let server = IoHandlerServer::new(&io, url);
		server.start().unwrap();

		assert_eq!(String::from_utf8(dummy_request(url, request.as_bytes())).unwrap(), response.to_string());

		server.stop();
	}
}
