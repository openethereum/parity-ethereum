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

const AES_KEY_LEN: usize = 32;
const AES_NONCE_LEN: usize = 12;

enum EncryptionInner {
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

	/// Encrypt the supplied plaintext
	pub fn encrypt(self, plain: &[u8]) -> Vec<u8> {
		match self.0 {
			EncryptionInner::ECIES(valid_public) => {
				::ethcrypto::ecies::encrypt(&valid_public, &[], plain)
					.expect("validity of public key an invariant of the type; qed")
			}
		}
	}
}

enum DecryptionInner {
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

	/// Decrypt ciphertext. Fails if it's an invalid message.
	pub fn decrypt(self, ciphertext: &[u8]) -> Option<Vec<u8>> {
		match self.0 {
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
		let message = vec![1, 2, 3, 4, 5];

		let instance = EncryptionInstance::ecies(key_pair.public().clone()).unwrap();
		let ciphertext = instance.encrypt(&message);

		let instance = DecryptionInstance::ecies(key_pair.secret().clone()).unwrap();
		let decrypted = instance.decrypt(&ciphertext).unwrap();

		assert_eq!(message, decrypted)
	}
}
