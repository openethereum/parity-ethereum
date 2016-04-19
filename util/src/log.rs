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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

static LOG_SIZE : usize = 128;

pub struct RotatingLogger {
	idx: AtomicUsize,
	levels: String,
	logs: RwLock<Vec<String>>,
}

impl RotatingLogger {

	pub fn new(levels: String) -> Self {
		RotatingLogger {
			idx: AtomicUsize::new(0),
			levels: levels,
			logs: RwLock::new(Vec::with_capacity(LOG_SIZE)),
		}
	}

	pub fn append(&self, log: String) {
		let idx = self.idx.fetch_add(1, Ordering::SeqCst);
		let idx = idx % LOG_SIZE;
		self.logs.write().unwrap().insert(idx, log);
	}

	pub fn levels(&self) -> &str {
		&self.levels
	}

	pub fn logs(&self) -> RwLockReadGuard<Vec<String>> {
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

