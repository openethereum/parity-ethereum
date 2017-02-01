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

//! RPC Requests Statistics

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize};
use std::time;
use futures::Future;
use jsonrpc_core as rpc;
use order_stat;
use util::RwLock;

const RATE_SECONDS: usize = 10;
const STATS_SAMPLES: usize = 60;

struct RateCalculator {
	era: time::Instant,
	samples: [u16; RATE_SECONDS],
}

impl fmt::Debug for RateCalculator {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{} req/s", self.rate())
	}
}

impl Default for RateCalculator {
	fn default() -> Self {
		RateCalculator {
			era: time::Instant::now(),
			samples: [0; RATE_SECONDS],
		}
	}
}

impl RateCalculator {
	fn elapsed(&self) -> u64 {
		self.era.elapsed().as_secs()
	}

	pub fn tick(&mut self) -> u16 {
		if self.elapsed() >= RATE_SECONDS as u64 {
			self.era = time::Instant::now();
			self.samples[0] = 0;
		}

		let pos = self.elapsed() as usize % RATE_SECONDS;
		let next = (pos + 1) % RATE_SECONDS;
		self.samples[next] = 0;
		self.samples[pos] = self.samples[pos].saturating_add(1);
		self.samples[pos]
	}

	fn current_rate(&self) -> usize {
		let now = match self.elapsed() {
			i if i >= RATE_SECONDS as u64 => RATE_SECONDS,
			i => i as usize + 1,
		};
		let sum: usize = self.samples[0..now].iter().map(|x| *x as usize).sum();
		sum / now
	}

	pub fn rate(&self) -> usize {
		if self.elapsed() > RATE_SECONDS as u64 {
			0
		} else {
			self.current_rate()
		}
	}
}

struct StatsCalculator<T = u32> {
	filled: bool,
	idx: usize,
	samples: [T; STATS_SAMPLES],
}

impl<T: Default + Copy> Default for StatsCalculator<T> {
	fn default() -> Self {
		StatsCalculator {
			filled: false,
			idx: 0,
			samples: [T::default(); STATS_SAMPLES],
		}
	}
}

impl<T: fmt::Display + Default + Copy + Ord> fmt::Debug for StatsCalculator<T> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "median: {} ms", self.approximated_median())
	}
}

impl<T: Default + Copy + Ord> StatsCalculator<T> {
	pub fn add(&mut self, sample: T) {
		self.idx += 1;
		if self.idx >= STATS_SAMPLES {
			self.filled = true;
			self.idx = 0;
		}

		self.samples[self.idx] = sample;
	}

	/// Returns aproximate of media
	pub fn approximated_median(&self) -> T {
		let mut copy = [T::default(); STATS_SAMPLES];
		copy.copy_from_slice(&self.samples);
		let bound = if self.filled { STATS_SAMPLES } else { self.idx + 1 };

		let (_, &mut median) = order_stat::median_of_medians(&mut copy[0..bound]);
		median
	}
}

/// RPC Statistics
#[derive(Default, Debug)]
pub struct RpcStats {
	requests: RwLock<RateCalculator>,
	roundtrips: RwLock<StatsCalculator<u32>>,
	active_sessions: AtomicUsize,
}

impl RpcStats {
	/// Count session opened
	pub fn open_session(&self) {
		self.active_sessions.fetch_add(1, atomic::Ordering::SeqCst);
	}

	/// Count session closed.
	/// Silently overflows if closing unopened session.
	pub fn close_session(&self) {
		self.active_sessions.fetch_sub(1, atomic::Ordering::SeqCst);
	}

	/// Count request. Returns number of requests in current second.
	pub fn count_request(&self) -> u16 {
		self.requests.write().tick()
	}

	/// Add roundtrip time (microseconds)
	pub fn add_roundtrip(&self, microseconds: u32) {
		self.roundtrips.write().add(microseconds)
	}

	/// Returns number of open sessions
	pub fn sessions(&self) -> usize {
		self.active_sessions.load(atomic::Ordering::Relaxed)
	}

	/// Returns requests rate
	pub fn requests_rate(&self) -> usize {
		self.requests.read().rate()
	}

	/// Returns approximated roundtrip in microseconds
	pub fn approximated_roundtrip(&self) -> u32 {
		self.roundtrips.read().approximated_median()
	}
}

/// Notifies about RPC activity.
pub trait ActivityNotifier: Send + Sync + 'static {
	/// Activity on RPC interface
	fn active(&self);
}

/// Stats-counting RPC middleware
pub struct Middleware<T: ActivityNotifier = ClientNotifier> {
	stats: Arc<RpcStats>,
	notifier: T,
}

impl<T: ActivityNotifier> Middleware<T> {
	/// Create new Middleware with stats counter and activity notifier.
	pub fn new(stats: Arc<RpcStats>, notifier: T) -> Self {
		Middleware {
			stats: stats,
			notifier: notifier,
		}
	}

	fn as_micro(dur: time::Duration) -> u32 {
		(dur.as_secs() * 1_000_000) as u32 + dur.subsec_nanos() / 1_000
	}
}

impl<M: rpc::Metadata, T: ActivityNotifier> rpc::Middleware<M> for Middleware<T> {
	fn on_request<F>(&self, request: rpc::Request, meta: M, process: F) -> rpc::FutureResponse where
		F: FnOnce(rpc::Request, M) -> rpc::FutureResponse,
	{
		let start = time::Instant::now();
		let response = process(request, meta);

		self.notifier.active();
		let stats = self.stats.clone();
		stats.count_request();
		response.map(move |res| {
			stats.add_roundtrip(Self::as_micro(start.elapsed()));
			res
		}).boxed()
	}
}

/// Client Notifier
pub struct ClientNotifier {
	/// Client
	pub client: Arc<::ethcore::client::Client>,
}

impl ActivityNotifier for ClientNotifier {
	fn active(&self) {
		self.client.keep_alive()
	}
}

#[cfg(test)]
mod tests {

	use super::{RateCalculator, StatsCalculator, RpcStats};

	#[test]
	fn should_calculate_rate() {
		// given
		let mut avg = RateCalculator::default();

		// when
		avg.tick();
		avg.tick();
		avg.tick();
		let rate = avg.rate();

		// then
		assert_eq!(rate, 3usize);
	}

	#[test]
	fn should_approximate_median() {
		// given
		let mut stats = StatsCalculator::default();
		stats.add(5);
		stats.add(100);
		stats.add(3);
		stats.add(15);
		stats.add(20);
		stats.add(6);

		// when
		let median = stats.approximated_median();

		// then
		assert_eq!(median, 5);
	}

	#[test]
	fn should_count_rpc_stats() {
		// given
		let stats = RpcStats::default();
		assert_eq!(stats.sessions(), 0);
		assert_eq!(stats.requests_rate(), 0);
		assert_eq!(stats.approximated_roundtrip(), 0);

		// when
		stats.open_session();
		stats.close_session();
		stats.open_session();
		stats.count_request();
		stats.count_request();
		stats.add_roundtrip(125);

		// then
		assert_eq!(stats.sessions(), 1);
		assert_eq!(stats.requests_rate(), 2);
		assert_eq!(stats.approximated_roundtrip(), 125);
	}

	#[test]
	fn should_be_sync_and_send() {
		let stats = RpcStats::default();
		is_sync(stats);
	}

	fn is_sync<F: Send + Sync>(x: F) {
		drop(x)
	}
}
