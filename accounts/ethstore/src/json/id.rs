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

//! Universaly unique identifier.
use std::{fmt, str};
use rustc_hex::{ToHex, FromHex};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde::de::{Visitor, Error as SerdeError};
use super::Error;

/// Universaly unique identifier.
#[derive(Debug, PartialEq)]
pub struct Uuid([u8; 16]);

impl From<[u8; 16]> for Uuid {
	fn from(uuid: [u8; 16]) -> Self {
		Uuid(uuid)
	}
}

impl<'a> Into<String> for &'a Uuid {
	fn into(self) -> String {
		let d1 = &self.0[0..4];
		let d2 = &self.0[4..6];
		let d3 = &self.0[6..8];
		let d4 = &self.0[8..10];
		let d5 = &self.0[10..16];
		[d1, d2, d3, d4, d5].iter().map(|d| d.to_hex()).collect::<Vec<String>>().join("-")
	}
}

impl Into<String> for Uuid {
	fn into(self) -> String {
		Into::into(&self)
	}
}

impl Into<[u8; 16]> for Uuid {
	fn into(self) -> [u8; 16] {
		self.0
	}
}

impl fmt::Display for Uuid {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		let s: String = (self as &Uuid).into();
		write!(f, "{}", s)
	}
}

fn copy_into(from: &str, into: &mut [u8]) -> Result<(), Error> {
	let from = from.from_hex().map_err(|_| Error::InvalidUuid)?;

	if from.len() != into.len() {
		return Err(Error::InvalidUuid);
	}

	into.copy_from_slice(&from);
	Ok(())
}

impl str::FromStr for Uuid {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<&str> = s.split("-").collect();

		if parts.len() != 5 {
			return Err(Error::InvalidUuid);
		}

		let mut uuid = [0u8; 16];

		copy_into(parts[0], &mut uuid[0..4])?;
		copy_into(parts[1], &mut uuid[4..6])?;
		copy_into(parts[2], &mut uuid[6..8])?;
		copy_into(parts[3], &mut uuid[8..10])?;
		copy_into(parts[4], &mut uuid[10..16])?;

		Ok(Uuid(uuid))
	}
}

impl From<&'static str> for Uuid {
	fn from(s: &'static str) -> Self {
		s.parse().expect(&format!("invalid string literal for {}: '{}'", stringify!(Self), s))
	}
}

impl Serialize for Uuid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		let s: String = self.into();
		serializer.serialize_str(&s)
	}
}

impl<'a> Deserialize<'a> for Uuid {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		deserializer.deserialize_any(UuidVisitor)
	}
}

struct UuidVisitor;

impl<'a> Visitor<'a> for UuidVisitor {
	type Value = Uuid;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid hex-encoded UUID")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		value.parse().map_err(SerdeError::custom)
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::Uuid;

	#[test]
	fn uuid_from_str() {
		let uuid: Uuid = "3198bc9c-6672-5ab3-d995-4942343ae5b6".into();
		assert_eq!(uuid, Uuid::from([0x31, 0x98, 0xbc, 0x9c, 0x66, 0x72, 0x5a, 0xb3, 0xd9, 0x95, 0x49, 0x42, 0x34, 0x3a, 0xe5, 0xb6]));
	}

	#[test]
	fn uuid_from_and_to_str() {
		let from = "3198bc9c-6672-5ab3-d995-4942343ae5b6";
		let uuid: Uuid = from.into();
		let to: String = uuid.into();
		assert_eq!(from, &to);
	}
}
