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

use util::numbers::U256;
use util::hash::{Address, H256};
use v1::types::Log;
use ethcore::receipt::LocalizedReceipt;

#[derive(Debug, Serialize)]
pub struct Receipt {
	#[serde(rename="transactionHash")]
	pub transaction_hash: H256,
	#[serde(rename="transactionIndex")]
	pub transaction_index: U256,
	#[serde(rename="blockHash")]
	pub block_hash: H256,
	#[serde(rename="blockNumber")]
	pub block_number: U256,
	#[serde(rename="cumulativeGasUsed")]
	pub cumulative_gas_used: U256,
	#[serde(rename="gasUsed")]
	pub gas_used: U256,
	#[serde(rename="contractAddress")]
	pub contract_address: Option<Address>,
	pub logs: Vec<Log>,
}

impl From<LocalizedReceipt> for Receipt {
	fn from(r: LocalizedReceipt) -> Self {
		Receipt {
			transaction_hash: r.transaction_hash,
			transaction_index: U256::from(r.transaction_index),
			block_hash: r.block_hash,
			block_number: U256::from(r.block_number),
			cumulative_gas_used: r.cumulative_gas_used,
			gas_used: r.gas_used,
			contract_address: r.contract_address,
			logs: r.logs.into_iter().map(From::from).collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::str::FromStr;
	use util::numbers::*;
	use v1::types::{Bytes, Log, Receipt};

	#[test]
	fn receipt_serialization() {
		let s = r#"{"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x00","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x04510c","cumulativeGasUsed":"0x20","gasUsed":"0x10","contractAddress":null,"logs":[{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"data":"0x","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x04510c","transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x00","logIndex":"0x01"}]}"#;

		let receipt = Receipt {
			transaction_hash: H256::zero(),
			transaction_index: U256::zero(),
			block_hash: H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap(),
			block_number: U256::from(0x4510c),
			cumulative_gas_used: U256::from(0x20),
			gas_used: U256::from(0x10),
			contract_address: None,
			logs: vec![Log {
				address: Address::from_str("33990122638b9132ca29c723bdf037f1a891a70c").unwrap(),
				topics: vec![
					H256::from_str("a6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc").unwrap(),
					H256::from_str("4861736852656700000000000000000000000000000000000000000000000000").unwrap()
				],
				data: Bytes::new(vec![]),
				block_hash: H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap(),
				block_number: U256::from(0x4510c),
				transaction_hash: H256::new(),
				transaction_index: U256::zero(),
				log_index: U256::one()
			}]
		};

		let serialized = serde_json::to_string(&receipt).unwrap();
		assert_eq!(serialized, s);
	}
}

