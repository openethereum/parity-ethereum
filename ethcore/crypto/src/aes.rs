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
use rcrypto::blockmodes::{CtrMode, CbcDecryptor, PkcsPadding};
use rcrypto::aessafe::{AesSafe128Encryptor, AesSafe128Decryptor};
use rcrypto::symmetriccipher::{Encryptor, Decryptor};
use rcrypto::buffer::{RefReadBuffer, RefWriteBuffer, WriteBuffer};

/// Encrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn encrypt_128_ctr(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = CtrMode::new(AesSafe128Encryptor::new(k), iv.to_vec());
	encryptor.encrypt(&mut RefReadBuffer::new(plain), &mut RefWriteBuffer::new(dest), true)?;
	Ok(())
}

/// Decrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn decrypt_128_ctr(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = CtrMode::new(AesSafe128Encryptor::new(k), iv.to_vec());
	encryptor.decrypt(&mut RefReadBuffer::new(encrypted), &mut RefWriteBuffer::new(dest), true)?;
	Ok(())
}

/// Decrypt a message (CBC mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn decrypt_128_cbc(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<usize, SymmError> {
	let mut encryptor = CbcDecryptor::new(AesSafe128Decryptor::new(k), PkcsPadding, iv.to_vec());
	let len = dest.len();
	let mut buffer = RefWriteBuffer::new(dest);
	encryptor.decrypt(&mut RefReadBuffer::new(encrypted), &mut buffer, true)?;
	Ok(len - buffer.remaining())
}

