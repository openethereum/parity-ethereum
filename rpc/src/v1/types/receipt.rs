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


