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

extern crate ansi_term;
extern crate app_dirs;
extern crate ctrlc;
extern crate docopt;
#[macro_use]
extern crate clap;
extern crate dir;
extern crate env_logger;
extern crate fdlimit;
extern crate futures;
extern crate futures_cpupool;
extern crate isatty;
extern crate jsonrpc_core;
extern crate num_cpus;
extern crate number_prefix;
extern crate parking_lot;
extern crate regex;
extern crate rlp;
extern crate rpassword;
extern crate rustc_hex;
extern crate semver;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;

extern crate ethcore;
extern crate ethcore_devtools as devtools;
extern crate ethcore_io as io;
extern crate ethcore_light as light;
extern crate ethcore_logger;
extern crate ethcore_util as util;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes as bytes;
extern crate ethcore_network as network;
extern crate migration as migr;
extern crate kvdb;
extern crate kvdb_rocksdb;
extern crate ethkey;
extern crate ethsync;
extern crate node_health;
extern crate panic_hook;
extern crate parity_hash_fetch as hash_fetch;
extern crate parity_ipfs_api;
extern crate parity_local_store as local_store;
extern crate parity_reactor;
extern crate parity_rpc;
extern crate parity_updater as updater;
extern crate parity_version;
extern crate parity_whisper;
extern crate path;
extern crate rpc_cli;
extern crate node_filter;
extern crate keccak_hash as hash;
extern crate journaldb;

#[macro_use]
extern crate log as rlog;

#[cfg(feature="stratum")]
extern crate ethcore_stratum;

#[cfg(feature="secretstore")]
extern crate ethcore_secretstore;

#[cfg(feature = "dapps")]
extern crate parity_dapps;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(windows)] extern crate ws2_32;
#[cfg(windows)] extern crate winapi;

mod account;
mod blockchain;
mod cache;
mod cli;
mod configuration;
mod dapps;
mod ipfs;
mod deprecated;
mod helpers;
mod informant;
mod light_helpers;
mod migration;
mod modules;
mod params;
mod presale;
mod rpc;
mod rpc_apis;
mod run;
mod secretstore;
mod signer;
mod snapshot;
mod upgrade;
mod url;
mod user_defaults;
mod whisper;

#[cfg(feature="stratum")]
mod stratum;

use std::{process, env};
use std::collections::HashMap;
use std::io::{self as stdio, BufReader, Read, Write};
use std::fs::{remove_file, metadata, File, create_dir_all};
use std::path::PathBuf;
use hash::keccak_buffer;
use cli::Args;
use configuration::{Cmd, Execute, Configuration};
use deprecated::find_deprecated;
use ethcore_logger::setup_log;
use dir::default_hypervisor_path;

fn print_hash_of(maybe_file: Option<String>) -> Result<String, String> {
	if let Some(file) = maybe_file {
		let mut f = BufReader::new(File::open(&file).map_err(|_| "Unable to open file".to_owned())?);
		let hash = keccak_buffer(&mut f).map_err(|_| "Unable to read from file".to_owned())?;
		Ok(hash.hex())
	} else {
		Err("Streaming from standard input not yet supported. Specify a file.".to_owned())
	}
}

enum PostExecutionAction {
	Print(String),
	Restart(Option<String>),
	Quit,
}

fn execute(command: Execute, can_restart: bool) -> Result<PostExecutionAction, String> {
	let logger = setup_log(&command.logger).expect("Logger is initialized only once; qed");

	match command.cmd {
		Cmd::Run(run_cmd) => {
			let (restart, spec_name) = run::execute(run_cmd, can_restart, logger)?;
			Ok(if restart { PostExecutionAction::Restart(spec_name) } else { PostExecutionAction::Quit })
		},
		Cmd::Version => Ok(PostExecutionAction::Print(Args::print_version())),
		Cmd::Hash(maybe_file) => print_hash_of(maybe_file).map(|s| PostExecutionAction::Print(s)),
		Cmd::Account(account_cmd) => account::execute(account_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::ImportPresaleWallet(presale_cmd) => presale::execute(presale_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::Blockchain(blockchain_cmd) => blockchain::execute(blockchain_cmd).map(|_| PostExecutionAction::Quit),
		Cmd::SignerToken(ws_conf, ui_conf, logger_config) => signer::execute(ws_conf, ui_conf, logger_config).map(|s| PostExecutionAction::Print(s)),
		Cmd::SignerSign { id, pwfile, port, authfile } => rpc_cli::signer_sign(id, pwfile, port, authfile).map(|s| PostExecutionAction::Print(s)),
		Cmd::SignerList { port, authfile } => rpc_cli::signer_list(port, authfile).map(|s| PostExecutionAction::Print(s)),
		Cmd::SignerReject { id, port, authfile } => rpc_cli::signer_reject(id, port, authfile).map(|s| PostExecutionAction::Print(s)),
		Cmd::Snapshot(snapshot_cmd) => snapshot::execute(snapshot_cmd).map(|s| PostExecutionAction::Print(s)),
	}
}

fn start(can_restart: bool) -> Result<PostExecutionAction, String> {
	let args: Vec<String> = env::args().collect();
	let conf = Configuration::parse(&args, take_spec_name_override()).unwrap_or_else(|e| e.exit());

	let deprecated = find_deprecated(&conf.args);
	for d in deprecated {
		println!("{}", d);
	}

	let cmd = conf.into_command()?;
	execute(cmd, can_restart)
}

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

fn take_spec_name_override() -> Option<String> {
	let p = updates_path("spec_name_overide");
	let r = File::open(p.clone()).ok()
		.and_then(|mut f| { let mut spec_name = String::new(); f.read_to_string(&mut spec_name).ok().map(|_| spec_name) });
	let _ = remove_file(p);
	r
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
fn main_direct(can_restart: bool) -> i32 {
	global_init();
	let mut alt_mains = HashMap::new();
	sync_main(&mut alt_mains);
	stratum_main(&mut alt_mains);
	let res = if let Some(f) = std::env::args().nth(1).and_then(|arg| alt_mains.get(&arg.to_string())) {
		f();
		0
	} else {
		match start(can_restart) {
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
		let can_restart = std::env::args().any(|arg| arg == "--can-restart");
		process::exit(main_direct(can_restart));
	}
}
