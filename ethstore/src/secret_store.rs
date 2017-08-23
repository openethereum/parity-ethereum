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

use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use ethkey::{Address, Message, Signature, Secret, Public};
use Error;
use json::{Uuid, OpaqueKeyFile};
use bigint::hash::H256;
use OpaqueSecret;

/// Key directory reference
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretVaultRef {
	/// Reference to key in root directory
	Root,
	/// Referenc to key in specific vault
	Vault(String),
}

/// Stored account reference
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoreAccountRef {
	/// Vault reference
	pub vault: SecretVaultRef,
	/// Account address
	pub address: Address,
}

/// Simple Secret Store API
pub trait SimpleSecretStore: Send + Sync {
	/// Inserts new accounts to the store (or vault) with given password.
	fn insert_account(&self, vault: SecretVaultRef, secret: Secret, password: &str) -> Result<StoreAccountRef, Error>;
	/// Inserts new derived account to the store (or vault) with given password.
	fn insert_derived(&self, vault: SecretVaultRef, account_ref: &StoreAccountRef, password: &str, derivation: Derivation) -> Result<StoreAccountRef, Error>;
	/// Changes accounts password.
	fn change_password(&self, account: &StoreAccountRef, old_password: &str, new_password: &str) -> Result<(), Error>;
	/// Exports key details for account.
	fn export_account(&self, account: &StoreAccountRef, password: &str) -> Result<OpaqueKeyFile, Error>;
	/// Entirely removes account from the store and underlying storage.
	fn remove_account(&self, account: &StoreAccountRef, password: &str) -> Result<(), Error>;
	/// Generates new derived account.
	fn generate_derived(&self, account_ref: &StoreAccountRef, password: &str, derivation: Derivation) -> Result<Address, Error>;
	/// Sign a message with given account.
	fn sign(&self, account: &StoreAccountRef, password: &str, message: &Message) -> Result<Signature, Error>;
	/// Sign a message with derived account.
	fn sign_derived(&self, account_ref: &StoreAccountRef, password: &str, derivation: Derivation, message: &Message) -> Result<Signature, Error>;
	/// Decrypt a messages with given account.
	fn decrypt(&self, account: &StoreAccountRef, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error>;
	/// Agree on shared key.
	fn agree(&self, account: &StoreAccountRef, password: &str, other: &Public) -> Result<Secret, Error>;

	/// Returns all accounts in this secret store.
	fn accounts(&self) -> Result<Vec<StoreAccountRef>, Error>;
	/// Get reference to some account with given address.
	/// This method could be removed if we will guarantee that there is max(1) account for given address.
	fn account_ref(&self, address: &Address) -> Result<StoreAccountRef, Error>;

	/// Create new vault with given password
	fn create_vault(&self, name: &str, password: &str) -> Result<(), Error>;
	/// Open vault with given password
	fn open_vault(&self, name: &str, password: &str) -> Result<(), Error>;
	/// Close vault
	fn close_vault(&self, name: &str) -> Result<(), Error>;
	/// List all vaults
	fn list_vaults(&self) -> Result<Vec<String>, Error>;
	/// List all currently opened vaults
	fn list_opened_vaults(&self) -> Result<Vec<String>, Error>;
	/// Change vault password
	fn change_vault_password(&self, name: &str, new_password: &str) -> Result<(), Error>;
	/// Cnage account' vault
	fn change_account_vault(&self, vault: SecretVaultRef, account: StoreAccountRef) -> Result<StoreAccountRef, Error>;
	/// Get vault metadata string.
	fn get_vault_meta(&self, name: &str) -> Result<String, Error>;
	/// Set vault metadata string.
	fn set_vault_meta(&self, name: &str, meta: &str) -> Result<(), Error>;
}

/// Secret Store API
pub trait SecretStore: SimpleSecretStore {

	/// Returns a raw opaque Secret that can be later used to sign a message.
	fn raw_secret(&self, account: &StoreAccountRef, password: &str) -> Result<OpaqueSecret, Error>;

	/// Signs a message with raw secret.
	fn sign_with_secret(&self, secret: &OpaqueSecret, message: &Message) -> Result<Signature, Error> {
		Ok(::ethkey::sign(&secret.0, message)?)
	}

	/// Imports presale wallet
	fn import_presale(&self, vault: SecretVaultRef, json: &[u8], password: &str) -> Result<StoreAccountRef, Error>;
	/// Imports existing JSON wallet
	fn import_wallet(&self, vault: SecretVaultRef, json: &[u8], password: &str) -> Result<StoreAccountRef, Error>;
	/// Copies account between stores and vaults.
	fn copy_account(&self, new_store: &SimpleSecretStore, new_vault: SecretVaultRef, account: &StoreAccountRef, password: &str, new_password: &str) -> Result<(), Error>;
	/// Checks if password matches given account.
	fn test_password(&self, account: &StoreAccountRef, password: &str) -> Result<bool, Error>;

	/// Returns a public key for given account.
	fn public(&self, account: &StoreAccountRef, password: &str) -> Result<Public, Error>;

	/// Returns uuid of an account.
	fn uuid(&self, account: &StoreAccountRef) -> Result<Uuid, Error>;
	/// Returns account's name.
	fn name(&self, account: &StoreAccountRef) -> Result<String, Error>;
	/// Returns account's metadata.
	fn meta(&self, account: &StoreAccountRef) -> Result<String, Error>;

	/// Modifies account metadata.
	fn set_name(&self, account: &StoreAccountRef, name: String) -> Result<(), Error>;
	/// Modifies account name.
	fn set_meta(&self, account: &StoreAccountRef, meta: String) -> Result<(), Error>;

	/// Returns local path of the store.
	fn local_path(&self) -> PathBuf;
	/// Lists all found geth accounts.
	fn list_geth_accounts(&self, testnet: bool) -> Vec<Address>;
	/// Imports geth accounts to the store/vault.
	fn import_geth_accounts(&self, vault: SecretVaultRef, desired: Vec<Address>, testnet: bool) -> Result<Vec<StoreAccountRef>, Error>;
}

impl StoreAccountRef {
	/// Create reference to root account with given address
	pub fn root(address: Address) -> Self {
		StoreAccountRef::new(SecretVaultRef::Root, address)
	}

	/// Create reference to vault account with given address
	pub fn vault(vault_name: &str, address: Address) -> Self {
		StoreAccountRef::new(SecretVaultRef::Vault(vault_name.to_owned()), address)
	}

	/// Create new account reference
	pub fn new(vault_ref: SecretVaultRef, address: Address) -> Self {
		StoreAccountRef {
			vault: vault_ref,
			address: address,
		}
	}
}

impl Hash for StoreAccountRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.address.hash(state);
	}
}

/// Node in hierarchical derivation.
pub struct IndexDerivation {
	/// Node is soft (allows proof of parent from parent node).
	pub soft: bool,
	/// Index sequence of the node.
	pub index: u32,
}

/// Derivation scheme for keys
pub enum Derivation {
	/// Hierarchical derivation
	Hierarchical(Vec<IndexDerivation>),
	/// Hash derivation, soft.
	SoftHash(H256),
	/// Hash derivation, hard.
	HardHash(H256),
}
