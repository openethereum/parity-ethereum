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

use tiny_keccak::Keccak;
use rcrypto::pbkdf2::pbkdf2;
use rcrypto::scrypt::{scrypt, ScryptParams};
use rcrypto::sha2::Sha256;
use rcrypto::hmac::Hmac;

pub const KEY_LENGTH: usize = 32;
pub const KEY_ITERATIONS: usize = 10240;
pub const KEY_LENGTH_AES: usize = KEY_LENGTH / 2;

pub fn derive_key_iterations(password: &str, salt: &[u8; 32], c: u32) -> (Vec<u8>, Vec<u8>) {
	let mut h_mac = Hmac::new(Sha256::new(), password.as_bytes());
	let mut derived_key = vec![0u8; KEY_LENGTH];
	pbkdf2(&mut h_mac, salt, c, &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES..KEY_LENGTH];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

pub fn derive_key_scrypt(password: &str, salt: &[u8; 32], n: u32, p: u32, r: u32) -> (Vec<u8>, Vec<u8>) {
	let mut derived_key = vec![0u8; KEY_LENGTH];
	let scrypt_params = ScryptParams::new(n.trailing_zeros() as u8, r, p);
	scrypt(password.as_bytes(), salt, &scrypt_params, &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES..KEY_LENGTH];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

pub fn derive_mac(derived_left_bits: &[u8], cipher_text: &[u8]) -> Vec<u8> {
	let mut mac = vec![0u8; KEY_LENGTH_AES + cipher_text.len()];
	mac[0..KEY_LENGTH_AES].copy_from_slice(derived_left_bits);
	mac[KEY_LENGTH_AES..cipher_text.len() + KEY_LENGTH_AES].copy_from_slice(cipher_text);
	mac
}

pub trait Keccak256<T> {
	fn keccak256(&self) -> T where T: Sized;
}

impl Keccak256<[u8; 32]> for [u8] {
	fn keccak256(&self) -> [u8; 32] {
		let mut keccak = Keccak::new_keccak256();
		let mut result = [0u8; 32];
		keccak.update(self);
		keccak.finalize(&mut result);
		result
	}
}

/// AES encryption
pub mod aes {
	use rcrypto::blockmodes::{CtrMode, CbcDecryptor, PkcsPadding};
	use rcrypto::aessafe::{AesSafe128Encryptor, AesSafe128Decryptor};
	use rcrypto::symmetriccipher::{Encryptor, Decryptor};
	use rcrypto::buffer::{RefReadBuffer, RefWriteBuffer};

	/// Encrypt a message
	pub fn encrypt(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) {
		let mut encryptor = CtrMode::new(AesSafe128Encryptor::new(k), iv.to_vec());
		encryptor.encrypt(&mut RefReadBuffer::new(plain), &mut RefWriteBuffer::new(dest), true).expect("Invalid length or padding");
	}

	/// Decrypt a message
	pub fn decrypt(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) {
		let mut encryptor = CtrMode::new(AesSafe128Encryptor::new(k), iv.to_vec());
		encryptor.decrypt(&mut RefReadBuffer::new(encrypted), &mut RefWriteBuffer::new(dest), true).expect("Invalid length or padding");
	}

	/// Decrypt a message using cbc mode
	pub fn decrypt_cbc(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) {
		let mut encryptor = CbcDecryptor::new(AesSafe128Decryptor::new(k), PkcsPadding, iv.to_vec());
		encryptor.decrypt(&mut RefReadBuffer::new(encrypted), &mut RefWriteBuffer::new(dest), true).expect("Invalid length or padding");
	}

}

