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

#![cfg_attr(feature="use-precompiled-js", allow(dead_code))]
#![cfg_attr(feature="use-precompiled-js", allow(unused_imports))]

use std::fmt;
use std::process::Command;

#[cfg(not(windows))]
mod platform {
	use std::process::Command;

	pub static NPM_CMD: &'static str = "npm";
	pub fn handle_cmd(cmd: &mut Command) -> &mut Command {
		cmd
	}
}

#[cfg(windows)]
mod platform {
	use std::process::{Command, Stdio};

	pub static NPM_CMD: &'static str = "cmd.exe";
	// NOTE [ToDr] For some reason on windows
	// The command doesn't have %~dp0 set properly
	// and it cannot load globally installed node.exe
	pub fn handle_cmd(cmd: &mut Command) -> &mut Command {
		cmd.stdin(Stdio::null())
			.arg("/c")
			.arg("npm.cmd")
	}
}

fn die<T : fmt::Debug>(s: &'static str, e: T) -> ! {
	panic!("Error: {}: {:?}", s, e);
}

#[cfg(feature = "use-precompiled-js")]
pub fn test(_path: &str) {
}
#[cfg(feature = "use-precompiled-js")]
pub fn build(_path: &str, _dest: &str) {
}

#[cfg(not(feature = "use-precompiled-js"))]
pub fn build(path: &str, dest: &str) {
	let child = platform::handle_cmd(&mut Command::new(platform::NPM_CMD))
		.arg("install")
		.arg("--no-progress")
		.current_dir(path)
		.status()
		.unwrap_or_else(|e| die("Installing node.js dependencies with npm", e));
	assert!(child.success(), "There was an error installing dependencies.");

	let child = platform::handle_cmd(&mut Command::new(platform::NPM_CMD))
		.arg("run")
		.arg("build")
		.env("NODE_ENV", "production")
		.env("BUILD_DEST", dest)
		.current_dir(path)
		.status()
		.unwrap_or_else(|e| die("Building JS code", e));
	assert!(child.success(), "There was an error build JS code.");
}

#[cfg(not(feature = "use-precompiled-js"))]
pub fn test(path: &str) {
	let child = Command::new(platform::NPM_CMD)
		.arg("run")
		.arg("test")
		.current_dir(path)
		.status()
		.unwrap_or_else(|e| die("Running test command", e));
	assert!(child.success(), "There was an error while running JS tests.");
}
