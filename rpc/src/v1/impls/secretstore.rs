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

//! SecretStore-specific rpc implementation.

use std::collections::BTreeSet;
use std::sync::Arc;

use crypto::DEFAULT_MAC;
use ethkey::Secret;
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Result;
use v1::helpers::errors;
use v1::helpers::accounts::unwrap_provider;
use v1::helpers::secretstore::{encrypt_document, decrypt_document, decrypt_document_with_shadow, ordered_servers_keccak};
use v1::traits::SecretStore;
use v1::types::{H160, H512, Bytes};

/// Parity implementation.
pub struct SecretStoreClient {
	accounts: Option<Arc<AccountProvider>>,
}

impl SecretStoreClient {
	/// Creates new SecretStoreClient
	pub fn new(store: &Option<Arc<AccountProvider>>) -> Self {
		SecretStoreClient {
			accounts: store.clone(),
		}
	}

	/// Attempt to get the `Arc<AccountProvider>`, errors if provider was not
	/// set.
	fn account_provider(&self) -> Result<Arc<AccountProvider>> {
		unwrap_provider(&self.accounts)
	}

	/// Decrypt public key using account' private key
	fn decrypt_key(&self, address: H160, password: String, key: Bytes) -> Result<Vec<u8>> {
		let store = self.account_provider()?;
		store.decrypt(address.into(), Some(password), &DEFAULT_MAC, &key.0)
			.map_err(|e| errors::account("Could not decrypt key.", e))
	}

	/// Decrypt secret key using account' private key
	fn decrypt_secret(&self, address: H160, password: String, key: Bytes) -> Result<Secret> {
		self.decrypt_key(address, password, key)
			.and_then(|s| Secret::from_unsafe_slice(&s).map_err(|e| errors::account("invalid secret", e)))
	}
}

impl SecretStore for SecretStoreClient {
	fn encrypt(&self, address: H160, password: String, key: Bytes, data: Bytes) -> Result<Bytes> {
		encrypt_document(self.decrypt_key(address, password, key)?, data.0)
			.map(Into::into)
	}

	fn decrypt(&self, address: H160, password: String, key: Bytes, data: Bytes) -> Result<Bytes> {
		decrypt_document(self.decrypt_key(address, password, key)?, data.0)
			.map(Into::into)
	}

	fn shadow_decrypt(&self, address: H160, password: String, decrypted_secret: H512, common_point: H512, decrypt_shadows: Vec<Bytes>, data: Bytes) -> Result<Bytes> {
		let mut shadows = Vec::with_capacity(decrypt_shadows.len());
		for decrypt_shadow in decrypt_shadows {
			shadows.push(self.decrypt_secret(address.clone(), password.clone(), decrypt_shadow)?);
		}

		decrypt_document_with_shadow(decrypted_secret.into(), common_point.into(), shadows, data.0)
			.map(Into::into)
	}

	fn sign_servers_set(&self, address: H160, password: String, servers_set: BTreeSet<H512>) -> Result<Bytes> {
		let servers_set_keccak_value = ordered_servers_keccak(servers_set);
		let store = self.account_provider()?;
		store
			.sign(address.into(), Some(password), servers_set_keccak_value.into())
			.map(|s| Bytes::new((*s).to_vec()))
			.map_err(|e| errors::account("Could not sign servers set.", e))
	}
}
