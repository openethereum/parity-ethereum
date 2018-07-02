// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

extern crate dir;
extern crate docopt;
extern crate ethstore;
extern crate num_cpus;
extern crate panic_hook;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use std::collections::VecDeque;
use std::io::Read;
use std::{env, process, fs, fmt};

use docopt::Docopt;
use ethstore::accounts_dir::{KeyDirectory, RootDiskDirectory};
use ethstore::ethkey::{Address, Password};
use ethstore::{EthStore, SimpleSecretStore, SecretStore, import_accounts, PresaleWallet, SecretVaultRef, StoreAccountRef};

mod crack;

pub const USAGE: &'static str = r#"
Ethereum key management.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    ethstore insert <secret> <password> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore change-pwd <address> <old-pwd> <new-pwd> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore list [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore import [--src DIR] [--dir DIR]
    ethstore import-wallet <path> <password> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore find-wallet-pass <path> <password>
    ethstore remove <address> <password> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore sign <address> <password> <message> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore public <address> <password> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore list-vaults [--dir DIR]
    ethstore create-vault <vault> <password> [--dir DIR]
    ethstore change-vault-pwd <vault> <old-pwd> <new-pwd> [--dir DIR]
    ethstore move-to-vault <address> <vault> <password> [--dir DIR] [--vault VAULT] [--vault-pwd VAULTPWD]
    ethstore move-from-vault <address> <vault> <password> [--dir DIR]
    ethstore [-h | --help]

Options:
    -h, --help               Display this message and exit.
    --dir DIR                Specify the secret store directory. It may be either
                             parity, parity-(chain), geth, geth-test
                             or a path [default: parity].
    --vault VAULT            Specify vault to use in this operation.
    --vault-pwd VAULTPWD     Specify vault password to use in this operation. Please note
                             that this option is required when vault option is set.
                             Otherwise it is ignored.
    --src DIR                Specify import source. It may be either
                             parity, parity-(chain), get, geth-test
                             or a path [default: geth].

Commands:
    insert             Save account with password.
    change-pwd         Change password.
    list               List accounts.
    import             Import accounts from src.
    import-wallet      Import presale wallet.
    find-wallet-pass   Tries to open a wallet with list of passwords given.
    remove             Remove account.
    sign               Sign message.
    public             Displays public key for an address.
    list-vaults        List vaults.
    create-vault       Create new vault.
    change-vault-pwd   Change vault password.
    move-to-vault      Move account to vault from another vault/root directory.
    move-from-vault    Move account to root directory from given vault.
"#;

#[derive(Debug, Deserialize)]
struct Args {
	cmd_insert: bool,
	cmd_change_pwd: bool,
	cmd_list: bool,
	cmd_import: bool,
	cmd_import_wallet: bool,
	cmd_find_wallet_pass: bool,
	cmd_remove: bool,
	cmd_sign: bool,
	cmd_public: bool,
	cmd_list_vaults: bool,
	cmd_create_vault: bool,
	cmd_change_vault_pwd: bool,
	cmd_move_to_vault: bool,
	cmd_move_from_vault: bool,
	arg_secret: String,
	arg_password: String,
	arg_old_pwd: String,
	arg_new_pwd: String,
	arg_address: String,
	arg_message: String,
	arg_path: String,
	arg_vault: String,
	flag_src: String,
	flag_dir: String,
	flag_vault: String,
	flag_vault_pwd: String,
}

enum Error {
	Ethstore(ethstore::Error),
	Docopt(docopt::Error),
}

impl From<ethstore::Error> for Error {
	fn from(err: ethstore::Error) -> Self {
		Error::Ethstore(err)
	}
}

impl From<docopt::Error> for Error {
	fn from(err: docopt::Error) -> Self {
		Error::Docopt(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Ethstore(ref err) => fmt::Display::fmt(err, f),
			Error::Docopt(ref err) => fmt::Display::fmt(err, f),
		}
	}
}

fn main() {
	panic_hook::set_abort();

	match execute(env::args()) {
		Ok(result) => println!("{}", result),
		Err(Error::Docopt(ref e)) => e.exit(),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		}
	}
}

fn key_dir(location: &str) -> Result<Box<KeyDirectory>, Error> {
	let dir: Box<KeyDirectory> = match location {
		"geth" => Box::new(RootDiskDirectory::create(dir::geth(false))?),
		"geth-test" => Box::new(RootDiskDirectory::create(dir::geth(true))?),
		path if path.starts_with("parity") => {
			let chain = path.split('-').nth(1).unwrap_or("ethereum");
			let path = dir::parity(chain);
			Box::new(RootDiskDirectory::create(path)?)
		},
		path => Box::new(RootDiskDirectory::create(path)?),
	};

	Ok(dir)
}

fn open_args_vault(store: &EthStore, args: &Args) -> Result<SecretVaultRef, Error> {
	if args.flag_vault.is_empty() {
		return Ok(SecretVaultRef::Root);
	}

	let vault_pwd = load_password(&args.flag_vault_pwd)?;
	store.open_vault(&args.flag_vault, &vault_pwd)?;
	Ok(SecretVaultRef::Vault(args.flag_vault.clone()))
}

