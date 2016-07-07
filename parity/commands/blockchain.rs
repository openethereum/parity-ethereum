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
use ethcore::client::ClientConfig;
use ethcore::spec::Spec;

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

#[derive(Debug, PartialEq)]
pub struct ImportBlockchain {
	pub spec: SpecType,
	pub logger_config: LoggerConfig,
	//pub client_config: ClientConfig,
	pub db_path: String,
	pub file_path: Option<String>,
	pub format: Option<DataFormat>,
}

#[cfg(test)]
mod test {
	use std::str::FromStr;
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
