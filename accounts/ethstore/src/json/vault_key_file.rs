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

use std::io::{Read, Write};
use serde::de::Error;
use serde_json;
use serde_json::value::Value;
use serde_json::error;
use super::{Uuid, Version, Crypto, H160};

/// Meta key name for vault field
const VAULT_NAME_META_KEY: &'static str = "vault";

/// Key file as stored in vaults
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultKeyFile {
	/// Key id
	pub id: Uuid,
	/// Key version
	pub version: Version,
	/// Secret, encrypted with account password
	pub crypto: Crypto,
	/// Serialized `VaultKeyMeta`, encrypted with vault password
	pub metacrypto: Crypto,
}

/// Data, stored in `VaultKeyFile::metacrypto`
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultKeyMeta {
	/// Key address
	pub address: H160,
	/// Key name
	pub name: Option<String>,
	/// Key metadata
	pub meta: Option<String>,
}

/// Insert vault name to the JSON meta field
pub fn insert_vault_name_to_json_meta(meta: &str, vault_name: &str) -> Result<String, error::Error> {
	let mut meta = if meta.is_empty() {
		Value::Object(serde_json::Map::new())
	} else {
		serde_json::from_str(meta)?
	};

	if let Some(meta_obj) = meta.as_object_mut() {
		meta_obj.insert(VAULT_NAME_META_KEY.to_owned(), Value::String(vault_name.to_owned()));
		serde_json::to_string(meta_obj)
	} else {
		Err(error::Error::custom("Meta is expected to be a serialized JSON object"))
	}
}

/// Remove vault name from the JSON meta field
pub fn remove_vault_name_from_json_meta(meta: &str) -> Result<String, error::Error> {
	let mut meta = if meta.is_empty() {
		Value::Object(serde_json::Map::new())
	} else {
		serde_json::from_str(meta)?
	};

	if let Some(meta_obj) = meta.as_object_mut() {
		meta_obj.remove(VAULT_NAME_META_KEY);
		serde_json::to_string(meta_obj)
	} else {
		Err(error::Error::custom("Meta is expected to be a serialized JSON object"))
	}
}

impl VaultKeyFile {
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: Read {
		serde_json::from_reader(reader)
	}

	pub fn write<W>(&self, writer: &mut W) -> Result<(), serde_json::Error> where W: Write {
		serde_json::to_writer(writer, self)
	}
}

impl VaultKeyMeta {
	pub fn load(bytes: &[u8]) -> Result<Self, serde_json::Error> {
		serde_json::from_slice(&bytes)
	}

	pub fn write(&self) -> Result<Vec<u8>, serde_json::Error> {
		let s = serde_json::to_string(self)?;
		Ok(s.as_bytes().into())
	}
}

#[cfg(test)]
mod test {
	use serde_json;
	use json::{VaultKeyFile, Version, Crypto, Cipher, Aes128Ctr, Kdf, Pbkdf2, Prf,
		insert_vault_name_to_json_meta, remove_vault_name_from_json_meta};

	#[test]
	fn to_and_from_json() {
		let file = VaultKeyFile {
			id: "08d82c39-88e3-7a71-6abb-89c8f36c3ceb".into(),
			version: Version::V3,
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "fecb968bbc8c7e608a89ebcfe53a41d0".into(),
				}),
				ciphertext: "4befe0a66d9a4b6fec8e39eb5c90ac5dafdeaab005fff1af665fd1f9af925c91".into(),
				kdf: Kdf::Pbkdf2(Pbkdf2 {
					c: 10240,
					dklen: 32,
					prf: Prf::HmacSha256,
					salt: "f17731e84ecac390546692dbd4ccf6a3a2720dc9652984978381e61c28a471b2".into(),
				}),
				mac: "7c7c3daafb24cf11eb3079dfb9064a11e92f309a0ee1dd676486bab119e686b7".into(),
			},
			metacrypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "9c353fb3f894fc05946843616c26bb3f".into(),
				}),
				ciphertext: "fef0d113d7576c1702daf380ad6f4c5408389e57991cae2a174facd74bd549338e1014850bddbab7eb486ff5f5c9c5532800c6a6d4db2be2212cd5cd3769244ab230e1f369e8382a9e6d7c0a".into(),
				kdf: Kdf::Pbkdf2(Pbkdf2 {
					c: 10240,
					dklen: 32,
					prf: Prf::HmacSha256,
					salt: "aca82865174a82249a198814b263f43a631f272cbf7ed329d0f0839d259c652a".into(),
				}),
				mac: "b7413946bfe459d2801268dc331c04b3a84d92be11ef4dd9a507f895e8d9b5bd".into(),
			}
		};

		let serialized = serde_json::to_string(&file).unwrap();
		let deserialized = serde_json::from_str(&serialized).unwrap();

		assert_eq!(file, deserialized);
	}

	#[test]
	fn vault_name_inserted_to_json_meta() {
		assert_eq!(insert_vault_name_to_json_meta(r#""#, "MyVault").unwrap(), r#"{"vault":"MyVault"}"#);
		assert_eq!(insert_vault_name_to_json_meta(r#"{"tags":["kalabala"]}"#, "MyVault").unwrap(), r#"{"tags":["kalabala"],"vault":"MyVault"}"#);
	}

	#[test]
	fn vault_name_not_inserted_to_json_meta() {
		assert!(insert_vault_name_to_json_meta(r#"///3533"#, "MyVault").is_err());
		assert!(insert_vault_name_to_json_meta(r#""string""#, "MyVault").is_err());
	}

	#[test]
	fn vault_name_removed_from_json_meta() {
		assert_eq!(remove_vault_name_from_json_meta(r#"{"vault":"MyVault"}"#).unwrap(), r#"{}"#);
		assert_eq!(remove_vault_name_from_json_meta(r#"{"tags":["kalabala"],"vault":"MyVault"}"#).unwrap(), r#"{"tags":["kalabala"]}"#);
	}

	#[test]
	fn vault_name_not_removed_from_json_meta() {
		assert!(remove_vault_name_from_json_meta(r#"///3533"#).is_err());
		assert!(remove_vault_name_from_json_meta(r#""string""#).is_err());
	}
}
