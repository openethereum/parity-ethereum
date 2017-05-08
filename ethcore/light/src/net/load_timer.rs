// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Request load timer and distribution manager.
//!
//! This uses empirical samples of the length of time taken to respond
//! to requests in order to inform request credit costs.
//!
//! The average request time is determined by an exponential moving average
//! of the mean request times during the last `MOVING_SAMPLE_SIZE` time periods of
//! length `TIME_PERIOD_MS`, with the exception that time periods where no data is
//! gathered are excluded.

use std::collections::{HashMap, VecDeque};

use request::{CompleteRequest, Kind};

use time;
use util::{U256, Uint, RwLock, Mutex};

/// Number of time periods samples should be kept for.
pub const MOVING_SAMPLE_SIZE: usize = 256;
/// Length of time periods.
pub const TIME_PERIOD_MS: isize = 60 * 60 * 1000;

/// Something which stores load timer samples.
pub trait SampleStore {
	/// Load the samples for a given request kind.
	fn load(&self, kind: Kind) -> VecDeque<u64>;

	/// Store the samples for a given request kind.
	fn store(&self, kind: Kind, items: &VecDeque<u64>);
}

/// Request load distributions.
pub struct LoadDistribution {
	active_period: RwLock<HashMap<Kind, Mutex<(u64, u64)>>>,
	samples: RwLock<HashMap<Kind, VecDeque<u64>>>,
}

impl LoadDistribution {
	/// Load rolling samples from the given store.
	pub fn load(store: &SampleStore) -> Self {
		let mut samples = HashMap::new();
		{
			let mut load_for_kind = |kind| {
				let mut kind_samples = store.load(kind);
				while kind_samples.len() > MOVING_SAMPLE_SIZE {
					kind_samples.pop_front();
				}

				samples.insert(kind, kind_samples);
			};

			load_for_kind(Kind::Headers);
			load_for_kind(Kind::HeaderProof);
			load_for_kind(Kind::Receipts);
			load_for_kind(Kind::Body);
			load_for_kind(Kind::Account);
			load_for_kind(Kind::Storage);
			load_for_kind(Kind::Code);
			load_for_kind(Kind::Execution);
		}

		LoadDistribution {
			active_period: RwLock::new(HashMap::new()),
			samples: RwLock::new(samples),
		}
	}

	/// Begin a timer.
	pub fn begin<'a>(&'a self, req: &CompleteRequest) -> LoadTimer<'a> {
		let kind = req.kind();
		let n = match *req {
			CompleteRequest::Headers(ref req) => req.max,
			CompleteRequest::Execution(ref req) => req.gas.low_u64(),
			_ => 1,
		};

		LoadTimer {
			start: time::precise_time_ns(),
			n: n,
			dist: self,
			kind: kind,
		}
	}

	/// Calculate EMA of load in nanoseconds for a specific request kind.
	/// If there is no data for the given request kind, no EMA will be calculated.
	pub fn moving_average(&self, kind: Kind) -> Option<u64> {
		let samples = self.samples.read();
		samples.get(&kind).and_then(|s| {
			if s.len() == 0 { return None }

			let alpha: f64 = 1f64 / s.len() as f64;
			let start = s.front().expect("length known to be non-zero; qed").clone();
			let ema = s.iter().skip(1).fold(start as f64, |a, &c| {
				(alpha * c as f64) + ((1.0 - alpha) * a)
			});

			Some(ema as u64)
		})
	}

	/// End the current time period. Provide a store to
	pub fn end_period(&self, store: &SampleStore) {
		let active_period = self.active_period.read();
		let mut samples = self.samples.write();

		for (&kind, set) in active_period.iter() {
			let (elapsed, n) = ::std::mem::replace(&mut *set.lock(), (0, 0));
			if n == 0 { continue }

			let kind_samples = samples.entry(kind)
				.or_insert_with(|| VecDeque::with_capacity(MOVING_SAMPLE_SIZE));

			if kind_samples.len() == MOVING_SAMPLE_SIZE { kind_samples.pop_front(); }
			kind_samples.push_back(elapsed / n);

			store.store(kind, &kind_samples);
		}
	}

	fn update(&self, kind: Kind, elapsed: u64, n: u64) {
		macro_rules! update_counters {
			($counters: expr) => {
				$counters.0 += elapsed;
				$counters.1 += n;
			}
		};

		{
			let set = self.active_period.read();
			if let Some(counters) = set.get(&kind) {
				let mut counters = counters.lock();
				update_counters!(counters);
				return;
			}
		}

		let mut set = self.active_period.write();
		let counters = set
			.entry(kind)
			.or_insert_with(|| Mutex::new((0, 0)));

		update_counters!(counters.get_mut());
	}
}

/// A timer for a single request.
/// On drop, this will update the distribution.
pub struct LoadTimer<'a> {
	start: u64,
	n: u64,
	dist: &'a LoadDistribution,
	kind: Kind,
}

impl<'a> Drop for LoadTimer<'a> {
	fn drop(&mut self) {
		let elapsed = time::precise_time_ns() - self.start;
		self.dist.update(self.kind, elapsed, self.n);
	}
}
