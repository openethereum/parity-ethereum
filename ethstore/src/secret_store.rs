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

use ethkey::{Address, Message, Signature, Secret, Public};
use Error;
use json::UUID;

pub trait SecretStore: Send + Sync {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error>;
	fn import_presale(&self, json: &[u8], password: &str) -> Result<Address, Error>;
	fn import_wallet(&self, json: &[u8], password: &str) -> Result<Address, Error>;
	fn change_password(&self, account: &Address, old_password: &str, new_password: &str) -> Result<(), Error>;
	fn remove_account(&self, account: &Address, password: &str) -> Result<(), Error>;

	fn sign(&self, account: &Address, password: &str, message: &Message) -> Result<Signature, Error>;
	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error>;
	fn public(&self, account: &Address, password: &str) -> Result<Public, Error>;

	fn accounts(&self) -> Result<Vec<Address>, Error>;
	fn uuid(&self, account: &Address) -> Result<UUID, Error>;
	fn name(&self, account: &Address) -> Result<String, Error>;
	fn meta(&self, account: &Address) -> Result<String, Error>;

	fn set_name(&self, address: &Address, name: String) -> Result<(), Error>;
	fn set_meta(&self, address: &Address, meta: String) -> Result<(), Error>;

	fn local_path(&self) -> String;
	fn list_geth_accounts(&self, testnet: bool) -> Vec<Address>;
	fn import_geth_accounts(&self, desired: Vec<Address>, testnet: bool) -> Result<Vec<Address>, Error>;
}

