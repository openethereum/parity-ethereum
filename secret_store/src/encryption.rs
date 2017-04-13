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

use std::iter::repeat;
use rand::{Rng, OsRng};
use ethcrypto;
use util::Bytes;
use types::all::Error;

/// Encrypt document with distributely generated key.
pub fn encrypt_document(key: Bytes, document: Bytes) -> Result<Bytes, Error> {
	// make document key
	let key = into_document_key(key)?;

	// use symmetric encryption to encrypt document
	let iv = initialization_vector();
	let mut encrypted_document = Vec::with_capacity(document.len() + iv.len());
	encrypted_document.extend(repeat(0).take(document.len()));
	ethcrypto::aes::encrypt(&key, &iv, &document, &mut encrypted_document);
	encrypted_document.extend_from_slice(&iv);

	Ok(encrypted_document)
}

/// Decrypt document with distributely generated key.
pub fn decrypt_document(key: Bytes, mut encrypted_document: Bytes) -> Result<Bytes, Error> {
	// initialization vector takes 16 bytes
	let encrypted_document_len = encrypted_document.len();
	if encrypted_document_len < 16 {
		return Err(Error::Serde("invalid encrypted data".into()));
	}

	// make document key
	let key = into_document_key(key)?;

	// use symmetric decryption to decrypt document
	let iv = encrypted_document.split_off(encrypted_document_len - 16);
	let mut document = Vec::with_capacity(encrypted_document_len - 16);
	document.extend(repeat(0).take(encrypted_document_len - 16));
	ethcrypto::aes::decrypt(&key, &iv, &encrypted_document, &mut document);

	Ok(document)
}

pub fn decrypt_document_with_shadow(_key: Bytes, mut _encrypted_document: Bytes) -> Result<Bytes, Error> {
	unimplemented!()
}

fn into_document_key(key: Bytes) -> Result<Bytes, Error> {
	// key is a previously distributely generated Public
	if key.len() != 64 {
		return Err(Error::Serde("invalid public key length".into()));
	}

	// use x coordinate of distributely generated point as encryption key
	Ok(key[..16].into())
}

fn initialization_vector() -> [u8; 16] {
	let mut result = [0u8; 16];
	let mut rng = OsRng::new().unwrap();
	rng.fill_bytes(&mut result);
	result
}

#[cfg(test)]
mod tests {
	use util::Bytes;
	use rustc_serialize::hex::FromHex;
	use super::{encrypt_document, decrypt_document};

	#[test]
	fn encrypt_and_decrypt_document() {
		let document_key: Bytes = "cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325".from_hex().unwrap();
		let document: Bytes = b"Hello, world!!!"[..].into();

		let encrypted_document = encrypt_document(document_key.clone(), document.clone()).unwrap();
		assert!(document != encrypted_document);

		let decrypted_document = decrypt_document(document_key.clone(), encrypted_document).unwrap();
		assert_eq!(decrypted_document, document);
	}

	#[test]
	fn encrypt_and_shadow_decrypt_document() {
	}
}
