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

use std::collections::HashMap;

use bigint::hash::H256;
use ethkey::{KeyPair, Public, Secret};
use rand::{Rng, OsRng};
use ring::error::Unspecified;

use rpc::crypto::DecryptionInstance;

/// A symmetric or asymmetric key used for encryption, decryption, and signing
/// of payloads.
pub enum Key {
	/// ECIES key pair for Secp2561k curve. Suitable for encryption, decryption,
	/// and signing.
	Asymmetric(KeyPair),
	// TODO: (requires a "metadata" to deal with IVs).
	// AES-256 GCM mode. Suitable for encryption, decryption, but not signing.
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
		let id = self.rng().gen();
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

	/// Get decryption instance for identity.
	/// If the identity is known, always succeeds.
	pub fn decryption_instance(&self, id: &H256) -> Option<DecryptionInstance> {
		self.get(id).map(|key| match key {
			&Key::Asymmetric(ref pair) => DecryptionInstance::ecies(pair.secret().clone())
				.expect("all keys stored are valid; qed"),
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
		let bad_secret = ::ethkey::Secret::from_slice(&[0xff; 32]);
		assert!(Key::from_secret(bad_secret).is_err());
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
