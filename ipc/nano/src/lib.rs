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

//! IPC over nanomsg transport

extern crate ethcore_ipc as ipc;
extern crate nanomsg;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

pub use ipc::{WithSocket, IpcInterface, IpcConfig};
pub use nanomsg::Socket as NanoSocket;

use std::sync::*;
use nanomsg::{Socket, Protocol, Error, Endpoint, PollRequest, PollFd, PollInOut};
use std::ops::Deref;

const POLL_TIMEOUT: isize = 200;
const DEFAULT_CONNECTION_TIMEOUT: isize = 30000;
const DEBUG_CONNECTION_TIMEOUT: isize = 5000;

/// Generic worker to handle service (binded) sockets
pub struct Worker<S: ?Sized> where S: IpcInterface {
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

impl<S> GuardedSocket<S> where S: WithSocket<Socket> {
	pub fn service(&self) -> Arc<S> {
		self.client.clone()
	}
}

impl<S> Deref for GuardedSocket<S> where S: WithSocket<Socket> {
    type Target = Arc<S>;

    fn deref(&self) -> &Arc<S> {
        &self.client
    }
}

/// Spawns client <`S`> over specified address
/// creates socket and connects endpoint to it
/// for duplex (paired) connections with the service
pub fn init_duplex_client<S>(socket_addr: &str) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	let mut socket = Socket::new(Protocol::Pair).map_err(|e| {
		warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
		SocketError::DuplexLink
	})?;

	socket.set_receive_timeout(DEFAULT_CONNECTION_TIMEOUT).unwrap();

	let endpoint = socket.connect(socket_addr).map_err(|e| {
		warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", socket_addr, e);
		SocketError::DuplexLink
	})?;

	Ok(GuardedSocket {
		client: Arc::new(S::init(socket)),
		_endpoint: endpoint,
	})
}

/// Spawns client <`S`> over specified address
/// creates socket and connects endpoint to it
/// for request-reply connections to the service
pub fn client<S>(socket_addr: &str, receive_timeout: Option<isize>) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	let mut socket = Socket::new(Protocol::Req).map_err(|e| {
		warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
		SocketError::RequestLink
	})?;

	if let Some(timeout) = receive_timeout {
		socket.set_receive_timeout(timeout).unwrap();
	}

	let endpoint = socket.connect(socket_addr).map_err(|e| {
		warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", socket_addr, e);
		SocketError::RequestLink
	})?;

	trace!(target: "ipc", "Created client for {}", socket_addr);
	Ok(GuardedSocket {
		client: Arc::new(S::init(socket)),
		_endpoint: endpoint,
	})
}

lazy_static! {
	/// Set PARITY_IPC_DEBUG=1 for fail-fast connectivity problems diagnostic
	pub static ref DEBUG_FLAG: bool = {
		use std::env;

		if let Ok(debug) = env::var("PARITY_IPC_DEBUG") {
			debug == "1" || debug.to_uppercase() == "TRUE"
		}
		else { false }
	};
}

/// Client with no default timeout on operations
pub fn generic_client<S>(socket_addr: &str) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	if *DEBUG_FLAG {
		client(socket_addr, Some(DEBUG_CONNECTION_TIMEOUT))
	} else {
		client(socket_addr, None)
	}
}

/// Client over interface that is supposed to give quick almost non-blocking responses
pub fn fast_client<S>(socket_addr: &str) -> Result<GuardedSocket<S>, SocketError> where S: WithSocket<Socket> {
	if *DEBUG_FLAG {
		client(socket_addr, Some(DEBUG_CONNECTION_TIMEOUT))
	} else {
		client(socket_addr, Some(DEFAULT_CONNECTION_TIMEOUT))
	}
}

/// Error occurred while establising socket or endpoint
#[derive(Debug)]
pub enum SocketError {
	/// Error establising duplex (paired) socket and/or endpoint
	DuplexLink,
	/// Error establising duplex (paired) socket and/or endpoint
	RequestLink,
}

impl<S: ?Sized> Worker<S> where S: IpcInterface {
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
		use std::io::Write;

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

							if let Err(e) = socket.write(&result) {
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
		let mut socket = Socket::new(Protocol::Pair).map_err(|e| {
			warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
			SocketError::DuplexLink
		})?;

		let endpoint = socket.bind(addr).map_err(|e| {
			warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", addr, e);
			SocketError::DuplexLink
		})?;

		self.sockets.push((socket, endpoint));

		self.rebuild_poll_request();

		trace!(target: "ipc", "Started duplex worker at {}", addr);

		Ok(())
	}

	/// Add generic socket for request-reply style communications
	/// with multiple clients
	pub fn add_reqrep(&mut self, addr: &str) -> Result<(), SocketError>  {
		let mut socket = Socket::new(Protocol::Rep).map_err(|e| {
			warn!(target: "ipc", "Failed to create ipc socket: {:?}", e);
			SocketError::DuplexLink
		})?;


		let endpoint = socket.bind(addr).map_err(|e| {
			warn!(target: "ipc", "Failed to bind socket to address '{}': {:?}", addr, e);
			SocketError::DuplexLink
		})?;

		self.sockets.push((socket, endpoint));

		self.rebuild_poll_request();

		trace!(target: "ipc", "Started request-reply worker at {}", addr);
		Ok(())
	}
}

#[cfg(test)]
mod service_tests {

	use super::Worker;
	use ipc::*;
	use std::io::{Read, Write};
	use std::sync::{Arc, RwLock};
	use nanomsg::{Socket, Protocol, Endpoint};

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

	impl IpcInterface for DummyService {
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
}
