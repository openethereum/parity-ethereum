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

//! Pub-Sub types.

use ethereum_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde_json::{Value, from_value};
use v1::types::{RichHeader, Filter, Log};

/// Subscription result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
	/// New block header.
	Header(Box<RichHeader>),
	/// Log
	Log(Box<Log>),
	/// Transaction hash
	TransactionHash(H256),
	/// SyncStatus
	SyncState(PubSubSyncStatus)
}

/// PubSbub sync status
#[derive(Debug, Serialize, Eq, PartialEq, Clone)]
#[serde(rename_all="camelCase")]
pub struct PubSubSyncStatus {
	/// is_major_syncing?
	pub syncing: bool,
}

impl Serialize for Result {
	fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
		where S: Serializer
	{
		match *self {
			Result::Header(ref header) => header.serialize(serializer),
			Result::Log(ref log) => log.serialize(serializer),
			Result::TransactionHash(ref hash) => hash.serialize(serializer),
			Result::SyncState(ref sync) => sync.serialize(serializer),
		}
	}
}

/// Subscription kind.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
	/// New block headers subscription.
	NewHeads,
	/// Logs subscription.
	Logs,
	/// New Pending Transactions subscription.
	NewPendingTransactions,
	/// Node syncing status subscription.
	Syncing,
}

/// Subscription kind.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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

impl<'a> Deserialize<'a> for Params {
	fn deserialize<D>(deserializer: D) -> ::std::result::Result<Params, D::Error>
	where D: Deserializer<'a> {
		let v: Value = Deserialize::deserialize(deserializer)?;

		if v.is_null() {
			return Ok(Params::None);
		}

		from_value(v.clone()).map(Params::Logs)
			.map_err(|e| D::Error::custom(format!("Invalid Pub-Sub parameters: {}", e)))
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::{Result, Kind, Params};
	use v1::types::{RichHeader, Header, Filter};
	use v1::types::filter::VariadicValue;

	#[test]
	fn should_deserialize_kind() {
		assert_eq!(serde_json::from_str::<Kind>(r#""newHeads""#).unwrap(), Kind::NewHeads);
		assert_eq!(serde_json::from_str::<Kind>(r#""logs""#).unwrap(), Kind::Logs);
		assert_eq!(serde_json::from_str::<Kind>(r#""newPendingTransactions""#).unwrap(), Kind::NewPendingTransactions);
		assert_eq!(serde_json::from_str::<Kind>(r#""syncing""#).unwrap(), Kind::Syncing);
	}

	#[test]
	fn should_deserialize_logs() {
		let none = serde_json::from_str::<Params>(r#"null"#).unwrap();
		assert_eq!(none, Params::None);

		let logs1 = serde_json::from_str::<Params>(r#"{}"#).unwrap();
		let logs2 = serde_json::from_str::<Params>(r#"{"limit":10}"#).unwrap();
		let logs3 = serde_json::from_str::<Params>(
			r#"{"topics":["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b"]}"#
		).unwrap();
		assert_eq!(logs1, Params::Logs(Filter {
			from_block: None,
			to_block: None,
			block_hash: None,
			address: None,
			topics: None,
			limit: None,
		}));
		assert_eq!(logs2, Params::Logs(Filter {
			from_block: None,
			to_block: None,
			block_hash: None,
			address: None,
			topics: None,
			limit: Some(10),
		}));
		assert_eq!(logs3, Params::Logs(Filter {
			from_block: None,
			to_block: None,
			block_hash: None,
			address: None,
			topics: Some(vec![
				VariadicValue::Single("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b".parse().unwrap()
			)]),
			limit: None,
		}));
	}

	#[test]
	fn should_serialize_header() {
		let header = Result::Header(Box::new(RichHeader {
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
		}));
		let expected = r#"{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x0","extraData":"0x","gasLimit":"0x0","gasUsed":"0x0","hash":"0x0000000000000000000000000000000000000000000000000000000000000000","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","number":"0x0","parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","receiptsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","sealFields":["0x","0x"],"sha3Uncles":"0x0000000000000000000000000000000000000000000000000000000000000000","size":"0x45","stateRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":"0x0","transactionsRoot":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#;
		assert_eq!(serde_json::to_string(&header).unwrap(), expected);
	}
}
