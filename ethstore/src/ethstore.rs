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

use std::collections::{BTreeMap, HashMap};
use std::mem;
use parking_lot::{Mutex, RwLock};

use crypto::KEY_ITERATIONS;
use random::Random;
use ethkey::{Signature, Address, Message, Secret, Public, KeyPair};
use dir::{KeyDirectory, VaultKeyDirectory, VaultKey, SetKeyError};
use account::SafeAccount;
use presale::PresaleWallet;
use json::{self, Uuid};
use {import, Error, SimpleSecretStore, SecretStore, SecretVaultRef, StoreAccountRef};

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

	fn get(&self, account: &StoreAccountRef) -> Result<SafeAccount, Error> {
		let mut accounts = self.store.get(account)?.into_iter();
		accounts.next().ok_or(Error::InvalidAccount)
	}
}

impl SimpleSecretStore for EthStore {
	fn insert_account(&self, vault: SecretVaultRef, secret: Secret, password: &str) -> Result<StoreAccountRef, Error> {
		self.store.insert_account(vault, secret, password)
	}

	fn accounts(&self) -> Result<Vec<StoreAccountRef>, Error> {
		self.store.accounts()
	}

	fn change_password(&self, account: &StoreAccountRef, old_password: &str, new_password: &str) -> Result<(), Error> {
		self.store.change_password(account, old_password, new_password)
	}

	fn remove_account(&self, account: &StoreAccountRef, password: &str) -> Result<(), Error> {
		self.store.remove_account(account, password)
	}

	fn sign(&self, account: &StoreAccountRef, password: &str, message: &Message) -> Result<Signature, Error> {
		let account = self.get(account)?;
		account.sign(password, message)
	}

	fn decrypt(&self, account: &StoreAccountRef, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let account = self.get(account)?;
		account.decrypt(password, shared_mac, message)
	}

	fn create_vault(&self, name: &str, password: &str) -> Result<(), Error> {
		self.store.create_vault(name, password)
	}

	fn open_vault(&self, name: &str, password: &str) -> Result<(), Error> {
		self.store.open_vault(name, password)
	}

	fn close_vault(&self, name: &str) -> Result<(), Error> {
		self.store.close_vault(name)
	}

	fn change_vault_password(&self, name: &str, password: &str, new_password: &str) -> Result<(), Error> {
		self.store.change_vault_password(name, password, new_password)
	}
}

