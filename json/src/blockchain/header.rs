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

use hash::Hash;
use uint::Uint;
use bytes::Bytes;

/// Blockchain test header deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Header {
	bloom: Hash, // TODO Bloom
	coinbase: Hash,
	difficulty: Uint,
	#[serde(rename="extraData")]
	extra_data: Bytes,
	#[serde(rename="gasLimit")]
	gas_limit: Uint,
	#[serde(rename="gasUsed")]
	gas_used: Uint,
	hash: Hash,
	#[serde(rename="mixHash")]
	mix_hash: Hash,
	nonce: Uint, // TODO fix parsing
	number: Uint,
	#[serde(rename="parentHash")]
	parent_hash: Hash,
	#[serde(rename="receiptTrie")]
	receipt_trie: Hash,
	#[serde(rename="stateRoot")]
	state_root: Hash,
	timestamp: Uint,
	#[serde(rename="transactionsTrie")]
	transactions_trie: Hash,
	#[serde(rename="uncleHash")]
	uncle_hash: Hash,
}
