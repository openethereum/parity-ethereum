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
use std::io::{Read, Write};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Error, Visitor, MapAccess, DeserializeOwned};
use serde_json;
use super::{Uuid, Version, Crypto, H160};

/// Public opaque type representing serializable `KeyFile`.
#[derive(Debug, PartialEq)]
pub struct OpaqueKeyFile {
	key_file: KeyFile
}

impl Serialize for OpaqueKeyFile {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
		S: Serializer,
	{
		self.key_file.serialize(serializer)
	}
}

impl<T> From<T> for OpaqueKeyFile where T: Into<KeyFile> {
	fn from(val: T) -> Self {
		OpaqueKeyFile { key_file: val.into() }
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct KeyFile {
	pub id: Uuid,
	pub version: Version,
	pub crypto: Crypto,
	pub address: Option<H160>,
	pub name: Option<String>,
	pub meta: Option<String>,
}

enum KeyFileField {
	Id,
	Version,
	Crypto,
	Address,
	Name,
	Meta,
}

impl<'a> Deserialize<'a> for KeyFileField {
	fn deserialize<D>(deserializer: D) -> Result<KeyFileField, D::Error>
		where D: Deserializer<'a>
	{
		deserializer.deserialize_any(KeyFileFieldVisitor)
	}
}

struct KeyFileFieldVisitor;

impl<'a> Visitor<'a> for KeyFileFieldVisitor {
	type Value = KeyFileField;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid key file field")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where E: Error
	{
		match value {
			"id" => Ok(KeyFileField::Id),
			"version" => Ok(KeyFileField::Version),
			"crypto" => Ok(KeyFileField::Crypto),
			"Crypto" => Ok(KeyFileField::Crypto),
			"address" => Ok(KeyFileField::Address),
			"name" => Ok(KeyFileField::Name),
			"meta" => Ok(KeyFileField::Meta),
			_ => Err(Error::custom(format!("Unknown field: '{}'", value))),
		}
	}
}

impl<'a> Deserialize<'a> for KeyFile {
	fn deserialize<D>(deserializer: D) -> Result<KeyFile, D::Error>
		where D: Deserializer<'a>
	{
		static FIELDS: &'static [&'static str] = &["id", "version", "crypto", "Crypto", "address"];
		deserializer.deserialize_struct("KeyFile", FIELDS, KeyFileVisitor)
	}
}

fn none_if_empty<'a, T>(v: Option<serde_json::Value>) -> Option<T> where
	T: DeserializeOwned
{
	v.and_then(|v| if v.is_null() {
		None
	} else {
		serde_json::from_value(v).ok()
	})

}

struct KeyFileVisitor;
impl<'a> Visitor<'a> for KeyFileVisitor {
	type Value = KeyFile;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid key object")
	}

	fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
		where V: MapAccess<'a>
	{
		let mut id = None;
		let mut version = None;
		let mut crypto = None;
		let mut address = None;
		let mut name = None;
		let mut meta = None;

		loop {
			match visitor.next_key()? {
				Some(KeyFileField::Id) => { id = Some(visitor.next_value()?); }
				Some(KeyFileField::Version) => { version = Some(visitor.next_value()?); }
				Some(KeyFileField::Crypto) => { crypto = Some(visitor.next_value()?); }
				Some(KeyFileField::Address) => { address = Some(visitor.next_value()?); }
				Some(KeyFileField::Name) => { name = none_if_empty(visitor.next_value().ok()) }
				Some(KeyFileField::Meta) => { meta = none_if_empty(visitor.next_value().ok()) }
				None => { break; }
			}
		}

		let id = match id {
			Some(id) => id,
			None => return Err(V::Error::missing_field("id")),
		};

		let version = match version {
			Some(version) => version,
			None => return Err(V::Error::missing_field("version")),
		};

		let crypto = match crypto {
			Some(crypto) => crypto,
			None => return Err(V::Error::missing_field("crypto")),
		};

		let result = KeyFile {
			id: id,
			version: version,
			crypto: crypto,
			address: address,
			name: name,
			meta: meta,
		};

		Ok(result)
	}
}

impl KeyFile {
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: Read {
		serde_json::from_reader(reader)
	}

	pub fn write<W>(&self, writer: &mut W) -> Result<(), serde_json::Error> where W: Write {
		serde_json::to_writer(writer, self)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use serde_json;
	use json::{KeyFile, Uuid, Version, Crypto, Cipher, Aes128Ctr, Kdf, Scrypt};

	#[test]
	fn basic_keyfile() {
		let json = r#"
		{
			"address": "6edddfc6349aff20bc6467ccf276c5b52487f7a8",
			"crypto": {
				"cipher": "aes-128-ctr",
				"ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
				"cipherparams": {
					"iv": "b5a7ec855ec9e2c405371356855fec83"
				},
				"kdf": "scrypt",
				"kdfparams": {
					"dklen": 32,
					"n": 262144,
					"p": 1,
					"r": 8,
					"salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
				},
				"mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
			},
			"id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
			"version": 3,
			"name": "Test",
			"meta": "{}"
		}"#;

		let expected = KeyFile {
			id: Uuid::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
			version: Version::V3,
			address: Some("6edddfc6349aff20bc6467ccf276c5b52487f7a8".into()),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "b5a7ec855ec9e2c405371356855fec83".into(),
				}),
				ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
				}),
				mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
			},
			name: Some("Test".to_owned()),
			meta: Some("{}".to_owned()),
		};

		let keyfile: KeyFile = serde_json::from_str(json).unwrap();
		assert_eq!(keyfile, expected);
	}

	#[test]
	fn capital_crypto_keyfile() {
		let json = r#"
		{
			"address": "6edddfc6349aff20bc6467ccf276c5b52487f7a8",
			"Crypto": {
				"cipher": "aes-128-ctr",
				"ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
				"cipherparams": {
					"iv": "b5a7ec855ec9e2c405371356855fec83"
				},
				"kdf": "scrypt",
				"kdfparams": {
					"dklen": 32,
					"n": 262144,
					"p": 1,
					"r": 8,
					"salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
				},
				"mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
			},
			"id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
			"version": 3
		}"#;

		let expected = KeyFile {
			id: "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73".into(),
			version: Version::V3,
			address: Some("6edddfc6349aff20bc6467ccf276c5b52487f7a8".into()),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "b5a7ec855ec9e2c405371356855fec83".into(),
				}),
				ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
				}),
				mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
			},
			name: None,
			meta: None,
		};

		let keyfile: KeyFile = serde_json::from_str(json).unwrap();
		assert_eq!(keyfile, expected);
	}

	#[test]
	fn to_and_from_json() {
		let file = KeyFile {
			id: "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73".into(),
			version: Version::V3,
			address: Some("6edddfc6349aff20bc6467ccf276c5b52487f7a8".into()),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "b5a7ec855ec9e2c405371356855fec83".into(),
				}),
				ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
				}),
				mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
			},
			name: Some("Test".to_owned()),
			meta: None,
		};

		let serialized = serde_json::to_string(&file).unwrap();
		println!("{}", serialized);
		let deserialized = serde_json::from_str(&serialized).unwrap();

		assert_eq!(file, deserialized);
	}
}
