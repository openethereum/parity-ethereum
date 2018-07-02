// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

extern crate parity;

extern crate ctrlc;
extern crate dir;
extern crate fdlimit;
#[macro_use]
extern crate log;
extern crate panic_hook;
extern crate parking_lot;

#[cfg(windows)] extern crate winapi;

use std::{process, env};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self as stdio, Read, Write};
use std::fs::{remove_file, metadata, File, create_dir_all};
use std::path::PathBuf;
use std::sync::Arc;
use ctrlc::CtrlC;
use dir::default_hypervisor_path;
use fdlimit::raise_fd_limit;
use parity::{start, ExecutionAction};
use parking_lot::{Condvar, Mutex};

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
		.and_then(|_| File::create(updates_path("spec_name_override"))
		.and_then(|mut f| f.write_all(spec_name.as_bytes())))
	{
		warn!("Couldn't override chain spec: {} at {:?}", e, updates_path("spec_name_override"));
	}
}

fn take_spec_name_override() -> Option<String> {
	let p = updates_path("spec_name_override");
	let r = File::open(p.clone()).ok()
		.and_then(|mut f| { let mut spec_name = String::new(); f.read_to_string(&mut spec_name).ok().map(|_| spec_name) });
	let _ = remove_file(p);
	r
}

#[cfg(windows)]
fn global_cleanup() {
	// We need to cleanup all sockets before spawning another Parity process. This makes sure everything is cleaned up.
	// The loop is required because of internal reference counter for winsock dll. We don't know how many crates we use do
	// initialize it. There's at least 2 now.
	for _ in 0.. 10 {
		unsafe { ::winapi::um::winsock2::WSACleanup(); }
	}
}

#[cfg(not(windows))]
fn global_init() {}

#[cfg(windows)]
fn global_init() {
	// When restarting in the same process this reinits windows sockets.
	unsafe {
		const WS_VERSION: u16 = 0x202;
		let mut wsdata: ::winapi::um::winsock2::WSADATA = ::std::mem::zeroed();
		::winapi::um::winsock2::WSAStartup(WS_VERSION, &mut wsdata);
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

#[derive(Debug)]
/// Status used to exit or restart the program.
struct ExitStatus {
	/// Whether the program panicked.
	panicking: bool,
	/// Whether the program should exit.
	should_exit: bool,
	/// Whether the program should restart.
	should_restart: bool,
	/// If a restart happens, whether a new chain spec should be used.
	spec_name_override: Option<String>,
}

// Run our version of parity.
// Returns the exit error code.
fn main_direct(force_can_restart: bool) -> i32 {
	global_init();

	let mut conf = {
		let args = std::env::args().collect::<Vec<_>>();
		parity::Configuration::parse_cli(&args).unwrap_or_else(|e| e.exit())
	};

	if let Some(spec_override) = take_spec_name_override() {
		conf.args.flag_testnet = false;
		conf.args.arg_chain = spec_override;
	}

	let can_restart = force_can_restart || conf.args.flag_can_restart;

	// increase max number of open files
	raise_fd_limit();

	let exit = Arc::new((Mutex::new(ExitStatus {
		panicking: false,
		should_exit: false,
		should_restart: false,
		spec_name_override: None
	}), Condvar::new()));

	// Double panic can happen. So when we lock `ExitStatus` after the main thread is notified, it cannot be locked
	// again.
	let exiting = Arc::new(AtomicBool::new(false));

	let exec = if can_restart {
		start(
			conf,
			{
				let e = exit.clone();
				let exiting = exiting.clone();
				move |new_chain: String| {
					if !exiting.swap(true, Ordering::SeqCst) {
						*e.0.lock() = ExitStatus {
							panicking: false,
							should_exit: true,
							should_restart: true,
							spec_name_override: Some(new_chain),
						};
						e.1.notify_all();
					}
				}
			},
			{
				let e = exit.clone();
				let exiting = exiting.clone();
				move || {
					if !exiting.swap(true, Ordering::SeqCst) {
						*e.0.lock() = ExitStatus {
							panicking: false,
							should_exit: true,
							should_restart: true,
							spec_name_override: None,
						};
						e.1.notify_all();
					}
				}
			}
		)

	} else {
		trace!(target: "mode", "Not hypervised: not setting exit handlers.");
		start(conf, move |_| {}, move || {})
	};

	let res = match exec {
		Ok(result) => match result {
			ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			ExecutionAction::Instant(None) => 0,
			ExecutionAction::Running(client) => {
				panic_hook::set_with({
					let e = exit.clone();
					let exiting = exiting.clone();
					move || {
						if !exiting.swap(true, Ordering::SeqCst) {
							*e.0.lock() = ExitStatus {
								panicking: true,
								should_exit: true,
								should_restart: false,
								spec_name_override: None,
							};
							e.1.notify_all();
						}
					}
				});

				CtrlC::set_handler({
					let e = exit.clone();
					let exiting = exiting.clone();
					move || {
						if !exiting.swap(true, Ordering::SeqCst) {
							*e.0.lock() = ExitStatus {
								panicking: false,
								should_exit: true,
								should_restart: false,
								spec_name_override: None,
							};
							e.1.notify_all();
						}
					}
				});

				// Wait for signal
				let mut lock = exit.0.lock();
				if !lock.should_exit {
					let _ = exit.1.wait(&mut lock);
				}

				client.shutdown();

				if lock.should_restart {
					if let Some(ref spec_name) = lock.spec_name_override {
						set_spec_name_override(spec_name.clone());
					}
					PLEASE_RESTART_EXIT_CODE
				} else {
					if lock.panicking {
						1
					} else {
						0
					}
				}
			},
		},
		Err(err) => {
			writeln!(&mut stdio::stderr(), "{}", err).expect("StdErr available; qed");
			1
		},
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
	panic_hook::set_abort();

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
