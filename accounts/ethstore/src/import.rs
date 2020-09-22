// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{collections::HashSet, fs, path::Path};

use accounts_dir::{DiskKeyFileManager, KeyDirectory, KeyFileManager};
use ethkey::Address;
use Error;

/// Import an account from a file.
pub fn import_account(path: &Path, dst: &dyn KeyDirectory) -> Result<Address, Error> {
    let key_manager = DiskKeyFileManager::default();
    let existing_accounts = dst
        .load()?
        .into_iter()
        .map(|a| a.address)
        .collect::<HashSet<_>>();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|f| f.to_owned());
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
pub fn import_accounts(
    src: &dyn KeyDirectory,
    dst: &dyn KeyDirectory,
) -> Result<Vec<Address>, Error> {
    let accounts = src.load()?;
    let existing_accounts = dst
        .load()?
        .into_iter()
        .map(|a| a.address)
        .collect::<HashSet<_>>();

    accounts
        .into_iter()
        .filter(|a| !existing_accounts.contains(&a.address))
        .map(|a| {
            let address = a.address.clone();
            dst.insert(a)?;
            Ok(address)
        })
        .collect()
}
