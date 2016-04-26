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

#[cfg(test)]
mod tests {

	use super::super::service::*;
	use nanoipc;
	use std::sync::Arc;
	use std::io::Write;
	use std::sync::atomic::{Ordering, AtomicBool};

	fn dummy_write(addr: &str, buf: &[u8]) -> (::nanomsg::Socket, ::nanomsg::Endpoint) {
		let mut socket = ::nanomsg::Socket::new(::nanomsg::Protocol::Pair).unwrap();
		let endpoint = socket.connect(addr).unwrap();
		socket.write(buf).unwrap();
		(socket, endpoint)
	}


	fn init_worker(addr: &str) -> nanoipc::Worker<Service> {
		let mut worker = nanoipc::Worker::<Service>::new(&Arc::new(Service::new()));
		worker.add_duplex(addr).unwrap();
		worker
	}

	#[test]
	fn can_create_client() {
		let client = nanoipc::init_duplex_client::<ServiceClient<_>>("ipc:///tmp/parity-nano-test10.ipc");
		assert!(client.is_ok());
	}

	#[test]
	fn can_call_handshake() {
		let url = "ipc:///tmp/parity-test-nano-20.ipc";
		let worker_should_exit = Arc::new(AtomicBool::new(false));
		let worker_is_ready = Arc::new(AtomicBool::new(false));
		let c_worker_should_exit = worker_should_exit.clone();
		let c_worker_is_ready = worker_is_ready.clone();

		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !c_worker_should_exit.load(Ordering::Relaxed) {
				worker.poll();
				c_worker_is_ready.store(true, Ordering::Relaxed);
			}
		});

		while !worker_is_ready.load(Ordering::Relaxed) { }
		let client = nanoipc::init_duplex_client::<ServiceClient<_>>(url).unwrap();

		let hs = client.handshake();

		worker_should_exit.store(true, Ordering::Relaxed);
		assert!(hs.is_ok());
	}
}
