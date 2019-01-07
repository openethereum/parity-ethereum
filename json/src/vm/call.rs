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

//! Vm call deserialization.

use bytes::Bytes;
use hash::Address;
use uint::Uint;
use maybe::MaybeEmpty;

/// Vm call deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Call {
	/// Call data.
	pub data: Bytes,
	/// Call destination.
	pub destination: MaybeEmpty<Address>,
	/// Gas limit.
	pub gas_limit: Uint,
	/// Call value.
	pub value: Uint,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use vm::Call;
	use ethereum_types::{U256, H160 as Hash160};
	use uint::Uint;
	use hash::Address;
	use maybe::MaybeEmpty;
	use std::str::FromStr;

	#[test]
	fn call_deserialization_empty_dest() {
		let s = r#"{
			"data" : "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
			"destination" : "",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;
		let call: Call = serde_json::from_str(s).unwrap();

		assert_eq!(&call.data[..],
			&[0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44, 0x55, 0x55, 0x66, 0x66, 0x77, 0x77,
			  0x88, 0x88, 0x99, 0x99, 0x00, 0x00, 0xaa, 0xaa, 0xbb, 0xbb, 0xcc, 0xcc, 0xdd, 0xdd,
			  0xee, 0xee, 0xff, 0xff]);

		assert_eq!(call.destination, MaybeEmpty::None);
		assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
		assert_eq!(call.value, Uint(U256::from(0)));
	}

	#[test]
	fn call_deserialization_full_dest() {
		let s = r#"{
			"data" : "0x1234",
			"destination" : "5a39ed1020c04d4d84539975b893a4e7c53eab6c",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;

		let call: Call = serde_json::from_str(s).unwrap();

		assert_eq!(&call.data[..], &[0x12, 0x34]);
		assert_eq!(call.destination, MaybeEmpty::Some(Address(Hash160::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c").unwrap())));
		assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
		assert_eq!(call.value, Uint(U256::from(0)));
	}
}
