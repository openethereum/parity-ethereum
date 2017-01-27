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

extern crate rustc_serialize;
extern crate docopt;
extern crate ethstore;

use std::{env, process, fs};
use std::io::Read;
use docopt::Docopt;
use ethstore::ethkey::Address;
use ethstore::dir::{KeyDirectory, ParityDirectory, DiskDirectory, GethDirectory, DirectoryType};
use ethstore::{EthStore, SecretStore, import_accounts, Error, PresaleWallet};

pub const USAGE: &'static str = r#"
Ethereum key management.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    ethstore insert <secret> <password> [--dir DIR]
    ethstore change-pwd <address> <old-pwd> <new-pwd> [--dir DIR]
    ethstore list [--dir DIR]
    ethstore import [--src DIR] [--dir DIR]
    ethstore import-wallet <path> <password> [--dir DIR]
    ethstore remove <address> <password> [--dir DIR]
    ethstore sign <address> <password> <message> [--dir DIR]
    ethstore public <address> <password>
    ethstore [-h | --help]

Options:
    -h, --help         Display this message and exit.
    --dir DIR          Specify the secret store directory. It may be either
                       parity, parity-test, geth, geth-test
                       or a path [default: parity].
    --src DIR          Specify import source. It may be either
                       parity, parity-test, get, geth-test
                       or a path [default: geth].

Commands:
    insert             Save account with password.
    change-pwd         Change password.
    list               List accounts.
    import             Import accounts from src.
    import-wallet      Import presale wallet.
    remove             Remove account.
    sign               Sign message.
    public             Displays public key for an address.
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
	cmd_insert: bool,
	cmd_change_pwd: bool,
	cmd_list: bool,
	cmd_import: bool,
	cmd_import_wallet: bool,
	cmd_remove: bool,
	cmd_sign: bool,
	cmd_public: bool,
	arg_secret: String,
	arg_password: String,
	arg_old_pwd: String,
	arg_new_pwd: String,
	arg_address: String,
	arg_message: String,
	arg_path: String,
	flag_src: String,
	flag_dir: String,
}

fn main() {
	match execute(env::args()) {
		Ok(result) => println!("{}", result),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		}
	}
}

fn key_dir(location: &str) -> Result<Box<KeyDirectory>, Error> {
	let dir: Box<KeyDirectory> = match location {
		"parity" => Box::new(ParityDirectory::create(DirectoryType::Main)?),
		"parity-test" => Box::new(ParityDirectory::create(DirectoryType::Testnet)?),
		"geth" => Box::new(GethDirectory::create(DirectoryType::Main)?),
		"geth-test" => Box::new(GethDirectory::create(DirectoryType::Testnet)?),
		path => Box::new(DiskDirectory::create(path)?),
	};

	Ok(dir)
}

fn format_accounts(accounts: &[Address]) -> String {
	accounts.iter()
		.enumerate()
		.map(|(i, a)| format!("{:2}: 0x{:?}", i, a))
		.collect::<Vec<String>>()
		.join("\n")
}

fn load_password(path: &str) -> Result<String, Error> {
	let mut file = fs::File::open(path)?;
	let mut password = String::new();
	file.read_to_string(&mut password)?;
	// drop EOF
	let _ = password.pop();
	Ok(password)
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).decode())
		.unwrap_or_else(|e| e.exit());

	let store = EthStore::open(key_dir(&args.flag_dir)?)?;

	return if args.cmd_insert {
		let secret = args.arg_secret.parse().map_err(|_| Error::InvalidSecret)?;
		let password = load_password(&args.arg_password)?;
		let address = store.insert_account(secret, &password)?;
		Ok(format!("0x{:?}", address))
	} else if args.cmd_change_pwd {
		let address = args.arg_address.parse().map_err(|_| Error::InvalidAccount)?;
		let old_pwd = load_password(&args.arg_old_pwd)?;
		let new_pwd = load_password(&args.arg_new_pwd)?;
		let ok = store.change_password(&address, &old_pwd, &new_pwd).is_ok();
		Ok(format!("{}", ok))
	} else if args.cmd_list {
		let accounts = store.accounts()?;
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
		let address = store.insert_account(kp.secret().clone(), &password)?;
		Ok(format!("0x{:?}", address))
	} else if args.cmd_remove {
		let address = args.arg_address.parse().map_err(|_| Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		let ok = store.remove_account(&address, &password).is_ok();
		Ok(format!("{}", ok))
	} else if args.cmd_sign {
		let address = args.arg_address.parse().map_err(|_| Error::InvalidAccount)?;
		let message = args.arg_message.parse().map_err(|_| Error::InvalidMessage)?;
		let password = load_password(&args.arg_password)?;
		let signature = store.sign(&address, &password, &message)?;
		Ok(format!("0x{:?}", signature))
	} else if args.cmd_public {
		let address = args.arg_address.parse().map_err(|_| Error::InvalidAccount)?;
		let password = load_password(&args.arg_password)?;
		let public = store.public(&address, &password)?;
		Ok(format!("0x{:?}", public))
	} else {
		Ok(format!("{}", USAGE))
	}
}

