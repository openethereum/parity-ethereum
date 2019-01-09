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

use std::{ops, str};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use rustc_hex::{ToHex, FromHex, FromHexError};

#[derive(Debug, PartialEq)]
pub struct Bytes(Vec<u8>);

impl ops::Deref for Bytes {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a> Deserialize<'a> for Bytes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		let s = String::deserialize(deserializer)?;
		let data = s.from_hex().map_err(|e| Error::custom(format!("Invalid hex value {}", e)))?;
		Ok(Bytes(data))
	}
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
