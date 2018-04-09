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

//! Ethcore client application.

#![warn(missing_docs)]

extern crate panic_hook;
extern crate parity_lib;
extern crate dir;

#[macro_use]
extern crate log;

#[cfg(windows)] extern crate ws2_32;
#[cfg(windows)] extern crate winapi;

use dir::default_hypervisor_path;
use std::{process, env};
use std::collections::HashMap;
use std::io::{self as stdio, Read, Write};
use std::fs::{metadata, File, create_dir_all};
use std::path::PathBuf;
use parity_lib::{start, PostExecutionAction};

#[cfg(not(feature="stratum"))]
fn stratum_main(_: &mut HashMap<String, fn()>) {}

#[cfg(feature="stratum")]
fn stratum_main(alt_mains: &mut HashMap<String, fn()>) {
	alt_mains.insert("stratum".to_owned(), stratum::main);
}

fn sync_main(_: &mut HashMap<String, fn()>) {}

fn updates_path(name: &str) -> PathBuf {
	let mut dest = PathBuf::from(default_hypervisor_path());
	dest.push(name);
	dest
}

fn latest_exe_path() -> Option<PathBuf> {
	File::open(updates_path("latest")).ok()
		.and_then(|mut f| { let mut exe = String::new(); f.read_to_string(&mut exe).ok().map(|_| updates_path(&exe)) })
}

fn set_spec_name_override(spec_name: String) {
	if let Err(e) = create_dir_all(default_hypervisor_path())
		.and_then(|_| File::create(updates_path("spec_name_overide"))
		.and_then(|mut f| f.write_all(spec_name.as_bytes())))
	{
		warn!("Couldn't override chain spec: {} at {:?}", e, updates_path("spec_name_overide"));
	}
}

#[cfg(windows)]
fn global_cleanup() {
	// We need to cleanup all sockets before spawning another Parity process. This makes shure everything is cleaned up.
	// The loop is required because of internal refernce counter for winsock dll. We don't know how many crates we use do
	// initialize it. There's at least 2 now.
	for _ in 0.. 10 {
		unsafe { ::ws2_32::WSACleanup(); }
	}
}

#[cfg(not(windows))]
fn global_init() {}

#[cfg(windows)]
fn global_init() {
	// When restarting in the same process this reinits windows sockets.
	unsafe {
		const WS_VERSION: u16 = 0x202;
		let mut wsdata: ::winapi::winsock2::WSADATA = ::std::mem::zeroed();
		::ws2_32::WSAStartup(WS_VERSION, &mut wsdata);
	}
}

#[cfg(not(windows))]
fn global_cleanup() {}

// Starts ~/.parity-updates/parity and returns the code it exits with.
fn run_parity() -> Option<i32> {
	global_init();
	use ::std::ffi::OsString;
	let prefix = vec![OsString::from("--can-restart"), OsString::from("--force-direct")];
	let res = latest_exe_path().and_then(|exe| process::Command::new(exe)
		.args(&(env::args_os().skip(1).chain(prefix.into_iter()).collect::<Vec<_>>()))
		.status()
		.map(|es| es.code().unwrap_or(128))
		.ok()
	);
	global_cleanup();
	res
}

const PLEASE_RESTART_EXIT_CODE: i32 = 69;

// Run our version of parity.
// Returns the exit error code.
fn main_direct(force_can_restart: bool) -> i32 {
	global_init();
	let mut alt_mains = HashMap::new();
	sync_main(&mut alt_mains);
	stratum_main(&mut alt_mains);
	let res = if let Some(f) = std::env::args().nth(1).and_then(|arg| alt_mains.get(&arg.to_string())) {
		f();
		0
	} else {
		let mut args = std::env::args().skip(1).collect::<Vec<_>>();
		if force_can_restart && !args.iter().any(|arg| arg == "--can-restart") {
			args.push("--can-restart".to_owned());
		}

		match start(args) {
			Ok(result) => match result {
				PostExecutionAction::Print(s) => { println!("{}", s); 0 },
				PostExecutionAction::Restart(spec_name_override) => {
					if let Some(spec_name) = spec_name_override {
						set_spec_name_override(spec_name);
					}
					PLEASE_RESTART_EXIT_CODE
				},
				PostExecutionAction::Quit => 0,
			},
			Err(err) => {
				writeln!(&mut stdio::stderr(), "{}", err).expect("StdErr available; qed");
				1
			},
		}
	};
	global_cleanup();
	res
}

fn println_trace_main(s: String) {
	if env::var("RUST_LOG").ok().and_then(|s| s.find("main=trace")).is_some() {
		println!("{}", s);
	}
}

#[macro_export]
macro_rules! trace_main {
	($arg:expr) => (println_trace_main($arg.into()));
	($($arg:tt)*) => (println_trace_main(format!("{}", format_args!($($arg)*))));
}

fn main() {
	panic_hook::set();

	// assuming the user is not running with `--force-direct`, then:
	// if argv[0] == "parity" and this executable != ~/.parity-updates/parity, run that instead.
	let force_direct = std::env::args().any(|arg| arg == "--force-direct");
	let exe = std::env::current_exe().ok();
	let development = exe.as_ref().and_then(|p| p.parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).map(|n| n == "target")).unwrap_or(false);
	let same_name = exe.as_ref().map(|p| p.file_stem().map_or(false, |s| s == "parity") && p.extension().map_or(true, |x| x == "exe")).unwrap_or(false);
	trace_main!("Starting up {} (force-direct: {}, development: {}, same-name: {})", std::env::current_exe().map(|x| format!("{}", x.display())).unwrap_or("<unknown>".to_owned()), force_direct, development, same_name);
	if !force_direct && !development && same_name {
		// looks like we're not running ~/.parity-updates/parity when the user is expecting otherwise.
		// Everything run inside a loop, so we'll be able to restart from the child into a new version seamlessly.
		loop {
			// If we fail to run the updated parity then fallback to local version.
			let latest_exe = latest_exe_path();
			let have_update = latest_exe.as_ref().map_or(false, |p| p.exists());
			let is_non_updated_current = exe.as_ref().map_or(false, |exe| latest_exe.as_ref().map_or(false, |lexe| exe.canonicalize().ok() != lexe.canonicalize().ok()));
			let update_is_newer = match (
				latest_exe.as_ref()
					.and_then(|p| metadata(p.as_path()).ok())
					.and_then(|m| m.modified().ok()),
				exe.as_ref()
					.and_then(|p| metadata(p.as_path()).ok())
					.and_then(|m| m.modified().ok())
			) {
				(Some(latest_exe_time), Some(this_exe_time)) if latest_exe_time > this_exe_time => true,
				_ => false,
			};
			trace_main!("Starting... (have-update: {}, non-updated-current: {}, update-is-newer: {})", have_update, is_non_updated_current, update_is_newer);
			let exit_code = if have_update && is_non_updated_current && update_is_newer {
				trace_main!("Attempting to run latest update ({})...", latest_exe.as_ref().expect("guarded by have_update; latest_exe must exist for have_update; qed").display());
				run_parity().unwrap_or_else(|| { trace_main!("Falling back to local..."); main_direct(true) })
			} else {
				trace_main!("No latest update. Attempting to direct...");
				main_direct(true)
			};
			trace_main!("Latest exited with {}", exit_code);
			if exit_code != PLEASE_RESTART_EXIT_CODE {
				trace_main!("Quitting...");
				process::exit(exit_code);
			}
			trace_main!("Rerunning...");
		}
	} else {
		trace_main!("Running direct");
		// Otherwise, we're presumably running the version we want. Just run and fall-through.
		process::exit(main_direct(false));
	}
}
