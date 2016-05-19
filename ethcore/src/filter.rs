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

//! Blockchain filter

use util::hash::*;
use util::sha3::*;
use client::BlockID;
use log_entry::LogEntry;

/// Blockchain Filter.
pub struct Filter {
	/// Blockchain will be searched from this block.
	pub from_block: BlockID,

	/// Till this block.
	pub to_block: BlockID,

	/// Search addresses. 
	/// 
	/// If None, match all.
	/// If specified, log must be produced by one of these addresses.
	pub address: Option<Vec<Address>>,

	/// Search topics.
	/// 
	/// If None, match all.
	/// If specified, log must contain one of these topics.
	pub topics: [Option<Vec<H256>>; 4],
}

impl Clone for Filter {
	fn clone(&self) -> Self {
		let mut topics = [None, None, None, None];
		for i in 0..4 {
			topics[i] = self.topics[i].clone();
		}

		Filter {
			from_block: self.from_block.clone(),
			to_block: self.to_block.clone(),
			address: self.address.clone(),
			topics: topics
		}
	}
}

impl Filter {
	/// Returns combinations of each address and topic.	
	pub fn bloom_possibilities(&self) -> Vec<H2048> {
		let blooms = match self.address {
			Some(ref addresses) if !addresses.is_empty() =>
				addresses.iter().map(|ref address| {
					let mut bloom = H2048::new();
					bloom.shift_bloomed(&address.sha3());
					bloom
				}).collect(),
			_ => vec![H2048::new()]
		};

		self.topics.iter().fold(blooms, | bs, topic | match *topic {
			None => bs,
			Some(ref topics) => bs.into_iter().flat_map(|bloom| {
				topics.into_iter().map(|topic| {
					let mut b = bloom.clone();
					b.shift_bloomed(&topic.sha3());
					b
				}).collect::<Vec<H2048>>()
			}).collect()
		})
	}

	/// Returns true if given log entry matches filter.
	pub fn matches(&self, log: &LogEntry) -> bool {
		let matches = match self.address {
			Some(ref addresses) if !addresses.is_empty() =>	addresses.iter().any(|address| &log.address == address),
			_ => true
		};

		matches && self.topics.iter().enumerate().all(|(i, topic)| match *topic {
			Some(ref topics) if !topics.is_empty() => topics.iter().any(|topic| log.topics.get(i) == Some(topic)),
			_ => true
		})
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use filter::Filter;
	use client::BlockID;
	use log_entry::LogEntry;

	#[test]
	fn test_bloom_possibilities_none() {
		let none_filter = Filter {
			from_block: BlockID::Earliest,
			to_block: BlockID::Latest,
			address: None,
			topics: [None, None, None, None]
		};

		let possibilities = none_filter.bloom_possibilities();
		assert_eq!(possibilities.len(), 1);
		assert!(possibilities[0].is_zero())
	}

	// block 399849
	#[test]
	fn test_bloom_possibilities_single_address_and_topic() {
		let filter = Filter {
			from_block: BlockID::Earliest,
			to_block: BlockID::Latest,
			address: Some(vec![Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap()]),
			topics: [
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()]),
				None, None, None
			]
		};

		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities, vec![H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()]);
	}

	#[test]
	fn test_bloom_possibilities_single_address_and_many_topics() {
		let filter = Filter {
			from_block: BlockID::Earliest,
			to_block: BlockID::Latest,
			address: Some(vec![Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap()]),
			topics: [
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()]),
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()]),
				None, None
			]
		};

		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities, vec![H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()]);
	}

	#[test]
	fn test_bloom_possibilites_multiple_addresses_and_topics() {
		let filter = Filter {
			from_block: BlockID::Earliest,
			to_block: BlockID::Latest,
			address: Some(vec![
						  Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
						  Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
			]),
			topics: [
				Some(vec![
					 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
					 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()
				]),
				Some(vec![
					 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
					 H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()
				]),
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()]),
				None
			]
		};

		// number of possibilites should be equal 2 * 2 * 2 * 1 = 8
		let possibilities = filter.bloom_possibilities();
		assert_eq!(possibilities.len(), 8);
		assert_eq!(possibilities[0], H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
	}

	#[test]
	fn test_filter_matches() {
		let filter = Filter {
			from_block: BlockID::Earliest,
			to_block: BlockID::Latest,
			address: Some(vec![Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap()]),
			topics: [
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap()]),
				Some(vec![H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa").unwrap()]),
				None, None
			]
		};

		let entry0 = LogEntry {
			address: Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
			topics: vec![
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa").unwrap(),
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
			],
			data: vec![]
		};

		let entry1 = LogEntry {
			address: Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5e").unwrap(),
			topics: vec![
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa").unwrap(),
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
			],
			data: vec![]
		};

		let entry2 = LogEntry {
			address: Address::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
			topics: vec![
				H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9").unwrap(),
			],
			data: vec![]
		};

		assert_eq!(filter.matches(&entry0), true);
		assert_eq!(filter.matches(&entry1), false);
		assert_eq!(filter.matches(&entry2), false);
	}
}
