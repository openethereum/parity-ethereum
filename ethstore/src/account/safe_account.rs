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

use ethkey::{KeyPair, sign, Address, Secret, Signature, Message, Public};
use {json, Error, crypto};
use crypto::Keccak256;
use random::Random;
use account::{Version, Cipher, Kdf, Aes128Ctr, Pbkdf2, Prf};

#[derive(Debug, PartialEq, Clone)]
pub struct Crypto {
	pub cipher: Cipher,
	pub ciphertext: Vec<u8>,
	pub kdf: Kdf,
	pub mac: [u8; 32],
}

#[derive(Debug, PartialEq, Clone)]
pub struct SafeAccount {
	pub id: [u8; 16],
	pub version: Version,
	pub address: Address,
	pub crypto: Crypto,
	pub filename: Option<String>,
	pub name: String,
	pub meta: String,
}

impl From<json::Crypto> for Crypto {
	fn from(json: json::Crypto) -> Self {
		Crypto {
			cipher: json.cipher.into(),
			ciphertext: json.ciphertext.into(),
			kdf: json.kdf.into(),
			mac: json.mac.into(),
		}
	}
}

impl Into<json::Crypto> for Crypto {
	fn into(self) -> json::Crypto {
		json::Crypto {
			cipher: self.cipher.into(),
			ciphertext: self.ciphertext.into(),
			kdf: self.kdf.into(),
			mac: self.mac.into(),
		}
	}
}

impl Into<json::KeyFile> for SafeAccount {
	fn into(self) -> json::KeyFile {
		json::KeyFile {
			id: From::from(self.id),
			version: self.version.into(),
			address: self.address.into(),
			crypto: self.crypto.into(),
			name: Some(self.name.into()),
			meta: Some(self.meta.into()),
		}
	}
}

impl Crypto {
	pub fn create(secret: &Secret, password: &str, iterations: u32) -> Self {
		let salt: [u8; 32] = Random::random();
		let iv: [u8; 16] = Random::random();

		// two parts of derived key
		// DK = [ DK[0..15] DK[16..31] ] = [derived_left_bits, derived_right_bits]
		let (derived_left_bits, derived_right_bits) = crypto::derive_key_iterations(password, &salt, iterations);

		let mut ciphertext = [0u8; 32];

		// aes-128-ctr with initial vector of iv
		crypto::aes::encrypt(&derived_left_bits, &iv, &**secret, &mut ciphertext);

		// KECCAK(DK[16..31] ++ <ciphertext>), where DK[16..31] - derived_right_bits
		let mac = crypto::derive_mac(&derived_right_bits, &ciphertext).keccak256();

		Crypto {
			cipher: Cipher::Aes128Ctr(Aes128Ctr {
				iv: iv,
			}),
			ciphertext: ciphertext.to_vec(),
			kdf: Kdf::Pbkdf2(Pbkdf2 {
				dklen: crypto::KEY_LENGTH as u32,
				salt: salt,
				c: iterations,
				prf: Prf::HmacSha256,
			}),
			mac: mac,
		}
	}

	pub fn secret(&self, password: &str) -> Result<Secret, Error> {
		if self.ciphertext.len() > 32 {
			return Err(Error::InvalidSecret);
		}

		let (derived_left_bits, derived_right_bits) = match self.kdf {
			Kdf::Pbkdf2(ref params) => crypto::derive_key_iterations(password, &params.salt, params.c),
			Kdf::Scrypt(ref params) => crypto::derive_key_scrypt(password, &params.salt, params.n, params.p, params.r)?,
		};

		let mac = crypto::derive_mac(&derived_right_bits, &self.ciphertext).keccak256();

		if mac != self.mac {
			return Err(Error::InvalidPassword);
		}

		match self.cipher {
			Cipher::Aes128Ctr(ref params) => {
				let from = 32 - self.ciphertext.len();
				let mut secret = [0; 32];
				crypto::aes::decrypt(&derived_left_bits, &params.iv, &self.ciphertext, &mut secret[from..]);
				Ok(Secret::from_slice(&secret)?)
			},
		}
	}
}

