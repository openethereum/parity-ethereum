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
//! Distributions can be persisted and updated between runs.

use request::{CompleteRequest, Kind};

use time;
use util::{U256, RwLock, Mutex};

const ROLLING_SAMPLE_SIZE: usize = 128;

/// Something which stores load timer samples.
pub trait LoadTimerStore {
}

/// Request load distribution.
pub struct LoadDistribution {
	active_set: RwLock<HashMap<Kind, Mutex<(U256, U256)>>>,
}

impl LoadDistribution {
	/// Begin a timer.
	pub fn begin<'a>(&'a self, req: &CompleteRequest) -> LoadTimer<'a> {
		let kind = req.kind();
		let n = match *req {
			CompleteRequest::Headers(ref req) => req.max.into(),
			CompleteRequest::Execution(ref req) => req.gas.clone(),
			_ => 1.into(),
		};

		LoadTimer {
			start: time::precise_time_ns(),
			n: n,
			dist: self,
			kind: kind,
		}
	}

	fn update(&self, kind: Kind, elapsed: u64, n: U256) {
		let update_counters = |&mut (ref mut c_e, ref mut c_n)| {
			*c_e += elapsed.into();
			*c_n += n;
		};

		{
			let set = self.active_set.read();
			if let Some(counters) = set.get(&kind) {
				update_counters(&mut *counters.lock());
				return;
			}
		}

		let mut set = self.active_set.write();
		let counters = set
			.entry(kind)
			.or_insert_with(|| Mutex::new((0.into(), 0.into())));

		update_counters(counters.get_mut());
	}
}

/// A timer for a single request.
/// On drop, this will update the distribution.
pub struct LoadTimer<'a> {
	start: u64,
	n: U256,
	dist: &'a LoadDistribution,
	kind: Kind,
}

impl<'a> Drop for LoadTimer<'a> {
	fn drop(&mut self) {
		let elapsed = time::precise_time_ns() - self.start;
		self.dist.update(self.kind, elapsed, self.n);
	}
}
