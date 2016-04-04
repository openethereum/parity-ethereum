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

pub use ipc::*;

use std::sync::*;
use std::io::{Write, Read};
use nanomsg::{Socket, Protocol, Error, Endpoint};

pub struct Worker<S> where S: IpcInterface<S> {
	service: Arc<S>,
	sockets: Vec<(Socket, Endpoint)>,
	method_buf: [u8;2],
}

#[derive(Debug)]
pub enum SocketError {
	DuplexLink
}

impl<S> Worker<S> where S: IpcInterface<S> {
	pub fn new(service: Arc<S>) -> Worker<S> {
		Worker::<S> {
			service: service.clone(),
			sockets: Vec::new(),
			method_buf: [0,0]
		}
	}

	pub fn poll(&mut self) {
		for item in self.sockets.iter_mut() {
			let socket = &mut item.0;
			// non-blocking read only ok if there is something to read from socket
			match socket.nb_read(&mut self.method_buf) {
				Ok(method_sign_len) => {
					if method_sign_len == 2 {
						let result = self.service.dispatch_buf(
							self.method_buf[1] as u16 * 256 + self.method_buf[0] as u16,
							socket);
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
					warn!(target: "ipc", "Error polling connection {:?}", x);
					panic!();
				}
			}
		}
	}

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
		Ok(())
	}
}

#[cfg(test)]
mod tests {

	use super::Worker;
	use ipc::*;
	use std::io::{Read, Write};
	use std::sync::{Arc, RwLock};
	use nanomsg::{Socket, Protocol};
	use std::thread;

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
		fn dispatch_buf<R>(&self, method_num: u16, r: &mut R) -> Vec<u8> where R: Read {
			let mut buf = vec![0u8; 4096];
			let size = r.read_to_end(&mut buf).unwrap();
			self.methods_stack.write().unwrap().push(
				TestInvoke {
					method_num: method_num,
					params: unsafe { Vec::from_raw_parts(buf.as_mut_ptr(), size, size) }
				});
			vec![]
		}
	}

	fn dummy_write(addr: &str, buf: &[u8]) {
		let mut socket = Socket::new(Protocol::Pair).unwrap();
		let endpoint = socket.connect(addr).unwrap();
		thread::sleep_ms(10);
		socket.write_all(buf).unwrap();
	}

	#[test]
	fn can_create_worker() {
		let worker = Worker::<DummyService>::new(Arc::new(DummyService::new()));
		assert_eq!(0, worker.sockets.len());
	}

	#[test]
	fn can_add_duplex_socket_to_worker() {
		let mut worker = Worker::<DummyService>::new(Arc::new(DummyService::new()));
		worker.add_duplex("ipc:///tmp/parity-test10.ipc").unwrap();
		assert_eq!(1, worker.sockets.len());
	}

	#[test]
	fn worker_can_poll_empty() {
		let service = Arc::new(DummyService::new());
		let mut worker = Worker::<DummyService>::new(service.clone());
		worker.add_duplex("ipc:///tmp/parity-test20.ipc").unwrap();
		worker.poll();
		assert_eq!(0, service.methods_stack.read().unwrap().len());
	}

	#[test]
	fn worker_can_poll() {
		let url = "ipc:///tmp/parity-test30.ipc";

		let mut worker = Worker::<DummyService>::new(Arc::new(DummyService::new()));
		worker.add_duplex(url).unwrap();

		dummy_write(url, &vec![0, 0, 7, 7, 6, 6]);
		worker.poll();

		assert_eq!(1, worker.service.methods_stack.read().unwrap().len());
	}
}
