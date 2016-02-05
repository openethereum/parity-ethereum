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

use util::hash::*;
use util::uint::*;

#[derive(Default, Debug, Serialize)]
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
	// TODO: figure out how to properly serialize bytes
	//#[serde(rename="extraData")]
	//extra_data: Vec<u8>,
	#[serde(rename="logsBloom")]
	pub logs_bloom: H2048,
	pub timestamp: U256,
	pub difficulty: U256,
	#[serde(rename="totalDifficulty")]
	pub total_difficulty: U256,
	pub uncles: Vec<U256>,
	pub transactions: Vec<U256>
}
