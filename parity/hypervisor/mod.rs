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

pub struct Hypervisor {
	ipc_worker: RwLock<nanoipc::Worker<HypervisorService>>,
}

impl Hypervisor {
	/// initializes the Hypervisor service with the open ipc socket for incoming clients
	pub fn init() -> Arc<Hypervisor>{
		let mut worker = nanoipc::Worker::<HypervisorService>::new(Arc::new(Service::new()));
		worker.add_reqrep(HYPERVISOR_IPC_URL);

		Arc::new(Hypervisor{
			ipc_worker: worker
		})
	}

	/// Waits for every required module to check in
	pub fn wait_for_startup() {

	}
}
