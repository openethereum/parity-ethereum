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

use ethcore_stratum::{JobDispatcher, RemoteWorkHandler};
use miner::miner::Miner;
use std::sync::{Weak, Arc};
use nanoipc::{NanoSocket, GuardedSocket};
use std::sync::atomic::Ordering;

pub struct StratumJobDispatcher {
	miner: Weak<Miner>,
}

impl JobDispatcher for StratumJobDispatcher {
}

pub struct Stratum {
	dispatcher: Arc<StratumJobDispatcher>,
	push_Handler: GuardedSocket<RemoteWorkHandler<NanoSocket>>,
	base_dir: String,
	stop: ::devtools::StopGuard,
}

impl Stratum {
	fn new(miner: Arc<Miner>, base_dir: &str) -> Stratum {
		let handler_url = format!("ipc://{}/stratum-job-handler.ipc", base_dir);
		Stratum {
			dispatcher: StratumJobDispatcher::new(miner),
			push_Handler: nanoipc::init_client::<RemoteWorkHandler<_>>(),
			base_dir: base_dir.to_owned(),
			stop: ::devtools::StopGuard::new(),
		}
	}

	fn run_async(&self) {
		let socket_url = format!("ipc://{}/stratum-job-dispatcher.ipc", &self.base_dir);
		let stop = self.stop.share();
		let service = self.dispatcher.clone() as Arc<JobDispatcher>;
		std::thread::spawn(move || {
			let mut worker = nanoipc::Worker::<T>::new(&service);
			worker.add_reqrep(&socket_url).unwrap();

			while !stop.load(Ordering::Relaxed) {
				worker.poll();
			}
		});
	}
}
