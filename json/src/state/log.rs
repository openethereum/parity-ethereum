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

//! State test log deserialization.
use hash::{Address, H256, Bloom};
use bytes::Bytes;

/// State test log deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Log {
	/// Address.
	pub address: Address,
	/// Topics.
	pub topics: Vec<H256>,
	/// Data.
	pub data: Bytes,
	/// Bloom.
	pub bloom: Bloom,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use state::Log;

	#[test]
	fn log_deserialization() {
		let s = r#"{
			"address" : "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6",
			"bloom" : "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008800000000000000000020000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000",
			"data" : "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			"topics" : [
				"0000000000000000000000000000000000000000000000000000000000000000"
			]
		}"#;
		let _deserialized: Log = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
