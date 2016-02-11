// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! SecretStore
//! module for managing key files, decrypting and encrypting arbitrary data

use common::*;

#[derive(PartialEq, Debug)]
enum CryptoCipherType {
	// aes-128-ctr with 128-bit initialisation vector(iv)
	Aes128Ctr(U128)
}

#[derive(PartialEq, Debug)]
enum KeyFileVersion {
	V3(u64)
}

#[derive(PartialEq, Debug)]
enum Pbkdf2CryptoFunction {
	HMacSha256
}

#[allow(non_snake_case)]
// Kdf of type `Pbkdf2`
// https://en.wikipedia.org/wiki/PBKDF2
struct KdfPbkdf2Params {
	// desired length of the derived key, in octets
	dkLen: u32,
	// cryptographic salt
	salt: H256,
	// number of iterations for derived key
	c: u32,
	// pseudo-random 2-parameters function
	prf: Pbkdf2CryptoFunction
}

#[derive(Debug)]
enum Pbkdf2ParseError {
	InvalidParameter(&'static str),
	InvalidPrf(Mismatch<String>),
	InvalidSaltFormat(UtilError),
	MissingParameter(&'static str),
}

impl KdfPbkdf2Params {
	fn new(json: &BTreeMap<String, Json>) -> Result<KdfPbkdf2Params, Pbkdf2ParseError> {
		Ok(KdfPbkdf2Params{
			salt: match try!(json.get("salt").ok_or(Pbkdf2ParseError::MissingParameter("salt"))).as_string() {
				None => { return Err(Pbkdf2ParseError::InvalidParameter("salt")) },
				Some(salt_value) => match H256::from_str(salt_value) {
					Ok(salt_hex_value) => salt_hex_value,
					Err(from_hex_error) => { return Err(Pbkdf2ParseError::InvalidSaltFormat(from_hex_error)); },
				}
			},
			prf: match try!(json.get("prf").ok_or(Pbkdf2ParseError::MissingParameter("prf"))).as_string() {
				Some("hmac-sha256") => Pbkdf2CryptoFunction::HMacSha256,
				Some(unexpected_prf) => { return Err(Pbkdf2ParseError::InvalidPrf(Mismatch { expected: "hmac-sha256".to_owned(), found: unexpected_prf.to_owned() })); },
				None => { return Err(Pbkdf2ParseError::InvalidParameter("prf")); },
			},
			dkLen: try!(try!(json.get("dklen").ok_or(Pbkdf2ParseError::MissingParameter("dklen"))).as_u64().ok_or(Pbkdf2ParseError::InvalidParameter("dkLen"))) as u32,
			c: try!(try!(json.get("c").ok_or(Pbkdf2ParseError::MissingParameter("c"))).as_u64().ok_or(Pbkdf2ParseError::InvalidParameter("c"))) as u32,
		})
	}
}

#[allow(non_snake_case)]
// Kdf of type `Scrypt`
// https://en.wikipedia.org/wiki/Scrypt
struct KdfScryptParams {
	// desired length of the derived key, in octets
	dkLen: u32,
	// parallelization
	p: u32,
	// cpu cost
	n: u32,
	// TODO: comment
	r: u32,
	// cryptographic salt
	salt: H256,
}

#[derive(Debug)]
enum ScryptParseError {
	InvalidParameter(&'static str),
	InvalidPrf(Mismatch<String>),
	InvalidSaltFormat(UtilError),
	MissingParameter(&'static str),
}

impl KdfScryptParams {
	fn new(json: &BTreeMap<String, Json>) -> Result<KdfScryptParams, ScryptParseError> {
		Ok(KdfScryptParams{
			salt: match try!(json.get("salt").ok_or(ScryptParseError::MissingParameter("salt"))).as_string() {
				None => { return Err(ScryptParseError::InvalidParameter("salt")) },
				Some(salt_value) => match H256::from_str(salt_value) {
					Ok(salt_hex_value) => salt_hex_value,
					Err(from_hex_error) => { return Err(ScryptParseError::InvalidSaltFormat(from_hex_error)); },
				}
			},
			dkLen: try!(try!(json.get("dklen").ok_or(ScryptParseError::MissingParameter("dklen"))).as_u64().ok_or(ScryptParseError::InvalidParameter("dkLen"))) as u32,
			p: try!(try!(json.get("p").ok_or(ScryptParseError::MissingParameter("p"))).as_u64().ok_or(ScryptParseError::InvalidParameter("p"))) as u32,
			n: try!(try!(json.get("n").ok_or(ScryptParseError::MissingParameter("n"))).as_u64().ok_or(ScryptParseError::InvalidParameter("n"))) as u32,
			r: try!(try!(json.get("r").ok_or(ScryptParseError::MissingParameter("r"))).as_u64().ok_or(ScryptParseError::InvalidParameter("r"))) as u32,
		})
	}
}

enum KeyFileKdf {
	Pbkdf2(KdfPbkdf2Params),
	Scrypt(KdfScryptParams)
}

struct KeyFileCrypto {
	cipher_type: CryptoCipherType,
	cipher_text: Bytes,
	kdf: KeyFileKdf,
}

impl KeyFileCrypto {
	fn new(json: &Json) -> Result<KeyFileCrypto, CryptoParseError> {
		let as_object = match json.as_object() {
			None => { return Err(CryptoParseError::InvalidJsonFormat); }
			Some(obj) => obj
		};

		let cipher_type = match as_object["cipher"].as_string() {
			None => { return Err(CryptoParseError::NoCipherType); }
			Some("aes-128-ctr") => CryptoCipherType::Aes128Ctr(
				match as_object["cipherparams"].as_object() {
					None => { return Err(CryptoParseError::NoCipherParameters); },
					Some(cipher_param) => match U128::from_str(match cipher_param["iv"].as_string() {
							None => { return Err(CryptoParseError::NoInitialVector); },
							Some(iv_hex_string) => iv_hex_string
						})
					{
						Ok(iv_value) => iv_value,
						Err(hex_error) => { return Err(CryptoParseError::InvalidInitialVector(hex_error)); }
					}
				}
			),
			Some(other_cipher_type) => {
				return Err(CryptoParseError::InvalidCipherType(
					Mismatch { expected: "aes-128-ctr".to_owned(), found: other_cipher_type.to_owned() }));
			}
		};

		let kdf = match (as_object["kdf"].as_string(), as_object["kdfparams"].as_object()) {
			(None, _) => { return Err(CryptoParseError::NoKdfType); },
			(Some("scrypt"), Some(kdf_params)) =>
				match KdfScryptParams::new(kdf_params) {
					Err(scrypt_params_error) => { return Err(CryptoParseError::Scrypt(scrypt_params_error)); },
					Ok(scrypt_params) => KeyFileKdf::Scrypt(scrypt_params)
				},
			(Some("pbkdf2"), Some(kdf_params)) =>
				match KdfPbkdf2Params::new(kdf_params) {
					Err(kdfPbkdf2_params_error) => { return Err(CryptoParseError::KdfPbkdf2(kdfPbkdf2_params_error)); },
					Ok(kdfPbkdf2_params) => KeyFileKdf::Pbkdf2(kdfPbkdf2_params)
				},
			(Some(other_kdf), _) => {
				return Err(CryptoParseError::InvalidKdfType(
					Mismatch { expected: "pbkdf2/scrypt".to_owned(), found: other_kdf.to_owned()}));
			}
		};

		let cipher_text = match as_object["ciphertext"].as_string() {
			None => { return Err(CryptoParseError::NoCipherText); }
			Some(text) => text
		};

		Ok(KeyFileCrypto {
			cipher_text: Bytes::from(cipher_text),
			cipher_type: cipher_type,
			kdf: kdf,
		})
	}
}

type Uuid = String;

struct KeyFileContent {
	version: KeyFileVersion,
	crypto: KeyFileCrypto,
	id: Uuid
}

#[derive(Debug)]
enum CryptoParseError {
	NoCryptoVersion,
	NoCipherText,
	NoCipherType,
	InvalidJsonFormat,
	InvalidCryptoVersion,
	InvalidKdfType(Mismatch<String>),
	InvalidCipherType(Mismatch<String>),
	NoInitialVector,
	NoCipherParameters,
	InvalidInitialVector(FromHexError),
	NoKdfType,
	NoKdfParams,
	Scrypt(ScryptParseError),
	KdfPbkdf2(Pbkdf2ParseError)
}

#[derive(Debug)]
enum KeyFileParseError {
	InvalidVersion,
	UnsupportedVersion(OutOfBounds<u64>),
	InvalidJsonFormat,
	InvalidIdentifier,
	NoCryptoSection,
	Crypto(CryptoParseError),
}

impl KeyFileContent {
	fn new(json: &Json) -> Result<KeyFileContent, KeyFileParseError> {
		let as_object = match json.as_object() {
			None => { return Err(KeyFileParseError::InvalidJsonFormat); },
			Some(obj) => obj
		};

		let version = match as_object["version"].as_u64() {
			None => { return Err(KeyFileParseError::InvalidVersion); },
			Some(json_version) => {
				if json_version <= 2 {
					return Err(KeyFileParseError::UnsupportedVersion(OutOfBounds { min: Some(3), max: None, found: json_version }))
				};
				KeyFileVersion::V3(json_version)
			}
		};

		let id = try!(as_object.get("id").and_then(|json| json.as_string()).ok_or(KeyFileParseError::InvalidIdentifier));

		let crypto = match as_object.get("crypto") {
			None => { return Err(KeyFileParseError::NoCryptoSection); }
			Some(crypto_json) => match KeyFileCrypto::new(crypto_json) {
					Ok(crypto) => crypto,
					Err(crypto_error) => { return Err(KeyFileParseError::Crypto(crypto_error)); }
				}
		};

		Ok(KeyFileContent {
			version: version,
			id: id.to_owned(),
			crypto: crypto
		})
	}
}

#[cfg(test)]
mod tests {
	use super::{KeyFileContent, KeyFileVersion, KeyFileKdf, KeyFileParseError};
	use common::*;

	#[test]
	fn can_read_keyfile() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "6087dab2f9fdbbfaddc31a909735c1e6"
						},
						"ciphertext" : "5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46",
						"kdf" : "pbkdf2",
						"kdfparams" : {
							"c" : 262144,
							"dklen" : 32,
							"prf" : "hmac-sha256",
							"salt" : "ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd"
						},
						"mac" : "517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::new(&json) {
			Ok(key_file) => {
				assert_eq!(KeyFileVersion::V3(3), key_file.version)
			},
			Err(e) => panic!("Error parsing valid file: {:?}", e)
		}
	}

	#[test]
	fn can_read_scrypt_krf() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::new(&json) {
			Ok(key_file) => {
				match key_file.crypto.kdf {
					KeyFileKdf::Scrypt(scrypt_params) => {},
					_ => { panic!("expected kdf params of crypto to be of scrypt type" ); }
				}
			},
			Err(e) => panic!("Error parsing valid file: {:?}", e)
		}
	}

	#[test]
	fn can_return_error_no_id() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::new(&json) {
			Ok(key_file) => {
				panic!("Should be error of no crypto section, got ok");
			},
			Err(KeyFileParseError::InvalidIdentifier) => { },
			Err(other_error) => { panic!("should be error of no crypto section, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_return_error_no_crypto() {
		let json = Json::from_str(
			r#"
				{
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::new(&json) {
			Ok(key_file) => {
				panic!("Should be error of no identifier, got ok");
			},
			Err(KeyFileParseError::NoCryptoSection) => { },
			Err(other_error) => { panic!("should be error of no identifier, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_return_error_unsupported_version() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"version" : 1
				}
			"#).unwrap();

		match KeyFileContent::new(&json) {
			Ok(key_file) => {
				panic!("should be error of unsupported version, got ok");
			},
			Err(KeyFileParseError::UnsupportedVersion(_)) => { },
			Err(other_error) => { panic!("should be error of unsupported version, got {:?}", other_error); }
		}
	}
}
