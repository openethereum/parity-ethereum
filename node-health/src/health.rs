// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Reporting node's health.

use std::sync::Arc;
use std::time::Duration;
use futures::Future;
use futures::sync::oneshot;
use types::{HealthInfo, HealthStatus, Health};
use time::{TimeChecker, MAX_DRIFT};
use parity_reactor::Remote;
use parking_lot::Mutex;
use {SyncStatus};

const TIMEOUT: Duration = Duration::from_secs(5);
const PROOF: &str = "Only one closure is invoked.";

/// A struct enabling you to query for node's health.
#[derive(Debug, Clone)]
pub struct NodeHealth {
	sync_status: Arc<SyncStatus>,
	time: TimeChecker,
	remote: Remote,
}

impl NodeHealth {
	/// Creates new `NodeHealth`.
	pub fn new(sync_status: Arc<SyncStatus>, time: TimeChecker, remote: Remote) -> Self {
		NodeHealth { sync_status, time, remote, }
	}

	/// Query latest health report.
	pub fn health(&self) -> Box<Future<Item = Health, Error = ()> + Send> {
		trace!(target: "dapps", "Checking node health.");
		// Check timediff
		let sync_status = self.sync_status.clone();
		let time = self.time.time_drift();
		let (tx, rx) = oneshot::channel();
		let tx = Arc::new(Mutex::new(Some(tx)));
		let tx2 = tx.clone();
		self.remote.spawn_with_timeout(
			move |_| time.then(move |result| {
				let _ = tx.lock().take().expect(PROOF).send(Ok(result));
				Ok(())
			}),
			TIMEOUT,
			move || {
				let _ = tx2.lock().take().expect(PROOF).send(Err(()));
			},
		);

		Box::new(rx.map_err(|err| {
			warn!(target: "dapps", "Health request cancelled: {:?}", err);
		}).and_then(move |time| {
			// Check peers
			let peers = {
				let (connected, max) = sync_status.peers();
				let (status, message) = match connected {
					0 => {
						(HealthStatus::Bad, "You are not connected to any peers. There is most likely some network issue. Fix connectivity.".into())
					},
					1 => (HealthStatus::NeedsAttention, "You are connected to only one peer. Your node might not be reliable. Check your network connection.".into()),
					_ => (HealthStatus::Ok, "".into()),
				};
				HealthInfo { status, message, details: (connected, max) }
			};

			// Check sync
			let sync = {
				let is_syncing = sync_status.is_major_importing();
				let (status, message) = if is_syncing {
					(HealthStatus::NeedsAttention, "Your node is still syncing, the values you see might be outdated. Wait until it's fully synced.".into())
				} else {
					(HealthStatus::Ok, "".into())
				};
				HealthInfo { status, message, details: is_syncing }
			};

			// Check time
			let time = {
				let (status, message, details) = match time {
					Ok(Ok(diff)) if diff < MAX_DRIFT && diff > -MAX_DRIFT => {
						(HealthStatus::Ok, "".into(), diff)
					},
					Ok(Ok(diff)) => {
						(HealthStatus::Bad, format!(
							"Your clock is not in sync. Detected difference is too big for the protocol to work: {}ms. Synchronize your clock.",
							diff,
						), diff)
					},
					Ok(Err(err)) => {
						(HealthStatus::NeedsAttention, format!(
							"Unable to reach time API: {}. Make sure that your clock is synchronized.",
							err,
						), 0)
					},
					Err(_) => {
						(HealthStatus::NeedsAttention, "Time API request timed out. Make sure that the clock is synchronized.".into(), 0)
					},
				};

				HealthInfo { status, message, details, }
			};

			Ok(Health { peers, sync, time})
		}))
	}
}
