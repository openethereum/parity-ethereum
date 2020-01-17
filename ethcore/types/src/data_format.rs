// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Data format for importing/exporting blocks from disk
use std::str::FromStr;

/// Format for importing/exporting blocks
#[derive(Debug, PartialEq)]
pub enum DataFormat {
	/// Hexadecimal format
	Hex,
	/// Binary format
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
