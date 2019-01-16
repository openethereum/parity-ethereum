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

use ethstore::{PresaleWallet, EthStore};
use ethstore::accounts_dir::RootDiskDirectory;
use helpers::{password_prompt, password_from_file};
use params::SpecType;

#[derive(Debug, PartialEq)]
pub struct ImportWallet {
	pub iterations: u32,
	pub path: String,
	pub spec: SpecType,
	pub wallet_path: String,
	pub password_file: Option<String>,
}

pub fn execute(cmd: ImportWallet) -> Result<String, String> {
	let password = match cmd.password_file {
		Some(file) => password_from_file(file)?,
		None => password_prompt()?,
	};

	let dir = Box::new(RootDiskDirectory::create(cmd.path).unwrap());
	let secret_store = Box::new(EthStore::open_with_iterations(dir, cmd.iterations).unwrap());
	let wallet = PresaleWallet::open(cmd.wallet_path).map_err(|_| "Unable to open presale wallet.")?;
	let kp = wallet.decrypt(&password).map_err(|_| "Invalid password.")?;
	let address = kp.address();
	import_account(kp);
	Ok(format!("{:?}", address))
}

#[cfg(feature = "accounts")]
pub fn import_account(kp: ethkey::KeyPair) {
	use accounts::{AccountProvider, AccountProviderSettings};

	let acc_provider = AccountProvider::new(secret_store, AccountProviderSettings::default());
	acc_provider.insert_account(kp.secret().clone(), &password).unwrap();
}

#[cfg(not(feature = "accounts"))]
pub fn import_account(_kp: ethkey::KeyPair) {}
