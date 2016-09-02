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

use v1::types::{Log, H160, H256, U256};
use ethcore::receipt::{Receipt as EthReceipt, RichReceipt, LocalizedReceipt};

/// Receipt
#[derive(Debug, Serialize)]
pub struct Receipt {
	/// Transaction Hash
	#[serde(rename="transactionHash")]
	pub transaction_hash: Option<H256>,
	/// Transaction index
	#[serde(rename="transactionIndex")]
	pub transaction_index: Option<U256>,
	/// Block hash
	#[serde(rename="blockHash")]
	pub block_hash: Option<H256>,
	/// Block number
	#[serde(rename="blockNumber")]
	pub block_number: Option<U256>,
	/// Cumulative gas used
	#[serde(rename="cumulativeGasUsed")]
	pub cumulative_gas_used: U256,
	/// Gas used
	#[serde(rename="gasUsed")]
	pub gas_used: Option<U256>,
	/// Contract address
	#[serde(rename="contractAddress")]
	pub contract_address: Option<H160>,
	/// Logs
	pub logs: Vec<Log>,
}

impl From<LocalizedReceipt> for Receipt {
	fn from(r: LocalizedReceipt) -> Self {
		Receipt {
			transaction_hash: Some(r.transaction_hash.into()),
			transaction_index: Some(r.transaction_index.into()),
			block_hash: Some(r.block_hash.into()),
			block_number: Some(r.block_number.into()),
			cumulative_gas_used: r.cumulative_gas_used.into(),
			gas_used: Some(r.gas_used.into()),
			contract_address: r.contract_address.map(Into::into),
			logs: r.logs.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<RichReceipt> for Receipt {
	fn from(r: RichReceipt) -> Self {
		Receipt {
			transaction_hash: Some(r.transaction_hash.into()),
			transaction_index: Some(r.transaction_index.into()),
			block_hash: None,
			block_number: None,
			cumulative_gas_used: r.cumulative_gas_used.into(),
			gas_used: Some(r.gas_used.into()),
			contract_address: r.contract_address.map(Into::into),
			logs: r.logs.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<EthReceipt> for Receipt {
	fn from(r: EthReceipt) -> Self {
		Receipt {
			transaction_hash: None,
			transaction_index: None,
			block_hash: None,
			block_number: None,
			cumulative_gas_used: r.gas_used.into(),
			gas_used: None,
			contract_address: None,
			logs: r.logs.into_iter().map(Into::into).collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::str::FromStr;
	use v1::types::{Log, Receipt, U256, H256, H160};

	#[test]
	fn receipt_serialization() {
		let s = r#"{"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","cumulativeGasUsed":"0x20","gasUsed":"0x10","contractAddress":null,"logs":[{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"data":"0x","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","logIndex":"0x1","type":"mined"}]}"#;

		let receipt = Receipt {
			transaction_hash: Some(H256::from(0)),
			transaction_index: Some(U256::from(0)),
			block_hash: Some(H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap()),
			block_number: Some(U256::from(0x4510c)),
			cumulative_gas_used: U256::from(0x20),
			gas_used: Some(U256::from(0x10)),
			contract_address: None,
			logs: vec![Log {
				address: H160::from_str("33990122638b9132ca29c723bdf037f1a891a70c").unwrap(),
				topics: vec![
					H256::from_str("a6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc").unwrap(),
					H256::from_str("4861736852656700000000000000000000000000000000000000000000000000").unwrap(),
				],
				data: vec![].into(),
				block_hash: Some(H256::from_str("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5").unwrap()),
				block_number: Some(U256::from(0x4510c)),
				transaction_hash: Some(H256::default()),
				transaction_index: Some(U256::default()),
				log_index: Some(U256::from(1)),
				log_type: "mined".to_owned(),
			}]
		};

		let serialized = serde_json::to_string(&receipt).unwrap();
		assert_eq!(serialized, s);
	}
}

