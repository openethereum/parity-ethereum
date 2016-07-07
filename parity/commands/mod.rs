// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

mod account;
mod presale;

pub use self::account::{AccountCmd, NewAccount, ImportAccounts};
pub use self::presale::ImportWallet;
use cli::print_version;
use configuration::Configuration;
use execute as main_execute;
use signer;

#[derive(Debug, PartialEq)]
pub enum Cmd {
	Run(Configuration),
	Version,
	Account(AccountCmd),
	ImportPresaleWallet(ImportWallet),
	Blockchain(BlockchainCmd),
	SignerToken(String),
}

#[derive(Debug, PartialEq)]
pub enum BlockchainCmd {
	Import,
	Export,
}

pub fn execute(command: Cmd) -> Result<String, String> {
	match command {
		Cmd::Run(configuration) => {
			main_execute(configuration);
			unimplemented!();
		},
		Cmd::Version => Ok(print_version()),
		Cmd::Account(account_cmd) => account::execute(account_cmd),
		Cmd::ImportPresaleWallet(presale_cmd) => presale::execute(presale_cmd),
		Cmd::Blockchain(_blockchain_cmd) => {
			unimplemented!();
		},
		Cmd::SignerToken(path) => {
			unimplemented!();
		},
	}
}
