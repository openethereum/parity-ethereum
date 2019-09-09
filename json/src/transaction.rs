// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Transaction deserialization.

use crate::{bytes::Bytes, hash::{Address, H256}, maybe::MaybeEmpty, uint::Uint};
use serde::Deserialize;

/// Plain transaction deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	/// Transaction data.
	pub data: Bytes,
	/// Gas limit.
	pub gas_limit: Uint,
	/// Gas price.
	pub gas_price: Uint,
	/// Nonce.
	pub nonce: Uint,
	/// To.
	pub to: MaybeEmpty<Address>,
	/// Value.
	pub value: Uint,
}

/// Unsigned transaction with signing information deserialization
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithSigningInfo {
	/// Transaction
	pub transaction: Transaction,
	/// R.
	pub r: Uint,
	/// S.
	pub s: Uint,
	/// V.
	pub v: Uint,
	/// Secret
	#[serde(rename = "secretKey")]
	pub secret: Option<H256>,
}

#[cfg(test)]
mod tests {
	use super::{Transaction, TransactionWithSigningInfo};

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"data" : "0x",
			"gasLimit" : "0xf388",
			"gasPrice" : "0x09184e72a000",
			"nonce" : "0x00",
			"to" : "",
			"value" : "0x00"
		}"#;
		let _deserialized: Transaction = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}

	#[test]
	fn transaction_with_signing_info_deserialization() {
		let s = r#"{
			"transaction": {
				"data" : "0x",
				"gasLimit" : "0xf388",
				"gasPrice" : "0x09184e72a000",
				"nonce" : "0x00",
				"to" : "",
				"value" : "0x00"
			},
			"r": 0,
			"s": 1,
			"v": 2,
			"secretKey": "0x0000000000000000000000000000000000000000000000000000000000000000"
		}"#;
		let _deserialized: TransactionWithSigningInfo = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
