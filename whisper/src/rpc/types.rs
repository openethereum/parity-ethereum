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

//! Types for Whisper RPC.

use std::fmt;
use std::ops::Deref;

use ethereum_types::{H32, H64, H128, H256, H264, H512};
use hex::{ToHex, FromHex};

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Error, Visitor};

/// Helper trait for generic hex bytes encoding.
pub trait HexEncodable: Sized + ::std::ops::Deref<Target=[u8]> {
	fn from_bytes(bytes: Vec<u8>) -> Option<Self>;
}

impl HexEncodable for Vec<u8> {
	fn from_bytes(bytes: Vec<u8>) -> Option<Self> { Some(bytes) }
}

macro_rules! impl_hex_for_hash {
	($($t: ident)*) => {
		$(
			impl HexEncodable for $t {
				fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
					if bytes.len() != $t::len() {
						None
					} else {
						Some($t::from_slice(&bytes))
					}
				}
			}
		)*
	}
}

impl_hex_for_hash!(
	H32 H64 H128 H256 H264 H512
);

/// Wrapper structure around hex-encoded data.
#[derive(Debug, PartialEq, Eq, Default, Hash, Clone)]
pub struct HexEncode<T>(pub T);

impl<T> From<T> for HexEncode<T> {
	fn from(x: T) -> Self {
		HexEncode(x)
	}
}

impl<T> HexEncode<T> {
	/// Create a new wrapper from the inner value.
	pub fn new(x: T) -> Self { HexEncode(x) }

	/// Consume the wrapper, yielding the inner value.
	pub fn into_inner(self) -> T { self.0 }
}

impl<T> Deref for HexEncode<T> {
	type Target = T;

	fn deref(&self) -> &T { &self.0 }
}

/// Hex-encoded arbitrary-byte vector.
pub type Bytes = HexEncode<Vec<u8>>;

/// 32-byte local identity
pub type Identity = HexEncode<H256>;

/// Public key for ECIES, SECP256k1
pub type Public = HexEncode<::ethkey::Public>;

/// Unvalidated private key for ECIES, SECP256k1
pub type Private = HexEncode<H256>;

/// Abridged topic is four bytes.
// only used in tests for now.
#[cfg(test)]
pub type AbridgedTopic = HexEncode<H32>;

/// 32-byte AES key.
pub type Symmetric = HexEncode<H256>;

impl<T: HexEncodable> Serialize for HexEncode<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let data = &self.0[..];
		let serialized = "0x".to_owned() + &data.to_hex();

		serializer.serialize_str(serialized.as_ref())
	}
}

impl<'a, T: 'a + HexEncodable> Deserialize<'a> for HexEncode<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		deserializer.deserialize_any(HexEncodeVisitor::<T>(::std::marker::PhantomData))
	}
}

// helper type for decoding anything from hex.
struct HexEncodeVisitor<T>(::std::marker::PhantomData<T>);

impl<'a, T: HexEncodable> Visitor<'a> for HexEncodeVisitor<T> {
	type Value = HexEncode<T>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a 0x-prefixed, hex-encoded vector of bytes")
	}

	fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
		let decoded = if value.len() >= 2 && &value[0..2] == "0x" && value.len() & 1 == 0 {
			Ok(Vec::from_hex(&value[2..]).map_err(|_| Error::custom("invalid hex"))?)
		} else {
			Err(Error::custom("invalid format"))
		};

		decoded
			.and_then(|x| T::from_bytes(x).ok_or(Error::custom("invalid format")))
			.map(HexEncode)
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

/// Receiver of a message. Either a public key, identity (presumably symmetric),
/// or broadcast over the topics.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Receiver {
	Public(Public),
	Identity(Identity),
}

/// A request to post a message to the whisper network.
#[derive(Deserialize)]
pub struct PostRequest {
	/// Receiver of the message. Either a public key or
	/// an identity. If the identity is symmetric, it will
	/// encrypt to that identity.
	///
	/// If the receiver is missing, this will be a broadcast message.
	pub to: Option<Receiver>,

