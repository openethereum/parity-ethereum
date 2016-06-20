use std::ops::{Deref, DerefMut};
use ethkey::{KeyPair, sign, Address, Secret, Signature, Message};
use {json, Error, crypto};
use crypto::Keccak256;
use random::Random;
use account::{Version, Cipher, Kdf, Aes128Ctr, Pbkdf2, Prf};

#[derive(Debug, PartialEq, Clone)]
pub struct Crypto {
	pub cipher: Cipher,
	pub ciphertext: [u8; 32],
	pub kdf: Kdf,
	pub mac: [u8; 32],
}

#[derive(Debug, PartialEq, Clone)]
pub struct SafeAccount {
	pub id: [u8; 16],
	pub version: Version,
	pub address: Address,
	pub crypto: Crypto,
}

impl From<json::Crypto> for Crypto {
	fn from(json: json::Crypto) -> Self {
		Crypto {
			cipher: From::from(json.cipher),
			ciphertext: json.ciphertext.into(),
			kdf: From::from(json.kdf),
			mac: json.mac.into(),
		}
	}
}

impl Into<json::Crypto> for Crypto {
	fn into(self) -> json::Crypto {
		json::Crypto {
			cipher: self.cipher.into(),
			ciphertext: From::from(self.ciphertext),
			kdf: self.kdf.into(),
			mac: From::from(self.mac),
		}
	}
}

impl From<json::KeyFile> for SafeAccount {
	fn from(json: json::KeyFile) -> Self {
		SafeAccount {
			id: json.id.into(),
			version: From::from(json.version),
			address: From::from(json.address), //json.address.into(),
			crypto: From::from(json.crypto),
		}
	}
}

impl Into<json::KeyFile> for SafeAccount {
	fn into(self) -> json::KeyFile {
		json::KeyFile {
			id: From::from(self.id),
			version: self.version.into(),
			address: self.address.into(), //From::from(self.address),
			crypto: self.crypto.into(),
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
		crypto::aes::encrypt(&derived_left_bits, &iv, secret.deref(), &mut ciphertext);

		// KECCAK(DK[16..31] ++ <ciphertext>), where DK[16..31] - derived_right_bits
		let mac = crypto::derive_mac(&derived_right_bits, &ciphertext).keccak256();

		Crypto {
			cipher: Cipher::Aes128Ctr(Aes128Ctr {
				iv: iv,
			}),
			ciphertext: ciphertext,
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
		let (derived_left_bits, derived_right_bits) = match self.kdf {
			Kdf::Pbkdf2(ref params) => crypto::derive_key_iterations(password, &params.salt, params.c),
			Kdf::Scrypt(ref params) => crypto::derive_key_scrypt(password, &params.salt, params.n, params.p, params.r),
		};

		let mac = crypto::derive_mac(&derived_right_bits, &self.ciphertext).keccak256();

		if mac != self.mac {
			return Err(Error::InvalidPassword);
		}

		let mut secret = Secret::default();

		match self.cipher {
			Cipher::Aes128Ctr(ref params) => {
				crypto::aes::decrypt(&derived_left_bits, &params.iv, &self.ciphertext, secret.deref_mut())
			},
		}

		Ok(secret)
	}
}

impl SafeAccount {
	pub fn create(keypair: &KeyPair, id: [u8; 16], password: &str, iterations: u32) -> Self {
		SafeAccount {
			id: id,
			version: Version::V3,
			crypto: Crypto::create(keypair.secret(), password, iterations),
			address: keypair.address(),
		}
	}

	pub fn sign(&self, password: &str, message: &Message) -> Result<Signature, Error> {
		let secret = try!(self.crypto.secret(password));
		sign(&secret, message).map_err(From::from)
	}

	pub fn change_password(&self, old_password: &str, new_password: &str, iterations: u32) -> Result<Self, Error> {
		let secret = try!(self.crypto.secret(old_password));
		let result = SafeAccount {
			id: self.id.clone(),
			version: self.version.clone(),
			crypto: Crypto::create(&secret, new_password, iterations),
			address: self.address.clone(),
		};
		Ok(result)
	}

	pub fn check_password(&self, password: &str) -> bool {
		self.crypto.secret(password).is_ok()
	}
}

#[cfg(test)]
mod tests {
	use ethkey::{Generator, Random, verify, Message};
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
	fn sign_and_verify() {
		let keypair = Random.generate().unwrap();
		let password = "hello world";
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], password, 10240);
		let signature = account.sign(password, &message).unwrap();
		assert!(verify(keypair.public(), &signature, &message).unwrap());
	}

	#[test]
	fn change_password() {
		let keypair = Random.generate().unwrap();
		let first_password = "hello world";
		let sec_password = "this is sparta";
		let i = 10240;
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], first_password, i);
		let new_account = account.change_password(first_password, sec_password, i).unwrap();
		assert!(account.sign(first_password, &message).is_ok());
		assert!(account.sign(sec_password, &message).is_err());
		assert!(new_account.sign(first_password, &message).is_err());
		assert!(new_account.sign(sec_password, &message).is_ok());
	}
}
