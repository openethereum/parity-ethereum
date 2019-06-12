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

//! Accounts Directory

use ethkey::Password;
use std::path::{PathBuf};
use {SafeAccount, Error};

mod disk;
mod memory;
mod vault;

/// `VaultKeyDirectory::set_key` error
#[derive(Debug)]
pub enum SetKeyError {
	/// Error is fatal and directory is probably in inconsistent state
	Fatal(Error),
	/// Error is non fatal, directory is reverted to pre-operation state
	NonFatalOld(Error),
	/// Error is non fatal, directory is consistent with new key
	NonFatalNew(Error),
}

/// Vault key
#[derive(Clone, PartialEq, Eq)]
pub struct VaultKey {
	/// Vault password
	pub password: Password,
	/// Number of iterations to produce a derived key from password
	pub iterations: u32,
}

/// Keys directory
pub trait KeyDirectory: Send + Sync {
	/// Read keys from directory
	fn load(&self) -> Result<Vec<SafeAccount>, Error>;
	/// Insert new key to directory
	fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	/// Update key in the directory
	fn update(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	/// Remove key from directory
	fn remove(&self, account: &SafeAccount) -> Result<(), Error>;
	/// Get directory filesystem path, if available
	fn path(&self) -> Option<&PathBuf> { None }
	/// Return vault provider, if available
	fn as_vault_provider(&self) -> Option<&VaultKeyDirectoryProvider> { None }
	/// Unique representation of directory account collection
	fn unique_repr(&self) -> Result<u64, Error>;
}

/// Vaults provider
pub trait VaultKeyDirectoryProvider {
	/// Create new vault with given key
	fn create(&self, name: &str, key: VaultKey) -> Result<Box<VaultKeyDirectory>, Error>;
	/// Open existing vault with given key
	fn open(&self, name: &str, key: VaultKey) -> Result<Box<VaultKeyDirectory>, Error>;
	/// List all vaults
	fn list_vaults(&self) -> Result<Vec<String>, Error>;
	/// Get vault meta
	fn vault_meta(&self, name: &str) -> Result<String, Error>;
}

/// Vault directory
pub trait VaultKeyDirectory: KeyDirectory {
	/// Cast to `KeyDirectory`
	fn as_key_directory(&self) -> &KeyDirectory;
	/// Vault name
	fn name(&self) -> &str;
	/// Get vault key
	fn key(&self) -> VaultKey;
	/// Set new key for vault
	fn set_key(&self, key: VaultKey) -> Result<(), SetKeyError>;
	/// Get vault meta
	fn meta(&self) -> String;
	/// Set vault meta
	fn set_meta(&self, meta: &str) -> Result<(), Error>;
}

pub use self::disk::{RootDiskDirectory, DiskKeyFileManager, KeyFileManager};
pub use self::memory::MemoryDirectory;
pub use self::vault::VaultDiskDirectory;

impl VaultKey {
	/// Create new vault key
	pub fn new(password: &Password, iterations: u32) -> Self {
		VaultKey {
			password: password.clone(),
			iterations: iterations,
		}
	}
}
