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

//! Spec genesis deserialization.

use uint::Uint;
use hash::{Address, H256};
use bytes::Bytes;
use spec::Seal;

/// Spec genesis.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Genesis {
	/// Seal.
	pub seal: Seal,
	/// Difficulty.
	pub difficulty: Uint,
	/// Block author.
	pub author: Address,
	/// Block timestamp.
	pub timestamp: Uint,
	/// Parent hash.
	#[serde(rename="parentHash")]
	pub parent_hash: H256,
	/// Gas limit.
	#[serde(rename="gasLimit")]
	pub gas_limit: Uint,
	/// Transactions root.
	#[serde(rename="transactionsRoot")]
	pub transactions_root: Option<H256>,
	/// Receipts root.
	#[serde(rename="receiptsRoot")]
	pub receipts_root: Option<H256>,
	/// State root.
	#[serde(rename="stateRoot")]
	pub state_root: Option<H256>,
	/// Gas used.
	#[serde(rename="gasUsed")]
	pub gas_used: Option<Uint>,
	/// Extra data.
	#[serde(rename="extraData")]
	pub extra_data: Option<Bytes>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::genesis::Genesis;

	#[test]
	fn genesis_deserialization() {
		let s = r#"{
			"nonce": "0x0000000000000042",
			"difficulty": "0x400000000",
			"seal": {
				"ethereum": {
					"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
					"nonce": "0x00006d6f7264656e"
				}
			},
			"author": "0x0000000000000000000000000000000000000000",
			"timestamp": "0x00",
			"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
			"extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
			"gasLimit": "0x1388",
			"stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
		}"#;
		let _deserialized: Genesis = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
