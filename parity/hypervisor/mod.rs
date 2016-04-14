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
		let service = HypervisorService::new();
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

	#[test]
	fn can_init() {
		let hypervisor = Hypervisor::with_url("ipc:///tmp/test-parity-hypervisor-10");
		assert_eq!(false, hypervisor.modules_ready());
	}

	#[test]
	fn can_wait_for_startup() {
		let hypervisor = Hypervisor::with_url("ipc:///tmp/test-parity-hypervisor-10");
		hypervisor.wait_for_startup();
		assert_eq!(false, hypervisor.modules_ready());
	}
}
