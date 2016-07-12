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
use ethcore::spec::Spec;
use ethcore::ethereum;
use util::contents;

#[derive(Eq, PartialEq, Debug)]
pub enum Policy {
	None,
	Dogmatic,
}

impl FromStr for Policy {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"none" => Ok(Policy::None),
			"dogmatic" => Ok(Policy::Dogmatic),
			other => Err(format!("Invalid policy value: {}", other)),
		}
	}
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
	pub fn spec(&self) -> Result<Spec, String> {
		match *self {
			SpecType::Mainnet => Ok(ethereum::new_frontier()),
			SpecType::Testnet => Ok(ethereum::new_morden()),
			SpecType::Olympic => Ok(ethereum::new_olympic()),
			SpecType::Custom(ref file) => Ok(Spec::load(&try!(contents(file).map_err(|_| "Could not load specification file."))))
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct LoggerConfig {
	pub mode: Option<String>,
	pub color: bool,
}

#[cfg(test)]
mod tests {
	use super::{Policy, SpecType};

	#[test]
	fn test_policy_parsing() {
		assert_eq!(Policy::None, "none".parse().unwrap());
		assert_eq!(Policy::Dogmatic, "dogmatic".parse().unwrap());
		assert!("sas".parse::<Policy>().is_err());
	}

	#[test]
	fn test_spec_type_parsing() {
		assert_eq!(SpecType::Mainnet, "frontier".parse().unwrap());
		assert_eq!(SpecType::Mainnet, "homestead".parse().unwrap());
		assert_eq!(SpecType::Mainnet, "mainnet".parse().unwrap());
		assert_eq!(SpecType::Testnet, "testnet".parse().unwrap());
		assert_eq!(SpecType::Testnet, "morden".parse().unwrap());
		assert_eq!(SpecType::Olympic, "olympic".parse().unwrap());
	}
}
