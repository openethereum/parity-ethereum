// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Crypto utils used ethstore and network.

extern crate crypto as rcrypto;
extern crate ethereum_types;
#[macro_use]
extern crate quick_error;
extern crate ring;
extern crate tiny_keccak;

pub mod aes;
pub mod aes_gcm;
pub mod error;
pub mod scrypt;
pub mod digest;
pub mod hmac;
pub mod pbkdf2;

pub use error::Error;

use tiny_keccak::Keccak;

pub const KEY_LENGTH: usize = 32;
pub const KEY_ITERATIONS: usize = 10240;
pub const KEY_LENGTH_AES: usize = KEY_LENGTH / 2;

/// Default authenticated data to use (in RPC).
pub const DEFAULT_MAC: [u8; 2] = [0, 0];

pub trait Keccak256<T> {
	fn keccak256(&self) -> T where T: Sized;
}

impl<T> Keccak256<[u8; 32]> for T where T: AsRef<[u8]> {
	fn keccak256(&self) -> [u8; 32] {
		let mut keccak = Keccak::new_keccak256();
		let mut result = [0u8; 32];
		keccak.update(self.as_ref());
		keccak.finalize(&mut result);
		result
	}
}

pub fn derive_key_iterations(password: &[u8], salt: &[u8; 32], c: u32) -> (Vec<u8>, Vec<u8>) {
	let mut derived_key = [0u8; KEY_LENGTH];
	pbkdf2::sha256(c, pbkdf2::Salt(salt), pbkdf2::Secret(password), &mut derived_key);
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

pub fn is_equal(a: &[u8], b: &[u8]) -> bool {
	ring::constant_time::verify_slices_are_equal(a, b).is_ok()
}
