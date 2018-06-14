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

//! Lenient bytes json deserialization for test json files.

use std::fmt;
use std::str::FromStr;
use std::ops::Deref;
use rustc_hex::FromHex;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};

/// Lenient bytes json deserialization for test json files.
#[derive(Default, Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Bytes(Vec<u8>);

impl Bytes {
	/// Creates bytes struct.
	pub fn new(v: Vec<u8>) -> Self {
		Bytes(v)
	}
}

impl Into<Vec<u8>> for Bytes {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

impl Deref for Bytes {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		&self.0
	}
}

impl FromStr for Bytes {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		let v = match value.len() {
			0 => vec![],
			2 if value.starts_with("0x") => vec![],
			_ if value.starts_with("0x") && value.len() % 2 == 1 => {
				let v = "0".to_owned() + &value[2..];
				FromHex::from_hex(v.as_str()).unwrap_or(vec![])
			},
			_ if value.starts_with("0x") => FromHex::from_hex(&value[2..]).unwrap_or(vec![]),
			_ => FromHex::from_hex(value).unwrap_or(vec![]),
		};

		Ok(Bytes(v))
	}
}

impl<'a> Deserialize<'a> for Bytes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a> {
		deserializer.deserialize_any(BytesVisitor)
	}
}

struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
	type Value = Bytes;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a hex encoded string of bytes")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
		Bytes::from_str(value).map_err(Error::custom)
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
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
