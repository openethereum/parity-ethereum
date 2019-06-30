// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! # AES-GCM Module
//!
//! ### Overview
//!
//! Whisper's network protocol is secured using the Galois/Counter Mode of
//! Operation (GCM) block cipher that provides confidentiality and message
//! authentication assurance to inputs on each invocation (cf. [[2]]). 
//!
//! It provides authenticated encryption and signing ("sealing") of data,
//! and authenticated decryption ("opening") of data that has been protected
//! with Authenticated Encryption with Associated Data (AEAD).
//! 
//! It is constructed from the approved symmetric key block cipher Advanced
//! Encryption Standard (AES) with a block size of 256-bits.
//!
//! [1]: https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf
//! [2]: http://luca-giuzzi.unibs.it/corsi/Support/papers-cryptography/gcm-spec.pdf

use ring;

/// Cryptographic secret key to seal the plaintext message, whose length is
/// appropriate for the underlying block cipher.
type SecretKey = [u8; 32];

/// Initialization Vector (IV) is a nonce used during the sealing process.
/// It must be distinct for each invocation of the encryption
/// operation for a fixed value of the Secret Key. It is authenticated but it is
/// not necessary to include it in the Additional Authenticated Data (AAD) field.
type Nonce = [u8; 12];

/// Authentication Tag Length that determines the authentication strength.
type AuthenticationTagLength = usize;

/// Plaintext message data to seal.
type Plaintext = Vec<u8>;

/// Ciphertext.
type Ciphertext = Vec<u8>;

/// AES-GCM encryptor provides the authenticated encryption operation.
pub struct Encryptor<'a> {
    key: ring::aead::SealingKey,
    ad: &'a [u8],
    offset: usize,
}

impl<'a> Encryptor<'a> {
    pub fn aes_256_gcm(key: &SecretKey) -> Result<Encryptor<'a>, ring::error::Unspecified> {
        let sk = ring::aead::SealingKey::new(&ring::aead::AES_256_GCM, key)?;
        Ok(Encryptor {
            key: sk,
            ad: &[],
            offset: 0,
        })
    }

    /// Optional offset value. Only the slice `[offset..]` will be encrypted.
    pub fn offset(&mut self, off: usize) -> &mut Self {
        self.offset = off;
        self
    }

    /// Please note that the pair (key, nonce) must never be reused. Using random nonces
    /// limits the number of messages encrypted with the same key to 2^32 (cf. [[1]])
    ///
    /// [1]: https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf
    pub fn encrypt(&self, nonce: &Nonce, mut data: Plaintext) -> Result<Ciphertext, ring::error::Unspecified> {
        if self.offset > data.len() {
            return Err(ring::error::Unspecified)
        }
        let tag_len: AuthenticationTagLength = ring::aead::AES_256_GCM.tag_len();
        data.extend(::std::iter::repeat(0).take(tag_len));
        let nonce = ring::aead::Nonce::assume_unique_for_key(*nonce);
        let aad = ring::aead::Aad::from(self.ad);
        let len = ring::aead::seal_in_place(&self.key, nonce, aad, &mut data[self.offset ..], tag_len)?;
        data.truncate(self.offset + len);
        Ok(data)
    }
}

/// AES-GCM decryptor provides the authenticated decryption operation.
pub struct Decryptor<'a> {
    key: ring::aead::OpeningKey,
    ad: &'a [u8],
    offset: usize,
}

impl<'a> Decryptor<'a> {
    pub fn aes_256_gcm(key: &SecretKey) -> Result<Decryptor<'a>, ring::error::Unspecified> {
        let ok = ring::aead::OpeningKey::new(&ring::aead::AES_256_GCM, key)?;
        Ok(Decryptor {
            key: ok,
            ad: &[],
            offset: 0,
        })
    }

    pub fn decrypt(&self, nonce: &Nonce, mut data: Ciphertext) -> Result<Plaintext, ring::error::Unspecified> {
        if self.offset > data.len() {
            return Err(ring::error::Unspecified)
        }
        let nonce = ring::aead::Nonce::assume_unique_for_key(*nonce);
        let aad = ring::aead::Aad::from(self.ad);
        let len = ring::aead::open_in_place(&self.key, nonce, aad, 0, &mut data[self.offset ..])?.len();
        data.truncate(self.offset + len);
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::{Encryptor, Decryptor};

    #[test]
    fn aes_gcm_256() {
        let secret = b"12345678901234567890123456789012";
        let nonce = b"123456789012";
        let message = b"So many books, so little time";

        let ciphertext = Encryptor::aes_256_gcm(secret)
            .unwrap()
            .encrypt(nonce, message.to_vec())
            .unwrap();

        assert!(ciphertext != message);

        let plaintext = Decryptor::aes_256_gcm(secret)
            .unwrap()
            .decrypt(nonce, ciphertext)
            .unwrap();

        assert_eq!(plaintext, message)
    }
}
