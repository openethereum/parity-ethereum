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
use util::sha3::*;
use v1::types::BlockNumber;

#[derive(Debug, PartialEq)]
pub enum VariadicValue<T> where T: Deserialize {
	Single(T),
	Multiple(Vec<T>),
	Null
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
			.map_err(|_| Error::syntax("")) // unreachable, but types must match
	}
}

pub type FilterAddress = VariadicValue<Address>;
pub type Topic = VariadicValue<H256>;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Filter {
	#[serde(rename="fromBlock")]
	pub from_block: Option<BlockNumber>,
	#[serde(rename="toBlock")]
	pub to_block: Option<BlockNumber>,
	pub address: Option<FilterAddress>,
	pub topics: Option<Vec<Topic>>
}

impl Filter {
	/// Returns combinations of each address and topic.	
	pub fn bloom_possibilities(&self) -> Vec<H2048> {
		let blooms = match self.address {
			Some(VariadicValue::Single(ref address)) => {
				let mut bloom = H2048::new();
				bloom.shift_bloomed(&address.sha3());
				vec![bloom]
			},
			Some(VariadicValue::Multiple(ref addresses)) => {
				addresses.iter().map(|ref address| {
					let mut bloom = H2048::new();
					bloom.shift_bloomed(&address.sha3());
					bloom
				}).collect()
			},
			_ => vec![H2048::new()]
		};

		match self.topics {
			None => blooms,
			Some(ref topics) => topics.iter().fold(blooms, | bs, topic | match *topic {
				VariadicValue::Null => bs,
				VariadicValue::Single(ref topic) => bs.into_iter().map(|mut bloom| {
					bloom.shift_bloomed(&topic.sha3());
					bloom
				}).collect(),
				VariadicValue::Multiple(ref topics) => bs.into_iter().map(|bloom| {
					topics.into_iter().map(|topic| {
						let mut b = bloom.clone();
						b.shift_bloomed(&topic.sha3());
						b
					}).collect::<Vec<H2048>>()
				}).flat_map(|m| m).collect::<Vec<H2048>>()
			})
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

	#[test]
	fn test_bloom_possibilities_none() {
		let none_filter = Filter {
			from_block: None,
			to_block: None,
			address: None,
			topics: None
		};

		let possibilities = none_filter.bloom_possibilities();
		assert_eq!(possibilities, vec![H2048::new()]); 
	}

	// block 399849
	#[test]
	fn test_bloom_possibilities_single_address_and_topic() {
		let filter = Filter {
			from_block: None,
			to_block: None,
			address: Some(VariadicValue::Single(Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap())),
			topics: Some(vec![VariadicValue::Single(H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap())])
		};

		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities, vec![H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()]);
	}

	#[test]
	fn test_bloom_possibilities_single_address_and_many_topics() {
		let filter = Filter {
			from_block: None,
			to_block: None,
			address: Some(VariadicValue::Single(Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap())),
			topics: Some(vec![
						 VariadicValue::Single(H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()),
						 VariadicValue::Single(H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap())
			])
		};

		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities, vec![H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()]);
	}

	#[test]
	fn test_bloom_possibilites_multiple_addresses_and_topics() {
		let filter = Filter {
			from_block: None,
			to_block: None,
			address: Some(VariadicValue::Multiple(vec![
												  Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
												  Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap()
			])),
			topics: Some(vec![
						 VariadicValue::Multiple(vec![
												 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
												 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()
						 ]),
						 VariadicValue::Multiple(vec![
												 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
												 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()
						 ]),
						 VariadicValue::Single(H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap())
			])
		};

		// number of possibilites should be equal 2 * 2 * 2 * 1 = 8
		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities.len(), 8);
		assert_eq!(possibilities[0], H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
	}
}
