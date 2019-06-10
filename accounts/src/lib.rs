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

#![warn(missing_docs)]

//! Account management.

mod account_data;
mod error;
mod stores;

use self::account_data::{Unlock, AccountData};
use self::stores::AddressBook;

use std::collections::HashMap;
use std::time::{Instant, Duration};

use ethkey::{Address, Message, Public, Secret, Password, Random, Generator};
use ethstore::accounts_dir::MemoryDirectory;
use ethstore::{
	SimpleSecretStore, SecretStore, EthStore, EthMultiStore,
	random_string, SecretVaultRef, StoreAccountRef, OpaqueSecret,
};
use log::warn;
use parking_lot::RwLock;

pub use ethkey::Signature;
pub use ethstore::{Derivation, IndexDerivation, KeyFile, Error};

pub use self::account_data::AccountMeta;
pub use self::error::SignError;

type AccountToken = Password;

/// Account management settings.
#[derive(Debug, Default)]
pub struct AccountProviderSettings {
	/// Store raw account secret when unlocking the account permanently.
	pub unlock_keep_secret: bool,
	/// Disallowed accounts.
	pub blacklisted_accounts: Vec<Address>,
}

/// Account management.
/// Responsible for unlocking accounts.
pub struct AccountProvider {
	/// For performance reasons some methods can re-use unlocked secrets.
	unlocked_secrets: RwLock<HashMap<StoreAccountRef, OpaqueSecret>>,
	/// Unlocked account data.
	unlocked: RwLock<HashMap<StoreAccountRef, AccountData>>,
	/// Address book.
	address_book: RwLock<AddressBook>,
	/// Accounts on disk
	sstore: Box<SecretStore>,
	/// Accounts unlocked with rolling tokens
	transient_sstore: EthMultiStore,
	/// When unlocking account permanently we additionally keep a raw secret in memory
	/// to increase the performance of transaction signing.
	unlock_keep_secret: bool,
	/// Disallowed accounts.
	blacklisted_accounts: Vec<Address>,
}

fn transient_sstore() -> EthMultiStore {
	EthMultiStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory load always succeeds; qed")
}

impl AccountProvider {
	/// Creates new account provider.
	pub fn new(sstore: Box<SecretStore>, settings: AccountProviderSettings) -> Self {
		if let Ok(accounts) = sstore.accounts() {
			for account in accounts.into_iter().filter(|a| settings.blacklisted_accounts.contains(&a.address)) {
				warn!("Local Account {} has a blacklisted (known to be weak) address and will be ignored",
					account.address);
			}
		}

		// Remove blacklisted accounts from address book.
		let mut address_book = AddressBook::new(&sstore.local_path());
		for addr in &settings.blacklisted_accounts {
			address_book.remove(*addr);
		}

		AccountProvider {
			unlocked_secrets: RwLock::new(HashMap::new()),
			unlocked: RwLock::new(HashMap::new()),
			address_book: RwLock::new(address_book),
			sstore,
			transient_sstore: transient_sstore(),
			unlock_keep_secret: settings.unlock_keep_secret,
			blacklisted_accounts: settings.blacklisted_accounts,
		}
	}