impl SafeAccount {
	pub fn create(
		keypair: &KeyPair,
		id: [u8; 16],
		password: &str,
		iterations: u32,
		name: String,
		meta: String
	) -> Self {
		SafeAccount {
			id: id,
			version: Version::V3,
			crypto: Crypto::create(keypair.secret(), password, iterations),
			address: keypair.address(),
			filename: None,
			name: name,
			meta: meta,
		}
	}

	/// Create a new `SafeAccount` from the given `json`; if it was read from a
	/// file, the `filename` should be `Some` name. If it is as yet anonymous, then it
	/// can be left `None`.
	pub fn from_file(json: json::KeyFile, filename: Option<String>) -> Self {
		SafeAccount {
			id: json.id.into(),
			version: json.version.into(),
			address: json.address.into(),
			crypto: json.crypto.into(),
			filename: filename,
			name: json.name.unwrap_or(String::new()),
			meta: json.meta.unwrap_or("{}".to_owned()),
		}
	}

	pub fn sign(&self, password: &str, message: &Message) -> Result<Signature, Error> {
		let secret = self.crypto.secret(password)?;
		sign(&secret, message).map_err(From::from)
	}

	pub fn decrypt(&self, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let secret = self.crypto.secret(password)?;
		crypto::ecies::decrypt(&secret, shared_mac, message).map_err(From::from)
	}

	pub fn public(&self, password: &str) -> Result<Public, Error> {
		let secret = self.crypto.secret(password)?;
		Ok(KeyPair::from_secret(secret)?.public().clone())
	}

	pub fn change_password(&self, old_password: &str, new_password: &str, iterations: u32) -> Result<Self, Error> {
		let secret = self.crypto.secret(old_password)?;
		let result = SafeAccount {
			id: self.id.clone(),
			version: self.version.clone(),
			crypto: Crypto::create(&secret, new_password, iterations),
			address: self.address.clone(),
			filename: self.filename.clone(),
			name: self.name.clone(),
			meta: self.meta.clone(),
		};
		Ok(result)
	}

	pub fn check_password(&self, password: &str) -> bool {
		self.crypto.secret(password).is_ok()
	}
}

#[cfg(test)]
mod tests {
	use ethkey::{Generator, Random, verify_public, Message};
	use super::{Crypto, SafeAccount};

	#[test]
	fn crypto_create() {
		let keypair = Random.generate().unwrap();
		let crypto = Crypto::create(keypair.secret(), "this is sparta", 10240);
		let secret = crypto.secret("this is sparta").unwrap();
		assert_eq!(keypair.secret(), &secret);
	}

	#[test]
	#[should_panic]
	fn crypto_invalid_password() {
		let keypair = Random.generate().unwrap();
		let crypto = Crypto::create(keypair.secret(), "this is sparta", 10240);
		let _ = crypto.secret("this is sparta!").unwrap();
	}

	#[test]
	fn sign_and_verify_public() {
		let keypair = Random.generate().unwrap();
		let password = "hello world";
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], password, 10240, "Test".to_owned(), "{}".to_owned());
		let signature = account.sign(password, &message).unwrap();
		assert!(verify_public(keypair.public(), &signature, &message).unwrap());
	}

	#[test]
	fn change_password() {
		let keypair = Random.generate().unwrap();
		let first_password = "hello world";
		let sec_password = "this is sparta";
		let i = 10240;
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], first_password, i, "Test".to_owned(), "{}".to_owned());
		let new_account = account.change_password(first_password, sec_password, i).unwrap();
		assert!(account.sign(first_password, &message).is_ok());
		assert!(account.sign(sec_password, &message).is_err());
		assert!(new_account.sign(first_password, &message).is_err());
		assert!(new_account.sign(sec_password, &message).is_ok());
	}
}
