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

/// Unsigned transaction with signing information deserialization.
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
	/// R.
	#[serde(default)]
	pub r: MaybeEmpty<Uint>,
	/// S.
	#[serde(default)]
	pub s: MaybeEmpty<Uint>,
	/// V.
	#[serde(default)]
	pub v: MaybeEmpty<Uint>,
	/// Secret
	#[serde(rename = "secretKey")]
	pub secret: Option<H256>,
}

#[cfg(test)]
mod tests {
	use super::{Bytes, H256, MaybeEmpty, Transaction, Uint};
	use ethereum_types::{H256 as Eth256, U256};

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"data" : "0x",
			"gasLimit" : "0xf388",
			"gasPrice" : "0x09184e72a000",
			"nonce" : "0x00",
			"to" : "",
			"value" : "0x00",
			"r": "0",
			"s": "1",
			"v": "2",
			"secretKey": "0x0000000000000000000000000000000000000000000000000000000000000000"
		}"#;
		let tx: Transaction = serde_json::from_str(s).expect("JSON string is valid");
		assert_eq!(tx.data, Bytes::new(Vec::new()));
		assert_eq!(tx.gas_limit, Uint(U256::from(0xf388)));
		assert_eq!(tx.gas_price, Uint(U256::from(0x09184e72a000_u64)));
		assert_eq!(tx.nonce, Uint(U256::zero()));
		assert_eq!(tx.to, MaybeEmpty::None);
		assert_eq!(tx.value, Uint(U256::zero()));
		assert_eq!(tx.r, Uint(U256::zero()).into());
		assert_eq!(tx.s, Uint(U256::one()).into());
		assert_eq!(tx.v, Uint(U256::from(2)).into());
		assert_eq!(tx.secret, Some(H256(Eth256::zero())));
	}
}
