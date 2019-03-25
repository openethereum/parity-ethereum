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


use std::sync::Weak;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread;
use std::sync::Arc;

use engines::Engine;
use machine::Machine;

/// Service that is managing the engine
pub struct StepService {
	shutdown: Arc<AtomicBool>,
	thread: Option<thread::JoinHandle<()>>,
}

impl StepService {
	/// Start the `StepService`
	pub fn start<M: Machine + 'static>(engine: Weak<Engine<M>>) -> Arc<Self> {
		let shutdown = Arc::new(AtomicBool::new(false));
		let s = shutdown.clone();

		let thread = thread::Builder::new()
			.name("CliqueStepService".into())
			.spawn(move || {
				// startup delay.
				thread::sleep(Duration::from_secs(5));

				loop {
					// see if we are in shutdown.
					if shutdown.load(Ordering::Acquire) {
							trace!(target: "miner", "CliqueStepService: received shutdown signal!");
							break;
					}

					trace!(target: "miner", "CliqueStepService: triggering sealing");

					// Try sealing
					engine.upgrade().map(|x| x.step());

					// Yield
					thread::sleep(Duration::from_millis(2000));
				}
				trace!(target: "miner", "CliqueStepService: shutdown.");
			}).expect("CliqueStepService thread failed");

		Arc::new(StepService {
			shutdown: s,
			thread: Some(thread),
		})
	}

	/// Stop the `StepService`
	pub fn stop(&mut self) {
		trace!(target: "miner", "CliqueStepService: shutting down.");
		self.shutdown.store(true, Ordering::Release);
		if let Some(t) = self.thread.take() {
			t.join().expect("CliqueStepService thread panicked!");
		}
	}
}
