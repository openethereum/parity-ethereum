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

//! Identity and keystore for Whisper sessions.
//!
//! Can handle symmetric and asymmetric keys.
//! Symmetric encryption is done via AES-256 in GCM mode.

use bigint::hash::H256;
use ethcrypto::{self, DEFAULT_MAC as DEFAULT_MAC_ECIES};
use ethkey::{KeyPair, Public, Secret};
use parking_lot::{RwLock, Mutex};
use rand::{Rng, OsRng};
use ring::Unspecified;

const KEY_LEN: usize = 32; // 256 bits
const NONCE_LEN: usize = 12; // 96 bits.

/// A symmetric or asymmetric key used for encryption, decryption, and signing
/// of payloads.
pub enum Key {
	/// ECIES key pair for Secp2561k curve. Suitable for encryption, decryption,
	/// and signing.
	Asymmetric(KeyPair),
	// TODO: (requires a "metadata" to deal with IVs).
	// AES-256 GCM mode. Suitable for encryption, decryption, but not signing.
	// (the two types actually wrap the same key.),
}

impl Key {
	/// Generate a random asymmetric key with the given cryptographic RNG.
	pub fn new_asymmetric(rng: &mut OsRng) -> Self {
		match ::ethkey::Generator::generate(rng) {
			Ok(pair) => Key::Asymmetric(pair),
			Err(void) => match void {},
		}
	}

	/// From secret asymmetric key. Fails if secret is invalid.
	pub fn from_secret(secret: Secret) -> Result<Self, Unspecified> {
		KeyPair::from_secret(secret)
			.map(Key::Asymmetric)
			.map_err(|_| Unspecified)
	}

	/// Get a handle to the public key if this is an asymmetric key.
	pub fn public(&self) -> Option<&Public> {
		match *self {
			Key::Asymmetric(ref pair) => Some(pair.public()),
		}
	}

	/// Get a handle to the secret key if this is an asymmetric key.
	pub fn secret(&self) -> Option<&Secret> {
		match *self {
			Key::Asymmetric(ref pair) => Some(pair.secret()),
		}
	}
}

/// Key store.
pub struct KeyStore {
	identities: HashMap<H256, Key>,
}

impl KeyStore {
	/// Create the key store. Returns any error in accessing the system's secure
	/// RNG.
	pub fn new() -> Result<Self, ::std::io::Error> {
		// create the RNG once so we can assume future creations will succeed.

		let _rng = OsRng::new()?;
		Ok(KeyStore {
			identities: RwLock::new(HashMap::new()),
		})
	}

	/// Import a key, generating a random identity for it.
	pub fn import(&mut self, key: Key) -> H256 {
		let id: H256(self.rng().gen());
		self.identities.insert(id, key);

		id
	}

	/// Get a key by ID.
	pub fn get<'a>(&'a self, id: &H256) -> Option<&'a Key> {
		self.identities.get()
	}

	/// Whether the store contains a key by this ID.
	pub fn contains(&self, id: &H256) -> bool {
		self.identities.contains_key(id)
	}

	/// Remove a key by ID.
	pub fn remove(&mut self, id: &H256) -> bool {
		self.identities.remove(id).is_some()
	}

	/// Get RNG.
	pub fn rng(&self) -> OsRng {
		OsRng::new().expect("Operating system RNG existence checked in constructor; qed")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn aes_key_len_should_be_equal_to_constant() {
		assert_eq!(::ring::aead::AES_256_GCM.key_len(), KEY_LEN);
	}

	#[test]
	fn aes_nonce_len_should_be_equal_to_constant() {
		assert_eq!(::ring::aead::AES_256_GCM.nonce_len(), KEY_LEN);
	}
}
