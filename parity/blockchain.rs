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

use std::str::{FromStr, from_utf8};
use std::{io, fs};
use std::io::{BufReader, BufRead};
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;
use ethcore_logger::{setup_log, Config as LogConfig};
use io::{PanicHandler, ForwardPanic};
use util::{ToPretty, Uint};
use rlp::PayloadInfo;
use ethcore::service::ClientService;
use ethcore::client::{Mode, DatabaseCompactionProfile, VMType, BlockImportError, BlockChainClient, BlockID};
use ethcore::error::ImportError;
use ethcore::miner::Miner;
use cache::CacheConfig;
use informant::{Informant, MillisecondDuration};
use params::{SpecType, Pruning, Switch, tracing_switch_to_bool, fatdb_switch_to_bool};
use io_handler::ImportIoHandler;
use helpers::{to_client_config, execute_upgrades};
use dir::Directories;
use user_defaults::UserDefaults;
use fdlimit;

#[derive(Debug, PartialEq)]
pub enum DataFormat {
	Hex,
	Binary,
}

impl Default for DataFormat {
	fn default() -> Self {
		DataFormat::Binary
	}
}

impl FromStr for DataFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"binary" | "bin" => Ok(DataFormat::Binary),
			"hex" => Ok(DataFormat::Hex),
			x => Err(format!("Invalid format: {}", x))
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum BlockchainCmd {
	Import(ImportBlockchain),
	Export(ExportBlockchain),
}

#[derive(Debug, PartialEq)]
pub struct ImportBlockchain {
	pub spec: SpecType,
	pub logger_config: LogConfig,
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub file_path: Option<String>,
	pub format: Option<DataFormat>,
	pub pruning: Pruning,
	pub pruning_history: u64,
	pub compaction: DatabaseCompactionProfile,
	pub wal: bool,
	pub mode: Mode,
	pub tracing: Switch,
	pub fat_db: Switch,
	pub vm_type: VMType,
	pub check_seal: bool,
}

#[derive(Debug, PartialEq)]
pub struct ExportBlockchain {
	pub spec: SpecType,
	pub logger_config: LogConfig,
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub file_path: Option<String>,
	pub format: Option<DataFormat>,
	pub pruning: Pruning,
	pub pruning_history: u64,
	pub compaction: DatabaseCompactionProfile,
	pub wal: bool,
	pub mode: Mode,
	pub fat_db: Switch,
	pub tracing: Switch,
	pub from_block: BlockID,
	pub to_block: BlockID,
	pub check_seal: bool,
}

pub fn execute(cmd: BlockchainCmd) -> Result<String, String> {
	match cmd {
		BlockchainCmd::Import(import_cmd) => execute_import(import_cmd),
		BlockchainCmd::Export(export_cmd) => execute_export(export_cmd),
	}
}

fn execute_import(cmd: ImportBlockchain) -> Result<String, String> {
	let timer = Instant::now();

	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	let _logger = setup_log(&cmd.logger_config);

	// create dirs used by parity
	try!(cmd.dirs.create_dirs());

	// load spec file
	let spec = try!(cmd.spec.spec());

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// database paths
	let db_dirs = cmd.dirs.database(genesis_hash, spec.fork_name.clone());

	// user defaults path
	let user_defaults_path = db_dirs.user_defaults_path();

	// load user defaults
	let mut user_defaults = try!(UserDefaults::load(&user_defaults_path));

	fdlimit::raise_fd_limit();

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&user_defaults);

	// check if tracing is on
	let tracing = try!(tracing_switch_to_bool(cmd.tracing, &user_defaults));

	// check if fatdb is on
	let fat_db = try!(fatdb_switch_to_bool(cmd.fat_db, &user_defaults, algorithm));

	// prepare client and snapshot paths.
	let client_path = db_dirs.client_path(algorithm);
	let snapshot_path = db_dirs.snapshot_path();

	// execute upgrades
	try!(execute_upgrades(&db_dirs, algorithm, cmd.compaction.compaction_profile(db_dirs.fork_path().as_path())));

	// prepare client config
	let client_config = to_client_config(&cmd.cache_config, cmd.mode, tracing, fat_db, cmd.compaction, cmd.wal, cmd.vm_type,  "".into(), algorithm, cmd.pruning_history, cmd.check_seal);

	// build client
	let service = try!(ClientService::start(
		client_config,
		&spec,
		&client_path,
		&snapshot_path,
		&cmd.dirs.ipc_path(),
		Arc::new(Miner::with_spec(&spec)),
	).map_err(|e| format!("Client service error: {:?}", e)));

	panic_handler.forward_from(&service);
	let client = service.client();

	let mut instream: Box<io::Read> = match cmd.file_path {
		Some(f) => Box::new(try!(fs::File::open(&f).map_err(|_| format!("Cannot open given file: {}", f)))),
		None => Box::new(io::stdin()),
	};

	const READAHEAD_BYTES: usize = 8;

	let mut first_bytes: Vec<u8> = vec![0; READAHEAD_BYTES];
	let mut first_read = 0;

	let format = match cmd.format {
		Some(format) => format,
		None => {
			first_read = try!(instream.read(&mut first_bytes).map_err(|_| "Error reading from the file/stream."));
			match first_bytes[0] {
				0xf9 => DataFormat::Binary,
				_ => DataFormat::Hex,
			}
		}
	};

	let informant = Informant::new(client.clone(), None, None, None, cmd.logger_config.color);

	try!(service.register_io_handler(Arc::new(ImportIoHandler {
		info: Arc::new(informant),
	})).map_err(|_| "Unable to register informant handler".to_owned()));

	let do_import = |bytes| {
		while client.queue_info().is_full() { sleep(Duration::from_secs(1)); }
		match client.import_block(bytes) {
			Err(BlockImportError::Import(ImportError::AlreadyInChain)) => {
				trace!("Skipping block already in chain.");
			}
			Err(e) => {
				return Err(format!("Cannot import block: {:?}", e));
			},
			Ok(_) => {},
		}
		Ok(())
	};


	match format {
		DataFormat::Binary => {
			loop {
				let mut bytes = if first_read > 0 {first_bytes.clone()} else {vec![0; READAHEAD_BYTES]};
				let n = if first_read > 0 {
					first_read
				} else {
					try!(instream.read(&mut bytes).map_err(|_| "Error reading from the file/stream."))
				};
				if n == 0 { break; }
				first_read = 0;
				let s = try!(PayloadInfo::from(&bytes).map_err(|e| format!("Invalid RLP in the file/stream: {:?}", e))).total();
				bytes.resize(s, 0);
				try!(instream.read_exact(&mut bytes[n..]).map_err(|_| "Error reading from the file/stream."));
				try!(do_import(bytes));
			}
		}
		DataFormat::Hex => {
			for line in BufReader::new(instream).lines() {
				let s = try!(line.map_err(|_| "Error reading from the file/stream."));
				let s = if first_read > 0 {from_utf8(&first_bytes).unwrap().to_owned() + &(s[..])} else {s};
				first_read = 0;
				let bytes = try!(s.from_hex().map_err(|_| "Invalid hex in file/stream."));
				try!(do_import(bytes));
			}
		}
	}
	client.flush_queue();

	// save user defaults
	user_defaults.pruning = algorithm;
	user_defaults.tracing = tracing;
	try!(user_defaults.save(&user_defaults_path));

	let report = client.report();

	let ms = timer.elapsed().as_milliseconds();
	Ok(format!("Import completed in {} seconds, {} blocks, {} blk/s, {} transactions, {} tx/s, {} Mgas, {} Mgas/s",
		ms / 1000,
		report.blocks_imported,
		(report.blocks_imported * 1000) as u64 / ms,
		report.transactions_applied,
		(report.transactions_applied * 1000) as u64 / ms,
		report.gas_processed / From::from(1_000_000),
		(report.gas_processed / From::from(ms * 1000)).low_u64(),
	).into())
}

