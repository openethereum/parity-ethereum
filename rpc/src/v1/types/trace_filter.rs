// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Trace filter deserialization.

use ethereum_types::H160;
use types::{
	ids::BlockId,
	trace_filter::Filter,
};
use v1::types::BlockNumber;

/// Trace filter
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TraceFilter {
	/// From block
	pub from_block: Option<BlockNumber>,
	/// To block
	pub to_block: Option<BlockNumber>,
	/// From address
	pub from_address: Option<Vec<H160>>,
	/// To address
	pub to_address: Option<Vec<H160>>,
	/// Output offset
	pub after: Option<usize>,
	/// Output amount
	pub count: Option<usize>,
}

impl Into<Filter> for TraceFilter {
	fn into(self) -> Filter {
		let num_to_id = |num| match num {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(n) => BlockId::Number(n),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,
			BlockNumber::Pending => {
				warn!("Pending traces are not supported and might be removed in future versions. Falling back to Latest");
				BlockId::Latest
			}
		};
		let start = self.from_block.map_or(BlockId::Latest, &num_to_id);
		let end = self.to_block.map_or(BlockId::Latest, &num_to_id);
		Filter {
			range: start..end,
			from_address: self.from_address.map_or_else(Vec::new, |x| x.into_iter().map(Into::into).collect()),
			to_address: self.to_address.map_or_else(Vec::new, |x| x.into_iter().map(Into::into).collect()),
			after: self.after,
			count: self.count,
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use ethereum_types::Address;
	use v1::types::{BlockNumber, TraceFilter};

	#[test]
	fn test_empty_trace_filter_deserialize() {
		let s = r#"{}"#;
		let deserialized: TraceFilter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, TraceFilter {
			from_block: None,
			to_block: None,
			from_address: None,
			to_address: None,
			after: None,
			count: None,
		});
	}

	#[test]
	fn test_trace_filter_deserialize() {
		let s = r#"{
			"fromBlock": "latest",
			"toBlock": "latest",
			"fromAddress": ["0x0000000000000000000000000000000000000003"],
			"toAddress": ["0x0000000000000000000000000000000000000005"],
			"after": 50,
			"count": 100
		}"#;
		let deserialized: TraceFilter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, TraceFilter {
			from_block: Some(BlockNumber::Latest),
			to_block: Some(BlockNumber::Latest),
			from_address: Some(vec![Address::from_low_u64_be(3).into()]),
			to_address: Some(vec![Address::from_low_u64_be(5).into()]),
			after: 50.into(),
			count: 100.into(),
		});
	}
}
