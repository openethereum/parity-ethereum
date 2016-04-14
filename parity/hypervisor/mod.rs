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

//! Parity interprocess hypervisor module

pub mod service;

pub const HYPERVISOR_IPC_URL: &'static str = "ipc:///tmp/parity-internal-hyper-status.ipc";

use nanoipc;
use std::sync::{Arc,RwLock};
use hypervisor::service::*;
use ipc::IpcInterface;

pub struct Hypervisor {
	service: Arc<HypervisorService>,
	ipc_worker: RwLock<nanoipc::Worker<HypervisorService>>,
}

impl Hypervisor {
	/// initializes the Hypervisor service with the open ipc socket for incoming clients
	pub fn init() -> Arc<Hypervisor>{
		Hypervisor::with_url(HYPERVISOR_IPC_URL)
	}

	fn with_url(addr: &str) -> Arc<Hypervisor>{
		Hypervisor::with_url_and_service(addr, HypervisorService::new())
	}

	fn with_url_and_service(addr: &str, service: Arc<HypervisorService>) -> Arc<Hypervisor> {
		let mut worker = nanoipc::Worker::new(&service);
		worker.add_reqrep(addr);

		Arc::new(Hypervisor{
			service: service,
			ipc_worker: RwLock::new(worker),
		})
	}

	pub fn modules_ready(&self) -> bool {
		self.service.unchecked_count() == 0
	}

	/// Waits for every required module to check in
	pub fn wait_for_startup(&self) {
		let mut worker = self.ipc_worker.write().unwrap();
		while !self.modules_ready() {
			worker.poll()
		}
	}
}

mod tests {
	use super::*;
	use std::sync::atomic::{AtomicBool,Ordering};
	use std::sync::Arc;
	use super::service::*;
	use nanoipc;

	#[test]
	fn can_init() {
		let url = "ipc:///tmp/test-parity-hypervisor-10";

		let hypervisor = Hypervisor::with_url(url);
		assert_eq!(false, hypervisor.modules_ready());
	}

	#[test]
	fn can_wait_for_startup() {
		let url = "ipc:///tmp/test-parity-hypervisor-20";
		let test_module_id = 8080u64;

		let hypervisor_ready = Arc::new(AtomicBool::new(false));
		let hypervisor_ready_local = hypervisor_ready.clone();

		::std::thread::spawn(move || {
			while !hypervisor_ready.load(Ordering::Relaxed) { }

			let client = nanoipc::init_client::<HypervisorServiceClient<_>>(url).unwrap();
			client.module_ready(test_module_id);
		});

		let hypervisor = Hypervisor::with_url_and_service(url, HypervisorService::with_modules(vec![test_module_id]));
		hypervisor_ready_local.store(true, Ordering::Relaxed);
		hypervisor.wait_for_startup();

		assert_eq!(true, hypervisor.modules_ready());
	}
}
