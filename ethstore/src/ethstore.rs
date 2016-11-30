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
use std::mem;
use ethkey::KeyPair;
use crypto::KEY_ITERATIONS;
use random::Random;
use ethkey::{Signature, Address, Message, Secret, Public};
use dir::KeyDirectory;
use account::SafeAccount;
use {Error, SimpleSecretStore, SecretStore};
use json;
use json::UUID;
use parking_lot::RwLock;
use presale::PresaleWallet;
use import;

pub struct EthStore {
	store: EthMultiStore,
}

impl EthStore {
	pub fn open(directory: Box<KeyDirectory>) -> Result<Self, Error> {
		Self::open_with_iterations(directory, KEY_ITERATIONS as u32)
	}

	pub fn open_with_iterations(directory: Box<KeyDirectory>, iterations: u32) -> Result<Self, Error> {
		Ok(EthStore {
			store: try!(EthMultiStore::open_with_iterations(directory, iterations)),
		})
	}

	fn get(&self, address: &Address) -> Result<SafeAccount, Error> {
		let mut accounts = try!(self.store.get(address)).into_iter();
		accounts.next().ok_or(Error::InvalidAccount)
	}
}

impl SimpleSecretStore for EthStore {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		self.store.insert_account(secret, password)
	}

	fn accounts(&self) -> Result<Vec<Address>, Error> {
		self.store.accounts()
	}

	fn change_password(&self, address: &Address, old_password: &str, new_password: &str) -> Result<(), Error> {
		self.store.change_password(address, old_password, new_password)
	}

	fn remove_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		self.store.remove_account(address, password)
	}

	fn sign(&self, address: &Address, password: &str, message: &Message) -> Result<Signature, Error> {
		let account = try!(self.get(address));
		account.sign(password, message)
	}

	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let account = try!(self.get(account));
		account.decrypt(password, shared_mac, message)
	}
}

impl SecretStore for EthStore {
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
		try!(self.store.save(safe_account));
		Ok(address)
	}

	fn test_password(&self, address: &Address, password: &str) -> Result<bool, Error> {
		let account = try!(self.get(address));
		Ok(account.check_password(password))
	}

	fn copy_account(&self, new_store: &SimpleSecretStore, address: &Address, password: &str, new_password: &str) -> Result<(), Error> {
		let account = try!(self.get(address));
		let secret = try!(account.crypto.secret(password));
		try!(new_store.insert_account(secret, new_password));
		Ok(())
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
		self.store.save(account)
	}

	fn set_meta(&self, address: &Address, meta: String) -> Result<(), Error> {
		let mut account = try!(self.get(address));
		account.meta = meta;

		// save to file
		self.store.save(account)
	}

	fn local_path(&self) -> String {
		self.store.dir.path().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| String::new())
	}

	fn list_geth_accounts(&self, testnet: bool) -> Vec<Address> {
		import::read_geth_accounts(testnet)
	}

	fn import_geth_accounts(&self, desired: Vec<Address>, testnet: bool) -> Result<Vec<Address>, Error> {
		import::import_geth_accounts(&*self.store.dir, desired.into_iter().collect(), testnet)
	}
}

/// Similar to `EthStore` but may store many accounts (with different passwords) for the same `Address`
pub struct EthMultiStore {
	dir: Box<KeyDirectory>,
	iterations: u32,
	cache: RwLock<BTreeMap<Address, Vec<SafeAccount>>>,
}

impl EthMultiStore {

	pub fn open(directory: Box<KeyDirectory>) -> Result<Self, Error> {
		Self::open_with_iterations(directory, KEY_ITERATIONS as u32)
	}

	pub fn open_with_iterations(directory: Box<KeyDirectory>, iterations: u32) -> Result<Self, Error> {
		let store = EthMultiStore {
			dir: directory,
			iterations: iterations,
			cache: Default::default(),
		};
		try!(store.reload_accounts());
		Ok(store)
	}

	fn reload_accounts(&self) -> Result<(), Error> {
		let mut cache = self.cache.write();
		let accounts = try!(self.dir.load());

		let mut new_accounts = BTreeMap::new();
		for account in accounts {
			let mut entry = new_accounts.entry(account.address.clone()).or_insert_with(Vec::new);
			entry.push(account);
		}
		mem::replace(&mut *cache, new_accounts);
		Ok(())
	}

	fn get(&self, address: &Address) -> Result<Vec<SafeAccount>, Error> {
		{
			let cache = self.cache.read();
			if let Some(accounts) = cache.get(address) {
				if !accounts.is_empty() {
					return Ok(accounts.clone())
				}
			}
		}

		try!(self.reload_accounts());
		let cache = self.cache.read();
		let accounts = try!(cache.get(address).cloned().ok_or(Error::InvalidAccount));
		if accounts.is_empty() {
			Err(Error::InvalidAccount)
		} else {
			Ok(accounts)
		}
	}

	fn save(&self, account: SafeAccount) -> Result<(), Error> {
		//save to file
		let account = try!(self.dir.insert(account));

		// update cache
		let mut cache = self.cache.write();
		let mut accounts = cache.entry(account.address.clone()).or_insert_with(Vec::new);
		accounts.push(account);
		Ok(())
	}

}

impl SimpleSecretStore for EthMultiStore {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		let keypair = try!(KeyPair::from_secret(secret).map_err(|_| Error::CreationFailed));
		let id: [u8; 16] = Random::random();
		let account = SafeAccount::create(&keypair, id, password, self.iterations, "".to_owned(), "{}".to_owned());
		let address = account.address.clone();
		try!(self.save(account));
		Ok(address)
	}

	fn accounts(&self) -> Result<Vec<Address>, Error> {
		try!(self.reload_accounts());
		Ok(self.cache.read().keys().cloned().collect())
	}

	fn remove_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		let accounts = try!(self.get(address));

		for account in accounts {
			// Skip if password is invalid
			if !account.check_password(password) {
				continue;
			}

			// Remove from dir
			try!(self.dir.remove(&account));

			// Remove from cache
			let mut cache = self.cache.write();
			let is_empty = {
				let mut accounts = cache.get_mut(address).expect("Entry exists, because it was returned by `get`; qed");
				if let Some(position) = accounts.iter().position(|acc| acc == &account) {
					accounts.remove(position);
				}
				accounts.is_empty()
			};

			if is_empty {
				cache.remove(address);
			}

			return Ok(());
		}
		Err(Error::InvalidPassword)
	}

	fn change_password(&self, address: &Address, old_password: &str, new_password: &str) -> Result<(), Error> {
		let accounts = try!(self.get(address));
		for account in accounts {
			// First remove
			try!(self.remove_account(&address, old_password));
			// Then insert back with new password
			let new_account = try!(account.change_password(old_password, new_password, self.iterations));
			try!(self.save(new_account));
		}
		Ok(())
	}

	fn sign(&self, address: &Address, password: &str, message: &Message) -> Result<Signature, Error> {
		let accounts = try!(self.get(address));
		for account in accounts {
			if account.check_password(password) {
				return account.sign(password, message);
			}
		}

		Err(Error::InvalidPassword)
	}

	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let accounts = try!(self.get(account));
		for account in accounts {
			if account.check_password(password) {
				return account.decrypt(password, shared_mac, message);
			}
		}
		Err(Error::InvalidPassword)
	}
}

#[cfg(test)]
mod tests {
	fn should_have_some_tests() {
		assert_eq!(true, false)
	}
}
