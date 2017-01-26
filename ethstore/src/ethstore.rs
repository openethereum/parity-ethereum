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

use std::collections::BTreeMap;
use std::mem;
use parking_lot::RwLock;

use crypto::KEY_ITERATIONS;
use random::Random;
use ethkey::{Signature, Address, Message, Secret, Public, KeyPair};
use dir::KeyDirectory;
use account::SafeAccount;
use presale::PresaleWallet;
use json::{self, Uuid};
use {import, Error, SimpleSecretStore, SecretStore};

pub struct EthStore {
	store: EthMultiStore,
}

impl EthStore {
	pub fn open(directory: Box<KeyDirectory>) -> Result<Self, Error> {
		Self::open_with_iterations(directory, KEY_ITERATIONS as u32)
	}

	pub fn open_with_iterations(directory: Box<KeyDirectory>, iterations: u32) -> Result<Self, Error> {
		Ok(EthStore {
			store: EthMultiStore::open_with_iterations(directory, iterations)?,
		})
	}

	fn get(&self, address: &Address) -> Result<SafeAccount, Error> {
		let mut accounts = self.store.get(address)?.into_iter();
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
		let account = self.get(address)?;
		account.sign(password, message)
	}

	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let account = self.get(account)?;
		account.decrypt(password, shared_mac, message)
	}
}

impl SecretStore for EthStore {
	fn import_presale(&self, json: &[u8], password: &str) -> Result<Address, Error> {
		let json_wallet = json::PresaleWallet::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned()))?;
		let wallet = PresaleWallet::from(json_wallet);
		let keypair = wallet.decrypt(password).map_err(|_| Error::InvalidPassword)?;
		self.insert_account(keypair.secret().clone(), password)
	}

	fn import_wallet(&self, json: &[u8], password: &str) -> Result<Address, Error> {
		let json_keyfile = json::KeyFile::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned()))?;
		let mut safe_account = SafeAccount::from_file(json_keyfile, None);
		let secret = safe_account.crypto.secret(password).map_err(|_| Error::InvalidPassword)?;
		safe_account.address = KeyPair::from_secret(secret)?.address();
		let address = safe_account.address.clone();
		self.store.import(safe_account)?;
		Ok(address)
	}

	fn test_password(&self, address: &Address, password: &str) -> Result<bool, Error> {
		let account = self.get(address)?;
		Ok(account.check_password(password))
	}

	fn copy_account(&self, new_store: &SimpleSecretStore, address: &Address, password: &str, new_password: &str) -> Result<(), Error> {
		let account = self.get(address)?;
		let secret = account.crypto.secret(password)?;
		new_store.insert_account(secret, new_password)?;
		Ok(())
	}

	fn public(&self, account: &Address, password: &str) -> Result<Public, Error> {
		let account = self.get(account)?;
		account.public(password)
	}

	fn uuid(&self, address: &Address) -> Result<Uuid, Error> {
		let account = self.get(address)?;
		Ok(account.id.into())
	}

	fn name(&self, address: &Address) -> Result<String, Error> {
		let account = self.get(address)?;
		Ok(account.name.clone())
	}

	fn meta(&self, address: &Address) -> Result<String, Error> {
		let account = self.get(address)?;
		Ok(account.meta.clone())
	}

	fn set_name(&self, address: &Address, name: String) -> Result<(), Error> {
		let old = self.get(address)?;
		let mut account = old.clone();
		account.name = name;

		// save to file
		self.store.update(old, account)
	}

	fn set_meta(&self, address: &Address, meta: String) -> Result<(), Error> {
		let old = self.get(address)?;
		let mut account = old.clone();
		account.meta = meta;

		// save to file
		self.store.update(old, account)
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
		store.reload_accounts()?;
		Ok(store)
	}

	fn reload_accounts(&self) -> Result<(), Error> {
		let mut cache = self.cache.write();
		let accounts = self.dir.load()?;

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

		self.reload_accounts()?;
		let cache = self.cache.read();
		let accounts = cache.get(address).cloned().ok_or(Error::InvalidAccount)?;
		if accounts.is_empty() {
			Err(Error::InvalidAccount)
		} else {
			Ok(accounts)
		}
	}

	fn import(&self, account: SafeAccount) -> Result<(), Error> {
		// save to file
		let account = self.dir.insert(account)?;

		// update cache
		let mut cache = self.cache.write();
		let mut accounts = cache.entry(account.address.clone()).or_insert_with(Vec::new);
		accounts.push(account);
		Ok(())
	}

	fn update(&self, old: SafeAccount, new: SafeAccount) -> Result<(), Error> {
		// save to file
		let account = self.dir.update(new)?;

		// update cache
		let mut cache = self.cache.write();
		let mut accounts = cache.entry(account.address.clone()).or_insert_with(Vec::new);
		// Remove old account
		accounts.retain(|acc| acc != &old);
		// And push updated to the end
		accounts.push(account);
		Ok(())

	}

}

