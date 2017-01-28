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

use std::collections::HashSet;
use ethkey::Address;
use dir::{GethDirectory, KeyDirectory, DirectoryType};
use Error;

pub fn import_accounts(src: &KeyDirectory, dst: &KeyDirectory) -> Result<Vec<Address>, Error> {
	let accounts = src.load()?;
	let existing_accounts = dst.load()?.into_iter().map(|a| a.address).collect::<HashSet<_>>();

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
	let t = if testnet {
		DirectoryType::Testnet
	} else {
		DirectoryType::Main
	};

	GethDirectory::open(t)
		.load()
		.map(|d| d.into_iter().map(|a| a.address).collect())
		.unwrap_or_else(|_| Vec::new())
}

/// Import specific `desired` accounts from the Geth keystore into `dst`. 
pub fn import_geth_accounts(dst: &KeyDirectory, desired: HashSet<Address>, testnet: bool) -> Result<Vec<Address>, Error> {
	let t = if testnet {
		DirectoryType::Testnet
	} else {
		DirectoryType::Main
	};

	let src = GethDirectory::open(t);
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
