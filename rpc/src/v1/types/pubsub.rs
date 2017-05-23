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

//! Pub-Sub types.

use serde::{Serialize, Serializer};
use v1::types::{RichHeader, Filter};

/// Subscription result.
#[derive(Debug, PartialEq, Eq)]
pub enum Result {
	/// New block header.
	Header(RichHeader),
}

impl Serialize for Result {
	fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
		where S: Serializer
	{
		match *self {
			Result::Header(ref header) => header.serialize(serializer),
		}
	}
}

/// Subscription kind.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum Kind {
	/// New block headers subscription.
	#[serde(rename="newHeads")]
	NewHeads,
	/// Logs subscription.
	#[serde(rename="logs")]
	Logs,
	/// New Pending Transactions subscription.
	#[serde(rename="newPendingTransactions")]
	NewPendingTransactions,
	/// Node syncing status subscription.
	#[serde(rename="syncing")]
	Syncing,
}

/// Subscription kind.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum Params {
	/// No parameters passed.
	None,
	/// Log parameters.
	Logs(Filter),
}

impl Default for Params {
	fn default() -> Self {
		Params::None
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::{Result, Kind};
	use v1::types::{RichHeader, Header};

	#[test]
	fn should_deserialize_kind() {
		assert_eq!(serde_json::from_str::<Kind>(r#""newHeads""#).unwrap(), Kind::NewHeads);
		assert_eq!(serde_json::from_str::<Kind>(r#""logs""#).unwrap(), Kind::Logs);
		assert_eq!(serde_json::from_str::<Kind>(r#""newPendingTransactions""#).unwrap(), Kind::NewPendingTransactions);
		assert_eq!(serde_json::from_str::<Kind>(r#""syncing""#).unwrap(), Kind::Syncing);
	}

	#[test]
	fn should_serialize_header() {
		let header = Result::Header(RichHeader {
			extra_info: Default::default(),
			inner: Header {
				hash: Some(Default::default()),
				parent_hash: Default::default(),
				uncles_hash: Default::default(),
				author: Default::default(),
				miner: Default::default(),
				state_root: Default::default(),
				transactions_root: Default::default(),
				receipts_root: Default::default(),
				number: Some(Default::default()),
				gas_used: Default::default(),
				gas_limit: Default::default(),
				extra_data: Default::default(),
				logs_bloom: Default::default(),
				timestamp: Default::default(),
				difficulty: Default::default(),
				seal_fields: vec![Default::default(), Default::default()],
				size: Some(69.into()),
			},
		});
		let expected = r#"{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x0","extraData":"0x","gasLimit":"0x0","gasUsed":"0x0","hash":"0x0000000000000000000000000000000000000000000000000000000000000000","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","sealFields":["0x","0x"],"sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","size":"0x45","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#;
		assert_eq!(serde_json::to_string(&header).unwrap(), expected);
	}
}
