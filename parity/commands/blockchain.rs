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
use std::time::Duration;
use std::thread::sleep;
use std::path::Path;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;
use util::panics::{PanicHandler, ForwardPanic};
use util::{version, contents, NetworkConfiguration, journaldb, PayloadInfo};
use util::log::Colour;
use ethcore::service::ClientService;
use ethcore::client::{ClientConfig, Mode, DatabaseCompactionProfile, Switch, VMType, BlockImportError, BlockChainClient, BlockID};
use ethcore::error::ImportError;
use ethcore::miner::Miner;
use ethcore::spec::Spec;
use ethcore::ethereum;
use cache::CacheConfig;
use setup_log::setup_log;
use informant::Informant;
use fdlimit;

#[derive(Debug, PartialEq)]
pub enum DataFormat {
	Hex,
	Binary,
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
pub struct LoggerConfig {
	pub mode: Option<String>,
	pub color: bool,
}

#[derive(Debug, PartialEq)]
pub enum SpecType {
	Mainnet,
	Testnet,
	Olympic,
	Custom(String),
}

impl FromStr for SpecType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let spec = match s {
			"frontier" | "homestead" | "mainnet" => SpecType::Mainnet,
			"morden" | "testnet" => SpecType::Testnet,
			"olympic" => SpecType::Olympic,
			other => SpecType::Custom(other.into()),
		};
		Ok(spec)
	}
}

impl SpecType {
	fn spec(&self) -> Result<Spec, String> {
		match *self {
			SpecType::Mainnet => Ok(ethereum::new_frontier()),
			SpecType::Testnet => Ok(ethereum::new_morden()),
			SpecType::Olympic => Ok(ethereum::new_olympic()),
			SpecType::Custom(ref file) => Ok(Spec::load(&try!(contents(file).map_err(|_| "Could not load specification file."))))
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct ImportBlockchain {
	pub spec: SpecType,
	pub logger_config: LoggerConfig,
	pub cache_config: CacheConfig,
	pub db_path: String,
	pub file_path: Option<String>,
	pub format: Option<DataFormat>,
	pub pruning: Option<journaldb::Algorithm>,
	pub compaction: DatabaseCompactionProfile,
	pub mode: Mode,
	pub tracing: Switch,
	pub vm_type: VMType,
}

#[derive(Debug, PartialEq)]
pub struct ExportBlockchain {
	pub spec: SpecType,
	pub logger_config: LoggerConfig,
	pub cache_config: CacheConfig,
	pub pruning: Option<journaldb::Algorithm>,
	pub compaction: DatabaseCompactionProfile,
	pub mode: Mode,
	pub tracing: Switch,
	pub from_block: BlockID,
	pub to_block: BlockID,
}

pub fn execute(cmd: BlockchainCmd) -> Result<String, String> {
	match cmd {
		BlockchainCmd::Import(import_cmd) => execute_import(import_cmd),
		BlockchainCmd::Export(export_cmd) => execute_export(export_cmd),
	}
}

fn client_config(
		cache_config: &CacheConfig,
		mode: Mode,
		tracing: Switch,
		pruning: Option<journaldb::Algorithm>,
		compaction: DatabaseCompactionProfile
	) -> ClientConfig {
	let mut client_config = ClientConfig::default();
	client_config.mode = mode;
	client_config.blockchain.max_cache_size = cache_config.blockchain as usize;
	client_config.blockchain.pref_cache_size = cache_config.blockchain as usize * 3 / 4;
	client_config.blockchain.db_cache_size = Some(cache_config.rocksdb_blockchain_cache_size() as usize);
	// state db cache size
	client_config.db_cache_size = Some(cache_config.rocksdb_state_cache_size() as usize);
	client_config.tracing.enabled = tracing;
	// chose best db here (requires state root hash)
	client_config.pruning = pruning.unwrap_or_else(|| { unimplemented!(); });
	client_config.db_compaction = compaction;
	client_config
}

fn execute_import(cmd: ImportBlockchain) -> Result<String, String> {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// load spec file
	let spec = try!(cmd.spec.spec());

	// setup network configuration
	let net_settings = NetworkConfiguration {
		config_path: None,
		listen_address: None,
		public_address: None,
		udp_port: None,
		nat_enabled: false,
		discovery_enabled: false,
		boot_nodes: vec![],
		use_secret: None,
		ideal_peers: 0,
		reserved_nodes: vec![],
		non_reserved_mode: ::util::network::NonReservedPeerMode::Accept,
	};

	// Setup logging
	let _logger = setup_log(&cmd.logger_config.mode, cmd.logger_config.color);

	unsafe { fdlimit::raise_fd_limit(); }

	info!("Starting {}", Colour::White.bold().paint(version()));

	let cfg = client_config(&cmd.cache_config, cmd.mode, cmd.tracing, cmd.pruning, cmd.compaction);

	// build client
	let service = ClientService::start(
		cfg, spec, net_settings, Path::new(&cmd.db_path), Arc::new(Miner::with_spec(try!(cmd.spec.spec()))), false
		// TODO: pretty error
	).unwrap();

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
			let first_read = try!(instream.read(&mut first_bytes).map_err(|_| "Error reading from the file/stream."));
			match first_bytes[0] {
				0xf9 => DataFormat::Binary,
				_ => DataFormat::Hex,
			}
		}
	};

	let informant = Informant::new(cmd.logger_config.color);

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
		informant.tick::<&'static ()>(&client, None);
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
				try!(instream.read_exact(&mut bytes[READAHEAD_BYTES..]).map_err(|_| "Error reading from the file/stream."));
				do_import(bytes);
			}
		}
		DataFormat::Hex => {
			for line in BufReader::new(instream).lines() {
				let s = try!(line.map_err(|_| "Error reading from the file/stream."));
				let s = if first_read > 0 {from_utf8(&first_bytes).unwrap().to_owned() + &(s[..])} else {s};
				first_read = 0;
				let bytes = try!(s.from_hex().map_err(|_| "Invalid hex in file/stream."));
				do_import(bytes);
			}
		}
	}
	client.flush_queue();

	unimplemented!();
}