	/// Creates not disk backed provider.
	pub fn transient_provider() -> Self {
		AccountProvider {
			unlocked_secrets: RwLock::new(HashMap::new()),
			unlocked: RwLock::new(HashMap::new()),
			address_book: RwLock::new(AddressBook::transient()),
			sstore: Box::new(EthStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory load always succeeds; qed")),
			transient_sstore: transient_sstore(),
			unlock_keep_secret: false,
			blacklisted_accounts: vec![],
		}
	}

	/// Creates new random account.
	pub fn new_account(&self, password: &Password) -> Result<Address, Error> {
		self.new_account_and_public(password).map(|d| d.0)
	}

	/// Creates new random account and returns address and public key
	pub fn new_account_and_public(&self, password: &Password) -> Result<(Address, Public), Error> {
		let acc = Random.generate().expect("secp context has generation capabilities; qed");
		let public = acc.public().clone();
		let secret = acc.secret().clone();
		let account = self.sstore.insert_account(SecretVaultRef::Root, secret, password)?;
		Ok((account.address, public))
	}

	/// Inserts new account into underlying store.
	/// Does not unlock account!
	pub fn insert_account(&self, secret: Secret, password: &Password) -> Result<Address, Error> {
		let account = self.sstore.insert_account(SecretVaultRef::Root, secret, password)?;
		if self.blacklisted_accounts.contains(&account.address) {
			self.sstore.remove_account(&account, password)?;
			return Err(Error::InvalidAccount.into());
		}
		Ok(account.address)
	}

	/// Generates new derived account based on the existing one
	/// If password is not provided, account must be unlocked
	/// New account will be created with the same password (if save: true)
	pub fn derive_account(&self, address: &Address, password: Option<Password>, derivation: Derivation, save: bool)
		-> Result<Address, SignError>
	{
		let account = self.sstore.account_ref(&address)?;
		let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
		Ok(
			if save { self.sstore.insert_derived(SecretVaultRef::Root, &account, &password, derivation)?.address }
			else { self.sstore.generate_derived(&account, &password, derivation)? }
		)
	}

	/// Import a new presale wallet.
	pub fn import_presale(&self, presale_json: &[u8], password: &Password) -> Result<Address, Error> {
		let account = self.sstore.import_presale(SecretVaultRef::Root, presale_json, password)?;
		Ok(Address::from(account.address).into())
	}

	/// Import a new wallet.
	pub fn import_wallet(&self, json: &[u8], password: &Password, gen_id: bool) -> Result<Address, Error> {
		let account = self.sstore.import_wallet(SecretVaultRef::Root, json, password, gen_id)?;
		if self.blacklisted_accounts.contains(&account.address) {
			self.sstore.remove_account(&account, password)?;
			return Err(Error::InvalidAccount.into());
		}
		Ok(Address::from(account.address).into())
	}

	/// Checks whether an account with a given address is present.
	pub fn has_account(&self, address: Address) -> bool {
		self.sstore.account_ref(&address).is_ok() && !self.blacklisted_accounts.contains(&address)
	}

	/// Returns addresses of all accounts.
	pub fn accounts(&self) -> Result<Vec<Address>, Error> {
		let accounts = self.sstore.accounts()?;
		Ok(accounts
			.into_iter()
			.map(|a| a.address)
			.filter(|address| !self.blacklisted_accounts.contains(address))
			.collect()
		)
	}

	/// Returns the address of default account.
	pub fn default_account(&self) -> Result<Address, Error> {
		Ok(self.accounts()?.first().cloned().unwrap_or_default())
	}

	/// Returns each address along with metadata.
	pub fn addresses_info(&self) -> HashMap<Address, AccountMeta> {
		self.address_book.read().get()
	}

	/// Returns each address along with metadata.
	pub fn set_address_name(&self, account: Address, name: String) {
		self.address_book.write().set_name(account, name)
	}

	/// Returns each address along with metadata.
	pub fn set_address_meta(&self, account: Address, meta: String) {
		self.address_book.write().set_meta(account, meta)
	}

	/// Removes and address from the address book
	pub fn remove_address(&self, addr: Address) {
		self.address_book.write().remove(addr)
	}

	/// Returns each account along with name and meta.
	pub fn accounts_info(&self) -> Result<HashMap<Address, AccountMeta>, Error> {
		let r = self.sstore.accounts()?
			.into_iter()
			.filter(|a| !self.blacklisted_accounts.contains(&a.address))
			.map(|a| (a.address.clone(), self.account_meta(a.address).ok().unwrap_or_default()))
			.collect();
		Ok(r)
	}

	/// Returns each account along with name and meta.
	pub fn account_meta(&self, address: Address) -> Result<AccountMeta, Error> {
		let account = self.sstore.account_ref(&address)?;
		Ok(AccountMeta {
			name: self.sstore.name(&account)?,
			meta: self.sstore.meta(&account)?,
			uuid: self.sstore.uuid(&account).ok().map(Into::into),	// allowed to not have a Uuid
		})
	}

	/// Returns account public key.
	pub fn account_public(&self, address: Address, password: &Password) -> Result<Public, Error> {
		self.sstore.public(&self.sstore.account_ref(&address)?, password)
	}

	/// Returns each account along with name and meta.
	pub fn set_account_name(&self, address: Address, name: String) -> Result<(), Error> {
		self.sstore.set_name(&self.sstore.account_ref(&address)?, name)?;
		Ok(())
	}

	/// Returns each account along with name and meta.
	pub fn set_account_meta(&self, address: Address, meta: String) -> Result<(), Error> {
		self.sstore.set_meta(&self.sstore.account_ref(&address)?, meta)?;
		Ok(())
	}

	/// Returns `true` if the password for `account` is `password`. `false` if not.
	pub fn test_password(&self, address: &Address, password: &Password) -> Result<bool, Error> {
		self.sstore.test_password(&self.sstore.account_ref(&address)?, password)
			.map_err(Into::into)
	}

	/// Permanently removes an account.
	pub fn kill_account(&self, address: &Address, password: &Password) -> Result<(), Error> {
		self.sstore.remove_account(&self.sstore.account_ref(&address)?, &password)?;
		Ok(())
	}

	/// Changes the password of `account` from `password` to `new_password`. Fails if incorrect `password` given.
	pub fn change_password(&self, address: &Address, password: Password, new_password: Password) -> Result<(), Error> {
		self.sstore.change_password(&self.sstore.account_ref(address)?, &password, &new_password)
	}

	/// Exports an account for given address.
	pub fn export_account(&self, address: &Address, password: Password) -> Result<KeyFile, Error> {
		self.sstore.export_account(&self.sstore.account_ref(address)?, &password)
	}

	/// Helper method used for unlocking accounts.
	fn unlock_account(&self, address: Address, password: Password, unlock: Unlock) -> Result<(), Error> {
		let account = self.sstore.account_ref(&address)?;

		// check if account is already unlocked permanently, if it is, do nothing
		let mut unlocked = self.unlocked.write();
		if let Some(data) = unlocked.get(&account) {
			if let Unlock::Perm = data.unlock {
				return Ok(())
			}
		}

		if self.unlock_keep_secret && unlock == Unlock::Perm {
			// verify password and get the secret
			let secret = self.sstore.raw_secret(&account, &password)?;
			self.unlocked_secrets.write().insert(account.clone(), secret);
		} else {
			// verify password by signing dump message
			// result may be discarded
			let _ = self.sstore.sign(&account, &password, &Default::default())?;
		}

		let data = AccountData { unlock, password };

		unlocked.insert(account, data);
		Ok(())
	}

	fn password(&self, account: &StoreAccountRef) -> Result<Password, SignError> {
		let mut unlocked = self.unlocked.write();
		let data = unlocked.get(account).ok_or(SignError::NotUnlocked)?.clone();
		if let Unlock::OneTime = data.unlock {
			unlocked.remove(account).expect("data exists: so key must exist: qed");
		}
		if let Unlock::Timed(ref end) = data.unlock {
			if Instant::now() > *end {
				unlocked.remove(account).expect("data exists: so key must exist: qed");
				return Err(SignError::NotUnlocked);
			}
		}
		Ok(data.password)
	}

	/// Unlocks account permanently.
	pub fn unlock_account_permanently(&self, account: Address, password: Password) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Perm)
	}

	/// Unlocks account temporarily (for one signing).
	pub fn unlock_account_temporarily(&self, account: Address, password: Password) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::OneTime)
	}

	/// Unlocks account temporarily with a timeout.
	pub fn unlock_account_timed(&self, account: Address, password: Password, duration: Duration) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Timed(Instant::now() + duration))
	}

	/// Checks if given account is unlocked
	pub fn is_unlocked(&self, address: &Address) -> bool {
		let unlocked = self.unlocked.read();
		let unlocked_secrets = self.unlocked_secrets.read();
		self.sstore.account_ref(address)
			.map(|r| unlocked.get(&r).is_some() || unlocked_secrets.get(&r).is_some())
			.unwrap_or(false)
	}

	/// Checks if given account is unlocked permanently
	pub fn is_unlocked_permanently(&self, address: &Address) -> bool {
		let unlocked = self.unlocked.read();
		self.sstore.account_ref(address)
			.map(|r| unlocked.get(&r).map_or(false, |account| account.unlock == Unlock::Perm))
			.unwrap_or(false)
	}

	/// Signs the message. If password is not provided the account must be unlocked.
	pub fn sign(&self, address: Address, password: Option<Password>, message: Message) -> Result<Signature, SignError> {
		let account = self.sstore.account_ref(&address)?;
		match self.unlocked_secrets.read().get(&account) {
			Some(secret) => {
				Ok(self.sstore.sign_with_secret(&secret, &message)?)
			},
			None => {
				let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
				Ok(self.sstore.sign(&account, &password, &message)?)
			}
		}
	}

	/// Signs message using the derived secret. If password is not provided the account must be unlocked.
	pub fn sign_derived(&self, address: &Address, password: Option<Password>, derivation: Derivation, message: Message)
		-> Result<Signature, SignError>
	{
		let account = self.sstore.account_ref(address)?;
		let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
		Ok(self.sstore.sign_derived(&account, &password, derivation, &message)?)
	}

	/// Signs given message with supplied token. Returns a token to use in next signing within this session.
	pub fn sign_with_token(&self, address: Address, token: AccountToken, message: Message) -> Result<(Signature, AccountToken), SignError> {
		let account = self.sstore.account_ref(&address)?;
		let is_std_password = self.sstore.test_password(&account, &token)?;

		let new_token = Password::from(random_string(16));
		let signature = if is_std_password {
			// Insert to transient store
			self.sstore.copy_account(&self.transient_sstore, SecretVaultRef::Root, &account, &token, &new_token)?;
			// sign
			self.sstore.sign(&account, &token, &message)?
		} else {
			// check transient store
			self.transient_sstore.change_password(&account, &token, &new_token)?;
			// and sign
			self.transient_sstore.sign(&account, &new_token, &message)?
		};

		Ok((signature, new_token))
	}

	/// Decrypts a message with given token. Returns a token to use in next operation for this account.
	pub fn decrypt_with_token(&self, address: Address, token: AccountToken, shared_mac: &[u8], message: &[u8])
		-> Result<(Vec<u8>, AccountToken), SignError>
	{
		let account = self.sstore.account_ref(&address)?;
		let is_std_password = self.sstore.test_password(&account, &token)?;

		let new_token = Password::from(random_string(16));
		let message = if is_std_password {
			// Insert to transient store
			self.sstore.copy_account(&self.transient_sstore, SecretVaultRef::Root, &account, &token, &new_token)?;
			// decrypt
			self.sstore.decrypt(&account, &token, shared_mac, message)?
		} else {
			// check transient store
			self.transient_sstore.change_password(&account, &token, &new_token)?;
			// and decrypt
			self.transient_sstore.decrypt(&account, &token, shared_mac, message)?
		};

		Ok((message, new_token))
	}

	/// Decrypts a message. If password is not provided the account must be unlocked.
	pub fn decrypt(&self, address: Address, password: Option<Password>, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, SignError> {
		let account = self.sstore.account_ref(&address)?;
		let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
		Ok(self.sstore.decrypt(&account, &password, shared_mac, message)?)
	}

	/// Agree on shared key.
	pub fn agree(&self, address: Address, password: Option<Password>, other_public: &Public) -> Result<Secret, SignError> {
		let account = self.sstore.account_ref(&address)?;
		let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
		Ok(self.sstore.agree(&account, &password, other_public)?)
	}

	/// Returns the underlying `SecretStore` reference if one exists.
	pub fn list_geth_accounts(&self, testnet: bool) -> Vec<Address> {
		self.sstore.list_geth_accounts(testnet).into_iter().map(|a| Address::from(a).into()).collect()
	}

	/// Returns the underlying `SecretStore` reference if one exists.
	pub fn import_geth_accounts(&self, desired: Vec<Address>, testnet: bool) -> Result<Vec<Address>, Error> {
		self.sstore.import_geth_accounts(SecretVaultRef::Root, desired, testnet)
			.map(|a| a.into_iter().map(|a| a.address).collect())
			.map_err(Into::into)
	}

	/// Create new vault.
	pub fn create_vault(&self, name: &str, password: &Password) -> Result<(), Error> {
		self.sstore.create_vault(name, password)
			.map_err(Into::into)
	}

	/// Open existing vault.
	pub fn open_vault(&self, name: &str, password: &Password) -> Result<(), Error> {
		self.sstore.open_vault(name, password)
			.map_err(Into::into)
	}

	/// Close previously opened vault.
	pub fn close_vault(&self, name: &str) -> Result<(), Error> {
		self.sstore.close_vault(name)
			.map_err(Into::into)
	}

	/// List all vaults
	pub fn list_vaults(&self) -> Result<Vec<String>, Error> {
		self.sstore.list_vaults()
			.map_err(Into::into)
	}

	/// List all currently opened vaults
	pub fn list_opened_vaults(&self) -> Result<Vec<String>, Error> {
		self.sstore.list_opened_vaults()
			.map_err(Into::into)
	}

	/// Change vault password.
	pub fn change_vault_password(&self, name: &str, new_password: &Password) -> Result<(), Error> {
		self.sstore.change_vault_password(name, new_password)
			.map_err(Into::into)
	}

	/// Change vault of the given address.
	pub fn change_vault(&self, address: Address, new_vault: &str) -> Result<(), Error> {
		let new_vault_ref = if new_vault.is_empty() { SecretVaultRef::Root } else { SecretVaultRef::Vault(new_vault.to_owned()) };
		let old_account_ref = self.sstore.account_ref(&address)?;
		self.sstore.change_account_vault(new_vault_ref, old_account_ref)
			.map_err(Into::into)
			.map(|_| ())
	}

	/// Get vault metadata string.
	pub fn get_vault_meta(&self, name: &str) -> Result<String, Error> {
		self.sstore.get_vault_meta(name)
			.map_err(Into::into)
	}

	/// Set vault metadata string.
	pub fn set_vault_meta(&self, name: &str, meta: &str) -> Result<(), Error> {
		self.sstore.set_vault_meta(name, meta)
			.map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use super::{AccountProvider, Unlock};
	use std::time::{Duration, Instant};
	use ethkey::{Generator, Random, Address};
	use ethstore::{StoreAccountRef, Derivation};
	use ethereum_types::H256;

	#[test]
	fn unlock_account_temp() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"test".into()).is_ok());
		assert!(ap.unlock_account_temporarily(kp.address(), "test1".into()).is_err());
		assert!(ap.unlock_account_temporarily(kp.address(), "test".into()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_err());
	}

	#[test]
	fn derived_account_nosave() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"base".into()).is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from_low_u64_be(999)),
			false,
		).expect("Derivation should not fail");

		assert!(ap.unlock_account_permanently(derived_addr, "base".into()).is_err(),
			"There should be an error because account is not supposed to be saved");
	}

	#[test]
	fn derived_account_save() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"base".into()).is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from_low_u64_be(999)),
			true,
		).expect("Derivation should not fail");

		assert!(ap.unlock_account_permanently(derived_addr, "base_wrong".into()).is_err(),
			"There should be an error because password is invalid");

		assert!(ap.unlock_account_permanently(derived_addr, "base".into()).is_ok(),
			"Should be ok because account is saved and password is valid");
	}

	#[test]
	fn derived_account_sign() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"base".into()).is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from_low_u64_be(1999)),
			true,
		).expect("Derivation should not fail");
		ap.unlock_account_permanently(derived_addr, "base".into())
			.expect("Should be ok because account is saved and password is valid");

		let msg = Default::default();
		let signed_msg1 = ap.sign(derived_addr, None, msg)
			.expect("Signing with existing unlocked account should not fail");
		let signed_msg2 = ap.sign_derived(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from_low_u64_be(1999)),
			msg,
		).expect("Derived signing with existing unlocked account should not fail");

		assert_eq!(signed_msg1, signed_msg2,
			"Signed messages should match");
	}

	#[test]
	fn unlock_account_perm() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"test".into()).is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "test1".into()).is_err());
		assert!(ap.unlock_account_permanently(kp.address(), "test".into()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		assert!(ap.unlock_account_temporarily(kp.address(), "test".into()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
	}

	#[test]
	fn unlock_account_timer() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"test".into()).is_ok());
		assert!(ap.unlock_account_timed(kp.address(), "test1".into(), Duration::from_secs(60)).is_err());
		assert!(ap.unlock_account_timed(kp.address(), "test".into(), Duration::from_secs(60)).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		ap.unlocked.write().get_mut(&StoreAccountRef::root(kp.address())).unwrap().unlock = Unlock::Timed(Instant::now());
		assert!(ap.sign(kp.address(), None, Default::default()).is_err());
	}

	#[test]
	fn should_sign_and_return_token() {
		// given
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), &"test".into()).is_ok());

		// when
		let (_signature, token) = ap.sign_with_token(kp.address(), "test".into(), Default::default()).unwrap();

		// then
		ap.sign_with_token(kp.address(), token.clone(), Default::default())
			.expect("First usage of token should be correct.");
		assert!(ap.sign_with_token(kp.address(), token, Default::default()).is_err(), "Second usage of the same token should fail.");
	}

	#[test]
	fn should_not_return_blacklisted_account() {
		// given
		let mut ap = AccountProvider::transient_provider();
		let acc = ap.new_account(&"test".into()).unwrap();
		ap.blacklisted_accounts = vec![acc];

		// then
		assert_eq!(ap.accounts_info().unwrap().keys().cloned().collect::<Vec<Address>>(), vec![]);
		assert_eq!(ap.accounts().unwrap(), vec![]);
	}
}
