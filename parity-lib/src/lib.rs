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

//! Ethcore client library.

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
extern crate toml;

extern crate ethcore;
extern crate ethcore_bytes as bytes;
extern crate ethcore_io as io;
extern crate ethcore_light as light;
extern crate ethcore_logger;
extern crate ethcore_migrations as migrations;
extern crate ethcore_miner as miner;
extern crate ethcore_network as network;
extern crate ethcore_service;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate migration as migr;
extern crate kvdb;
extern crate kvdb_rocksdb;
extern crate ethkey;
extern crate ethsync;
extern crate node_health;
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
extern crate registrar;

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

#[cfg(test)]
extern crate tempdir;

mod account;
mod blockchain;
mod cache;
mod cli;
mod configuration;
mod dapps;
mod export_hardcoded_sync;
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

use std::io::BufReader;
use std::fs::File;
use hash::keccak_buffer;
use cli::Args;
use configuration::{Cmd, Execute, Configuration};
use deprecated::find_deprecated;
use ethcore_logger::setup_log;

fn print_hash_of(maybe_file: Option<String>) -> Result<String, String> {
	if let Some(file) = maybe_file {
		let mut f = BufReader::new(File::open(&file).map_err(|_| "Unable to open file".to_owned())?);
		let hash = keccak_buffer(&mut f).map_err(|_| "Unable to read from file".to_owned())?;
		Ok(format!("{:x}", hash))
	} else {
		Err("Streaming from standard input not yet supported. Specify a file.".to_owned())
	}
}

/// Action at the end of the parity client running.
pub enum PostExecutionAction {
	/// Something should be printed on stdout.
	Print(String),
	/// Parity should be retarted with the given chain spec.
	Restart(Option<String>),
	/// Parity should quit.
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
		Cmd::ExportHardcodedSync(export_hs_cmd) => export_hardcoded_sync::execute(export_hs_cmd).map(|s| PostExecutionAction::Print(s)),
	}
}

/// Runs Parity by passing a list of command-line arguments.
///
/// The `args` must **not** contain the name of the executable.
///
/// On error, returns what to print on stderr.
///
/// # Example
///
/// ```
/// start(vec!["--light".to_owned(), "--logging".to_owned(), "eth=trace".to_owned()])
/// ```
pub fn start(mut args: Vec<String>, spec_name_overide: Option<String>) -> Result<PostExecutionAction, String> {
	let can_restart = args.iter().any(|arg| arg == "--can-restart");
	args.insert(0, "parity".to_owned());
	let conf = Configuration::parse(&args, spec_name_overide).unwrap_or_else(|e| e.exit());

	let deprecated = find_deprecated(&conf.args);
	for d in deprecated {
		println!("{}", d);
	}

	let cmd = conf.into_command()?;
	execute(cmd, can_restart)
}
