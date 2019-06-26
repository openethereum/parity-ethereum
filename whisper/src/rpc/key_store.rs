// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Identity and keystore for Whisper sessions.
//!
//! Can handle symmetric and asymmetric keys.
//! Symmetric encryption is done via AES-256 in GCM mode.

use std::collections::HashMap;

use ethereum_types::H256;
use ethkey::{KeyPair, Public, Secret};
use parity_util_mem::Memzero;
use rand::{Rng, rngs::OsRng};

use rpc::crypto::{AES_KEY_LEN, EncryptionInstance, DecryptionInstance};

/// A symmetric or asymmetric key used for encryption, decryption, and signing
/// of payloads.
pub enum Key {
	/// ECIES key pair for Secp2561k curve. Suitable for encryption, decryption,
	/// and signing.
	Asymmetric(KeyPair),
	/// AES-256 GCM mode. Suitable for encryption, decryption, but not signing.
	Symmetric(Memzero<[u8; AES_KEY_LEN]>),
}

impl Key {
	/// Generate a random asymmetric key with the given cryptographic RNG.
	pub fn new_asymmetric(rng: &mut OsRng) -> Self {
		match ::ethkey::Generator::generate(rng) {
			Ok(pair) => Key::Asymmetric(pair),
			Err(void) => match void {},
		}
	}

	/// Generate a random symmetric key with the given cryptographic RNG.
	pub fn new_symmetric(rng: &mut OsRng) -> Self {
		Key::Symmetric(Memzero::from(rng.gen::<[u8; 32]>()))
	}

	/// From secret asymmetric key. Fails if secret is invalid.
	pub fn from_secret(secret: Secret) -> Option<Self> {
		KeyPair::from_secret(secret).map(Key::Asymmetric).ok()
	}

	/// From raw symmetric key.
	pub fn from_raw_symmetric(key: [u8; AES_KEY_LEN]) -> Self {
		Key::Symmetric(Memzero::from(key))
	}

	/// Get a handle to the public key if this is an asymmetric key.
	pub fn public(&self) -> Option<&Public> {
		match *self {
			Key::Asymmetric(ref pair) => Some(pair.public()),
			Key::Symmetric(_) => None,
		}
	}

	/// Get a handle to the secret key if this is an asymmetric key.
	pub fn secret(&self) -> Option<&Secret> {
		match *self {
			Key::Asymmetric(ref pair) => Some(pair.secret()),
			Key::Symmetric(_) => None,
		}
	}

	/// Get a handle to the symmetric key.
	pub fn symmetric(&self) -> Option<&[u8; AES_KEY_LEN]>  {
		match *self {
			Key::Asymmetric(_) => None,
			Key::Symmetric(ref key) => Some(key),
		}
	}
}

/// Key store.
pub struct KeyStore {
	rng: OsRng,
	identities: HashMap<H256, Key>,
}

impl KeyStore {
	/// Create the key store. Returns any error in accessing the system's secure
	/// RNG.
	pub fn new() -> Result<Self, ::std::io::Error> {
		Ok(KeyStore {
			rng: OsRng::new()?,
			identities: HashMap::new(),
		})
	}

	/// Import a key, generating a random identity for it.
	pub fn insert(&mut self, key: Key) -> H256 {
		let id = H256::random_using(self.rng());
		self.identities.insert(id, key);

		id
	}

	/// Get a key by ID.
	pub fn get<'a>(&'a self, id: &H256) -> Option<&'a Key> {
		self.identities.get(id)
	}

	/// Get asymmetric ID's public key.
	pub fn public<'a>(&'a self, id: &H256) -> Option<&'a Public> {
		self.get(id).and_then(Key::public)
	}

	/// Get asymmetric ID's secret key.
	pub fn secret<'a>(&'a self, id: &H256) -> Option<&'a Secret> {
		self.get(id).and_then(Key::secret)
	}

	/// Get symmetric ID's key.
	pub fn symmetric<'a>(&'a self, id: &H256) -> Option<&'a [u8; AES_KEY_LEN]> {
		self.get(id).and_then(Key::symmetric)
	}

	/// Get encryption instance for identity.
	pub fn encryption_instance(&self, id: &H256) -> Result<EncryptionInstance, &'static str> {
		self.get(id).ok_or("no such identity").and_then(|key| match *key {
			Key::Asymmetric(ref pair) => EncryptionInstance::ecies(pair.public().clone())
				.map_err(|_| "could not create encryption instance for id"),
			Key::Symmetric(ref key) =>
				 OsRng::new()
					.map(|mut rng| EncryptionInstance::aes(key.clone(), rng.gen()))
				 	.map_err(|_| "unable to get secure randomness")
		})
	}

	/// Get decryption instance for identity.
	/// If the identity is known, always succeeds.
	pub fn decryption_instance(&self, id: &H256) -> Option<DecryptionInstance> {
		self.get(id).map(|key| match *key {
			Key::Asymmetric(ref pair) => DecryptionInstance::ecies(pair.secret().clone())
				.expect("all keys stored are valid; qed"),
			Key::Symmetric(ref key) => DecryptionInstance::aes(key.clone()),
		})
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
	pub fn rng(&mut self) -> &mut OsRng {
		&mut self.rng
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rejects_invalid_secret() {
		let bad_secret = ::ethkey::Secret::from([0xff; 32]);
		assert!(Key::from_secret(bad_secret).is_none());
	}

	#[test]
	fn generated_key_should_exist() {
		let mut store = KeyStore::new().unwrap();
		let key = Key::new_asymmetric(store.rng());

		assert!(key.public().is_some());
		assert!(key.secret().is_some());

		let id = store.insert(key);

		assert!(store.contains(&id));
		assert!(store.get(&id).is_some());
	}
}
