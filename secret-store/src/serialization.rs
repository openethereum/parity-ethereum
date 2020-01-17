// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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
use std::ops::Deref;
use rustc_hex::{self, FromHex};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use crypto::publickey::{Public, Secret, Signature};
use ethereum_types::{H160, H256};
use bytes::Bytes;
use types::Requester;

trait ToHex {
	fn to_hex(&self) -> String;
}

impl ToHex for Bytes {
	fn to_hex(&self) -> String {
		format!("0x{}", rustc_hex::ToHex::to_hex(&self[..]))
	}
}

impl ToHex for Signature {
	fn to_hex(&self) -> String {
		format!("0x{}", self)
	}
}

impl ToHex for Secret {
	fn to_hex(&self) -> String {
		format!("0x{}", self.to_hex())
	}
}

macro_rules! impl_to_hex {
	($name: ident) => (
		impl ToHex for $name {
			fn to_hex(&self) -> String {
				format!("{:#x}", self)
			}
		}
	);
}

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
				serializer.serialize_str(<$other as ToHex>::to_hex(&self.0).as_ref())
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

impl_to_hex!(H256);
impl_to_hex!(H160);
impl_to_hex!(Public);

impl_bytes!(SerializableBytes, Bytes, true, (Default));
impl_bytes!(SerializableH256, H256, false, (Default, PartialOrd, Ord));
impl_bytes!(SerializableH160, H160, false, (Default));
impl_bytes!(SerializablePublic, Public, false, (Default, PartialOrd, Ord));
impl_bytes!(SerializableSecret, Secret, false, ());
impl_bytes!(SerializableSignature, Signature, false, ());

/// Serializable shadow decryption result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableEncryptedDocumentKeyShadow {
	/// Decrypted secret point. It is partially decrypted if shadow decryption was requested.
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
	use super::*;
	use std::str::FromStr;

	macro_rules! do_test {
		($value: expr, $expected: expr, $expected_type: ident) => (
			let serialized = serde_json::to_string(&$value).unwrap();
			assert_eq!(serialized, $expected);
			let deserialized: $expected_type = serde_json::from_str(&serialized).unwrap();
			assert_eq!(deserialized, $value);
		);
	}

	#[test]
	fn serialize_and_deserialize_bytes() {
		do_test!(SerializableBytes(vec![1, 2, 3, 4]), "\"0x01020304\"".to_owned(), SerializableBytes);
	}

	#[test]
	fn serialize_and_deserialize_h256() {
		let s = "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae";
		let h256 = SerializableH256(H256::from_str(s).unwrap());
		do_test!(h256, format!("\"0x{}\"", s), SerializableH256);
	}

	#[test]
	fn serialize_and_deserialize_h160() {
		let s = "c6d9d2cd449a754c494264e1809c50e34d64562b";
		let h160 = SerializableH160(H160::from_str(s).unwrap());
		do_test!(h160, format!("\"0x{}\"", s), SerializableH160);
	}

	#[test]
	fn serialize_and_deserialize_public() {
		let s = "cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325";
		let public = SerializablePublic(s.parse().unwrap());
		do_test!(public, format!("\"0x{}\"", s), SerializablePublic);
	}

	#[test]
	fn serialize_and_deserialize_secret() {
		let s = "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae";
		let secret = SerializableSecret(Secret::from_str(s).unwrap());
		do_test!(secret, format!("\"0x{}\"", s), SerializableSecret);
	}

	#[test]
	fn serialize_and_deserialize_signature() {
		let raw_r = "afafafafafafafafafafafbcbcbcbcbcbcbcbcbcbeeeeeeeeeeeeedddddddddd";
		let raw_s = "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae";
		let r = H256::from_str(raw_r).unwrap();
		let s = H256::from_str(raw_s).unwrap();
		let v = 42u8;
		let public = SerializableSignature(Signature::from_rsv(&r, &s, v));
		do_test!(public, format!("\"0x{}{}{:x}\"", raw_r, raw_s, v), SerializableSignature);
	}
}
