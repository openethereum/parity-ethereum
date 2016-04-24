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


use std::env;
use std::sync::Arc;
use time;
use env_logger::LogBuilder;
use util::{RotatingLogger};

/// Sets up the logger
pub fn setup_log(init: &Option<String>) -> Arc<RotatingLogger> {
	use rlog::*;

	let mut levels = String::new();
	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		let lvl = &env::var("RUST_LOG").unwrap();
		levels.push_str(&lvl);
		levels.push_str(",");
		builder.parse(lvl);
	}

	if let Some(ref s) = *init {
		levels.push_str(s);
		builder.parse(s);
	}

	let logs = Arc::new(RotatingLogger::new(levels));
	let logger = logs.clone();
	let format = move |record: &LogRecord| {
		let timestamp = time::strftime("%Y-%m-%d %H:%M:%S %Z", &time::now()).unwrap();
		let format = if max_log_level() <= LogLevelFilter::Info {
			format!("{}{}", timestamp, record.args())
		} else {
			format!("{}{}:{}: {}", timestamp, record.level(), record.target(), record.args())
		};
		logger.append(format.clone());
		format
    };
	builder.format(format);
	builder.init().unwrap();
	logs
}