impl SecretStore for EthStore {
	fn import_presale(&self, vault: SecretVaultRef, json: &[u8], password: &str) -> Result<StoreAccountRef, Error> {
		let json_wallet = json::PresaleWallet::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned()))?;
		let wallet = PresaleWallet::from(json_wallet);
		let keypair = wallet.decrypt(password).map_err(|_| Error::InvalidPassword)?;
		self.insert_account(vault, keypair.secret().clone(), password)
	}

	fn import_wallet(&self, vault: SecretVaultRef, json: &[u8], password: &str) -> Result<StoreAccountRef, Error> {
		let json_keyfile = json::KeyFile::load(json).map_err(|_| Error::InvalidKeyFile("Invalid JSON format".to_owned()))?;
		let mut safe_account = SafeAccount::from_file(json_keyfile, None);
		let secret = safe_account.crypto.secret(password).map_err(|_| Error::InvalidPassword)?;
		safe_account.address = KeyPair::from_secret(secret)?.address();
		self.store.import(vault, safe_account)
	}

	fn test_password(&self, account: &StoreAccountRef, password: &str) -> Result<bool, Error> {
		let account = self.get(account)?;
		Ok(account.check_password(password))
	}

	fn copy_account(&self, new_store: &SimpleSecretStore, new_vault: SecretVaultRef, account: &StoreAccountRef, password: &str, new_password: &str) -> Result<(), Error> {
		let account = self.get(account)?;
		let secret = account.crypto.secret(password)?;
		new_store.insert_account(new_vault, secret, new_password)?;
		Ok(())
	}

	fn move_account(&self, new_store: &SimpleSecretStore, new_vault: SecretVaultRef, account: &StoreAccountRef, password: &str, new_password: &str) -> Result<(), Error> {
		self.copy_account(new_store, new_vault, account, password, new_password)?;
		self.remove_account(account, password)?;
		Ok(())
	}

	fn public(&self, account: &StoreAccountRef, password: &str) -> Result<Public, Error> {
		let account = self.get(account)?;
		account.public(password)
	}

	fn uuid(&self, account: &StoreAccountRef) -> Result<Uuid, Error> {
		let account = self.get(account)?;
		Ok(account.id.into())
	}

	fn name(&self, account: &StoreAccountRef) -> Result<String, Error> {
		let account = self.get(account)?;
		Ok(account.name.clone())
	}

	fn meta(&self, account: &StoreAccountRef) -> Result<String, Error> {
		let account = self.get(account)?;
		Ok(account.meta.clone())
	}

	fn set_name(&self, account_ref: &StoreAccountRef, name: String) -> Result<(), Error> {
		let old = self.get(account_ref)?;
		let mut safe_account = old.clone();
		safe_account.name = name;

		// save to file
		self.store.update(account_ref, old, safe_account)
	}

	fn set_meta(&self, account_ref: &StoreAccountRef, meta: String) -> Result<(), Error> {
		let old = self.get(account_ref)?;
		let mut safe_account = old.clone();
		safe_account.meta = meta;

		// save to file
		self.store.update(account_ref, old, safe_account)
	}

	fn local_path(&self) -> String {
		self.store.dir.path().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| String::new())
	}

	fn list_geth_accounts(&self, testnet: bool) -> Vec<Address> {
		import::read_geth_accounts(testnet)
	}

	fn import_geth_accounts(&self, vault: SecretVaultRef, desired: Vec<Address>, testnet: bool) -> Result<Vec<StoreAccountRef>, Error> {
		let imported_addresses = match vault {
			SecretVaultRef::Root => import::import_geth_accounts(&*self.store.dir, desired.into_iter().collect(), testnet),
			SecretVaultRef::Vault(vault_name) => {
				if let Some(vault) = self.store.vaults.lock().get(&vault_name) {
					import::import_geth_accounts(vault.as_key_directory(), desired.into_iter().collect(), testnet)
				} else {
					Err(Error::VaultNotFound)
				}
			},
		};

		imported_addresses
			.map(|a| a.into_iter().map(|a| StoreAccountRef::root(a)).collect())
	}
}

/// Similar to `EthStore` but may store many accounts (with different passwords) for the same `Address`
pub struct EthMultiStore {
	dir: Box<KeyDirectory>,
	iterations: u32,
	// order lock: cache, then vaults
	cache: RwLock<BTreeMap<StoreAccountRef, Vec<SafeAccount>>>,
	vaults: Mutex<HashMap<String, Box<VaultKeyDirectory>>>,
}

impl EthMultiStore {

	pub fn open(directory: Box<KeyDirectory>) -> Result<Self, Error> {
		Self::open_with_iterations(directory, KEY_ITERATIONS as u32)
	}

	pub fn open_with_iterations(directory: Box<KeyDirectory>, iterations: u32) -> Result<Self, Error> {
		let store = EthMultiStore {
			dir: directory,
			vaults: Mutex::new(HashMap::new()),
			iterations: iterations,
			cache: Default::default(),
		};
		store.reload_accounts()?;
		Ok(store)
	}

	fn reload_accounts(&self) -> Result<(), Error> {
		let mut cache = self.cache.write();

		let mut new_accounts = BTreeMap::new();
		for account in self.dir.load()? {
			let account_ref = StoreAccountRef::root(account.address);
			new_accounts
				.entry(account_ref)
				.or_insert_with(Vec::new)
				.push(account);
		}
		for (vault_name, vault) in &*self.vaults.lock() {
			for account in vault.load()? {
				let account_ref = StoreAccountRef::vault(vault_name, account.address);
				new_accounts
					.entry(account_ref)
					.or_insert_with(Vec::new)
					.push(account);
			}
		}

		mem::replace(&mut *cache, new_accounts);
		Ok(())
	}

