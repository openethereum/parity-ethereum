// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Ethcore client application.

#![warn(missing_docs)]

extern crate ctrlc;
extern crate dir;
extern crate fdlimit;
#[macro_use]
extern crate log;
extern crate panic_hook;
extern crate parity_ethereum;
extern crate parking_lot;
extern crate parity_daemonize;
extern crate ansi_term;

#[cfg(windows)] extern crate winapi;
extern crate ethcore_logger;

use std::ffi::OsString;
use std::fs::{remove_file, metadata, File, create_dir_all};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{process, env};

use ansi_term::Colour;
use ctrlc::CtrlC;
use dir::default_hypervisor_path;
use fdlimit::raise_fd_limit;
use ethcore_logger::setup_log;
use parity_ethereum::{start, ExecutionAction};
use parity_daemonize::AsHandle;
use parking_lot::{Condvar, Mutex};

const PLEASE_RESTART_EXIT_CODE: i32 = 69;
const PARITY_EXECUTABLE_NAME: &str = "parity";

#[derive(Debug)]
enum Error {
	BinaryNotFound,
	ExitCode(i32),
	Restart,
	Unknown
}

fn update_path(name: &str) -> PathBuf {
	let mut dest = default_hypervisor_path();
	dest.push(name);
	dest
}

fn latest_exe_path() -> Result<PathBuf, Error> {
	File::open(update_path("latest")).and_then(|mut f| {
			let mut exe_path = String::new();
			trace!(target: "updater", "latest binary path: {:?}", f);
			f.read_to_string(&mut exe_path).map(|_| update_path(&exe_path))
	})
	.or(Err(Error::BinaryNotFound))
}

fn latest_binary_is_newer(current_binary: &Option<PathBuf>, latest_binary: &Option<PathBuf>) -> bool {
	match (
		current_binary
			.as_ref()
			.and_then(|p| metadata(p.as_path()).ok())
			.and_then(|m| m.modified().ok()),
		latest_binary
			.as_ref()
			.and_then(|p| metadata(p.as_path()).ok())
			.and_then(|m| m.modified().ok())
	) {
			(Some(latest_exe_time), Some(this_exe_time)) if latest_exe_time > this_exe_time => true,
			_ => false,
	}
}

fn set_spec_name_override(spec_name: &str) {
	if let Err(e) = create_dir_all(default_hypervisor_path())
		.and_then(|_| File::create(update_path("spec_name_override"))
		.and_then(|mut f| f.write_all(spec_name.as_bytes())))
	{
		warn!("Couldn't override chain spec: {} at {:?}", e, update_path("spec_name_override"));
	}
}

fn take_spec_name_override() -> Option<String> {
	let p = update_path("spec_name_override");
	let r = File::open(p.clone())
		.ok()
		.and_then(|mut f| {
			let mut spec_name = String::new();
			f.read_to_string(&mut spec_name).ok().map(|_| spec_name)
		});
	let _ = remove_file(p);
	r
}

