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

	fn init_worker(addr: &str) -> nanoipc::Worker<Service> {
		let mut worker = nanoipc::Worker::<Service>::new(Arc::new(Service::new()));
		worker.add_duplex(addr).unwrap();
		worker
	}

	#[test]
	fn can_create_client() {
		let client = nanoipc::init_client::<ServiceClient<_>>("ipc:///tmp/parity-examples-test10.ipc");
		assert!(client.is_ok());
	}

	#[test]
	fn can_call_handshake() {
		let exit = Arc::new(::std::sync::atomic::AtomicBool::new(false));
		let url = "ipc:///tmp/parity-test-examples-20.ipc";

		let worker_exit = exit.clone();
		::std::thread::spawn(move || {
			let mut worker = init_worker(url);
    		while !worker_exit.load(::std::sync::atomic::Ordering::Relaxed) { worker.poll() }
		});
		let client = nanoipc::init_client::<ServiceClient<_>>(url).unwrap();

		let hs = client.handshake();

		exit.store(true, ::std::sync::atomic::Ordering::Relaxed);
		assert!(hs.is_ok());
	}

}
