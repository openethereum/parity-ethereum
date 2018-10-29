// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Executed transaction.
use hash::Address;
use uint::Uint;
use bytes::Bytes;

/// Executed transaction.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	/// Contract address.
	pub address: Address,
	/// Transaction sender.
	#[serde(rename = "caller")]
	pub sender: Address,
	/// Contract code.
	pub code: Bytes,
	/// Input data.
	pub data: Bytes,
	/// Gas.
	pub gas: Uint,
	/// Gas price.
	pub gas_price: Uint,
	/// Transaction origin.
	pub origin: Address,
	/// Sent value.
	pub value: Uint,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use vm::Transaction;

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"address" : "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6",
			"caller" : "cd1722f2947def4cf144679da39c4c32bdc35681",
			"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055",
			"data" : "0x",
			"gas" : "0x0186a0",
			"gasPrice" : "0x5af3107a4000",
			"origin" : "cd1722f2947def4cf144679da39c4c32bdc35681",
			"value" : "0x0de0b6b3a7640000"
		}"#;
		let _deserialized: Transaction = serde_json::from_str(s).unwrap();
	}
}
