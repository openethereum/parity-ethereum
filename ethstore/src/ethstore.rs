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

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::mem;
use ethkey::KeyPair;
use crypto::KEY_ITERATIONS;
use random::Random;
use ethkey::{Signature, Address, Message, Secret, Public};
use dir::KeyDirectory;
use account::SafeAccount;
use {Error, SecretStore};
use json;
use json::UUID;
use presale::PresaleWallet;
use import;

pub struct EthStore {
	dir: Box<KeyDirectory>,
	iterations: u32,
	cache: RwLock<BTreeMap<Address, SafeAccount>>,
}

impl EthStore {
	pub fn open(directory: Box<KeyDirectory>) -> Result<Self, Error> {
		Self::open_with_iterations(directory, KEY_ITERATIONS as u32)
	}

	pub fn open_with_iterations(directory: Box<KeyDirectory>, iterations: u32) -> Result<Self, Error> {
		let accounts = try!(directory.load());
		let cache = accounts.into_iter().map(|account| (account.address.clone(), account)).collect();
		let store = EthStore {
			dir: directory,
			iterations: iterations,
			cache: RwLock::new(cache),
		};
		Ok(store)
	}

	fn save(&self, account: SafeAccount) -> Result<(), Error> {
		// save to file
		let account = try!(self.dir.insert(account.clone()));

		// update cache
		let mut cache = self.cache.write().unwrap();
		cache.insert(account.address.clone(), account);
		Ok(())
	}

	fn reload_accounts(&self) -> Result<(), Error> {
		let mut cache = self.cache.write().unwrap();
		let accounts = try!(self.dir.load());
		let new_accounts: BTreeMap<_, _> = accounts.into_iter().map(|account| (account.address.clone(), account)).collect();
		mem::replace(&mut *cache, new_accounts);
		Ok(())
	}

	fn get(&self, address: &Address) -> Result<SafeAccount, Error> {
		{
			let cache = self.cache.read().unwrap();
			if let Some(account) = cache.get(address) {
				return Ok(account.clone())
			}
		}
		try!(self.reload_accounts());
		let cache = self.cache.read().unwrap();
		cache.get(address).cloned().ok_or(Error::InvalidAccount)
	}
}

impl SecretStore for EthStore {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		let keypair = try!(KeyPair::from_secret(secret).map_err(|_| Error::CreationFailed));
		let id: [u8; 16] = Random::random();
		let account = SafeAccount::create(&keypair, id, password, self.iterations, UUID::from(id).into(), "{}".to_owned());
		let address = account.address.clone();
		try!(self.save(account));
		Ok(address)
	}

	fn import_presale(&self, json: &[u8], password: &str) -> Result<Address, Error> {
		let json_wallet = try!(json::PresaleWallet::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned())));
		let wallet = PresaleWallet::from(json_wallet);
		let keypair = try!(wallet.decrypt(password).map_err(|_| Error::InvalidPassword));
		self.insert_account(keypair.secret().clone(), password)
	}

	fn import_wallet(&self, json: &[u8], password: &str) -> Result<Address, Error> {
		let json_keyfile = try!(json::KeyFile::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned())));
		let mut safe_account = SafeAccount::from_file(json_keyfile, None);
		let secret = try!(safe_account.crypto.secret(password).map_err(|_| Error::InvalidPassword));
		safe_account.address = try!(KeyPair::from_secret(secret)).address();
		let address = safe_account.address.clone();
		try!(self.save(safe_account));
		Ok(address)
	}

	fn accounts(&self) -> Result<Vec<Address>, Error> {
		try!(self.reload_accounts());
		Ok(self.cache.read().unwrap().keys().cloned().collect())
	}

	fn change_password(&self, address: &Address, old_password: &str, new_password: &str) -> Result<(), Error> {
		// change password
		let account = try!(self.get(address));
		let account = try!(account.change_password(old_password, new_password, self.iterations));

		// save to file
		self.save(account)
	}

	fn remove_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		let can_remove = {
			let account = try!(self.get(address));
			account.check_password(password)
		};

		if can_remove {
			try!(self.dir.remove(address));
			let mut cache = self.cache.write().unwrap();
			cache.remove(address);
			Ok(())
		} else {
			Err(Error::InvalidPassword)
		}
	}

	fn sign(&self, address: &Address, password: &str, message: &Message) -> Result<Signature, Error> {
		let account = try!(self.get(address));
		account.sign(password, message)
	}

	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let account = try!(self.get(account));
		account.decrypt(password, shared_mac, message)
	}

	fn public(&self, account: &Address, password: &str) -> Result<Public, Error> {
		let account = try!(self.get(account));
		account.public(password)
	}

	fn uuid(&self, address: &Address) -> Result<UUID, Error> {
		let account = try!(self.get(address));
		Ok(account.id.into())
	}

	fn name(&self, address: &Address) -> Result<String, Error> {
		let account = try!(self.get(address));
		Ok(account.name.clone())
	}

	fn meta(&self, address: &Address) -> Result<String, Error> {
		let account = try!(self.get(address));
		Ok(account.meta.clone())
	}

	fn set_name(&self, address: &Address, name: String) -> Result<(), Error> {
		let mut account = try!(self.get(address));
		account.name = name;

		// save to file
		self.save(account)
	}

	fn set_meta(&self, address: &Address, meta: String) -> Result<(), Error> {
		let mut account = try!(self.get(address));
		account.meta = meta;

		// save to file
		self.save(account)
	}

	fn local_path(&self) -> String {
		self.dir.path().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| String::new())
	}

	fn list_geth_accounts(&self, testnet: bool) -> Vec<Address> {
		import::read_geth_accounts(testnet)
	}

	fn import_geth_accounts(&self, desired: Vec<Address>, testnet: bool) -> Result<Vec<Address>, Error> {
		import::import_geth_accounts(&*self.dir, desired.into_iter().collect(), testnet)
	}
}