fn execute_export(cmd: ExportBlockchain) -> Result<String, String> {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	let _logger = setup_log(&cmd.logger_config);

	// create dirs used by parity
	try!(cmd.dirs.create_dirs());

	let format = cmd.format.unwrap_or_default();

	// load spec file
	let spec = try!(cmd.spec.spec());

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// database paths
	let db_dirs = cmd.dirs.database(genesis_hash, spec.fork_name.clone());

	// user defaults path
	let user_defaults_path = db_dirs.user_defaults_path();

	// load user defaults
	let user_defaults = try!(UserDefaults::load(&user_defaults_path));

	fdlimit::raise_fd_limit();

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&user_defaults);

	// check if tracing is on
	let tracing = try!(tracing_switch_to_bool(cmd.tracing, &user_defaults));

	// check if fatdb is on
	let fat_db = try!(fatdb_switch_to_bool(cmd.fat_db, &user_defaults, algorithm));

	// prepare client and snapshot paths.
	let client_path = db_dirs.client_path(algorithm);
	let snapshot_path = db_dirs.snapshot_path();

	// execute upgrades
	try!(execute_upgrades(&db_dirs, algorithm, cmd.compaction.compaction_profile(db_dirs.fork_path().as_path())));

	// prepare client config
	let client_config = to_client_config(&cmd.cache_config, cmd.mode, tracing, fat_db, cmd.compaction, cmd.wal, VMType::default(), "".into(), algorithm, cmd.pruning_history, cmd.check_seal);

	let service = try!(ClientService::start(
		client_config,
		&spec,
		&client_path,
		&snapshot_path,
		&cmd.dirs.ipc_path(),
		Arc::new(Miner::with_spec(&spec)),
	).map_err(|e| format!("Client service error: {:?}", e)));

	panic_handler.forward_from(&service);
	let client = service.client();

	let mut out: Box<io::Write> = match cmd.file_path {
		Some(f) => Box::new(try!(fs::File::create(&f).map_err(|_| format!("Cannot write to file given: {}", f)))),
		None => Box::new(io::stdout()),
	};

	let from = try!(client.block_number(cmd.from_block).ok_or("From block could not be found"));
	let to = try!(client.block_number(cmd.to_block).ok_or("To block could not be found"));

	for i in from..(to + 1) {
		let b = try!(client.block(BlockID::Number(i)).ok_or("Error exporting incomplete chain"));
		match format {
			DataFormat::Binary => { out.write(&b).expect("Couldn't write to stream."); }
			DataFormat::Hex => { out.write_fmt(format_args!("{}", b.pretty())).expect("Couldn't write to stream."); }
		}
	}

	Ok("Export completed.".into())
}

#[cfg(test)]
mod test {
	use super::DataFormat;

	#[test]
	fn test_data_format_parsing() {
		assert_eq!(DataFormat::Binary, "binary".parse().unwrap());
		assert_eq!(DataFormat::Binary, "bin".parse().unwrap());
		assert_eq!(DataFormat::Hex, "hex".parse().unwrap());
	}
}
