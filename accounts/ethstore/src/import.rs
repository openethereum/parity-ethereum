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

use std::collections::HashSet;
use std::path::Path;
use std::fs;

use crypto::publickey::Address;
use accounts_dir::{KeyDirectory, RootDiskDirectory, DiskKeyFileManager, KeyFileManager};
use dir;
use Error;

/// Import an account from a file.
pub fn import_account(path: &Path, dst: &dyn KeyDirectory) -> Result<Address, Error> {
	let key_manager = DiskKeyFileManager::default();
	let existing_accounts = dst.load()?.into_iter().map(|a| a.address).collect::<HashSet<_>>();
	let filename = path.file_name().and_then(|n| n.to_str()).map(|f| f.to_owned());
	let account = fs::File::open(&path)
		.map_err(Into::into)
		.and_then(|file| key_manager.read(filename, file))?;

	let address = account.address.clone();
	if !existing_accounts.contains(&address) {
		dst.insert(account)?;
	}
	Ok(address)
}

/// Import all accounts from one directory to the other.
pub fn import_accounts(src: &dyn KeyDirectory, dst: &dyn KeyDirectory) -> Result<Vec<Address>, Error> {
	let accounts = src.load()?;
	let existing_accounts = dst.load()?.into_iter()
		.map(|a| a.address)
		.collect::<HashSet<_>>();

	accounts.into_iter()
		.filter(|a| !existing_accounts.contains(&a.address))
		.map(|a| {
			let address = a.address.clone();
			dst.insert(a)?;
			Ok(address)
		}).collect()
}

/// Provide a `HashSet` of all accounts available for import from the Geth keystore.
pub fn read_geth_accounts(testnet: bool) -> Vec<Address> {
	RootDiskDirectory::at(dir::geth(testnet))
		.load()
		.map(|d| d.into_iter().map(|a| a.address).collect())
		.unwrap_or_else(|_| Vec::new())
}

/// Import specific `desired` accounts from the Geth keystore into `dst`.
pub fn import_geth_accounts(dst: &dyn KeyDirectory, desired: HashSet<Address>, testnet: bool) -> Result<Vec<Address>, Error> {
	let src = RootDiskDirectory::at(dir::geth(testnet));
	let accounts = src.load()?;
	let existing_accounts = dst.load()?.into_iter().map(|a| a.address).collect::<HashSet<_>>();

	accounts.into_iter()
		.filter(|a| !existing_accounts.contains(&a.address))
		.filter(|a| desired.contains(&a.address))
		.map(|a| {
			let address = a.address.clone();
			dst.insert(a)?;
			Ok(address)
		}).collect()
}
