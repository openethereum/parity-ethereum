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
use std::ops::Deref;
use rustc_hex::{ToHex, FromHex};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use ethkey::{Public, Secret, Signature};
use ethereum_types::{H160, H256};
use bytes::Bytes;
use types::all::Requester;

macro_rules! impl_bytes_deserialize {
	($name: ident, $value: expr, true) => {
		$value[2..].from_hex().map($name).map_err(SerdeError::custom)
	};
	($name: ident, $value: expr, false) => {
		$value[2..].parse().map($name).map_err(SerdeError::custom)
	}
}

macro_rules! impl_bytes {
	($name: ident, $other: ident, $from_hex: ident, ($($trait: ident),*)) => {
		#[derive(Clone, Debug, PartialEq, Eq, $($trait,)*)]
		pub struct $name(pub $other);

		impl<T> From<T> for $name where $other: From<T> {
			fn from(s: T) -> $name {
				$name(s.into())
			}
		}

		impl Into<$other> for $name {
			fn into(self) -> $other {
				self.0
			}
		}

		impl Deref for $name {
			type Target = $other;

			fn deref(&self) -> &$other {
				&self.0
			}
		}

		impl Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
				let mut serialized = "0x".to_owned();
				serialized.push_str(self.0.to_hex().as_ref());
				serializer.serialize_str(serialized.as_ref())
			}
		}

		impl<'a> Deserialize<'a> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
				struct HexBytesVisitor;

				impl<'b> Visitor<'b> for HexBytesVisitor {
					type Value = $name;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						write!(formatter, "a hex-encoded bytes string")
					}

					fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
						if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
							impl_bytes_deserialize!($name, value, $from_hex)
						} else {
							Err(SerdeError::custom("invalid format"))
						}
					}

					fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize_any(HexBytesVisitor)
			}
		}
	}
}

/// Serializable message hash.
pub type SerializableMessageHash = SerializableH256;
/// Serializable address;
pub type SerializableAddress = SerializableH160;

/// Serializable Bytes.
impl_bytes!(SerializableBytes, Bytes, true, (Default));
/// Serializable H256.
impl_bytes!(SerializableH256, H256, false, (Default, PartialOrd, Ord));
/// Serializable H160.
impl_bytes!(SerializableH160, H160, false, (Default));
/// Serializable H512 (aka Public).
impl_bytes!(SerializablePublic, Public, false, (Default, PartialOrd, Ord));
/// Serializable Secret.
impl_bytes!(SerializableSecret, Secret, false, ());
/// Serializable Signature.
impl_bytes!(SerializableSignature, Signature, false, ());

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

/// Serializable requester identification data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerializableRequester {
	/// Requested with server key id signature.
	Signature(SerializableSignature),
	/// Requested with public key.
	Public(SerializablePublic),
	/// Requested with verified address.
	Address(SerializableAddress),
}

impl From<SerializableRequester> for Requester {
	fn from(requester: SerializableRequester) -> Requester {
		match requester {
			SerializableRequester::Signature(signature) => Requester::Signature(signature.into()),
			SerializableRequester::Public(public) => Requester::Public(public.into()),
			SerializableRequester::Address(address) => Requester::Address(address.into()),
		}
	}
}

impl From<Requester> for SerializableRequester {
	fn from(requester: Requester) -> SerializableRequester {
		match requester {
			Requester::Signature(signature) => SerializableRequester::Signature(signature.into()),
			Requester::Public(public) => SerializableRequester::Public(public.into()),
			Requester::Address(address) => SerializableRequester::Address(address.into()),
		}
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