#[cfg(windows)]
fn global_cleanup() {
	// We need to clean up all sockets before spawning another Parity process. This makes sure everything is cleaned up.
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

// Starts parity binary installed via `parity-updater` and returns the code it exits with.
fn run_parity() -> Result<(), Error> {
	global_init();

	let prefix = vec![OsString::from("--can-restart"), OsString::from("--force-direct")];

	let res: Result<(), Error> = latest_exe_path()
		.and_then(|exe| process::Command::new(exe)
		.args(&(env::args_os().skip(1).chain(prefix.into_iter()).collect::<Vec<_>>()))
		.status()
		.ok()
		.map_or(Err(Error::Unknown), |es| {
			match es.code() {
				// Process success
				Some(0) => Ok(()),
				// Please restart
				Some(PLEASE_RESTART_EXIT_CODE) => Err(Error::Restart),
				// Process error code `c`
				Some(c) => Err(Error::ExitCode(c)),
				// Unknown error, couldn't determine error code
				_ => Err(Error::Unknown),
			}
		})
	);

	global_cleanup();
	res
}

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

// Run `locally installed version` of parity (i.e, not installed via `parity-updater`)
// Returns the exit error code.
fn main_direct(force_can_restart: bool) -> i32 {
	global_init();

	let mut conf = {
		let args = std::env::args().collect::<Vec<_>>();
		parity_ethereum::Configuration::parse_cli(&args).unwrap_or_else(|e| e.exit())
	};

	let logger = setup_log(&conf.logger_config()).unwrap_or_else(|e| {
		eprintln!("{}", e);
		process::exit(2)
	});

	if let Some(spec_override) = take_spec_name_override() {
		conf.args.flag_testnet = false;
		conf.args.arg_chain = spec_override;
	}

	// FIXME: `pid_file` shouldn't need to cloned here
	// see: `https://github.com/paritytech/parity-daemonize/pull/13` for more info
	let handle = if let Some(pid) = conf.args.arg_daemon_pid_file.clone() {
		info!("{}", Colour::Blue.paint("starting in daemon mode").to_string());
		let _ = std::io::stdout().flush();

		match parity_daemonize::daemonize(pid) {
			Ok(h) => Some(h),
			Err(e) => {
				error!(
					"{}",
					Colour::Red.paint(format!("{}", e))
				);
				return 1;
			}
		}
	} else {
		None
	};

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
			logger,
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
		start(conf, logger, move |_| {}, move || {})
	};

	let res = match exec {
		Ok(result) => match result {
			ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			ExecutionAction::Instant(None) => 0,
			ExecutionAction::Running(client) => {
				panic_hook::set_with({
					let e = exit.clone();
					let exiting = exiting.clone();
					move |panic_msg| {
						warn!("Panic occured, see stderr for details");
						eprintln!("{}", panic_msg);
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

				// so the client has started successfully
				// if this is a daemon, detach from the parent process
				if let Some(mut handle) = handle {
					handle.detach()
				}

				// Wait for signal
				let mut lock = exit.0.lock();
				if !lock.should_exit {
					let _ = exit.1.wait(&mut lock);
				}

				client.shutdown();

				if lock.should_restart {
					if let Some(ref spec_name) = lock.spec_name_override {
						set_spec_name_override(&spec_name.clone());
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
			// error occured during start up
			// if this is a daemon, detach from the parent process
			if let Some(mut handle) = handle {
				handle.detach_with_msg(format!("{}", Colour::Red.paint(&err)))
			}
			eprintln!("{}", err);
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

macro_rules! trace_main {
	($arg:expr) => (println_trace_main($arg.into()));
	($($arg:tt)*) => (println_trace_main(format!("{}", format_args!($($arg)*))));
}

fn main() {
	panic_hook::set_abort();

	// the user has specified to run its originally installed binary (not via `parity-updater`)
	let force_direct = std::env::args().any(|arg| arg == "--force-direct");

	// absolute path to the current `binary`
	let exe_path = std::env::current_exe().ok();

	// the binary is named `target/xx/yy`
	let development = exe_path
		.as_ref()
		.and_then(|p| {
			p.parent()
				.and_then(|p| p.parent())
				.and_then(|p| p.file_name())
				.map(|n| n == "target")
		})
		.unwrap_or(false);

	// the binary is named `parity`
	let same_name = exe_path
		.as_ref()
		.map_or(false, |p| {
			p.file_stem().map_or(false, |n| n == PARITY_EXECUTABLE_NAME)
		});

	trace_main!("Starting up {} (force-direct: {}, development: {}, same-name: {})",
				std::env::current_exe().ok().map_or_else(|| "<unknown>".into(), |x| format!("{}", x.display())),
				force_direct,
				development,
				same_name);

	if !force_direct && !development && same_name {
		// Try to run the latest installed version of `parity`,
		// Upon failure it falls back to the locally installed version of `parity`
		// Everything run inside a loop, so we'll be able to restart from the child into a new version seamlessly.
		loop {
			// `Path` to the latest downloaded binary
			let latest_exe = latest_exe_path().ok();

			// `Latest´ binary exist
			let have_update = latest_exe.as_ref().map_or(false, |p| p.exists());

			// Canonicalized path to the current binary is not the same as to latest binary
			let canonicalized_path_not_same = exe_path
				.as_ref()
				.map_or(false, |exe| latest_exe.as_ref()
				.map_or(false, |lexe| exe.canonicalize().ok() != lexe.canonicalize().ok()));

			// Downloaded `binary` is newer
			let update_is_newer = latest_binary_is_newer(&latest_exe, &exe_path);
			trace_main!("Starting... (have-update: {}, non-updated-current: {}, update-is-newer: {})", have_update, canonicalized_path_not_same, update_is_newer);

			let exit_code = if have_update && canonicalized_path_not_same && update_is_newer {
				trace_main!("Attempting to run latest update ({})...",
							latest_exe.as_ref().expect("guarded by have_update; latest_exe must exist for have_update; qed").display());
				match run_parity() {
					Ok(_) => 0,
					// Restart parity
					Err(Error::Restart) => PLEASE_RESTART_EXIT_CODE,
					// Fall back to local version
					Err(e) => {
						error!(target: "updater", "Updated binary could not be executed error: {:?}. Falling back to local version", e);
						main_direct(true)
					}
				}
			} else {
				trace_main!("No latest update. Attempting to direct...");
				main_direct(true)
			};
			trace_main!("Latest binary exited with exit code: {}", exit_code);
			if exit_code != PLEASE_RESTART_EXIT_CODE {
				trace_main!("Quitting...");
				process::exit(exit_code);
			}
			trace!(target: "updater", "Re-running updater loop");
		}
	} else {
		trace_main!("Running direct");
		// Otherwise, we're presumably running the version we want. Just run and fall-through.
		process::exit(main_direct(false));
	}
}
