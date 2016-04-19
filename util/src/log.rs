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

//! Common log helper functions

use std::env;
use rlog::{LogLevelFilter};
use env_logger::LogBuilder;
use std::sync::{RwLock, RwLockReadGuard};
use arrayvec::ArrayVec;

lazy_static! {
	static ref LOG_DUMMY: bool = {
		let mut builder = LogBuilder::new();
		builder.filter(None, LogLevelFilter::Info);

		if let Ok(log) = env::var("RUST_LOG") {
			builder.parse(&log);
		}

		if let Ok(_) = builder.init() {
			println!("logger initialized");
		}
		true
	};
}

/// Intialize log with default settings
pub fn init_log() {
	let _ = *LOG_DUMMY;
}

const LOG_SIZE : usize = 128;

/// Logger implementation that keeps up to `LOG_SIZE` log elements.
pub struct RotatingLogger {
	/// Defined logger levels
	levels: String,
	/// Logs array. Latest log is always at index 0
	logs: RwLock<ArrayVec<[String; LOG_SIZE]>>,
}

impl RotatingLogger {

	/// Creates new `RotatingLogger` with given levels.
	/// It does not enforce levels - it's just read only.
	pub fn new(levels: String) -> Self {
		RotatingLogger {
			levels: levels,
			logs: RwLock::new(ArrayVec::<[_; LOG_SIZE]>::new()),
		}
	}

	/// Append new log entry
	pub fn append(&self, log: String) {
		self.logs.write().unwrap().insert(0, log);
	}

	/// Return levels
	pub fn levels(&self) -> &str {
		&self.levels
	}

	/// Return logs
	pub fn logs(&self) -> RwLockReadGuard<ArrayVec<[String; LOG_SIZE]>> {
		self.logs.read().unwrap()
	}

}

#[cfg(test)]
mod test {
	#[test]
	fn should_have_some_tests() {
		assert_eq!(true, false);
	}
}

