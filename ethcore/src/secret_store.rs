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

enum CryptoCipherType {
	// aes-128-ctr with 128-bit initialisation vector(iv)
	Aes128Ctr(U128)
}

enum KeyFileVersion {
	V3(u64)
}

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
enum KdfPbkdf2ParseError {
	InvalidParameter(String)
}

impl KdfPbkdf2Params {
	fn new(_json: &Json) -> Result<KdfPbkdf2Params, KdfPbkdf2ParseError> {
		KdfPbkdf2Params{
			dkLen: 0,
			salt: H256::zero(),
			c: 0,
			prf: Pbkdf2CryptoFunction::HMacSha256
		}
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
}

#[derive(Debug)]
enum ScryptParseError {
	InvalidParameter(String)
}

impl KdfScryptParams {
	fn new(_json: &Json) -> Result<KdfScryptParams, KdfPbkdf2ParseError> {
		Ok(KdfScryptParams{
			dkLen: 0,
			p: 0,
			n: 0,
			r: 0
		})
	}
}

enum Kdf {
	Pbkdf2(KdfPbkdf2Params),
	Scrypt(KdfScryptParams)
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
				match as_object["cipherparams"].as_string() {
					None => { return Err(CryptoParseError::NoCipherParameters); },
					Some(cipher_param) => H128::from(cipher_param)
				}
			),
			Some(oter_cipher_type) => {
				return Err(CryptoParseError::InvalidCipherType(
					Mismatch { expected: "aes-128-ctr".to_owned(), found: other_cipher_type.to_owned() }));
			}
		};

		let kdf = match (as_object["kdf"].as_string(), as_object["kdfparams"]) {
			(None, _) => { return Err(CryptoParseError::NoKdfType); },
			(_, None) => { return Err(CryptoParseError::NoKdfParams); },
			(Some("scrypt"), Some(kdf_params)) =>
				match KdfScryptParams::new(kdf_params) {
					Err(scrypt_params_error) => return Err(CryptoParseError::Scrypt(scrypt_params_error)),
					Ok(scrypt_params) => scrypt_params
				},
			(Some("pbkdf2"), Some(kdf_params)) =>
				match KdfPbkdf2Params::new(kdf_params) {
					Err(kdfPbkdf2_params_error) => return Err(CryptoParseError::Scrypt(scrypt_params_error)),
					Ok(kdfPbkdf2_params) => kdfPbkdf2_params
				},
			(Some(other_kdf), _) => {
				return Err(CryptoParseError::InvalidKdfType(
					Mismatch { expected: "pbkdf2/scrypt".to_owned(), found: other_kdf.to_ownded()}));
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
	InvalidJsonFormat,
	InvalidCryptoVersion,
	NoCryptoVersion,
	InvalidKdfType(Mismatch<String>),
	InvalidCipherType(Mismatch<String>),
	NoCipherText,
	NoKdfType,
	NoKdfParams,
	Scrypt(ScryptParseError),
	KdfPbkdf2(KdfPbkdf2ParseError)
}

#[derive(Debug)]
enum KeyFileParseError {
	InvalidVersion,
	UnsupportedVersion(OutOfBounds<u64>),
	InvalidJsonFormat,
	NoIdentifier,
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

		let id = match as_object["id"].as_string() {
			None => { return Err(KeyFileParseError::NoIdentifier); },
			Some(id) => id
		};

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