	fn get(&self, account: &StoreAccountRef) -> Result<Vec<SafeAccount>, Error> {
		{
			let cache = self.cache.read();
			if let Some(accounts) = cache.get(account) {
				if !accounts.is_empty() {
					return Ok(accounts.clone())
				}
			}
		}

		self.reload_accounts()?;
		let cache = self.cache.read();
		let accounts = cache.get(account).ok_or(Error::InvalidAccount)?;
		if accounts.is_empty() {
			Err(Error::InvalidAccount)
		} else {
			Ok(accounts.clone())
		}
	}

	fn import(&self, vault: SecretVaultRef, account: SafeAccount) -> Result<StoreAccountRef, Error> {
		// save to file
		let account = match vault {
			SecretVaultRef::Root => self.dir.insert(account)?,
			SecretVaultRef::Vault(ref vault_name) => self.vaults.lock().get_mut(vault_name).ok_or(Error::VaultNotFound)?.insert(account)?,
		};

		// update cache
		let account_ref = StoreAccountRef::new(vault, account.address.clone());
		let mut cache = self.cache.write();
		cache.entry(account_ref.clone())
			.or_insert_with(Vec::new)
			.push(account);

		Ok(account_ref)
	}

	fn update(&self, account_ref: &StoreAccountRef, old: SafeAccount, new: SafeAccount) -> Result<(), Error> {
		// save to file
		let account = match account_ref.vault {
			SecretVaultRef::Root => self.dir.update(new)?,
			SecretVaultRef::Vault(ref vault_name) => self.vaults.lock().get_mut(vault_name).ok_or(Error::VaultNotFound)?.update(new)?,
		};

		// update cache
		let mut cache = self.cache.write();
		let mut accounts = cache.entry(account_ref.clone()).or_insert_with(Vec::new);
		// Remove old account
		accounts.retain(|acc| acc != &old);
		// And push updated to the end
		accounts.push(account);
		Ok(())

	}

}

impl SimpleSecretStore for EthMultiStore {
	fn insert_account(&self, vault: SecretVaultRef, secret: Secret, password: &str) -> Result<StoreAccountRef, Error> {
		let keypair = KeyPair::from_secret(secret).map_err(|_| Error::CreationFailed)?;
		let id: [u8; 16] = Random::random();
		let account = SafeAccount::create(&keypair, id, password, self.iterations, "".to_owned(), "{}".to_owned());
		self.import(vault, account)
	}

	fn accounts(&self) -> Result<Vec<StoreAccountRef>, Error> {
		self.reload_accounts()?;
		Ok(self.cache.read().keys().cloned().collect())
	}

	fn remove_account(&self, account_ref: &StoreAccountRef, password: &str) -> Result<(), Error> {
		let accounts = self.get(account_ref)?;

		for account in accounts {
			// Skip if password is invalid
			if !account.check_password(password) {
				continue;
			}

			// Remove from dir
			match account_ref.vault {
				SecretVaultRef::Root => self.dir.remove(&account)?,
				SecretVaultRef::Vault(ref vault_name) => self.vaults.lock().get(vault_name).ok_or(Error::VaultNotFound)?.remove(&account)?,
			};

			// Remove from cache
			let mut cache = self.cache.write();
			let is_empty = {
				if let Some(accounts) = cache.get_mut(account_ref) {
					if let Some(position) = accounts.iter().position(|acc| acc == &account) {
						accounts.remove(position);
					}
					accounts.is_empty()
				} else {
					false
				}
			};

			if is_empty {
				cache.remove(account_ref);
			}

			return Ok(());
		}
		Err(Error::InvalidPassword)
	}

	fn change_password(&self, account_ref: &StoreAccountRef, old_password: &str, new_password: &str) -> Result<(), Error> {
		match account_ref.vault {
			SecretVaultRef::Root => {
				let accounts = self.get(account_ref)?;

				for account in accounts {
					// Change password
					let new_account = account.change_password(old_password, new_password, self.iterations)?;
					self.update(account_ref, account, new_account)?;
				}
				Ok(())
			},
			SecretVaultRef::Vault(ref vault_name) => {
				self.change_vault_password(vault_name, old_password, new_password)
			},
		}
	}

