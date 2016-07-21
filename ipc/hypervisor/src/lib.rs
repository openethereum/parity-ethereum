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

#![cfg_attr(feature="dev", allow(used_underscore_binding))]

extern crate ethcore_ipc as ipc;
extern crate ethcore_ipc_nano as nanoipc;
extern crate semver;
#[macro_use] extern crate log;

pub mod service;

/// Default value for hypervisor ipc listener
pub const HYPERVISOR_IPC_URL: &'static str = "ipc:///tmp/parity-internal-hyper-status.ipc";

use std::sync::{Arc,RwLock};
use service::{HypervisorService, IpcModuleId};
use std::process::{Command,Child};
use std::collections::HashMap;

pub use service::{HypervisorServiceClient, CLIENT_MODULE_ID, SYNC_MODULE_ID};

pub type BinaryId = &'static str;

pub struct Hypervisor {
	ipc_addr: String,
	service: Arc<HypervisorService>,
	ipc_worker: RwLock<nanoipc::Worker<HypervisorService>>,
	processes: RwLock<HashMap<BinaryId, Child>>,
	modules: HashMap<IpcModuleId, (BinaryId, BootArgs)>,
}

/// Boot arguments for binary
pub struct BootArgs {
	cli: Option<Vec<String>>,
	stdin: Option<Vec<u8>>,
}

impl BootArgs {
	/// New empty boot arguments
	pub fn new() -> BootArgs {
		BootArgs {
			cli: None,
			stdin: None,
		}
	}

	/// Set command-line arguments for boot
	pub fn cli(mut self, cli: Vec<String>) -> BootArgs {
		self.cli = Some(cli);
		self
	}

	/// Set std-in stream for boot
	pub fn stdin(mut self, stdin: Vec<u8>) -> BootArgs {
		self.stdin = Some(stdin);
		self
	}
}

impl Hypervisor {
	/// initializes the Hypervisor service with the open ipc socket for incoming clients
	pub fn new() -> Hypervisor {
		Hypervisor::with_url(HYPERVISOR_IPC_URL)
	}

	pub fn module(mut self, module_id: IpcModuleId, binary_id: BinaryId, args: BootArgs) -> Hypervisor {
		self.modules.insert(module_id, (binary_id, args));
		self.service.add_module(module_id);
		self
	}

	pub fn local_module(self, module_id: IpcModuleId) -> Hypervisor {
		self.service.add_module(module_id);
		self
	}

	/// Starts with the specified address for the ipc listener and
	/// the specified list of modules in form of created service
	pub fn with_url(addr: &str) -> Hypervisor {
		let service = HypervisorService::new();
		let worker = nanoipc::Worker::new(&service);
		Hypervisor{
			ipc_addr: addr.to_owned(),
			service: service,
			ipc_worker: RwLock::new(worker),
			processes: RwLock::new(HashMap::new()),
			modules: HashMap::new(),
		}
	}

	/// Since one binary can host multiple modules
	/// we match binaries
	fn match_module(&self, module_id: &IpcModuleId) -> Option<&(BinaryId, BootArgs)> {
		self.modules.get(module_id)
	}

	/// Creates IPC listener and starts all binaries
	pub fn start(&self) {
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
		use std::io::Write;

		self.match_module(&module_id).map(|&(ref binary_id, ref binary_args)| {
			let mut processes = self.processes.write().unwrap();
			{
				if processes.get(binary_id).is_some() {
					// already started for another module
					return;
				}
			}

			let mut executable_path = std::env::current_exe().unwrap();
			executable_path.pop();
			executable_path.push(binary_id);

			let executable_path = executable_path.to_str().unwrap();
			let mut command = Command::new(&executable_path);
			command.stderr(std::process::Stdio::inherit());

			if let Some(ref cli_args) = binary_args.cli {
				for arg in cli_args { command.arg(arg); }
			}

			command.stdin(std::process::Stdio::piped());

			trace!(target: "hypervisor", "Spawn executable: {:?}", command);

			let mut child = command.spawn().unwrap_or_else(
				|e| panic!("Hypervisor cannot start binary ({:?}): {}", executable_path, e));

			if let Some(ref std_in) = binary_args.stdin {
				trace!(target: "hypervisor", "Pushing std-in payload...");
				child.stdin.as_mut()
					.expect("std-in should be piped above")
					.write(std_in)
					.unwrap_or_else(|e| panic!(format!("Error trying to pipe stdin for {}: {:?}", &executable_path, e)));
				drop(child.stdin.take());
			}

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

	/// Shutdown the ipc and all managed child processes
	pub fn shutdown(&self, wait_time: Option<std::time::Duration>) {
		if wait_time.is_some() { std::thread::sleep(wait_time.unwrap()) }

		let mut childs = self.processes.write().unwrap();
		for (ref mut binary, ref mut child) in childs.iter_mut() {
			trace!(target: "hypervisor", "Stopping process module: {}", binary);
			child.kill().unwrap();
		}
	}
}

impl Drop for Hypervisor {
	fn drop(&mut self) {
		self.shutdown(Some(std::time::Duration::new(1, 0)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::atomic::{AtomicBool,Ordering};
	use std::sync::Arc;
	use nanoipc;

	#[test]
	fn can_init() {
		let url = "ipc:///tmp/test-parity-hypervisor-10.ipc";
		let test_module_id = 8080u64;

		let hypervisor = Hypervisor::with_url(url).local_module(test_module_id);
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

		let hypervisor = Hypervisor::with_url(url).local_module(test_module_id);
		hypervisor.start();
		hypervisor_ready_local.store(true, Ordering::Relaxed);
		hypervisor.wait_for_startup();

		assert_eq!(true, hypervisor.modules_ready());
	}
}
