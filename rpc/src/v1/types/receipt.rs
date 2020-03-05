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

use ethereum_types::{H160, H256, U64, U256, Bloom as H2048};
use v1::types::Log;
use types::receipt::{Receipt as EthReceipt, RichReceipt, LocalizedReceipt, TransactionOutcome};

/// Receipt
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
	/// Transaction Hash
	pub transaction_hash: Option<H256>,
	/// Transaction index
	pub transaction_index: Option<U256>,
	/// Block hash
	pub block_hash: Option<H256>,
	/// Sender
	pub from: Option<H160>,
	/// Recipient
	pub to: Option<H160>,
	/// Block number
	pub block_number: Option<U256>,
	/// Cumulative gas used
	pub cumulative_gas_used: U256,
	/// Gas used
	pub gas_used: Option<U256>,
	/// Contract address
	pub contract_address: Option<H160>,
	/// Logs
	pub logs: Vec<Log>,
	/// State Root
	// NOTE(niklasad1): EIP98 makes this optional field, if it's missing then skip serializing it
	#[serde(skip_serializing_if = "Option::is_none", rename = "root")]
	pub state_root: Option<H256>,
	/// Logs bloom
	pub logs_bloom: H2048,
	/// Status code
	// NOTE(niklasad1): Unknown after EIP98 rules, if it's missing then skip serializing it
	#[serde(skip_serializing_if = "Option::is_none", rename = "status")]
	pub status_code: Option<U64>,
}

impl Receipt {
	fn outcome_to_state_root(outcome: TransactionOutcome) -> Option<H256> {
		match outcome {
			TransactionOutcome::Unknown | TransactionOutcome::StatusCode(_) => None,
			TransactionOutcome::StateRoot(root) => Some(root),
		}
	}

	fn outcome_to_status_code(outcome: &TransactionOutcome) -> Option<U64> {
		match *outcome {
			TransactionOutcome::Unknown | TransactionOutcome::StateRoot(_) => None,
			TransactionOutcome::StatusCode(ref code) => Some((*code as u64).into()),
		}
	}
}

impl From<LocalizedReceipt> for Receipt {
	fn from(r: LocalizedReceipt) -> Self {
		Receipt {
			to: r.to.map(Into::into),
			from: Some(r.from),
			transaction_hash: Some(r.transaction_hash),
			transaction_index: Some(r.transaction_index.into()),
			block_hash: Some(r.block_hash),
			block_number: Some(r.block_number.into()),
			cumulative_gas_used: r.cumulative_gas_used,
			gas_used: Some(r.gas_used),
			contract_address: r.contract_address.map(Into::into),
			logs: r.logs.into_iter().map(Into::into).collect(),
			status_code: Self::outcome_to_status_code(&r.outcome),
			state_root: Self::outcome_to_state_root(r.outcome),
			logs_bloom: r.log_bloom,
		}
	}
}

impl From<RichReceipt> for Receipt {
	fn from(r: RichReceipt) -> Self {
		Receipt {
			from: Some(r.from),
			to: r.to.map(Into::into),
			transaction_hash: Some(r.transaction_hash),
			transaction_index: Some(r.transaction_index.into()),
			block_hash: None,
			block_number: None,
			cumulative_gas_used: r.cumulative_gas_used,
			gas_used: Some(r.gas_used),
			contract_address: r.contract_address.map(Into::into),
			logs: r.logs.into_iter().map(Into::into).collect(),
			status_code: Self::outcome_to_status_code(&r.outcome),
			state_root: Self::outcome_to_state_root(r.outcome),
			logs_bloom: r.log_bloom,
		}
	}
}

impl From<EthReceipt> for Receipt {
	fn from(r: EthReceipt) -> Self {
		Receipt {
			from: None,
			to: None,
			transaction_hash: None,
			transaction_index: None,
			block_hash: None,
			block_number: None,
			cumulative_gas_used: r.gas_used,
			gas_used: None,
			contract_address: None,
			logs: r.logs.into_iter().map(Into::into).collect(),
			status_code: Self::outcome_to_status_code(&r.outcome),
			state_root: Self::outcome_to_state_root(r.outcome),
			logs_bloom: r.log_bloom,
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use v1::types::{Log, Receipt};
	use ethereum_types::{H256, Bloom};

	#[test]
	fn receipt_serialization() {
		let s = r#"{"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","from":null,"to":null,"blockNumber":"0x4510c","cumulativeGasUsed":"0x20","gasUsed":"0x10","contractAddress":null,"logs":[{"address":"0x33990122638b9132ca29c723bdf037f1a891a70c","topics":["0xa6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc","0x4861736852656700000000000000000000000000000000000000000000000000"],"data":"0x","blockHash":"0xed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5","blockNumber":"0x4510c","transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000000","transactionIndex":"0x0","logIndex":"0x1","transactionLogIndex":null,"type":"mined","removed":false}],"root":"0x000000000000000000000000000000000000000000000000000000000000000a","logsBloom":"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f","status":"0x1"}"#;

		let receipt = Receipt {
			from: None,
			to: None,
			transaction_hash: Some(H256::zero()),
			transaction_index: Some(0.into()),
			block_hash: Some("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5".parse().unwrap()),
			block_number: Some(0x4510c.into()),
			cumulative_gas_used: 0x20.into(),
			gas_used: Some(0x10.into()),
			contract_address: None,
			logs: vec![Log {
				address: "33990122638b9132ca29c723bdf037f1a891a70c".parse().unwrap(),
				topics: vec![
					"a6697e974e6a320f454390be03f74955e8978f1a6971ea6730542e37b66179bc".parse().unwrap(),
					"4861736852656700000000000000000000000000000000000000000000000000".parse().unwrap(),
				],
				data: vec![].into(),
				block_hash: Some("ed76641c68a1c641aee09a94b3b471f4dc0316efe5ac19cf488e2674cf8d05b5".parse().unwrap()),
				block_number: Some(0x4510c.into()),
				transaction_hash: Some(H256::zero()),
				transaction_index: Some(0.into()),
				transaction_log_index: None,
				log_index: Some(1.into()),
				log_type: "mined".into(),
				removed: false,
			}],
			logs_bloom: Bloom::from_low_u64_be(15),
			state_root: Some(H256::from_low_u64_be(10)),
			status_code: Some(1u64.into()),
		};

		let serialized = serde_json::to_string(&receipt).unwrap();
		assert_eq!(serialized, s);
	}
}
