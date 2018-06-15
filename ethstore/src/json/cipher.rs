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

use std::fmt;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use super::{Error, H128};

#[derive(Debug, PartialEq)]
pub enum CipherSer {
	Aes128Ctr,
}

impl Serialize for CipherSer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			CipherSer::Aes128Ctr => serializer.serialize_str("aes-128-ctr"),
		}
	}
}

impl<'a> Deserialize<'a> for CipherSer {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		deserializer.deserialize_any(CipherSerVisitor)
	}
}

struct CipherSerVisitor;

impl<'a> Visitor<'a> for CipherSerVisitor {
	type Value = CipherSer;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid cipher identifier")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"aes-128-ctr" => Ok(CipherSer::Aes128Ctr),
			_ => Err(SerdeError::custom(Error::UnsupportedCipher))
		}
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Aes128Ctr {
	pub iv: H128,
}

#[derive(Debug, PartialEq)]
pub enum CipherSerParams {
	Aes128Ctr(Aes128Ctr),
}

impl Serialize for CipherSerParams {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			CipherSerParams::Aes128Ctr(ref params) => params.serialize(serializer),
		}
	}
}

impl<'a> Deserialize<'a> for CipherSerParams {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'a> {
		Aes128Ctr::deserialize(deserializer)
			.map(CipherSerParams::Aes128Ctr)
			.map_err(|_| Error::InvalidCipherParams)
			.map_err(SerdeError::custom)
	}
}

#[derive(Debug, PartialEq)]
pub enum Cipher {
	Aes128Ctr(Aes128Ctr),
}
