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

use serde::{Serialize, Serializer};
use util::hash::*;
use util::uint::*;
use v1::types::{Bytes, Transaction};

#[derive(Debug)]
pub enum BlockTransactions {
	Hashes(Vec<U256>),
	Full(Vec<Transaction>)
}

impl Serialize for BlockTransactions {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		match *self {
			BlockTransactions::Hashes(ref hashes) => hashes.serialize(serializer),
			BlockTransactions::Full(ref ts) => ts.serialize(serializer)
		}
	}
}

#[derive(Debug, Serialize)]
pub struct Block {
	pub hash: H256,
	#[serde(rename="parentHash")]
	pub parent_hash: H256,
	#[serde(rename="sha3Uncles")]
	pub uncles_hash: H256,
	pub author: Address,
	// TODO: get rid of this one
	pub miner: Address,
	#[serde(rename="stateRoot")]
	pub state_root: H256,
	#[serde(rename="transactionsRoot")]
	pub transactions_root: H256,
	#[serde(rename="receiptsRoot")]
	pub receipts_root: H256,
	pub number: U256,
	#[serde(rename="gasUsed")]
	pub gas_used: U256,
	#[serde(rename="gasLimit")]
	pub gas_limit: U256,
	#[serde(rename="extraData")]
	pub extra_data: Bytes,
	#[serde(rename="logsBloom")]
	pub logs_bloom: H2048,
	pub timestamp: U256,
	pub difficulty: U256,
	#[serde(rename="totalDifficulty")]
	pub total_difficulty: U256,
	pub uncles: Vec<U256>,
	pub transactions: BlockTransactions
}
