// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
use std::cmp::{Ord, PartialOrd, Ordering};
use std::ops::Deref;
use rustc_serialize::hex::ToHex;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use ethkey::{Public, Secret, Signature};
use util::H256;

#[derive(Clone, Debug)]
/// Serializable Signature.
pub struct SerializableSignature(Signature);

impl<T> From<T> for SerializableSignature where Signature: From<T> {
	fn from(s: T) -> SerializableSignature {
		SerializableSignature(s.into())
	}
}

impl Into<Signature> for SerializableSignature {
	fn into(self) -> Signature {
		self.0
	}
}

impl Deref for SerializableSignature {
	type Target = Signature;

	fn deref(&self) -> &Signature {
		&self.0
	}
}

impl Serialize for SerializableSignature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for SerializableSignature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = SerializableSignature;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded Signature")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| SerializableSignature(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable H256.
pub struct SerializableH256(H256);

impl<T> From<T> for SerializableH256 where H256: From<T> {
	fn from(s: T) -> SerializableH256 {
		SerializableH256(s.into())
	}
}

impl Into<H256> for SerializableH256 {
	fn into(self) -> H256 {
		self.0
	}
}

impl Deref for SerializableH256 {
	type Target = H256;

	fn deref(&self) -> &H256 {
		&self.0
	}
}

impl Serialize for SerializableH256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for SerializableH256 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = SerializableH256;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded H256")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| SerializableH256(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable EC scalar/secret key.
pub struct SerializableSecret(Secret);

impl<T> From<T> for SerializableSecret where Secret: From<T> {
	fn from(s: T) -> SerializableSecret {
		SerializableSecret(s.into())
	}
}

impl Into<Secret> for SerializableSecret {
	fn into(self) -> Secret {
		self.0
	}
}

impl Deref for SerializableSecret {
	type Target = Secret;

	fn deref(&self) -> &Secret {
		&self.0
	}
}

impl Serialize for SerializableSecret {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for SerializableSecret {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = SerializableSecret;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC scalar")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| SerializableSecret(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}

#[derive(Clone, Debug)]
/// Serializable EC point/public key.
pub struct SerializablePublic(Public);

impl<T> From<T> for SerializablePublic where Public: From<T> {
	fn from(p: T) -> SerializablePublic {
		SerializablePublic(p.into())
	}
}

impl Into<Public> for SerializablePublic {
	fn into(self) -> Public {
		self.0
	}
}

impl Deref for SerializablePublic {
	type Target = Public;

	fn deref(&self) -> &Public {
		&self.0
	}
}

impl Eq for SerializablePublic { }

impl PartialEq for SerializablePublic {
	fn eq(&self, other: &SerializablePublic) -> bool {
		self.0.eq(&other.0)
	}
}

impl Ord for SerializablePublic {
	fn cmp(&self, other: &SerializablePublic) -> Ordering {
		self.0.cmp(&other.0)
	}
}

impl PartialOrd for SerializablePublic {
	fn partial_cmp(&self, other: &SerializablePublic) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl Serialize for SerializablePublic {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&(*self.0).to_hex())
	}
}

impl Deserialize for SerializablePublic {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		struct HashVisitor;

		impl Visitor for HashVisitor {
			type Value = SerializablePublic;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC point")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				value.parse().map(|s| SerializablePublic(s)).map_err(SerdeError::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize(HashVisitor)
	}
}
