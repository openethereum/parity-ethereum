use std::io::{Read, Write};
use serde::{Deserialize, Deserializer, Error};
use serde::de::{Visitor, MapVisitor};
use serde_json;
use super::{UUID, Version, Crypto, H160};

#[derive(Debug, PartialEq, Serialize)]
pub struct KeyFile {
	pub id: UUID,
	pub version: Version,
	pub crypto: Crypto,
	pub address: H160,
}

enum KeyFileField {
	ID,
	Version,
	Crypto,
	Address,
}

impl Deserialize for KeyFileField {
	fn deserialize<D>(deserializer: &mut D) -> Result<KeyFileField, D::Error>
		where D: Deserializer
	{
		deserializer.deserialize(KeyFileFieldVisitor)
	}
}

struct KeyFileFieldVisitor;

impl Visitor for KeyFileFieldVisitor {
	type Value = KeyFileField;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
		where E: Error
	{
		match value {
			"id" => Ok(KeyFileField::ID),
			"version" => Ok(KeyFileField::Version),
			"crypto" => Ok(KeyFileField::Crypto),
			"Crypto" => Ok(KeyFileField::Crypto),
			"address" => Ok(KeyFileField::Address),
			_ => Err(Error::custom(format!("Unknown field: '{}'", value))),
		}
	}
}

impl Deserialize for KeyFile {
	fn deserialize<D>(deserializer: &mut D) -> Result<KeyFile, D::Error>
		where D: Deserializer
	{
		static FIELDS: &'static [&'static str] = &["id", "version", "crypto", "Crypto", "address"];
		deserializer.deserialize_struct("KeyFile", FIELDS, KeyFileVisitor)
	}
}

struct KeyFileVisitor;

impl Visitor for KeyFileVisitor {
	type Value = KeyFile;

	fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
		where V: MapVisitor
	{
		let mut id = None;
		let mut version = None;
		let mut crypto = None;
		let mut address = None;

		loop {
			match try!(visitor.visit_key()) {
				Some(KeyFileField::ID) => { id = Some(try!(visitor.visit_value())); }
				Some(KeyFileField::Version) => { version = Some(try!(visitor.visit_value())); }
				Some(KeyFileField::Crypto) => { crypto = Some(try!(visitor.visit_value())); }
				Some(KeyFileField::Address) => { address = Some(try!(visitor.visit_value())); }
				None => { break; }
			}
		}

		let id = match id {
			Some(id) => id,
			None => try!(visitor.missing_field("id")),
		};

		let version = match version {
			Some(version) => version,
			None => try!(visitor.missing_field("version")),
		};

		let crypto = match crypto {
			Some(crypto) => crypto,
			None => try!(visitor.missing_field("crypto")),
		};

		let address = match address {
			Some(address) => address,
			None => try!(visitor.missing_field("address")),
		};

		try!(visitor.end());

		let result = KeyFile {
			id: id,
			version: version,
			crypto: crypto,
			address: address,
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
	use json::{KeyFile, UUID, Version, Crypto, Cipher, Aes128Ctr, Kdf, Scrypt, H128, H160, H256};

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
			"version": 3
		}"#;

		let expected = KeyFile {
			id: UUID::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
			version: Version::V3,
			address: H160::from_str("6edddfc6349aff20bc6467ccf276c5b52487f7a8").unwrap(),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: H128::from_str("b5a7ec855ec9e2c405371356855fec83").unwrap(),
				}),
				ciphertext: H256::from_str("7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc").unwrap(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: H256::from_str("1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209").unwrap(),
				}),
				mac: H256::from_str("46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f").unwrap(),
			},
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
			id: UUID::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
			version: Version::V3,
			address: H160::from_str("6edddfc6349aff20bc6467ccf276c5b52487f7a8").unwrap(),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: H128::from_str("b5a7ec855ec9e2c405371356855fec83").unwrap(),
				}),
				ciphertext: H256::from_str("7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc").unwrap(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: H256::from_str("1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209").unwrap(),
				}),
				mac: H256::from_str("46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f").unwrap(),
			},
		};

		let keyfile: KeyFile = serde_json::from_str(json).unwrap();
		assert_eq!(keyfile, expected);
	}

	#[test]
	fn to_and_from_json() {
		let file = KeyFile {
			id: UUID::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
			version: Version::V3,
			address: H160::from_str("6edddfc6349aff20bc6467ccf276c5b52487f7a8").unwrap(),
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: H128::from_str("b5a7ec855ec9e2c405371356855fec83").unwrap(),
				}),
				ciphertext: H256::from_str("7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc").unwrap(),
				kdf: Kdf::Scrypt(Scrypt {
					n: 262144,
					dklen: 32,
					p: 1,
					r: 8,
					salt: H256::from_str("1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209").unwrap(),
				}),
				mac: H256::from_str("46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f").unwrap(),
			},
		};

		let serialized = serde_json::to_string(&file).unwrap();
		let deserialized = serde_json::from_str(&serialized).unwrap();

		assert_eq!(file, deserialized);
	}
}
