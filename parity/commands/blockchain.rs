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

use std::str::FromStr;
use util::panics::PanicHandler;
use util::journaldb;
use util::{version, contents, NetworkConfiguration};
use util::log::Colour;
use ethcore::client::ClientConfig;
use ethcore::spec::Spec;
use ethcore::ethereum;
use cache::CacheConfig;
use setup_log::setup_log;
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
	Export,
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
	//pub client_config: ClientConfig,
	pub db_path: String,
	pub file_path: Option<String>,
	pub format: Option<DataFormat>,
	pub pruning: Option<journaldb::Algorithm>,
}

pub fn execute(cmd: BlockchainCmd) -> Result<String, String> {
	match cmd {
		BlockchainCmd::Import(import_cmd) => execute_import(import_cmd),
		BlockchainCmd::Export => execute_export(),
	}
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


	unimplemented!();
}

fn execute_export() -> Result<String, String> {
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
