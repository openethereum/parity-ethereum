use std::{ops, str};
use serde::{Deserialize, Deserializer, Error, Serialize, Serializer};
use rustc_serialize::hex::{ToHex, FromHex, FromHexError};

#[derive(Debug, PartialEq)]
pub struct Bytes(Vec<u8>);

impl ops::Deref for Bytes {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Deserialize for Bytes {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer
	{
		let s = try!(String::deserialize(deserializer));
		let data = try!(s.from_hex().map_err(|e| Error::custom(format!("Invalid hex value {}", e))));
		Ok(Bytes(data))
	}
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		serializer.serialize_str(&self.0.to_hex())
	}
}

impl str::FromStr for Bytes {
	type Err = FromHexError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.from_hex().map(Bytes)
	}
}

impl From<&'static str> for Bytes {
	fn from(s: &'static str) -> Self {
		s.parse().expect(&format!("invalid string literal for {}: '{}'", stringify!(Self), s))
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(v: Vec<u8>) -> Self {
		Bytes(v)
	}
}

impl From<Bytes> for Vec<u8> {
	fn from(b: Bytes) -> Self {
		b.0
	}
}

