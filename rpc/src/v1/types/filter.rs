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
use util::numbers::*;
use v1::types::BlockNumber;
use ethcore::filter::Filter as EthFilter;
use ethcore::client::BlockId;

#[derive(Debug, PartialEq)]
pub enum VariadicValue<T> where T: Deserialize {
	Single(T),
	Multiple(Vec<T>),
	Null,
}

impl<T> Deserialize for VariadicValue<T> where T: Deserialize {
	fn deserialize<D>(deserializer: &mut D) -> Result<VariadicValue<T>, D::Error>
	where D: Deserializer {
		let v = try!(Value::deserialize(deserializer));

		if v.is_null() {
			return Ok(VariadicValue::Null);
		}

		Deserialize::deserialize(&mut value::Deserializer::new(v.clone())).map(VariadicValue::Single)
			.or_else(|_| Deserialize::deserialize(&mut value::Deserializer::new(v.clone())).map(VariadicValue::Multiple))
			.map_err(|_| Error::custom("")) // unreachable, but types must match
	}
}

pub type FilterAddress = VariadicValue<Address>;
pub type Topic = VariadicValue<H256>;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
	#[serde(rename="fromBlock")]
	pub from_block: Option<BlockNumber>,
	#[serde(rename="toBlock")]
	pub to_block: Option<BlockNumber>,
	pub address: Option<FilterAddress>,
	pub topics: Option<Vec<Topic>>,
}

impl Into<EthFilter> for Filter {
	fn into(self) -> EthFilter {
		EthFilter {
			from_block: self.from_block.map_or_else(|| BlockId::Latest, Into::into),
			to_block: self.to_block.map_or_else(|| BlockId::Latest, Into::into),
			address: self.address.and_then(|address| match address {
				VariadicValue::Null => None,
				VariadicValue::Single(a) => Some(vec![a]),
				VariadicValue::Multiple(a) => Some(a)
			}),
			topics: {
				let mut iter = self.topics.map_or_else(Vec::new, |topics| topics.into_iter().take(4).map(|topic| match topic {
					VariadicValue::Null => None,
					VariadicValue::Single(t) => Some(vec![t]),
					VariadicValue::Multiple(t) => Some(t)
				}).filter_map(|m| m).collect()).into_iter();
				[iter.next(), iter.next(), iter.next(), iter.next()]
			}
		}
	}
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
		assert_eq!(deserialized, vec![
				   VariadicValue::Single(H256::from_str("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap()),
				   VariadicValue::Null,
				   VariadicValue::Multiple(vec![
								   H256::from_str("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap(),
								   H256::from_str("0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc").unwrap()
				   ])
		]);
	}

	#[test]
	fn filter_deserialization() {
		let s = r#"{"fromBlock":"earliest","toBlock":"latest"}"#;
		let deserialized: Filter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, Filter {
			from_block: Some(BlockNumber::Earliest),
			to_block: Some(BlockNumber::Latest),
			address: None,
			topics: None
		});
	}
}
