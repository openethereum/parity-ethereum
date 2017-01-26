// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Trace filter deserialization.

use ethcore::client::BlockId;
use ethcore::client;
use v1::types::{BlockNumber, H160};

/// Trace filter
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceFilter {
	/// From block
	#[serde(rename="fromBlock")]
	pub from_block: Option<BlockNumber>,
	/// To block
	#[serde(rename="toBlock")]
	pub to_block: Option<BlockNumber>,
	/// From address
	#[serde(rename="fromAddress")]
	pub from_address: Option<Vec<H160>>,
	/// To address
	#[serde(rename="toAddress")]
	pub to_address: Option<Vec<H160>>,
}

impl Into<client::TraceFilter> for TraceFilter {
	fn into(self) -> client::TraceFilter {
		let start = self.from_block.map_or(BlockId::Latest, Into::into);
		let end = self.to_block.map_or(BlockId::Latest, Into::into);
		client::TraceFilter {
			range: start..end,
			from_address: self.from_address.map_or_else(Vec::new, |x| x.into_iter().map(Into::into).collect()),
			to_address: self.to_address.map_or_else(Vec::new, |x| x.into_iter().map(Into::into).collect()),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use util::Address;
	use v1::types::{BlockNumber, TraceFilter};

	#[test]
	fn test_empty_trace_filter_deserialize() {
		let s = r#"{}"#;
		let deserialized: TraceFilter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, TraceFilter {
			from_block: None,
			to_block: None,
			from_address: None,
			to_address: None
		});
	}

	#[test]
	fn test_trace_filter_deserialize() {
		let s = r#"{
			"fromBlock": "latest",
			"toBlock": "latest",
			"fromAddress": ["0x0000000000000000000000000000000000000003"],
			"toAddress": ["0x0000000000000000000000000000000000000005"]
		}"#;
		let deserialized: TraceFilter = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, TraceFilter {
			from_block: Some(BlockNumber::Latest),
			to_block: Some(BlockNumber::Latest),
			from_address: Some(vec![Address::from(3).into()]),
			to_address: Some(vec![Address::from(5).into()]),
		});
	}
}
