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

use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;
use ethcore::client::BlockID;

/// Represents rpc api block number param.
#[derive(Debug, PartialEq, Clone)]
pub enum BlockNumber {
	Num(u64),
	Latest,
	Earliest,
	Pending
}

impl Deserialize for BlockNumber {
	fn deserialize<D>(deserializer: &mut D) -> Result<BlockNumber, D::Error>
	where D: Deserializer {
		deserializer.deserialize(BlockNumberVisitor)
	}
}

struct BlockNumberVisitor;

impl Visitor for BlockNumberVisitor {
	type Value = BlockNumber;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"latest" => Ok(BlockNumber::Latest),
			"earliest" => Ok(BlockNumber::Earliest),
			"pending" => Ok(BlockNumber::Pending),
			_ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16).map(BlockNumber::Num).map_err(|_| Error::custom("invalid block number")),
			_ => value.parse::<u64>().map(BlockNumber::Num).map_err(|_| Error::custom("invalid block number"))
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

impl Into<BlockID> for BlockNumber {
	fn into(self) -> BlockID {
		match self {
			BlockNumber::Num(n) => BlockID::Number(n),
			BlockNumber::Earliest => BlockID::Earliest,
			// TODO: change this once blockid support pendingst,
			BlockNumber::Pending | BlockNumber::Latest => BlockID::Latest,
		}
	}
}

#[cfg(test)]
mod tests {
	use ethcore::client::BlockID;
	use super::*;
	use serde_json;

	#[test]
	fn block_number_deserialization() {
		let s = r#"["0xa", "10", "latest", "earliest", "pending"]"#;
		let deserialized: Vec<BlockNumber> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![BlockNumber::Num(10), BlockNumber::Num(10), BlockNumber::Latest, BlockNumber::Earliest, BlockNumber::Pending])
	}

	#[test]
	fn block_number_into() {
		assert_eq!(BlockID::Number(100), BlockNumber::Num(100).into());
		assert_eq!(BlockID::Earliest, BlockNumber::Earliest.into());
		assert_eq!(BlockID::Latest, BlockNumber::Latest.into());
		assert_eq!(BlockID::Latest, BlockNumber::Pending.into());
	}
}

