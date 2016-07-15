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
use std::fs::File;
use std::io::Write;
use isatty::{stderr_isatty};
use time;
use env_logger::LogBuilder;
use regex::Regex;
use util::RotatingLogger;
use util::log::Colour;

/// Sets up the logger
pub fn setup_log(init: &Option<String>, enable_color: bool, log_to_file: &Option<String>) -> Arc<RotatingLogger> {
	use rlog::*;

	let mut levels = String::new();
	let mut builder = LogBuilder::new();
	// Disable ws info logging by default.
	builder.filter(Some("ws"), LogLevelFilter::Warn);
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		let lvl = &env::var("RUST_LOG").unwrap();
		levels.push_str(lvl);
		levels.push_str(",");
		builder.parse(lvl);
	}

	if let Some(ref s) = *init {
		levels.push_str(s);
		builder.parse(s);
	}

	let enable_color = enable_color && stderr_isatty();
	let logs = Arc::new(RotatingLogger::new(levels));
	let logger = logs.clone();
	let maybe_file = log_to_file.as_ref().map(|f| File::create(f).unwrap_or_else(|_| die!("Cannot write to log file given: {}", f)));
	let format = move |record: &LogRecord| {
		let timestamp = time::strftime("%Y-%m-%d %H:%M:%S %Z", &time::now()).unwrap();

		let with_color = if max_log_level() <= LogLevelFilter::Info {
			format!("{}{}", Colour::Black.bold().paint(timestamp), record.args())
		} else {
			format!("{}{}:{}: {}", Colour::Black.bold().paint(timestamp), record.level(), record.target(), record.args())
		};

		let removed_color = kill_color(with_color.as_ref());

		let ret = match enable_color {
			true => with_color,
			false => removed_color.clone(),
		};

		if let Some(mut file) = maybe_file.as_ref() {
			// ignore errors - there's nothing we can do
			let _ = file.write_all(removed_color.as_bytes());
			let _ = file.write_all(b"\n");
		}
		logger.append(removed_color);

		ret
    };
	builder.format(format);
	builder.init().unwrap();
	logs
}

fn kill_color(s: &str) -> String {
	lazy_static! {
		static ref RE: Regex = Regex::new("\x1b\\[[^m]+m").unwrap();
	}
	RE.replace_all(s, "")
}

#[test]
fn should_remove_colour() {
	let before = "test";
	let after = kill_color(&Colour::Red.bold().paint(before));
	assert_eq!(after, "test");
}

#[test]
fn should_remove_multiple_colour() {
	let t = format!("{} {}", Colour::Red.bold().paint("test"), Colour::White.normal().paint("again"));
	let after = kill_color(&t);
	assert_eq!(after, "test again");
}