	/// Sender of the message.
	///
	/// If present, the payload will be signed by this
	/// identity. The call will fail if the whisper node doesn't store the
	/// signing key for this identity.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub from: Option<Identity>,

	/// Full topics to identify a message by.
	/// At least one topic must be specified if the receiver is
	/// not specified.
	pub topics: Vec<Bytes>,

	/// Payload of the message
	pub payload: Bytes,

	/// Optional padding of the message. No larger than 2^24 - 1.
	pub padding: Option<Bytes>,

	/// Priority of the message: how many milliseconds to spend doing PoW
	pub priority: u64,

	/// Time-To-Live of the message in seconds.
	pub ttl: u64,
}

/// Request for filter or subscription creation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
	/// ID of key used for decryption.
	///
	/// If this identity is removed, then no further messages will be returned.
	///
	/// If optional, this will listen for broadcast messages.
	pub decrypt_with: Option<Identity>,

	/// Accept only messages signed by given public key.
	pub from: Option<Public>,

	/// Possible topics. Cannot be empty if the identity is `None`
	pub topics: Vec<Bytes>,
}

/// A message captured by a filter or subscription.
#[derive(Serialize, Clone)]
pub struct FilterItem {
	/// Public key that signed this message.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub from: Option<Public>,

	/// Identity of recipient. If the filter wasn't registered with a
	/// recipient, this will be `None`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub recipient: Option<Identity>,

	/// Time to live in seconds.
	pub ttl: u64,

	/// Topics that matched the filter.
	pub topics: Vec<Bytes>,

	/// Unix timestamp of the message generation.
	pub timestamp: u64,

	/// Decrypted/Interpreted payload.
	pub payload: Bytes,

	/// Optional padding data.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub padding: Option<Bytes>,
}

/// Whisper node info.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
	/// min PoW to be accepted into the local pool.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "minPow")]
	pub required_pow: Option<f64>,

	/// Number of messages in the pool.
	pub messages: usize,

	/// Memory used by messages in the pool.
	pub memory: usize,

	/// Target memory of the pool.
	pub target_memory: usize,
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;
	use hex::FromHex;

	#[test]
	fn test_bytes_serialize() {
		let bytes = Bytes::new(Vec::from_hex("0123456789abcdef").unwrap());
		let serialized = serde_json::to_string(&bytes).unwrap();
		assert_eq!(serialized, r#""0x0123456789abcdef""#);
	}

	#[test]
	fn test_bytes_deserialize() {
		let bytes2: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""0x123""#);
		let bytes3: Result<Bytes, serde_json::Error> = serde_json::from_str(r#""0xgg""#);

		let bytes4: Bytes = serde_json::from_str(r#""0x""#).unwrap();
		let bytes5: Bytes = serde_json::from_str(r#""0x12""#).unwrap();
		let bytes6: Bytes = serde_json::from_str(r#""0x0123""#).unwrap();

		assert!(bytes2.is_err());
		assert!(bytes3.is_err());
		assert_eq!(bytes4, Bytes::new(vec![]));
		assert_eq!(bytes5, Bytes::new(vec![0x12]));
		assert_eq!(bytes6, Bytes::new(vec![0x1, 0x23]));
	}

	#[test]
	fn deserialize_topic() {
		let topic = AbridgedTopic::new([1, 2, 3, 15].into());

		let topic1: Result<AbridgedTopic, _> = serde_json::from_str(r#""0x010203""#);
		let topic2: Result<AbridgedTopic, _> = serde_json::from_str(r#""0102030F""#);
		let topic3: AbridgedTopic = serde_json::from_str(r#""0x0102030F""#).unwrap();

		assert!(topic1.is_err());
		assert!(topic2.is_err());
		assert_eq!(topic3, topic);
	}
}
