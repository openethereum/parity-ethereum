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

//! Performance timer with logging

extern crate time;
#[macro_use]
extern crate log;

use time::precise_time_ns;

#[macro_export]
macro_rules! trace_time {
	($name: expr) => {
		let _timer = $crate::PerfTimer::new($name);
	}
}

/// Performance timer with logging. Starts measuring time in the constructor, prints
/// elapsed time in the destructor or when `stop` is called.
pub struct PerfTimer {
	name: &'static str,
	start: u64,
}

impl PerfTimer {
	/// Create an instance with given name.
	pub fn new(name: &'static str) -> PerfTimer {
		PerfTimer {
			name,
			start: precise_time_ns(),
		}
	}
}

impl Drop for PerfTimer {
	fn drop(&mut self) {
		trace!(target: "perf", "{}: {:.2}ms", self.name, (precise_time_ns()  - self.start) as f32 / 1000_000.0);
	}
}
