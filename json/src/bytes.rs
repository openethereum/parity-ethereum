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

//! Lenient bytes json deserialization for test json files.

use rustc_serialize::hex::FromHex;
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;

/// Lenient bytes json deserialization for test json files.
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Bytes(Vec<u8>);

impl Into<Vec<u8>> for Bytes {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

impl Deserialize for Bytes {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer {
		deserializer.deserialize(BytesVisitor)
	}
}

struct BytesVisitor;

impl Visitor for BytesVisitor {
	type Value = Bytes;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		let v = match value.len() {
			0 => vec![],
			2 if value.starts_with("0x") => vec![],
			_ if value.starts_with("0x") && value.len() % 2 == 1 => {
				let v = "0".to_owned() + &value[2..];
				FromHex::from_hex(v.as_ref() as &str).unwrap_or(vec![]),
			},
			_ if value.starts_with("0x") => FromHex::from_hex(&value[2..]).unwrap_or(vec![]),
			_ => FromHex::from_hex(value).unwrap_or(vec![]),
		};
		Ok(Bytes(v))
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
mod test {
	use serde_json;
	use bytes::Bytes;

	#[test]
	fn bytes_deserialization() {
		let s = r#"["", "0x", "0x12", "1234", "0x001"]"#;
		let deserialized: Vec<Bytes> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
			Bytes(vec![]),
			Bytes(vec![]),
			Bytes(vec![0x12]),
			Bytes(vec![0x12, 0x34]),
			Bytes(vec![0, 1])
		]);
	}

	#[test]
	fn bytes_into() {
		let bytes = Bytes(vec![0xff, 0x11]);
		let v: Vec<u8> = bytes.into();
		assert_eq!(vec![0xff, 0x11], v);
	}
}