impl SimpleSecretStore for EthMultiStore {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		let keypair = KeyPair::from_secret(secret).map_err(|_| Error::CreationFailed)?;
		let id: [u8; 16] = Random::random();
		let account = SafeAccount::create(&keypair, id, password, self.iterations, "".to_owned(), "{}".to_owned());
		let address = account.address.clone();
		self.import(account)?;
		Ok(address)
	}

	fn accounts(&self) -> Result<Vec<Address>, Error> {
		self.reload_accounts()?;
		Ok(self.cache.read().keys().cloned().collect())
	}

	fn remove_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		let accounts = self.get(address)?;

		for account in accounts {
			// Skip if password is invalid
			if !account.check_password(password) {
				continue;
			}

			// Remove from dir
			self.dir.remove(&account)?;

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
		let accounts = self.get(address)?;
		for account in accounts {
			// Change password
			let new_account = account.change_password(old_password, new_password, self.iterations)?;
			self.update(account, new_account)?;
		}
		Ok(())
	}

	fn sign(&self, address: &Address, password: &str, message: &Message) -> Result<Signature, Error> {
		let accounts = self.get(address)?;
		for account in accounts {
			if account.check_password(password) {
				return account.sign(password, message);
			}
		}

		Err(Error::InvalidPassword)
	}

	fn decrypt(&self, account: &Address, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let accounts = self.get(account)?;
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

	use dir::MemoryDirectory;
	use ethkey::{Random, Generator, KeyPair};
	use secret_store::{SimpleSecretStore, SecretStore};
	use super::{EthStore, EthMultiStore};

	fn keypair() -> KeyPair {
		Random.generate().unwrap()
	}

	fn store() -> EthStore {
		EthStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory always load successfuly; qed")
	}

	fn multi_store() -> EthMultiStore {
		EthMultiStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory always load successfuly; qed")
	}

	#[test]
	fn should_insert_account_successfully() {
		// given
		let store = store();
		let keypair = keypair();

		// when
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();

		// then
		assert_eq!(address, keypair.address());
		assert!(store.get(&address).is_ok(), "Should contain account.");
		assert_eq!(store.accounts().unwrap().len(), 1, "Should have one account.");
	}

	#[test]
	fn should_update_meta_and_name() {
		// given
		let store = store();
		let keypair = keypair();
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();
		assert_eq!(&store.meta(&address).unwrap(), "{}");
		assert_eq!(&store.name(&address).unwrap(), "");

		// when
		store.set_meta(&address, "meta".into()).unwrap();
		store.set_name(&address, "name".into()).unwrap();

		// then
		assert_eq!(&store.meta(&address).unwrap(), "meta");
		assert_eq!(&store.name(&address).unwrap(), "name");
		assert_eq!(store.accounts().unwrap().len(), 1);
	}

	#[test]
	fn should_remove_account() {
		// given
		let store = store();
		let keypair = keypair();
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();

		// when
		store.remove_account(&address, "test").unwrap();

		// then
		assert_eq!(store.accounts().unwrap().len(), 0, "Should remove account.");
	}

	#[test]
	fn should_return_true_if_password_is_correct() {
		// given
		let store = store();
		let keypair = keypair();
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();

		// when
		let res1 = store.test_password(&address, "x").unwrap();
		let res2 = store.test_password(&address, "test").unwrap();

		assert!(!res1, "First password should be invalid.");
		assert!(res2, "Second password should be correct.");
	}

	#[test]
	fn multistore_should_be_able_to_have_the_same_account_twice() {
		// given
		let store = multi_store();
		let keypair = keypair();
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();
		let address2 = store.insert_account(keypair.secret().clone(), "xyz").unwrap();
		assert_eq!(address, address2);

		// when
		assert!(store.remove_account(&address, "test").is_ok(), "First password should work.");
		assert_eq!(store.accounts().unwrap().len(), 1);

		assert!(store.remove_account(&address, "xyz").is_ok(), "Second password should work too.");
		assert_eq!(store.accounts().unwrap().len(), 0);
	}

	#[test]
	fn should_copy_account() {
		// given
		let store = store();
		let multi_store = multi_store();
		let keypair = keypair();
		let address = store.insert_account(keypair.secret().clone(), "test").unwrap();
		assert_eq!(multi_store.accounts().unwrap().len(), 0);

		// when
		store.copy_account(&multi_store, &address, "test", "xyz").unwrap();

		// then
		assert!(store.test_password(&address, "test").unwrap(), "First password should work for store.");
		assert!(multi_store.sign(&address, "xyz", &Default::default()).is_ok(), "Second password should work for second store.");
		assert_eq!(multi_store.accounts().unwrap().len(), 1);
	}

}
