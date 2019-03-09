// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use engines::Engine;
use parity_machine::Machine;
use std::sync::Weak;
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct StepService {
	shutdown: Arc<RwLock<bool>>,
	thread: Option<thread::JoinHandle<()>>,
}

impl StepService {
	pub fn start<M: Machine + 'static>(engine: Weak<Engine<M>>) -> Arc<Self> {
		let shutdown = Arc::new(RwLock::new(false));
		let shutdown1 = shutdown.clone();

		let thread = thread::Builder::new()
			.name("CliqueMiner".into())
			.spawn(move || {
				// startup delay.
				thread::sleep(Duration::from_secs(5));

				loop {
					// see if we are in shutdown.
					if *shutdown.read() == true {
							trace!(target:"miner", "StepService: received shutdown signal!");
							break;
					}

					trace!(target: "miner", "StepService: triggering sealing");

					// Try sealing
					engine.upgrade().map(|x| x.step());

					// Yield
					thread::sleep(Duration::from_millis(2000));
				}
				trace!(target: "miner", "StepService: shut down.");
			}).expect("Unable to launch thread.");

		Arc::new(StepService {
			shutdown: shutdown1,
			thread: Some(thread),
		})
	}

	pub fn stop(&mut self) {
		trace!(target: "miner", "StepService: shutting down.");
		*self.shutdown.write() = true;
		if let Some(t) = self.thread.take() {
			t.join().expect("CliqueMiner thread panicked!");
		}
	}
}
