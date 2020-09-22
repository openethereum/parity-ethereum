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

use params::SpecType;
use std::num::NonZeroU32;

#[derive(Debug, PartialEq)]
pub enum AccountCmd {
    New(NewAccount),
    List(ListAccounts),
    Import(ImportAccounts),
}

#[derive(Debug, PartialEq)]
pub struct ListAccounts {
    pub path: String,
    pub spec: SpecType,
}

#[derive(Debug, PartialEq)]
pub struct NewAccount {
    pub iterations: NonZeroU32,
    pub path: String,
    pub spec: SpecType,
    pub password_file: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ImportAccounts {
    pub from: Vec<String>,
    pub to: String,
    pub spec: SpecType,
}

#[cfg(not(feature = "accounts"))]
pub fn execute(_cmd: AccountCmd) -> Result<String, String> {
    Err("Account management is deprecated. Please see #9997 for alternatives:\nhttps://github.com/openethereum/openethereum/issues/9997".into())
}

#[cfg(feature = "accounts")]
mod command {
    use super::*;
    use accounts::{AccountProvider, AccountProviderSettings};
    use ethstore::{accounts_dir::RootDiskDirectory, import_account, import_accounts, EthStore};
    use helpers::{password_from_file, password_prompt};
    use std::path::PathBuf;

    pub fn execute(cmd: AccountCmd) -> Result<String, String> {
        match cmd {
            AccountCmd::New(new_cmd) => new(new_cmd),
            AccountCmd::List(list_cmd) => list(list_cmd),
            AccountCmd::Import(import_cmd) => import(import_cmd),
        }
    }

    fn keys_dir(path: String, spec: SpecType) -> Result<RootDiskDirectory, String> {
        let spec = spec.spec(&::std::env::temp_dir())?;
        let mut path = PathBuf::from(&path);
        path.push(spec.data_dir);
        RootDiskDirectory::create(path).map_err(|e| format!("Could not open keys directory: {}", e))
    }

    fn secret_store(
        dir: Box<RootDiskDirectory>,
        iterations: Option<NonZeroU32>,
    ) -> Result<EthStore, String> {
        match iterations {
            Some(i) => EthStore::open_with_iterations(dir, i),
            _ => EthStore::open(dir),
        }
        .map_err(|e| format!("Could not open keys store: {}", e))
    }

    fn new(n: NewAccount) -> Result<String, String> {
        let password = match n.password_file {
            Some(file) => password_from_file(file)?,
            None => password_prompt()?,
        };

        let dir = Box::new(keys_dir(n.path, n.spec)?);
        let secret_store = Box::new(secret_store(dir, Some(n.iterations))?);
        let acc_provider = AccountProvider::new(secret_store, AccountProviderSettings::default());
        let new_account = acc_provider
            .new_account(&password)
            .map_err(|e| format!("Could not create new account: {}", e))?;
        Ok(format!("0x{:x}", new_account))
    }

    fn list(list_cmd: ListAccounts) -> Result<String, String> {
        let dir = Box::new(keys_dir(list_cmd.path, list_cmd.spec)?);
        let secret_store = Box::new(secret_store(dir, None)?);
        let acc_provider = AccountProvider::new(secret_store, AccountProviderSettings::default());
        let accounts = acc_provider.accounts().map_err(|e| format!("{}", e))?;
        let result = accounts
            .into_iter()
            .map(|a| format!("0x{:x}", a))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(result)
    }

    fn import(i: ImportAccounts) -> Result<String, String> {
        let to = keys_dir(i.to, i.spec)?;
        let mut imported = 0;

        for path in &i.from {
            let path = PathBuf::from(path);
            if path.is_dir() {
                let from = RootDiskDirectory::at(&path);
                imported += import_accounts(&from, &to)
                    .map_err(|e| format!("Importing accounts from {:?} failed: {}", path, e))?
                    .len();
            } else if path.is_file() {
                import_account(&path, &to)
                    .map_err(|e| format!("Importing account from {:?} failed: {}", path, e))?;
                imported += 1;
            }
        }

        Ok(format!("{} account(s) imported", imported))
    }
}

#[cfg(feature = "accounts")]
pub use self::command::execute;