fn open_args_vault_account(store: &EthStore, address: Address, args: &Args) -> Result<StoreAccountRef, Error> {
	match open_args_vault(store, args)? {
		SecretVaultRef::Root => Ok(StoreAccountRef::root(address)),
		SecretVaultRef::Vault(name) => Ok(StoreAccountRef::vault(&name, address)),
	}
}

fn format_accounts(accounts: &[Address]) -> String {
	accounts.iter()
		.enumerate()
		.map(|(i, a)| format!("{:2}: 0x{:x}", i, a))
		.collect::<Vec<String>>()
		.join("\n")
}

fn format_vaults(vaults: &[String]) -> String {
	vaults.join("\n")
}

fn load_password(path: &str) -> Result<Password, Error> {
	let mut file = fs::File::open(path).map_err(|e| ethstore::Error::Custom(format!("Error opening password file {}: {}", path, e)))?;
	let mut password = String::new();
	file.read_to_string(&mut password).map_err(|e| ethstore::Error::Custom(format!("Error reading password file {}: {}", path, e)))?;
	// drop EOF
	let _ = password.pop();
	Ok(password.into())
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).deserialize())?;

	let store = EthStore::open(key_dir(&args.flag_dir)?)?;

	return if args.cmd_insert {
		let secret = args.arg_secret.parse().map_err(|_| ethstore::Error::InvalidSecret)?;
		let password = load_password(&args.arg_password)?;
		let vault_ref = open_args_vault(&store, &args)?;
		let account_ref = store.insert_account(vault_ref, secret, &password)?;
		Ok(format!("0x{:x}", account_ref.address))
	} else if args.cmd_change_pwd {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let old_pwd = load_password(&args.arg_old_pwd)?;
		let new_pwd = load_password(&args.arg_new_pwd)?;
		let account_ref = open_args_vault_account(&store, address, &args)?;
		let ok = store.change_password(&account_ref, &old_pwd, &new_pwd).is_ok();
		Ok(format!("{}", ok))
	} else if args.cmd_list {
		let vault_ref = open_args_vault(&store, &args)?;
		let accounts = store.accounts()?;
		let accounts: Vec<_> = accounts
			.into_iter()
			.filter(|a| &a.vault == &vault_ref)
			.map(|a| a.address)
			.collect();
		Ok(format_accounts(&accounts))
	} else if args.cmd_import {
		let src = key_dir(&args.flag_src)?;
		let dst = key_dir(&args.flag_dir)?;
		let accounts = import_accounts(&*src, &*dst)?;
		Ok(format_accounts(&accounts))
	} else if args.cmd_import_wallet {
		let wallet = PresaleWallet::open(&args.arg_path)?;
		let password = load_password(&args.arg_password)?;
		let kp = wallet.decrypt(&password)?;
		let vault_ref = open_args_vault(&store, &args)?;
		let account_ref = store.insert_account(vault_ref, kp.secret().clone(), &password)?;
		Ok(format!("0x{:x}", account_ref.address))
	} else if args.cmd_find_wallet_pass {
		let passwords = load_password(&args.arg_password)?;
		let passwords = passwords.as_str().lines().map(|line| str::to_owned(line).into()).collect::<VecDeque<_>>();
		crack::run(passwords, &args.arg_path)?;
		Ok(format!("Password not found."))
	} else if args.cmd_remove {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		let account_ref = open_args_vault_account(&store, address, &args)?;
		let ok = store.remove_account(&account_ref, &password).is_ok();
		Ok(format!("{}", ok))
	} else if args.cmd_sign {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let message = args.arg_message.parse().map_err(|_| ethstore::Error::InvalidMessage)?;
		let password = load_password(&args.arg_password)?;
		let account_ref = open_args_vault_account(&store, address, &args)?;
		let signature = store.sign(&account_ref, &password, &message)?;
		Ok(format!("0x{}", signature))
	} else if args.cmd_public {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		let account_ref = open_args_vault_account(&store, address, &args)?;
		let public = store.public(&account_ref, &password)?;
		Ok(format!("0x{:x}", public))
	} else if args.cmd_list_vaults {
		let vaults = store.list_vaults()?;
		Ok(format_vaults(&vaults))
	} else if args.cmd_create_vault {
		let password = load_password(&args.arg_password)?;
		store.create_vault(&args.arg_vault, &password)?;
		Ok("OK".to_owned())
	} else if args.cmd_change_vault_pwd {
		let old_pwd = load_password(&args.arg_old_pwd)?;
		let new_pwd = load_password(&args.arg_new_pwd)?;
		store.open_vault(&args.arg_vault, &old_pwd)?;
		store.change_vault_password(&args.arg_vault, &new_pwd)?;
		Ok("OK".to_owned())
	} else if args.cmd_move_to_vault {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		let account_ref = open_args_vault_account(&store, address, &args)?;
		store.open_vault(&args.arg_vault, &password)?;
		store.change_account_vault(SecretVaultRef::Vault(args.arg_vault), account_ref)?;
		Ok("OK".to_owned())
	} else if args.cmd_move_from_vault {
		let address = args.arg_address.parse().map_err(|_| ethstore::Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		store.open_vault(&args.arg_vault, &password)?;
		store.change_account_vault(SecretVaultRef::Root, StoreAccountRef::vault(&args.arg_vault, address))?;
		Ok("OK".to_owned())
	} else {
		Ok(format!("{}", USAGE))
	}
}
