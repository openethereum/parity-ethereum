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

// while not included in binary
#![allow(dead_code)]
#![cfg_attr(feature="dev", allow(used_underscore_binding))]

pub mod service;

/// Default value for hypervisor ipc listener
pub const HYPERVISOR_IPC_URL: &'static str = "ipc:///tmp/parity-internal-hyper-status.ipc";

use nanoipc;
use std::sync::{Arc,RwLock};
use hypervisor::service::*;
use std::process::{Command,Child};
use std::collections::HashMap;

type BinaryId = &'static str;

const BLOCKCHAIN_DB_BINARY: BinaryId = "blockchain";

pub struct Hypervisor {
	ipc_addr: String,
	service: Arc<HypervisorService>,
	ipc_worker: RwLock<nanoipc::Worker<HypervisorService>>,
	processes: RwLock<HashMap<BinaryId, Child>>,
}

impl Default for Hypervisor {
	fn default() -> Self {
		Hypervisor::new()
	}
}

impl Hypervisor {
	/// initializes the Hypervisor service with the open ipc socket for incoming clients
	pub fn new() -> Hypervisor {
		Hypervisor::with_url(HYPERVISOR_IPC_URL)
	}

	/// Starts on the specified address for ipc listener
	fn with_url(addr: &str) -> Hypervisor{
		Hypervisor::with_url_and_service(addr, HypervisorService::new())
	}

	/// Starts with the specified address for the ipc listener and
	/// the specified list of modules in form of created service
	fn with_url_and_service(addr: &str, service: Arc<HypervisorService>) -> Hypervisor {
		let worker = nanoipc::Worker::new(&service);
		Hypervisor{
			ipc_addr: addr.to_owned(),
			service: service,
			ipc_worker: RwLock::new(worker),
			processes: RwLock::new(HashMap::new()),
		}
	}

	/// Since one binary can host multiple modules
	/// we match binaries
	fn match_module(module_id: &IpcModuleId) -> Option<BinaryId> {
		match *module_id {
			BLOCKCHAIN_MODULE_ID => Some(BLOCKCHAIN_DB_BINARY),
			// none means the module is inside the main binary
			_ => None
		}
	}

	/// Creates IPC listener and starts all binaries
	fn start(&self) {
		let mut worker = self.ipc_worker.write().unwrap();
		worker.add_reqrep(&self.ipc_addr).unwrap_or_else(|e| panic!("Hypervisor ipc worker can not start - critical! ({:?})", e));

		for module_id in self.service.module_ids() {
			self.start_module(module_id);
		}
	}

	/// Start binary for the specified module
	/// Does nothing when it is already started on module is inside the
	/// main binary
	fn start_module(&self, module_id: IpcModuleId) {
		Self::match_module(&module_id).map(|binary_id| {
			let mut processes = self.processes.write().unwrap();
			{
				if processes.get(binary_id).is_some() {
					// already started for another module
					return;
				}
			}
			let child = Command::new(binary_id).spawn().unwrap_or_else(
				|e| panic!("Hypervisor cannot start binary: {}", e));
			processes.insert(binary_id, child);
		});
	}

	/// Reports if all modules are checked in
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::atomic::{AtomicBool,Ordering};
	use std::sync::Arc;
	use super::service::*;
	use nanoipc;

	#[test]
	fn can_init() {
		let url = "ipc:///tmp/test-parity-hypervisor-10.ipc";
		let test_module_id = 8080u64;

		let hypervisor = Hypervisor::with_url_and_service(url, HypervisorService::with_modules(vec![test_module_id]));
		assert_eq!(false, hypervisor.modules_ready());
	}

	#[test]
	fn can_wait_for_startup() {
		let url = "ipc:///tmp/test-parity-hypervisor-20.ipc";
		let test_module_id = 8080u64;

		let hypervisor_ready = Arc::new(AtomicBool::new(false));
		let hypervisor_ready_local = hypervisor_ready.clone();

		::std::thread::spawn(move || {
			while !hypervisor_ready.load(Ordering::Relaxed) { }

			let client = nanoipc::init_client::<HypervisorServiceClient<_>>(url).unwrap();
			client.handshake().unwrap();
			client.module_ready(test_module_id);
		});

		let hypervisor = Hypervisor::with_url_and_service(url, HypervisorService::with_modules(vec![test_module_id]));
		hypervisor.start();
		hypervisor_ready_local.store(true, Ordering::Relaxed);
		hypervisor.wait_for_startup();

		assert_eq!(true, hypervisor.modules_ready());
	}
}
