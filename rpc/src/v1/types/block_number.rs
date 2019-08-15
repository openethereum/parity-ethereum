// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor, MapAccess};
use types::ids::BlockId;
use ethereum_types::H256;

/// Represents rpc api block number param.
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum BlockNumber {
	/// Hash
	Hash {
		/// block hash
		hash: H256,
		/// only return blocks part of the canon chain
		require_canonical: bool,
	},
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
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
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
			BlockNumber::Hash{ hash, require_canonical } => serializer.serialize_str(
				&format!("{{ 'hash': '{}', 'requireCanonical': '{}'  }}", hash, require_canonical)
			),
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

	fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapAccess<'a> {
		let (mut require_canonical, mut block_number, mut block_hash) = (false, None::<u64>, None::<H256>);

		loop {
			let key_str: Option<String> = visitor.next_key()?;

			match key_str {
				Some(key) => match key.as_str() {
					"blockNumber" => {
						let value: String = visitor.next_value()?;
						if value.starts_with("0x") {
							let number = u64::from_str_radix(&value[2..], 16).map_err(|e| {
								Error::custom(format!("Invalid block number: {}", e))
							})?;

							block_number = Some(number);
							break;
						} else {
							return Err(Error::custom("Invalid block number: missing 0x prefix".to_string()))
						}
					}
					"blockHash" => {
						block_hash = Some(visitor.next_value()?);
					}
					"requireCanonical" => {
						require_canonical = visitor.next_value()?;
					}
					key => {
						return Err(Error::custom(format!("Unknown key: {}", key)))
					}
				}
				None => {
					break
				}
			};
		}

		if let Some(number) = block_number {
			return Ok(BlockNumber::Num(number))
		}

		if let Some(hash) = block_hash {
			return Ok(BlockNumber::Hash { hash, require_canonical })
		}

		return Err(Error::custom("Invalid input"))
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"latest" => Ok(BlockNumber::Latest),
			"earliest" => Ok(BlockNumber::Earliest),
			"pending" => Ok(BlockNumber::Pending),
			_ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16).map(BlockNumber::Num).map_err(|e| {
				Error::custom(format!("Invalid block number: {}", e))
			}),
			_ => {
				Err(Error::custom("Invalid block number: missing 0x prefix".to_string()))
			},
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

/// Converts `BlockNumber` to `BlockId`, panics on `BlockNumber::Pending`
pub fn block_number_to_id(number: BlockNumber) -> BlockId {
	match number {
		BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
		BlockNumber::Num(num) => BlockId::Number(num),
		BlockNumber::Earliest => BlockId::Earliest,
		BlockNumber::Latest => BlockId::Latest,
		BlockNumber::Pending => panic!("`BlockNumber::Pending` should be handled manually")
	}
}

#[cfg(test)]
mod tests {
	use types::ids::BlockId;
	use super::*;
	use std::str::FromStr;
	use serde_json;

	#[test]
	fn block_number_deserialization() {
		let s = r#"[
			"0xa",
			"latest",
			"earliest",
			"pending",
			{"blockNumber": "0xa"},
			{"blockHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"},
			{"blockHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347", "requireCanonical": true}
		]"#;
		let deserialized: Vec<BlockNumber> = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			vec![
				BlockNumber::Num(10),
				BlockNumber::Latest,
				BlockNumber::Earliest,
				BlockNumber::Pending,
				BlockNumber::Num(10),
				BlockNumber::Hash { hash: H256::from_str("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").unwrap(), require_canonical: false },
				BlockNumber::Hash { hash: H256::from_str("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").unwrap(), require_canonical: true }
			]
		)
	}

	#[test]
	fn should_not_deserialize() {
		let s = r#"[{}, "10"]"#;
		assert!(serde_json::from_str::<Vec<BlockNumber>>(s).is_err());
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
