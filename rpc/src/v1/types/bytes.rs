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

use rustc_serialize::hex::ToHex;
use serde::{Serialize, Serializer};

/// Wrapper structure around vector of bytes.
#[derive(Debug)]
pub struct Bytes(Vec<u8>);

impl Bytes {
	/// Simple constructor.
	pub fn new(bytes: Vec<u8>) -> Bytes {
		Bytes(bytes)
	}
}

impl Default for Bytes {
	fn default() -> Self {
		// default serialized value is 0x00
		Bytes(vec![0])
	}
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
		where S: Serializer,
	{
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.visit_str(serialized.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;
	use rustc_serialize::hex::FromHex;

	#[test]
	fn test_bytes_serialize() {
		let bytes = Bytes("0123456789abcdef".from_hex().unwrap());
		let serialized = serde_json::to_string(&bytes).unwrap();
		assert_eq!(serialized, r#""0x0123456789abcdef""#);
	}
}
