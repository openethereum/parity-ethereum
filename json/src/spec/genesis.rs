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

/// Spec genesis.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Genesis {
	// old seal
	nonce: Option<Uint>,
	#[serde(rename="mixHash")]
	mix_hash: Option<H256>,

	// new seal // TODO: consider moving it to a separate seal structure
	#[serde(rename="sealFields")]
	seal_fields: Option<Uint>,
	#[serde(rename="sealRlp")]
	seal_rlp: Option<Bytes>,

	difficulty: Uint,
	author: Address,
	timestamp: Uint,
	#[serde(rename="parentHash")]
	parent_hash: H256,
	#[serde(rename="gasLimit")]
	gas_limit: Uint,
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
			"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
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
