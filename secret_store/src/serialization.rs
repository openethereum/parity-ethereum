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
use rustc_hex::{ToHex, FromHex};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use ethkey::{Public, Secret, Signature};
use bigint::hash::H256;
use bytes::Bytes;

/// Serializable message hash.
pub type SerializableMessageHash = SerializableH256;

/// Serializable shadow decryption result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableEncryptedDocumentKeyShadow {
	/// Decrypted secret point. It is partially decrypted if shadow decrpytion was requested.
	pub decrypted_secret: SerializablePublic,
	/// Shared common point.
	pub common_point: SerializablePublic,
	/// If shadow decryption was requested: shadow decryption coefficients, encrypted with requestor public.
	pub decrypt_shadows: Vec<SerializableBytes>,
}

/// Serializable Bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializableBytes(pub Bytes);

impl<T> From<T> for SerializableBytes where Bytes: From<T> {
	fn from(s: T) -> SerializableBytes {
		SerializableBytes(s.into())
	}
}

impl Into<Bytes> for SerializableBytes {
	fn into(self) -> Bytes {
		self.0
	}
}

impl Deref for SerializableBytes {
	type Target = Bytes;

	fn deref(&self) -> &Bytes {
		&self.0
	}
}

impl Serialize for SerializableBytes {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for SerializableBytes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		let s = String::deserialize(deserializer)?;
		if s.len() >= 2 && &s[0..2] == "0x" && s.len() & 1 == 0 {
			let data = s[2..].from_hex().map_err(SerdeError::custom)?;
			Ok(SerializableBytes(data))
		} else {
			Err(SerdeError::custom("invalid format"))
		}
	}
}

/// Serializable Signature.
#[derive(Clone, Debug)]
pub struct SerializableSignature(pub Signature);

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
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for SerializableSignature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
		struct HashVisitor;

		impl<'b> Visitor<'b> for HashVisitor {
			type Value = SerializableSignature;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded Signature")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
					value[2..].parse().map(|s| SerializableSignature(s)).map_err(SerdeError::custom)
				} else {
					Err(SerdeError::custom("invalid format"))
				}
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize_any(HashVisitor)
	}
}

/// Serializable H256.
#[derive(Clone, Debug)]
pub struct SerializableH256(pub H256);

impl Default for SerializableH256 {
	fn default() -> Self {
		SerializableH256(Default::default())
	}
}

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
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for SerializableH256 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
		struct HashVisitor;

		impl<'b> Visitor<'b> for HashVisitor {
			type Value = SerializableH256;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded H256")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
					value[2..].parse().map(|s| SerializableH256(s)).map_err(SerdeError::custom)
				} else {
					Err(SerdeError::custom("invalid format"))
				}
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize_any(HashVisitor)
	}
}

impl PartialEq<SerializableH256> for SerializableH256 {
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl Eq for SerializableH256 {
}

impl PartialOrd for SerializableH256 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl Ord for SerializableH256 {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.cmp(&other.0)
	}
}

/// Serializable EC scalar/secret key.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializableSecret(pub Secret);

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

impl AsRef<[u8]> for SerializableSecret {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		&*self.0
	}
}

impl Serialize for SerializableSecret {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for SerializableSecret {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
		struct HashVisitor;

		impl<'b> Visitor<'b> for HashVisitor {
			type Value = SerializableSecret;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC scalar")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
					value[2..].parse().map(|s| SerializableSecret(s)).map_err(SerdeError::custom)
				} else {
					Err(SerdeError::custom("invalid format"))
				}
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize_any(HashVisitor)
	}
}

/// Serializable EC point/public key.
#[derive(Clone, Debug)]
pub struct SerializablePublic(pub Public);

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

impl AsRef<[u8]> for SerializablePublic {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		&*self.0
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
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a> Deserialize<'a> for SerializablePublic {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
		struct HashVisitor;

		impl<'b> Visitor<'b> for HashVisitor {
			type Value = SerializablePublic;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a hex-encoded EC point")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
				if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
					value[2..].parse().map(|s| SerializablePublic(s)).map_err(SerdeError::custom)
				} else {
					Err(SerdeError::custom("invalid format"))
				}
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
				self.visit_str(value.as_ref())
			}
		}

		deserializer.deserialize_any(HashVisitor)
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::{SerializableBytes, SerializablePublic};

	#[test]
	fn serialize_and_deserialize_bytes() {
		let bytes = SerializableBytes(vec![1, 2, 3, 4]);
		let bytes_serialized = serde_json::to_string(&bytes).unwrap();
		assert_eq!(&bytes_serialized, r#""0x01020304""#);
		let bytes_deserialized: SerializableBytes = serde_json::from_str(&bytes_serialized).unwrap();
		assert_eq!(bytes_deserialized, bytes);
	}

	#[test]
	fn serialize_and_deserialize_public() {
		let public = SerializablePublic("cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325".parse().unwrap());
		let public_serialized = serde_json::to_string(&public).unwrap();
		assert_eq!(&public_serialized, r#""0xcac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325""#);
		let public_deserialized: SerializablePublic = serde_json::from_str(&public_serialized).unwrap();
		assert_eq!(public_deserialized, public);
	}
}
