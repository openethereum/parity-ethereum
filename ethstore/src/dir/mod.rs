// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::path::{PathBuf};
use {SafeAccount, Error};

mod disk;
mod geth;
mod memory;
mod parity;
mod vault;

pub enum DirectoryType {
	Testnet,
	Main,
}

#[derive(Debug)]
pub enum SetKeyError {
	Fatal(Error),
	NonFatalOld(Error),
	NonFatalNew(Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultKey {
	pub password: String,
	pub iterations: u32,
}

pub trait KeyDirectory: Send + Sync {
	fn load(&self) -> Result<Vec<SafeAccount>, Error>;
	fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	fn update(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	fn remove(&self, account: &SafeAccount) -> Result<(), Error>;
	fn path(&self) -> Option<&PathBuf> { None }
	fn as_vault_provider(&self) -> Option<&VaultKeyDirectoryProvider> { None }
}

pub trait VaultKeyDirectoryProvider {
	fn create(&self, name: &str, key: VaultKey) -> Result<Box<VaultKeyDirectory>, Error>;
	fn open(&self, name: &str, key: VaultKey) -> Result<Box<VaultKeyDirectory>, Error>;
}

pub trait VaultKeyDirectory: KeyDirectory {
	fn as_key_directory(&self) -> &KeyDirectory;
	fn name(&self) -> &str;
	fn set_key(&self, old_key: VaultKey, key: VaultKey) -> Result<(), SetKeyError>;
}

pub use self::disk::RootDiskDirectory;
pub use self::geth::GethDirectory;
pub use self::memory::MemoryDirectory;
pub use self::parity::ParityDirectory;
pub use self::vault::VaultDiskDirectory;

impl VaultKey {
	pub fn new(password: &str, iterations: u32) -> Self {
		VaultKey {
			password: password.to_owned(),
			iterations: iterations,
		}
	}
}

impl SetKeyError {
	pub fn fatal(err: Error) -> Self {
		SetKeyError::Fatal(err)
	}

	pub fn nonfatal_old(err: Error) -> Self {
		SetKeyError::NonFatalOld(err)
	}

	pub fn nonfatal_new(err: Error) -> Self {
		SetKeyError::NonFatalNew(err)
	}
}
