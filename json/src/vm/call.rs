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

//! Vm call deserialization.

use bytes::Bytes;
use hash::Address;
use uint::Uint;
use maybe::MaybeEmpty;

/// Vm call deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Call {
	/// Call data.
	pub data: Bytes,
	/// Call destination.
	pub destination: MaybeEmpty<Address>,
	/// Gas limit.
	#[serde(rename="gasLimit")]
	pub gas_limit: Uint,
	/// Call value.
	pub value: Uint,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use vm::Call;

	#[test]
	fn call_deserialization() {
		let s = r#"{
			"data" : "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
			"destination" : "",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;
		let _deserialized: Call = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
