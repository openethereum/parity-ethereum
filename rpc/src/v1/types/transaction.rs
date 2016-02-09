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
use v1::types::{Bytes, OptionalValue};

#[derive(Debug, Default, Serialize)]
pub struct Transaction {
	pub hash: H256,
	pub nonce: U256,
	#[serde(rename="blockHash")]
	pub block_hash: OptionalValue<H256>,
	#[serde(rename="blockNumber")]
	pub block_number: OptionalValue<U256>,
	#[serde(rename="transactionIndex")]
	pub transaction_index: OptionalValue<U256>,
	pub from: Address,
	pub to: OptionalValue<Address>,
	pub value: U256,
	#[serde(rename="gasPrice")]
	pub gas_price: U256,
	pub gas: U256,
	pub input: Bytes
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn test_transaction_serialize() {
		let t = Transaction::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x00","blockHash":null,"blockNumber":null,"transactionIndex":null,"from":"0x0000000000000000000000000000000000000000","to":null,"value":"0x00","gasPrice":"0x00","gas":"0x00","input":"0x00"}"#);
	}
}