	fn sign(&self, account: &StoreAccountRef, password: &str, message: &Message) -> Result<Signature, Error> {
		let accounts = self.get(account)?;
		for account in accounts {
			if account.check_password(password) {
				return account.sign(password, message);
			}
		}

		Err(Error::InvalidPassword)
	}

	fn decrypt(&self, account: &StoreAccountRef, password: &str, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
		let accounts = self.get(account)?;
		for account in accounts {
			if account.check_password(password) {
				return account.decrypt(password, shared_mac, message);
			}
		}
		Err(Error::InvalidPassword)
	}

	fn create_vault(&self, name: &str, password: &str) -> Result<(), Error> {
		let is_vault_created = { // lock border
			let mut vaults = self.vaults.lock();
			if !vaults.contains_key(&name.to_owned()) {
				let vault_provider = self.dir.as_vault_provider().ok_or(Error::VaultsAreNotSupported)?;
				let vault = vault_provider.create(name, VaultKey::new(password, self.iterations))?;
				vaults.insert(name.to_owned(), vault);
				true
			} else {
				false
			}
		};

		if is_vault_created {
			self.reload_accounts()?;
		}

		Ok(())
	}

	fn open_vault(&self, name: &str, password: &str) -> Result<(), Error> {
		let is_vault_opened = { // lock border
			let mut vaults = self.vaults.lock();
			if !vaults.contains_key(&name.to_owned()) {
				let vault_provider = self.dir.as_vault_provider().ok_or(Error::VaultsAreNotSupported)?;
				let vault = vault_provider.open(name, VaultKey::new(password, self.iterations))?;
				vaults.insert(name.to_owned(), vault);
				true
			} else {
				false
			}
		};

		if is_vault_opened {
			self.reload_accounts()?;
		}

		Ok(())
	}

	fn close_vault(&self, name: &str) -> Result<(), Error> {
		let is_vault_removed = self.vaults.lock().remove(&name.to_owned()).is_some();
		if is_vault_removed {
			self.reload_accounts()?;
		}
		Ok(())
	}

	fn change_vault_password(&self, name: &str, password: &str, new_password: &str) -> Result<(), Error> {
		let vault_provider = self.dir.as_vault_provider().ok_or(Error::VaultsAreNotSupported)?;
		let vault = vault_provider.open(name, VaultKey::new(password, self.iterations))?;
		match vault.set_key(VaultKey::new(password, self.iterations), VaultKey::new(new_password, self.iterations)) {
			Ok(_) => {
				self.close_vault(name)
					.and_then(|_| self.open_vault(name, new_password))
			},
			Err(SetKeyError::Fatal(err)) => {
				let _ = self.close_vault(name);
				Err(err)
			}
			Err(SetKeyError::NonFatalNew(err)) => {
				let _ = self.close_vault(name)
					.and_then(|_| self.open_vault(name, new_password));
				Err(err)
			},
			Err(SetKeyError::NonFatalOld(err)) => Err(err),
		}
	}
}

#[cfg(test)]
mod tests {

	use std::{env, fs};
	use std::path::PathBuf;
	use dir::{KeyDirectory, MemoryDirectory, RootDiskDirectory};
	use ethkey::{Random, Generator, KeyPair};
	use secret_store::{SimpleSecretStore, SecretStore, SecretVaultRef, StoreAccountRef};
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

	struct RootDiskDirectoryGuard {
		pub key_dir: Option<Box<KeyDirectory>>,
		path: Option<PathBuf>,
	}

	impl RootDiskDirectoryGuard {
		pub fn new(test_name: &str) -> Self {
			let mut path = env::temp_dir();
			path.push(test_name);
			fs::create_dir_all(&path).unwrap();

			RootDiskDirectoryGuard {
				key_dir: Some(Box::new(RootDiskDirectory::create(&path).unwrap())),
				path: Some(path),
			}
		}
	}

