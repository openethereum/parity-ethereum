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

//! Encryption schemes supported by RPC layer.

use ethkey::{self, Public, Secret};
use ring::aead::{self, AES_256_GCM, SealingKey, OpeningKey};

/// Length of AES key
pub const AES_KEY_LEN: usize = 32;
/// Length of AES nonce (IV)
pub const AES_NONCE_LEN: usize = 12;

enum EncryptionInner {
	AES([u8; AES_KEY_LEN], [u8; AES_NONCE_LEN]),
	ECIES(Public),
}

/// Encryption good for single usage.
pub struct EncryptionInstance(EncryptionInner);

impl EncryptionInstance {
	/// ECIES encryption using public key. Fails if invalid public key.
	pub fn ecies(public: Public) -> Result<Self, &'static str> {
		if !ethkey::public_is_valid(&public) {
			return Err("Invalid public key");
		}

		Ok(EncryptionInstance(EncryptionInner::ECIES(public)))
	}

	/// 256-bit AES GCM encryption with given nonce.
	/// It is extremely insecure to reuse nonces.
	///
	/// If generating nonces with a secure RNG, limit uses such that
	/// the chance of collision is negligible.
	pub fn aes(key: [u8; AES_KEY_LEN], nonce: [u8; AES_NONCE_LEN]) -> Self {
		EncryptionInstance(EncryptionInner::AES(key, nonce))
	}

	/// Encrypt the supplied plaintext
	pub fn encrypt(self, plain: &[u8]) -> Vec<u8> {
		match self.0 {
			EncryptionInner::AES(key, nonce) => {
				let sealing_key = SealingKey::new(&AES_256_GCM, &key)
					.expect("key is of correct len; qed");

				let out_suffix_capacity = AES_256_GCM.tag_len();
				let mut buf = plain.to_vec();
				buf.resize(plain.len() + out_suffix_capacity, 0);

				let out_size = aead::seal_in_place(
					&sealing_key,
					&nonce,
					&[], // no authenticated data.
					&mut buf,
					out_suffix_capacity,
				).expect("key, nonce, buf are valid and out suffix large enough; qed");

				// truncate to the output size and append the nonce.
				buf.truncate(out_size);
				buf.extend(&nonce[..]);

				buf
			}
			EncryptionInner::ECIES(valid_public) => {
				::ethcrypto::ecies::encrypt(&valid_public, &[], plain)
					.expect("validity of public key an invariant of the type; qed")
			}
		}
	}
}

enum DecryptionInner {
	AES([u8; AES_KEY_LEN]),
	ECIES(Secret),
}

/// Decryption instance good for single usage.
pub struct DecryptionInstance(DecryptionInner);

impl DecryptionInstance {
	/// ECIES decryption using secret key. Fails if invalid secret.
	pub fn ecies(secret: Secret) -> Result<Self, &'static str> {
		secret.check_validity().map_err(|_| "Invalid secret key")?;

		Ok(DecryptionInstance(DecryptionInner::ECIES(secret)))
	}

	/// 256-bit AES GCM decryption.
	pub fn aes(key: [u8; AES_KEY_LEN]) -> Self {
		DecryptionInstance(DecryptionInner::AES(key))
	}

	/// Decrypt ciphertext. Fails if it's an invalid message.
	pub fn decrypt(self, ciphertext: &[u8]) -> Option<Vec<u8>> {
		match self.0 {
			DecryptionInner::AES(key) => {
				let min_size = AES_NONCE_LEN + AES_256_GCM.tag_len();
				if ciphertext.len() < min_size { return None }

				let opening_key = OpeningKey::new(&AES_256_GCM, &key)
					.expect("key length is valid for mode; qed");

				// nonce is the suffix of ciphertext.
				let mut nonce = [0; AES_NONCE_LEN];
				let nonce_offset = ciphertext.len() - AES_NONCE_LEN;

				nonce.copy_from_slice(&ciphertext[nonce_offset..]);

				let mut buf = ciphertext[..nonce_offset].to_vec();

				// decrypted plaintext always ends up at the
				// front of the buffer.
				let maybe_decrypted = aead::open_in_place(
					&opening_key,
					&nonce,
					&[], // no authenticated data
					0, // no header.
					&mut buf,
				).ok().map(|plain_slice| plain_slice.len());

				maybe_decrypted.map(move |len| { buf.truncate(len); buf })
			}
			DecryptionInner::ECIES(secret) => {
				// secret is checked for validity, so only fails on invalid message.
				::ethcrypto::ecies::decrypt(&secret, &[], ciphertext).ok()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn aes_key_len_should_be_equal_to_constant() {
		assert_eq!(::ring::aead::AES_256_GCM.key_len(), AES_KEY_LEN);
	}

	#[test]
	fn aes_nonce_len_should_be_equal_to_constant() {
		assert_eq!(::ring::aead::AES_256_GCM.nonce_len(), AES_NONCE_LEN);
	}

	#[test]
	fn encrypt_asymmetric() {
		use ethkey::{Generator, Random};

		let key_pair = Random.generate().unwrap();
		let test_message = move |message: &[u8]| {
			let instance = EncryptionInstance::ecies(key_pair.public().clone()).unwrap();
			let ciphertext = instance.encrypt(&message);

			if !message.is_empty() {
				assert!(&ciphertext[..message.len()] != message)
			}

			let instance = DecryptionInstance::ecies(key_pair.secret().clone()).unwrap();
			let decrypted = instance.decrypt(&ciphertext).unwrap();

			assert_eq!(message, &decrypted[..])
		};

		test_message(&[1, 2, 3, 4, 5]);
		test_message(&[]);
		test_message(&[255; 512]);
	}

	#[test]
	fn encrypt_symmetric() {
		use rand::{Rng, OsRng};

		let mut rng = OsRng::new().unwrap();
		let mut test_message = move |message: &[u8]| {
			let key = rng.gen();

			let instance = EncryptionInstance::aes(key, rng.gen());
			let ciphertext = instance.encrypt(message);

			if !message.is_empty() {
				assert!(&ciphertext[..message.len()] != message)
			}

			let instance = DecryptionInstance::aes(key);
			let decrypted = instance.decrypt(&ciphertext).unwrap();

			assert_eq!(message, &decrypted[..])
		};

		test_message(&[1, 2, 3, 4, 5]);
		test_message(&[]);
		test_message(&[255; 512]);
	}
}
