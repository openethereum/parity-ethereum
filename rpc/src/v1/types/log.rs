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

use util::numbers::*;
use ethcore::log_entry::{LocalizedLogEntry, LogEntry};
use v1::types::Bytes;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct Log {
	pub address: Address,
	pub topics: Vec<H256>,
	pub data: Bytes,
	#[serde(rename="blockHash")]
	pub block_hash: Option<H256>,
	#[serde(rename="blockNumber")]
	pub block_number: Option<U256>,
	#[serde(rename="transactionHash")]
	pub transaction_hash: Option<H256>,
	#[serde(rename="transactionIndex")]
	pub transaction_index: Option<U256>,
	#[serde(rename="logIndex")]
	pub log_index: Option<U256>,
	#[serde(rename="type")]
	pub log_type: String,
}

impl From<LocalizedLogEntry> for Log {
	fn from(e: LocalizedLogEntry) -> Log {
		Log {
			address: e.entry.address,
			topics: e.entry.topics,
			data: Bytes::new(e.entry.data),
			block_hash: Some(e.block_hash),
			block_number: Some(From::from(e.block_number)),
			transaction_hash: Some(e.transaction_hash),
			transaction_index: Some(From::from(e.transaction_index)),
			log_index: Some(From::from(e.log_index)),
			log_type: "mined".to_owned(),
		}
	}
}

impl From<LogEntry> for Log {
	fn from(e: LogEntry) -> Log {
		Log {
			address: e.address,
			topics: e.topics,
			data: Bytes::new(e.data),
			block_hash: None,
			block_number: None,
			transaction_hash: None,
			transaction_index: None,
			log_index: None,
			log_type: "pending".to_owned(),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::str::FromStr;
	use util::numbers::*;
	use v1::types::{Bytes, Log};

	#[test]
	fn log_serialization() {
		let s = r#"{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"data":"0x","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x04510c","transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x00","logIndex":"0x01","type":"mined"}"#;

		let log = Log {
			address: Address::from_str("33990122638b9132ca29c723bdf037f1a891a70c").unwrap(),
			topics: vec![
				H256::from_str("a6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc").unwrap(),
				H256::from_str("4861736852656700000000000000000000000000000000000000000000000000").unwrap()
			],
			data: Bytes::new(vec![]),
			block_hash: Some(H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap()),
			block_number: Some(U256::from(0x4510c)),
			transaction_hash: Some(H256::new()),
			transaction_index: Some(U256::zero()),
			log_index: Some(U256::one()),
			log_type: "mined".to_owned(),
		};

		let serialized = serde_json::to_string(&log).unwrap();
		assert_eq!(serialized, s);
	}
}
