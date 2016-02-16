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
use serde_json::value;
use jsonrpc_core::Value;
use util::hash::*;
use v1::types::BlockNumber;

#[derive(Debug, PartialEq)]
pub enum Topic {
	Single(H256),
	Multiple(Vec<H256>),
	Null,
}

impl Deserialize for Topic {
	fn deserialize<D>(deserializer: &mut D) -> Result<Topic, D::Error>
		where D: Deserializer,
	{
		let v = try!(Value::deserialize(deserializer));

		if v.is_null() {
			return Ok(Topic::Null);
		}

		Deserialize::deserialize(&mut value::Deserializer::new(v.clone()))
			.map(Topic::Single)
			.or_else(|_| Deserialize::deserialize(&mut value::Deserializer::new(v.clone())).map(Topic::Multiple))
			.map_err(|_| Error::syntax("")) // unreachable, but types must match
	}
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Filter {
	#[serde(rename="fromBlock")]
	pub from_block: Option<BlockNumber>,
	#[serde(rename="toBlock")]
	pub to_block: Option<BlockNumber>,
	pub address: Option<Address>,
	pub topics: Option<Vec<Topic>>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::str::FromStr;
	use util::hash::*;
	use super::*;
	use v1::types::BlockNumber;

	#[test]
	fn topic_deserialization() {
		let s = r#"["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b", null, ["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b", "0x0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc"]]"#;
		let deserialized: Vec<Topic> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized,
		           vec![Topic::Single(H256::from_str("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap()),
		                Topic::Null,
		                Topic::Multiple(vec![H256::from_str("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap(),
		                                     H256::from_str("0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc").unwrap()])]);
	}

	#[test]
	fn filter_deserialization() {
		let s = r#"{"fromBlock":"earliest","toBlock":"latest"}"#;
		let deserialized: Filter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized,
		           Filter {
			           from_block: Some(BlockNumber::Earliest),
			           to_block: Some(BlockNumber::Latest),
			           address: None,
			           topics: None,
		           });
	}
}
