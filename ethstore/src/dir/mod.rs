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

use std::path::{PathBuf};
use {SafeAccount, Error};

mod disk;
mod geth;
mod memory;
mod parity;

pub enum DirectoryType {
	Testnet,
	Main,
}

pub trait KeyDirectory: Send + Sync {
	fn load(&self) -> Result<Vec<SafeAccount>, Error>;
	fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	fn update(&self, account: SafeAccount) -> Result<SafeAccount, Error>;
	fn remove(&self, account: &SafeAccount) -> Result<(), Error>;
	fn path(&self) -> Option<&PathBuf> { None }
}

pub use self::disk::DiskDirectory;
pub use self::geth::GethDirectory;
pub use self::memory::MemoryDirectory;
pub use self::parity::ParityDirectory;
