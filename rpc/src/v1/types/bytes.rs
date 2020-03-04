// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Serializable wrapper around vector of bytes

use std::fmt;
use rustc_hex::{ToHex, FromHex};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Error, Visitor};

/// Wrapper structure around vector of bytes.
#[derive(Debug, PartialEq, Eq, Default, Hash, Clone)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
	/// Simple constructor.
	pub fn new(bytes: Vec<u8>) -> Bytes {
		Bytes(bytes)
	}
	/// Convert back to vector
	pub fn into_vec(self) -> Vec<u8> {
		self.0
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(bytes: Vec<u8>) -> Bytes {
		Bytes(bytes)
	}
}

impl Into<Vec<u8>> for Bytes {
	fn into(self) -> Vec<u8> {
		self.0
	}
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer
	{
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex::<String>().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for Bytes {
	fn deserialize<D>(deserializer: D) -> Result<Bytes, D::Error>
	where D: Deserializer<'a> {
		deserializer.deserialize_any(BytesVisitor)
	}
}

struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
	type Value = Bytes;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a 0x-prefixed, hex-encoded vector of bytes")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
		if value.len() >= 2 && value.starts_with("0x") && value.len() & 1 == 0 {
			Ok(Bytes::new(FromHex::from_hex(&value[2..]).map_err(|e| Error::custom(format!("Invalid hex: {}", e)))?))
		} else {
			Err(Error::custom("Invalid bytes format. Expected a 0x-prefixed hex string with even length"))
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rustc_hex::FromHex;

	#[test]
	fn test_bytes_serialize() {
		let bytes = Bytes("0123456789abcdef".from_hex().unwrap());
		let serialized = serde_json::to_string(&bytes).unwrap();
		assert_eq!(serialized, r#""0x0123456789abcdef""#);
	}

	#[test]
	fn test_bytes_deserialize() {
		let bytes0: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""∀∂""#);
		let bytes1: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""""#);
		let bytes2: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""0x123""#);
		let bytes3: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""0xgg""#);

		let bytes4: Bytes = serde_json::from_str(r#""0x""#).unwrap();
		let bytes5: Bytes = serde_json::from_str(r#""0x12""#).unwrap();
		let bytes6: Bytes = serde_json::from_str(r#""0x0123""#).unwrap();

		assert!(bytes0.is_err());
		assert!(bytes1.is_err());
		assert!(bytes2.is_err());
		assert!(bytes3.is_err());
		assert_eq!(bytes4, Bytes(vec![]));
		assert_eq!(bytes5, Bytes(vec![0x12]));
		assert_eq!(bytes6, Bytes(vec![0x1, 0x23]));
	}
}
