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
use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use request::{CompleteRequest, Kind};

use bincode;
use parking_lot::{RwLock, Mutex};

/// Number of time periods samples should be kept for.
pub const MOVING_SAMPLE_SIZE: usize = 256;

/// Stores rolling load timer samples.
// TODO: switch to bigint if possible (FP casts aren't available)
pub trait SampleStore: Send + Sync {
	/// Load samples.
	fn load(&self) -> HashMap<Kind, VecDeque<u64>>;

	/// Store all samples.
	fn store(&self, samples: &HashMap<Kind, VecDeque<u64>>);
}

// get a hardcoded, arbitrarily determined (but intended overestimate)
// of the time it takes to serve a request of the given kind.
//
// TODO: seed this with empirical data.
fn hardcoded_serve_time(kind: Kind) -> Duration {
	Duration::new(0, match kind {
		Kind::Headers => 500_000,
		Kind::HeaderProof => 500_000,
		Kind::TransactionIndex => 500_000,
		Kind::Receipts => 1_000_000,
		Kind::Body => 1_000_000,
		Kind::Account => 1_500_000,
		Kind::Storage => 2_000_000,
		Kind::Code => 1_500_000,
		Kind::Execution => 250, // per gas.
		Kind::Signal => 500_000,
	})
}

/// A no-op store.
pub struct NullStore;

impl SampleStore for NullStore {
	fn load(&self) -> HashMap<Kind, VecDeque<u64>> { HashMap::new() }
	fn store(&self, _samples: &HashMap<Kind, VecDeque<u64>>) { }
}

/// Request load distributions.
pub struct LoadDistribution {
	active_period: RwLock<HashMap<Kind, Mutex<(u64, u64)>>>,
	samples: RwLock<HashMap<Kind, VecDeque<u64>>>,
}

impl LoadDistribution {
	/// Load rolling samples from the given store.
	pub fn load(store: &dyn SampleStore) -> Self {
		let mut samples = store.load();

		for kind_samples in samples.values_mut() {
			while kind_samples.len() > MOVING_SAMPLE_SIZE {
				kind_samples.pop_front();
			}
		}

		LoadDistribution {
			active_period: RwLock::new(HashMap::new()),
			samples: RwLock::new(samples),
		}
	}

	/// Begin a timer.
	pub fn begin_timer<'a>(&'a self, req: &CompleteRequest) -> LoadTimer<'a> {
		let kind = req.kind();
		let n = match *req {
			CompleteRequest::Headers(ref req) => req.max,
			CompleteRequest::Execution(ref req) => req.gas.low_u64(),
			_ => 1,
		};

		LoadTimer {
			start: Instant::now(),
			n,
			dist: self,
			kind,
		}
	}

	/// Calculate EMA of load for a specific request kind.
	/// If there is no data for the given request kind, no EMA will be calculated,
	/// but a hardcoded time will be returned.
	pub fn expected_time(&self, kind: Kind) -> Duration {
		let samples = self.samples.read();
		samples.get(&kind).and_then(|s| {
			if s.is_empty() { return None }

			let alpha: f64 = 1_f64 / s.len() as f64;
			let start = *s.front().expect("length known to be non-zero; qed") as f64;
			let ema = s.iter().skip(1).fold(start, |a, &c| {
				(alpha * c as f64) + ((1.0 - alpha) * a)
			});

			Some(Duration::from_nanos(ema as u64))
		}).unwrap_or_else(move || hardcoded_serve_time(kind))
	}

	/// End the current time period. Provide a store to
	pub fn end_period(&self, store: &dyn SampleStore) {
		let active_period = self.active_period.read();
		let mut samples = self.samples.write();

		for (&kind, set) in active_period.iter() {
			let (elapsed, n) = ::std::mem::replace(&mut *set.lock(), (0, 0));
			if n == 0 { continue }

			let kind_samples = samples.entry(kind)
				.or_insert_with(|| VecDeque::with_capacity(MOVING_SAMPLE_SIZE));

			if kind_samples.len() == MOVING_SAMPLE_SIZE { kind_samples.pop_front(); }
			kind_samples.push_back(elapsed / n);
		}

		store.store(&*samples);
	}

	fn update(&self, kind: Kind, elapsed: Duration, n: u64) {
		macro_rules! update_counters {
			($counters: expr) => {
				$counters.0 = $counters.0.saturating_add({ elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64 });
				$counters.1 = $counters.1.saturating_add(n);
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
	start: Instant,
	n: u64,
	dist: &'a LoadDistribution,
	kind: Kind,
}

impl<'a> Drop for LoadTimer<'a> {
	fn drop(&mut self) {
		let elapsed = self.start.elapsed();
		self.dist.update(self.kind, elapsed, self.n);
	}
}

/// A store which writes directly to a file.
pub struct FileStore(pub PathBuf);

impl SampleStore for FileStore {
	fn load(&self) -> HashMap<Kind, VecDeque<u64>> {
		File::open(&self.0)
			.map_err(|e| Box::new(bincode::ErrorKind::Io(e)))
			.and_then(|mut file| bincode::deserialize_from(&mut file))
			.unwrap_or_else(|_| HashMap::new())
	}

	fn store(&self, samples: &HashMap<Kind, VecDeque<u64>>) {
		let res = File::create(&self.0)
			.map_err(|e| Box::new(bincode::ErrorKind::Io(e)))
			.and_then(|mut file| bincode::serialize_into(&mut file, samples));

		if let Err(e) = res {
			warn!(target: "pip", "Error writing light request timing samples to file: {}", e);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use request::Kind;

	#[test]
	fn hardcoded_before_data() {
		let dist = LoadDistribution::load(&NullStore);
		assert_eq!(dist.expected_time(Kind::Headers), hardcoded_serve_time(Kind::Headers));

		dist.update(Kind::Headers, Duration::new(0, 100_000), 100);
		dist.end_period(&NullStore);

		assert_eq!(dist.expected_time(Kind::Headers), Duration::new(0, 1000));
	}

	#[test]
	fn moving_average() {
		let dist = LoadDistribution::load(&NullStore);

		let mut sum = 0;

		for (i, x) in (0..10).map(|x| x * 10_000).enumerate() {
			dist.update(Kind::Headers, Duration::new(0, x), 1);
			dist.end_period(&NullStore);

			sum += x;
			if i == 0 { continue }

			let moving_average = dist.expected_time(Kind::Headers);

			// should be weighted below the maximum entry.
			let arith_average = (sum as f64 / (i + 1) as f64) as u32;
			assert!(moving_average < Duration::new(0, x));

			// when there are only 2 entries, they should be equal due to choice of
			// ALPHA = 1/N.
			// otherwise, the weight should be below the arithmetic mean because the much
			// smaller previous values are discounted less.
			if i == 1 {
				assert_eq!(moving_average, Duration::new(0, arith_average));
			} else {
				assert!(moving_average < Duration::new(0, arith_average))
			}
		}
	}

	#[test]
	fn file_store() {
		let tempdir = ::tempdir::TempDir::new("").unwrap();
		let path = tempdir.path().join("file");
		let store = FileStore(path);

		let mut samples = store.load();
		assert!(samples.is_empty());
		samples.insert(Kind::Headers, vec![5, 2, 7, 2, 2, 4].into());
		samples.insert(Kind::Execution, vec![1, 1, 100, 250].into());

		store.store(&samples);

		let dup = store.load();

		assert_eq!(samples, dup);
	}
}
