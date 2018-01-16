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

//! Node Health status reporting.

#![warn(missing_docs)]

extern crate futures;
extern crate futures_cpupool;
extern crate ntp;
extern crate time as time_crate;
extern crate parity_reactor;
extern crate parking_lot;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod health;
mod time;
mod types;

pub use futures_cpupool::CpuPool;
pub use health::NodeHealth;
pub use types::{Health, HealthInfo, HealthStatus};
pub use time::{TimeChecker, Error};

/// Indicates sync status
pub trait SyncStatus: ::std::fmt::Debug + Send + Sync {
	/// Returns true if there is a major sync happening.
	fn is_major_importing(&self) -> bool;

	/// Returns number of connected and ideal peers.
	fn peers(&self) -> (usize, usize);
}
