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

use ethkey::{self, KeyPair, sign, Address, Password, Signature, Message, Public, Secret};
use ethkey::crypto::ecdh::agree;
use {json, Error};
use account::Version;
use crypto;
use super::crypto::Crypto;

/// Account representation.
#[derive(Debug, PartialEq, Clone)]
pub struct SafeAccount {
	/// Account ID
	pub id: [u8; 16],
	/// Account version
	pub version: Version,
	/// Account address
	pub address: Address,
	/// Account private key derivation definition.
	pub crypto: Crypto,
	/// Account filename
	pub filename: Option<String>,
	/// Account name
	pub name: String,
	/// Account metadata
	pub meta: String,
}

impl Into<json::KeyFile> for SafeAccount {
	fn into(self) -> json::KeyFile {
		json::KeyFile {
			id: From::from(self.id),
			version: self.version.into(),
			address: self.address.into(),
			crypto: self.crypto.into(),
			name: Some(self.name.into()),
			meta: Some(self.meta.into()),
		}
	}
}

impl SafeAccount {
	/// Create a new account
	pub fn create(
		keypair: &KeyPair,
		id: [u8; 16],
		password: &Password,
		iterations: u32,
		name: String,
		meta: String
	) -> Result<Self, crypto::Error> {
		Ok(SafeAccount {
			id: id,
			version: Version::V3,
			crypto: Crypto::with_secret(keypair.secret(), password, iterations)?,
			address: keypair.address(),
			filename: None,
			name: name,
			meta: meta,
		})
	}

	/// Create a new `SafeAccount` from the given `json`; if it was read from a
	/// file, the `filename` should be `Some` name. If it is as yet anonymous, then it
	/// can be left `None`.
	pub fn from_file(json: json::KeyFile, filename: Option<String>) -> Self {
		SafeAccount {
			id: json.id.into(),
			version: json.version.into(),
			address: json.address.into(),
			crypto: json.crypto.into(),
			filename: filename,
			name: json.name.unwrap_or(String::new()),
			meta: json.meta.unwrap_or("{}".to_owned()),
		}
	}

	/// Create a new `SafeAccount` from the given vault `json`; if it was read from a
	/// file, the `filename` should be `Some` name. If it is as yet anonymous, then it
	/// can be left `None`.
	pub fn from_vault_file(password: &Password, json: json::VaultKeyFile, filename: Option<String>) -> Result<Self, Error> {
		let meta_crypto: Crypto = json.metacrypto.into();
		let meta_plain = meta_crypto.decrypt(password)?;
		let meta_plain = json::VaultKeyMeta::load(&meta_plain).map_err(|e| Error::Custom(format!("{:?}", e)))?;

		Ok(SafeAccount::from_file(json::KeyFile {
			id: json.id,
			version: json.version,
			crypto: json.crypto,
			address: meta_plain.address,
			name: meta_plain.name,
			meta: meta_plain.meta,
		}, filename))
	}

	/// Create a new `VaultKeyFile` from the given `self`
	pub fn into_vault_file(self, iterations: u32, password: &Password) -> Result<json::VaultKeyFile, Error> {
		let meta_plain = json::VaultKeyMeta {
			address: self.address.into(),
			name: Some(self.name),
			meta: Some(self.meta),
		};
		let meta_plain = meta_plain.write().map_err(|e| Error::Custom(format!("{:?}", e)))?;
		let meta_crypto = Crypto::with_plain(&meta_plain, password, iterations)?;

		Ok(json::VaultKeyFile {
			id: self.id.into(),
			version: self.version.into(),
			crypto: self.crypto.into(),
			metacrypto: meta_crypto.into(),
		})
	}

	/// Sign a message.
	pub fn sign(&self, password: &Password, message: &Message) -> Result<Signature, Error> {
		let secret = self.crypto.secret(password)?;
		sign(&secret, message).map_err(From::from)
	}

	/// Decrypt a message.
	pub fn decrypt(&self, password: &Password, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let secret = self.crypto.secret(password)?;
		ethkey::crypto::ecies::decrypt(&secret, shared_mac, message).map_err(From::from)
	}

	/// Agree on shared key.
	pub fn agree(&self, password: &Password, other: &Public) -> Result<Secret, Error> {
		let secret = self.crypto.secret(password)?;
		agree(&secret, other).map_err(From::from)
	}

	/// Derive public key.
	pub fn public(&self, password: &Password) -> Result<Public, Error> {
		let secret = self.crypto.secret(password)?;
		Ok(KeyPair::from_secret(secret)?.public().clone())
	}

	/// Change account's password.
	pub fn change_password(&self, old_password: &Password, new_password: &Password, iterations: u32) -> Result<Self, Error> {
		let secret = self.crypto.secret(old_password)?;
		let result = SafeAccount {
			id: self.id.clone(),
			version: self.version.clone(),
			crypto: Crypto::with_secret(&secret, new_password, iterations)?,
			address: self.address.clone(),
			filename: self.filename.clone(),
			name: self.name.clone(),
			meta: self.meta.clone(),
		};
		Ok(result)
	}

	/// Check if password matches the account.
	pub fn check_password(&self, password: &Password) -> bool {
		self.crypto.secret(password).is_ok()
	}
}

#[cfg(test)]
mod tests {
	use ethkey::{Generator, Random, verify_public, Message};
	use super::SafeAccount;

	#[test]
	fn sign_and_verify_public() {
		let keypair = Random.generate().unwrap();
		let password = "hello world".into();
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], &password, 10240, "Test".to_owned(), "{}".to_owned());
		let signature = account.unwrap().sign(&password, &message).unwrap();
		assert!(verify_public(keypair.public(), &signature, &message).unwrap());
	}

	#[test]
	fn change_password() {
		let keypair = Random.generate().unwrap();
		let first_password = "hello world".into();
		let sec_password = "this is sparta".into();
		let i = 10240;
		let message = Message::default();
		let account = SafeAccount::create(&keypair, [0u8; 16], &first_password, i, "Test".to_owned(), "{}".to_owned()).unwrap();
		let new_account = account.change_password(&first_password, &sec_password, i).unwrap();
		assert!(account.sign(&first_password, &message).is_ok());
		assert!(account.sign(&sec_password, &message).is_err());
		assert!(new_account.sign(&first_password, &message).is_err());
		assert!(new_account.sign(&sec_password, &message).is_ok());
	}
}