	impl Drop for RootDiskDirectoryGuard {
		fn drop(&mut self) {
			if let Some(path) = self.path.take() {
				let _ = fs::remove_dir_all(path);
			}
		}
	}

	#[test]
	fn should_insert_account_successfully() {
		// given
		let store = store();
		let keypair = keypair();

		// when
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();

		// then
		assert_eq!(address, StoreAccountRef::root(keypair.address()));
		assert!(store.get(&address).is_ok(), "Should contain account.");
		assert_eq!(store.accounts().unwrap().len(), 1, "Should have one account.");
	}

	#[test]
	fn should_update_meta_and_name() {
		// given
		let store = store();
		let keypair = keypair();
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();
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
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();

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
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();

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
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();
		let address2 = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "xyz").unwrap();
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
		let address = store.insert_account(SecretVaultRef::Root, keypair.secret().clone(), "test").unwrap();
		assert_eq!(multi_store.accounts().unwrap().len(), 0);

		// when
		store.copy_account(&multi_store, SecretVaultRef::Root, &address, "test", "xyz").unwrap();

		// then
		assert!(store.test_password(&address, "test").unwrap(), "First password should work for store.");
		assert!(multi_store.sign(&address, "xyz", &Default::default()).is_ok(), "Second password should work for second store.");
		assert_eq!(multi_store.accounts().unwrap().len(), 1);
	}

	#[test]
	fn should_create_and_open_vaults() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_create_and_open_vaults");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let name2 = "vault2"; let password2 = "password2";
		let keypair1 = keypair();
		let keypair2 = keypair();
		let keypair3 = keypair(); let password3 = "password3";

		// when
		store.create_vault(name1, password1).unwrap();
		store.create_vault(name2, password2).unwrap();

		// then [can create vaults] ^^^

		// and when
		store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password1).unwrap();
		store.insert_account(SecretVaultRef::Vault(name2.to_owned()), keypair2.secret().clone(), password2).unwrap();
		store.insert_account(SecretVaultRef::Root, keypair3.secret().clone(), password3).unwrap();
		store.insert_account(SecretVaultRef::Vault("vault3".to_owned()), keypair1.secret().clone(), password3).unwrap_err();
		let accounts = store.accounts().unwrap();

		// then [can create accounts in vaults]
		assert_eq!(accounts.len(), 3);
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Root));
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Vault(name1.to_owned())));
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Vault(name2.to_owned())));

		// and when
		store.close_vault(name1).unwrap();
		store.close_vault(name2).unwrap();
		store.close_vault("vault3").unwrap();
		let accounts = store.accounts().unwrap();

		// then [can close vaults + accounts from vaults disappear]
		assert_eq!(accounts.len(), 1);
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Root));

		// and when
		store.open_vault(name1, password2).unwrap_err();
		store.open_vault(name2, password1).unwrap_err();
		store.open_vault(name1, password1).unwrap();
		store.open_vault(name2, password2).unwrap();
		let accounts = store.accounts().unwrap();

		// then [can check vaults on open + can reopen vaults + accounts from vaults appear]
		assert_eq!(accounts.len(), 3);
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Root));
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Vault(name1.to_owned())));
		assert!(accounts.iter().any(|a| a.vault == SecretVaultRef::Vault(name2.to_owned())));
	}

	#[test]
	fn should_move_vault_acounts() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_move_vault_acounts");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let name2 = "vault2"; let password2 = "password2";
		let password3 = "password3";
		let keypair1 = keypair();
		let keypair2 = keypair();
		let keypair3 = keypair();

		// when
		store.create_vault(name1, password1).unwrap();
		store.create_vault(name2, password2).unwrap();
		let account1 = store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password1).unwrap();
		let account2 = store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair2.secret().clone(), password1).unwrap();
		let account3 = store.insert_account(SecretVaultRef::Root, keypair3.secret().clone(), password3).unwrap();

		// then
		store.move_account(&store, SecretVaultRef::Root, &account1, password1, password2).unwrap();
		store.move_account(&store, SecretVaultRef::Vault(name2.to_owned()), &account2, password1, password2).unwrap();
		store.move_account(&store, SecretVaultRef::Vault(name2.to_owned()), &account3, password3, password2).unwrap();
		let accounts = store.accounts().unwrap();
		assert_eq!(accounts.len(), 3);
		assert!(accounts.iter().any(|a| a == &StoreAccountRef::root(account1.address.clone())));
		assert!(accounts.iter().any(|a| a == &StoreAccountRef::vault(name2, account2.address.clone())));
		assert!(accounts.iter().any(|a| a == &StoreAccountRef::vault(name2, account3.address.clone())));
	}

	#[test]
	fn should_not_remove_account_when_moving_to_self() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_not_remove_account_when_moving_to_self");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let password1 = "password1";
		let keypair1 = keypair();

		// when
		let account1 = store.insert_account(SecretVaultRef::Root, keypair1.secret().clone(), password1).unwrap();
		store.move_account(&store, SecretVaultRef::Root, &account1, password1, password1).unwrap();

		// then
		let accounts = store.accounts().unwrap();
		assert_eq!(accounts.len(), 1);
	}

	#[test]
	fn should_not_move_account_when_vault_password_incorrect() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_not_move_account_when_vault_password_incorrect");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let name2 = "vault2"; let password2 = "password2";
		let keypair1 = keypair();

		// when
		store.create_vault(name1, password1).unwrap();
		store.create_vault(name2, password2).unwrap();
		let account1 = store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password1).unwrap();

		// then
		store.move_account(&store, SecretVaultRef::Root, &account1, password2, password1).unwrap_err();
		store.move_account(&store, SecretVaultRef::Vault(name2.to_owned()), &account1, password1, password1).unwrap_err();
	}

	#[test]
	fn should_not_insert_account_when_vault_password_incorrect() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_not_insert_account_when_vault_password_incorrect");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let password2 = "password2";
		let keypair1 = keypair();

		// when
		store.create_vault(name1, password1).unwrap();

		// then
		store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password2).unwrap_err();
	}

	#[test]
	fn should_remove_account_from_vault() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_remove_account_from_vault");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let keypair1 = keypair();

		// when
		store.create_vault(name1, password1).unwrap();
		let account1 = store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password1).unwrap();
		assert_eq!(store.accounts().unwrap().len(), 1);

		// then
		store.remove_account(&account1, password1).unwrap();
		assert_eq!(store.accounts().unwrap().len(), 0);
	}

	#[test]
	fn should_not_remove_account_from_vault_when_password_is_incorrect() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_not_remove_account_from_vault_when_password_is_incorrect");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name1 = "vault1"; let password1 = "password1";
		let password2 = "password2";
		let keypair1 = keypair();

		// when
		store.create_vault(name1, password1).unwrap();
		let account1 = store.insert_account(SecretVaultRef::Vault(name1.to_owned()), keypair1.secret().clone(), password1).unwrap();
		assert_eq!(store.accounts().unwrap().len(), 1);

		// then
		store.remove_account(&account1, password2).unwrap_err();
		assert_eq!(store.accounts().unwrap().len(), 1);
	}

	#[test]
	fn should_change_vault_password() {
		// given
		let mut dir = RootDiskDirectoryGuard::new("should_change_vault_password");
		let store = EthStore::open(dir.key_dir.take().unwrap()).unwrap();
		let name = "vault"; let password = "password";
		let keypair = keypair();

		// when
		store.create_vault(name, password).unwrap();
		store.insert_account(SecretVaultRef::Vault(name.to_owned()), keypair.secret().clone(), password).unwrap();

		// then
		assert_eq!(store.accounts().unwrap().len(), 1);
		let new_password = "new_password";
		store.change_vault_password(name, "bad_password", new_password).unwrap_err();
		assert_eq!(store.accounts().unwrap().len(), 1);
		store.change_vault_password(name, password, new_password).unwrap();
		assert_eq!(store.accounts().unwrap().len(), 1);

		// and when
		store.close_vault(name).unwrap();

		// then
		store.open_vault(name, new_password).unwrap();
		assert_eq!(store.accounts().unwrap().len(), 1);
	}
}
