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

use ethcore_stratum::{JobDispatcher, RemoteWorkHandler, PushWorkHandler};
use miner::miner::Miner;
use std::sync::{Weak, Arc};
use nanoipc::{NanoSocket, GuardedSocket};
use std::sync::atomic::Ordering;
use std::thread;
use nanoipc;
use util::{H256, U256, FixedHash};
use ethereum::ethash::Ethash;
use ethash::SeedHashCompute;
use util::Mutex;

pub struct StratumJobDispatcher {
	miner: Weak<Miner>,
}

impl JobDispatcher for StratumJobDispatcher {
}

impl StratumJobDispatcher {
	fn new(miner: Arc<Miner>) -> StratumJobDispatcher {
		StratumJobDispatcher { miner: Arc::downgrade(&miner), }
	}
}

pub struct Stratum {
	dispatcher: Arc<StratumJobDispatcher>,
	push_handler: GuardedSocket<RemoteWorkHandler<NanoSocket>>,
	base_dir: String,
	stop: ::devtools::StopGuard,
	seed_compute: Mutex<SeedHashCompute>,
}

pub enum Error {
	Nano(nanoipc::SocketError),
}

impl From<nanoipc::SocketError> for Error {
	fn from(socket_err: nanoipc::SocketError) -> Error { Error::Nano(socket_err) }
}

impl super::work_notify::NotifyWork for Stratum {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		// TODO: move this to engine
		let target = Ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().get_seedhash(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		let body = format!(
			r#"{{ "result": ["0x{}","0x{}","0x{}","0x{:x}"] }}"#,
			pow_hash.hex(), seed_hash.hex(), target.hex(), number
		);

		self.push_handler.push_work_all(body).unwrap_or_else(
			|e| warn!(target: "stratum", "Error while pushing work: {:?}", e)
		);
	}
}

impl Stratum {
	pub fn new(miner: Arc<Miner>, base_dir: &str) -> Result<Stratum, Error> {
		let handler_url = format!("ipc://{}/stratum-job-handler.ipc", base_dir);
		Ok(Stratum {
			dispatcher: Arc::new(StratumJobDispatcher::new(miner)),
			push_handler: try!(nanoipc::init_client::<RemoteWorkHandler<_>>(&handler_url)),
			base_dir: base_dir.to_owned(),
			stop: ::devtools::StopGuard::new(),
			seed_compute: Mutex::new(SeedHashCompute::new()),
		})
	}

	pub fn run_async(&self) {
		let socket_url = format!("ipc://{}/stratum-job-dispatcher.ipc", &self.base_dir);
		let stop = self.stop.share();
		let service = self.dispatcher.clone() as Arc<JobDispatcher>;
		thread::spawn(move || {
			let mut worker = nanoipc::Worker::<JobDispatcher>::new(&service);
			worker.add_reqrep(&socket_url).unwrap();

			while !stop.load(Ordering::Relaxed) {
				worker.poll();
			}
		});
	}
}
