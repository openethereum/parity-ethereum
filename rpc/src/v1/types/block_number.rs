// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use ethcore::client::BlockId;

/// Represents rpc api block number param.
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum BlockNumber {
	/// Number
	Num(u64),
	/// Latest block
	Latest,
	/// Earliest block (genesis)
	Earliest,
	/// Pending block (being mined)
	Pending,
}

impl Default for BlockNumber {
	fn default() -> Self {
		BlockNumber::Latest
	}
}

impl<'a> Deserialize<'a> for BlockNumber {
	fn deserialize<D>(deserializer: D) -> Result<BlockNumber, D::Error> where D: Deserializer<'a> {
		deserializer.deserialize_any(BlockNumberVisitor)
	}
}

impl BlockNumber {
	/// Convert block number to min block target.
	pub fn to_min_block_num(&self) -> Option<u64> {
		match *self {
			BlockNumber::Num(ref x) => Some(*x),
			_ => None,
		}
	}
}

/// BlockNumber to BlockId conversion
///
/// NOTE use only for light clients.
pub trait LightBlockNumber {
	/// Convert block number to block id.
	fn to_block_id(self) -> BlockId;
}

impl LightBlockNumber for BlockNumber {
	fn to_block_id(self) -> BlockId {
		// NOTE Here we treat `Pending` as `Latest`.
		// Since light clients don't produce pending blocks
		// (they don't have state) we can safely fallback to `Latest`.
		match self {
			BlockNumber::Num(n) => BlockId::Number(n),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,
			BlockNumber::Pending => {
				warn!("`Pending` is deprecated and may be removed in future versions. Falling back to `Latest`");
				BlockId::Latest
			}
		}
	}
}

impl Serialize for BlockNumber {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		match *self {
			BlockNumber::Num(ref x) => serializer.serialize_str(&format!("0x{:x}", x)),
			BlockNumber::Latest => serializer.serialize_str("latest"),
			BlockNumber::Earliest => serializer.serialize_str("earliest"),
			BlockNumber::Pending => serializer.serialize_str("pending"),
		}
	}
}

struct BlockNumberVisitor;

impl<'a> Visitor<'a> for BlockNumberVisitor {
	type Value = BlockNumber;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a block number or 'latest', 'earliest' or 'pending'")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"latest" => Ok(BlockNumber::Latest),
			"earliest" => Ok(BlockNumber::Earliest),
			"pending" => Ok(BlockNumber::Pending),
			_ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16).map(BlockNumber::Num).map_err(|e| {
				Error::custom(format!("Invalid block number: {}", e))
			}),
			_ => Err(Error::custom(format!("Invalid block number: missing 0x prefix"))),
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

/// Converts `BlockNumber` to `BlockId`, panics on `BlockNumber::Pending`
pub fn block_number_to_id(number: BlockNumber) -> BlockId {
	match number {
		BlockNumber::Num(num) => BlockId::Number(num),
		BlockNumber::Earliest => BlockId::Earliest,
		BlockNumber::Latest => BlockId::Latest,

		BlockNumber::Pending => panic!("`BlockNumber::Pending` should be handled manually")
	}
}

#[cfg(test)]
mod tests {
	use ethcore::client::BlockId;
	use super::*;
	use serde_json;

	#[test]
	fn block_number_deserialization() {
		let s = r#"["0xa", "latest", "earliest", "pending"]"#;
		let deserialized: Vec<BlockNumber> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![BlockNumber::Num(10), BlockNumber::Latest, BlockNumber::Earliest, BlockNumber::Pending])
	}

	#[test]
	fn should_not_deserialize_decimal() {
		let s = r#""10""#;
		assert!(serde_json::from_str::<BlockNumber>(s).is_err());
	}

	#[test]
	fn normal_block_number_to_id() {
		assert_eq!(block_number_to_id(BlockNumber::Num(100)), BlockId::Number(100));
		assert_eq!(block_number_to_id(BlockNumber::Earliest), BlockId::Earliest);
		assert_eq!(block_number_to_id(BlockNumber::Latest), BlockId::Latest);
	}

	#[test]
	#[should_panic]
	fn pending_block_number_to_id() {
		// Since this function is not allowed to be called in such way, panic should happen
		block_number_to_id(BlockNumber::Pending);
	}
}
