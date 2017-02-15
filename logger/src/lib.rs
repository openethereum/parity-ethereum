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

//! Logger for parity executables

extern crate ethcore_util as util;
extern crate log as rlog;
extern crate isatty;
extern crate regex;
extern crate env_logger;
extern crate time;
#[macro_use]
extern crate lazy_static;

use std::{env, thread, fs};
use std::sync::{Weak, Arc};
use std::io::Write;
use isatty::{stderr_isatty, stdout_isatty};
use env_logger::LogBuilder;
use regex::Regex;
use util::{Mutex, RotatingLogger}	;
use util::log::Colour;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	pub mode: Option<String>,
	pub color: bool,
	pub file: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			mode: None,
			color: !cfg!(windows),
			file: None,
		}
	}
}

lazy_static! {
	static ref ROTATING_LOGGER : Mutex<Weak<RotatingLogger>> = Mutex::new(Default::default());
}

/// Sets up the logger
pub fn setup_log(config: &Config) -> Result<Arc<RotatingLogger>, String> {
	use rlog::*;

	let mut levels = String::new();
	let mut builder = LogBuilder::new();
	// Disable ws info logging by default.
	builder.filter(Some("ws"), LogLevelFilter::Warn);
	// Disable rustls info logging by default.
	builder.filter(Some("rustls"), LogLevelFilter::Warn);
	builder.filter(None, LogLevelFilter::Info);

	if let Ok(lvl) = env::var("RUST_LOG") {
		levels.push_str(&lvl);
		levels.push_str(",");
		builder.parse(&lvl);
	}

	if let Some(ref s) = config.mode {
		levels.push_str(s);
		builder.parse(s);
	}

	let isatty = stderr_isatty();
	let enable_color = config.color && isatty;
	let logs = Arc::new(RotatingLogger::new(levels));
	let logger = logs.clone();
	let mut open_options = fs::OpenOptions::new();

	let maybe_file = match config.file.as_ref() {
		Some(f) => Some(open_options
			.append(true).create(true).open(f)
			.map_err(|_| format!("Cannot write to log file given: {}", f))?),
		None => None,
	};

	let format = move |record: &LogRecord| {
		let timestamp = time::strftime("%Y-%m-%d %H:%M:%S %Z", &time::now()).unwrap();

		let with_color = if max_log_level() <= LogLevelFilter::Info {
			format!("{} {}", Colour::Black.bold().paint(timestamp), record.args())
		} else {
			let name = thread::current().name().map_or_else(Default::default, |x| format!("{}", Colour::Blue.bold().paint(x)));
			format!("{} {} {} {}  {}", Colour::Black.bold().paint(timestamp), name, record.level(), record.target(), record.args())
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
		if !isatty && record.level() <= LogLevel::Info && stdout_isatty() {
			// duplicate INFO/WARN output to console
			println!("{}", ret);
		}

		ret
    };

	builder.format(format);
	builder.init()
		.and_then(|_| {
			*ROTATING_LOGGER.lock() = Arc::downgrade(&logs);
			Ok(logs)
		})
		// couldn't create new logger - try to fall back on previous logger.
		.or_else(|err| match ROTATING_LOGGER.lock().upgrade() {
			Some(l) => Ok(l),
			// no previous logger. fatal.
			None => Err(format!("{:?}", err)),
		})
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
