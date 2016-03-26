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
use serde::{Serialize, Serializer, Deserialize, Deserializer, Error};
use serde::de::Visitor;
use util::common::FromHex;

/// Wrapper structure around vector of bytes.
#[derive(Debug, PartialEq, Default)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
	/// Simple constructor.
	pub fn new(bytes: Vec<u8>) -> Bytes {
		Bytes(bytes)
	}
	pub fn to_vec(self) -> Vec<u8> { let Bytes(x) = self; x }
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl Deserialize for Bytes {
	fn deserialize<D>(deserializer: &mut D) -> Result<Bytes, D::Error>
	where D: Deserializer {
		deserializer.deserialize(BytesVisitor)
	}
}

struct BytesVisitor;

impl Visitor for BytesVisitor {
	type Value = Bytes;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		if value.len() >= 2 && &value[0..2] == "0x" {
			Ok(Bytes::new(FromHex::from_hex(&value[2..]).unwrap_or_else(|_| vec![])))
		} else {
			Err(Error::custom("invalid hex"))
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
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