fn execute_export(cmd: ExportBlockchain) -> Result<String, String> {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// load spec file
	let spec = try!(cmd.spec.spec());

	// setup network configuration
	let net_settings = NetworkConfiguration {
		config_path: None,
		listen_address: None,
		public_address: None,
		udp_port: None,
		nat_enabled: false,
		discovery_enabled: false,
		boot_nodes: vec![],
		use_secret: None,
		ideal_peers: 0,
		reserved_nodes: vec![],
		non_reserved_mode: ::util::network::NonReservedPeerMode::Accept,
	};

	// Setup logging
	let _logger = setup_log(&cmd.logger_config.mode, cmd.logger_config.color);

	unsafe { fdlimit::raise_fd_limit(); }

	info!("Starting {}", Colour::White.bold().paint(version()));

	let cfg = client_config(&cmd.cache_config, cmd.mode, cmd.tracing, cmd.pruning, cmd.compaction);

	unimplemented!();
}

#[cfg(test)]
mod test {
	use std::str::FromStr;
	use ethcore::ethereum;
	use super::{DataFormat, SpecType};

	#[test]
	fn test_data_format_parsing() {
		assert_eq!(DataFormat::from_str("binary").unwrap(), DataFormat::Binary);
		assert_eq!(DataFormat::from_str("bin").unwrap(), DataFormat::Binary);
		assert_eq!(DataFormat::from_str("hex").unwrap(), DataFormat::Hex);
	}

	#[test]
	fn test_spec_type_parsing() {
		assert_eq!(SpecType::from_str("frontier").unwrap(), SpecType::Mainnet);
		assert_eq!(SpecType::from_str("homestead").unwrap(), SpecType::Mainnet);
		assert_eq!(SpecType::from_str("mainnet").unwrap(), SpecType::Mainnet);
		assert_eq!(SpecType::from_str("morden").unwrap(), SpecType::Testnet);
		assert_eq!(SpecType::from_str("testnet").unwrap(), SpecType::Testnet);
		assert_eq!(SpecType::from_str("olympic").unwrap(), SpecType::Olympic);
	}
}
