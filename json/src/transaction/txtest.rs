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

//! Transaction test deserialization.

use uint::Uint;
use bytes::Bytes;
use hash::Address;
use transaction::Transaction;

/// Transaction test deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionTest {
	/// Block number.
	#[serde(rename="blocknumber")]
	pub block_number: Option<Uint>,
	/// Transaction rlp.
	pub rlp: Bytes,
	/// Transaction sender.
	pub sender: Option<Address>,
	/// Transaction
	pub transaction: Option<Transaction>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use transaction::TransactionTest;

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"blocknumber" : "0",
			"rlp" : "0xf83f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870b801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a3664935301",
			"sender" : "e115cf6bb5656786569dd273705242ca72d84bc0",
			"transaction" : {
				"data" : "",
				"gasLimit" : "0x5208",
				"gasPrice" : "0x01",
				"nonce" : "0x00",
				"r" : "0x48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353",
				"s" : "0x01",
				"to" : "095e7baea6a6c7c4c2dfeb977efac326af552d87",
				"v" : "0x1b",
				"value" : "0x0b"
			}
		}"#;
		let _deserialized: TransactionTest = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
