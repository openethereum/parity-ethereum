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
use std::sync::Arc;
use nanoipc::{NanoSocket, GuardedSocket};
use std::sync::atomic::Ordering;
use std::thread;
use nanoipc;
use util::{H256, U256, FixedHash};
use ethereum::ethash::Ethash;
use ethash::SeedHashCompute;
use util::{Mutex, RwLock};

pub struct StratumJobDispatcher {
	last_work: RwLock<Option<(H256, U256, u64)>>,
	seed_compute: Mutex<SeedHashCompute>,
}

impl JobDispatcher for StratumJobDispatcher {
	fn initial(&self) -> Option<String> {
		self.last_work.read().map(|(pow_hash, difficulty, number)| {
			self.payload(pow_hash, difficulty, number)
		})
	}
}

impl StratumJobDispatcher {
	fn new() -> StratumJobDispatcher {
		StratumJobDispatcher {
			seed_compute: Mutex::new(SeedHashCompute::new()),
			last_work: RwLock::new(None),
		}
	}

	fn payload(&self, pow_hash: H256, difficulty: U256, number: u64) -> String {
		// TODO: move this to engine
		let target = Ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().get_seedhash(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		format!(
			r#"{{ "result": ["0x{}","0x{}","0x{}","0x{:x}"] }}"#,
			pow_hash.hex(), seed_hash.hex(), target.hex(), number
		)
	}
}

pub struct Stratum {
	dispatcher: Arc<StratumJobDispatcher>,
	base_dir: String,
	stop: ::devtools::StopGuard,
}

pub enum Error {
	Nano(nanoipc::SocketError),
}

impl From<nanoipc::SocketError> for Error {
	fn from(socket_err: nanoipc::SocketError) -> Error { Error::Nano(socket_err) }
}

impl super::work_notify::NotifyWork for Stratum {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		let client = nanoipc::init_client::<RemoteWorkHandler<_>>(
			&format!("ipc://{}/stratum-job-handler.ipc", self.base_dir)
		).unwrap_or_else(|e|
			warn!(target: "stratum", "Unable to push work for stratum service: {:?}", e)
		);
		client.push_work_all(
			self.dispatcher.payload(pow_hash, difficulty, number)
		).unwrap_or_else(
			|e| warn!(target: "stratum", "Error while pushing work: {:?}", e)
		);
		*self.dispatcher.last_work.write() = Some((pow_hash, difficulty, number))
	}
}

impl Stratum {
	pub fn new(base_dir: &str) -> Result<Stratum, Error> {
		Ok(Stratum {
			dispatcher: Arc::new(StratumJobDispatcher::new()),
			base_dir: base_dir.to_owned(),
			stop: ::devtools::StopGuard::new(),
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
