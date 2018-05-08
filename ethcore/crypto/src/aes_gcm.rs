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

use error::SymmError;
use ring;

enum Mode { Aes128Gcm, Aes256Gcm }

/// AES GCM encryptor.
pub struct Encryptor<'a> {
	mode: Mode,
	key: ring::aead::SealingKey,
	ad: &'a [u8],
	offset: usize,
}

impl<'a> Encryptor<'a> {
	pub fn aes_128_gcm(key: &[u8; 16]) -> Result<Encryptor<'a>, SymmError> {
		let sk = ring::aead::SealingKey::new(&ring::aead::AES_128_GCM, key)?;
		Ok(Encryptor {
			mode: Mode::Aes128Gcm,
			key: sk,
			ad: &[],
			offset: 0,
		})
	}

	pub fn aes_256_gcm(key: &[u8; 32]) -> Result<Encryptor<'a>, SymmError> {
		let sk = ring::aead::SealingKey::new(&ring::aead::AES_256_GCM, key)?;
		Ok(Encryptor {
			mode: Mode::Aes256Gcm,
			key: sk,
			ad: &[],
			offset: 0,
		})
	}

	/// Optional associated data which is not encrypted but authenticated.
	pub fn associate(&mut self, data: &'a [u8]) -> &mut Self {
		self.ad = data;
		self
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
	pub fn encrypt(&self, nonce: &[u8; 12], mut data: Vec<u8>) -> Result<Vec<u8>, SymmError> {
		if self.offset > data.len() {
			return Err(SymmError::offset_error(self.offset))
		}
		let tag_len = match self.mode {
			Mode::Aes128Gcm => ring::aead::AES_128_GCM.tag_len(),
			Mode::Aes256Gcm => ring::aead::AES_256_GCM.tag_len(),
		};
		data.extend(::std::iter::repeat(0).take(tag_len));
		let len = ring::aead::seal_in_place(&self.key, nonce, self.ad, &mut data[self.offset ..], tag_len)?;
		data.truncate(self.offset + len);
		Ok(data)
	}
}

/// AES GCM decryptor.
pub struct Decryptor<'a> {
	key: ring::aead::OpeningKey,
	ad: &'a [u8],
	offset: usize,
}

impl<'a> Decryptor<'a> {
	pub fn aes_128_gcm(key: &[u8; 16]) -> Result<Decryptor<'a>, SymmError> {
		let ok = ring::aead::OpeningKey::new(&ring::aead::AES_128_GCM, key)?;
		Ok(Decryptor {
			key: ok,
			ad: &[],
			offset: 0,
		})
	}

	pub fn aes_256_gcm(key: &[u8; 32]) -> Result<Decryptor<'a>, SymmError> {
		let ok = ring::aead::OpeningKey::new(&ring::aead::AES_256_GCM, key)?;
		Ok(Decryptor {
			key: ok,
			ad: &[],
			offset: 0,
		})
	}

	/// Optional associated data which is not encrypted but authenticated.
	pub fn associate(&mut self, data: &'a [u8]) -> &mut Self {
		self.ad = data;
		self
	}

	/// Optional offset value. Only the slice `[offset..]` will be decrypted.
	pub fn offset(&mut self, off: usize) -> &mut Self {
		self.offset = off;
		self
	}

	pub fn decrypt(&self, nonce: &[u8; 12], mut data: Vec<u8>) -> Result<Vec<u8>, SymmError> {
		if self.offset > data.len() {
			return Err(SymmError::offset_error(self.offset))
		}
		let len = ring::aead::open_in_place(&self.key, nonce, self.ad, 0, &mut data[self.offset ..])?.len();
		data.truncate(self.offset + len);
		Ok(data)
	}
}

#[cfg(test)]
mod tests {
	use super::{Encryptor, Decryptor};

	#[test]
	fn aes_gcm_128() {
		let secret = b"1234567890123456";
		let nonce = b"123456789012";
		let message = b"So many books, so little time";

		let ciphertext = Encryptor::aes_128_gcm(secret)
			.unwrap()
			.encrypt(nonce, message.to_vec())
			.unwrap();

		assert!(ciphertext != message);

		let plaintext = Decryptor::aes_128_gcm(secret)
			.unwrap()
			.decrypt(nonce, ciphertext)
			.unwrap();

		assert_eq!(plaintext, message)
	}

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

	#[test]
	fn aes_gcm_256_offset() {
		let secret = b"12345678901234567890123456789012";
		let nonce = b"123456789012";
		let message = b"prefix data; So many books, so little time";

		let ciphertext = Encryptor::aes_256_gcm(secret)
			.unwrap()
			.offset(13) // length of "prefix data; "
			.encrypt(nonce, message.to_vec())
			.unwrap();

		assert!(ciphertext != &message[..]);

		let plaintext = Decryptor::aes_256_gcm(secret)
			.unwrap()
			.offset(13) // length of "prefix data; "
			.decrypt(nonce, ciphertext)
			.unwrap();

		assert_eq!(plaintext, &message[..])
	}
}

