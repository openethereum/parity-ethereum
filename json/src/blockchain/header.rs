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

//! Blockchain test header deserializer.

use hash::{H64, H256, Bloom};
use uint::Uint;
use bytes::Bytes;

/// Blockchain test header deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Header {
	bloom: Bloom,
	coinbase: H256,
	difficulty: Uint,
	#[serde(rename="extraData")]
	extra_data: Bytes,
	#[serde(rename="gasLimit")]
	gas_limit: Uint,
	#[serde(rename="gasUsed")]
	gas_used: Uint,
	hash: H256,
	#[serde(rename="mixHash")]
	mix_hash: H256,
	nonce: H64,
	number: Uint,
	#[serde(rename="parentHash")]
	parent_hash: H256,
	#[serde(rename="receiptTrie")]
	receipt_trie: H256,
	#[serde(rename="stateRoot")]
	state_root: H256,
	timestamp: Uint,
	#[serde(rename="transactionsTrie")]
	transactions_trie: H256,
	#[serde(rename="uncleHash")]
	uncle_hash: H256,
}
