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

use std::fmt;
use std::num::NonZeroU32;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use super::{Error, Bytes};

#[derive(Debug, PartialEq)]
pub enum KdfSer {
	Pbkdf2,
	Scrypt,
}

impl Serialize for KdfSer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			KdfSer::Pbkdf2 => serializer.serialize_str("pbkdf2"),
			KdfSer::Scrypt => serializer.serialize_str("scrypt"),
		}
	}
}

impl<'a> Deserialize<'a> for KdfSer {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		deserializer.deserialize_any(KdfSerVisitor)
	}
}

struct KdfSerVisitor;

impl<'a> Visitor<'a> for KdfSerVisitor {
	type Value = KdfSer;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a kdf algorithm identifier")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"pbkdf2" => Ok(KdfSer::Pbkdf2),
			"scrypt" => Ok(KdfSer::Scrypt),
			_ => Err(SerdeError::custom(Error::UnsupportedKdf))
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq)]
pub enum Prf {
	HmacSha256,
}

impl Serialize for Prf {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			Prf::HmacSha256 => serializer.serialize_str("hmac-sha256"),
		}
	}
}

impl<'a> Deserialize<'a> for Prf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		deserializer.deserialize_any(PrfVisitor)
	}
}

struct PrfVisitor;

impl<'a> Visitor<'a> for PrfVisitor {
	type Value = Prf;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a prf algorithm identifier")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"hmac-sha256" => Ok(Prf::HmacSha256),
			_ => Err(SerdeError::custom(Error::InvalidPrf)),
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pbkdf2 {
	pub c: NonZeroU32,
	pub dklen: u32,
	pub prf: Prf,
	pub salt: Bytes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Scrypt {
	pub dklen: u32,
	pub p: u32,
	pub n: u32,
	pub r: u32,
	pub salt: Bytes,
}

#[derive(Debug, PartialEq)]
pub enum KdfSerParams {
	Pbkdf2(Pbkdf2),
	Scrypt(Scrypt),
}

impl Serialize for KdfSerParams {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			KdfSerParams::Pbkdf2(ref params) => params.serialize(serializer),
			KdfSerParams::Scrypt(ref params) => params.serialize(serializer),
		}
	}
}

impl<'a> Deserialize<'a> for KdfSerParams {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		use serde_json::{Value, from_value};

		let v: Value = Deserialize::deserialize(deserializer)?;

		from_value(v.clone()).map(KdfSerParams::Pbkdf2)
			.or_else(|_| from_value(v).map(KdfSerParams::Scrypt))
			.map_err(|_| D::Error::custom("Invalid KDF algorithm"))
	}
}

#[derive(Debug, PartialEq)]
pub enum Kdf {
	Pbkdf2(Pbkdf2),
	Scrypt(Scrypt),
}
