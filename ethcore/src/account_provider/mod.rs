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

//! Account management.

mod stores;

use self::stores::{AddressBook, DappsSettingsStore, NewDappsPolicy};

use std::fmt;
use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};
use util::{RwLock};
use ethstore::{
	SimpleSecretStore, SecretStore, Error as SSError, EthStore, EthMultiStore,
	random_string, SecretVaultRef, StoreAccountRef, OpaqueSecret,
};
use ethstore::dir::MemoryDirectory;
use ethstore::ethkey::{Address, Message, Public, Secret, Random, Generator};
use ethjson::misc::AccountMeta;
use hardware_wallet::{Error as HardwareError, HardwareWalletManager, KeyPath};
pub use ethstore::ethkey::Signature;
pub use ethstore::{Derivation, IndexDerivation, KeyFile};

/// Type of unlock.
#[derive(Clone, PartialEq)]
enum Unlock {
	/// If account is unlocked temporarily, it should be locked after first usage.
	OneTime,
	/// Account unlocked permantently can always sign message.
	/// Use with caution.
	Perm,
	/// Account unlocked with a timeout
	Timed(Instant),
}

/// Data associated with account.
#[derive(Clone)]
struct AccountData {
	unlock: Unlock,
	password: String,
}

/// Signing error
#[derive(Debug)]
pub enum SignError {
	/// Account is not unlocked
	NotUnlocked,
	/// Account does not exist.
	NotFound,
	/// Low-level hardware device error.
	Hardware(HardwareError),
	/// Low-level error from store
	SStore(SSError),
	/// Inappropriate chain
	InappropriateChain,
}

impl fmt::Display for SignError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			SignError::NotUnlocked => write!(f, "Account is locked"),
			SignError::NotFound => write!(f, "Account does not exist"),
			SignError::Hardware(ref e) => write!(f, "{}", e),
			SignError::SStore(ref e) => write!(f, "{}", e),
			SignError::InappropriateChain => write!(f, "Inappropriate chain"),
		}
	}
}

impl From<HardwareError> for SignError {
	fn from(e: HardwareError) -> Self {
		SignError::Hardware(e)
	}
}

impl From<SSError> for SignError {
	fn from(e: SSError) -> Self {
		SignError::SStore(e)
	}
}

/// `AccountProvider` errors.
pub type Error = SSError;

/// Dapp identifier
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DappId(String);

impl From<DappId> for String {
	fn from(id: DappId) -> String { id.0 }
}
impl From<String> for DappId {
	fn from(id: String) -> DappId { DappId(id) }
}
impl<'a> From<&'a str> for DappId {
	fn from(id: &'a str) -> DappId { DappId(id.to_owned()) }
}

fn transient_sstore() -> EthMultiStore {
	EthMultiStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory load always succeeds; qed")
}

type AccountToken = String;

/// Account management.
/// Responsible for unlocking accounts.
pub struct AccountProvider {
	/// For performance reasons some methods can re-use unlocked secrets.
	unlocked_secrets: RwLock<HashMap<StoreAccountRef, OpaqueSecret>>,
	/// Unlocked account data.
	unlocked: RwLock<HashMap<StoreAccountRef, AccountData>>,
	/// Address book.
	address_book: RwLock<AddressBook>,
	/// Dapps settings.
	dapps_settings: RwLock<DappsSettingsStore>,
	/// Accounts on disk
	sstore: Box<SecretStore>,
	/// Accounts unlocked with rolling tokens
	transient_sstore: EthMultiStore,
	/// Accounts in hardware wallets.
	hardware_store: Option<HardwareWalletManager>,
	/// When unlocking account permanently we additionally keep a raw secret in memory
	/// to increase the performance of transaction signing.
	unlock_keep_secret: bool,
	/// Disallowed accounts.
	blacklisted_accounts: Vec<Address>,
}

/// Account management settings.
pub struct AccountProviderSettings {
	/// Enable hardware wallet support.
	pub enable_hardware_wallets: bool,
	/// Use the classic chain key on the hardware wallet.
	pub hardware_wallet_classic_key: bool,
	/// Store raw account secret when unlocking the account permanently.
	pub unlock_keep_secret: bool,
	/// Disallowed accounts.
	pub blacklisted_accounts: Vec<Address>,
}

impl Default for AccountProviderSettings {
	fn default() -> Self {
		AccountProviderSettings {
			enable_hardware_wallets: false,
			hardware_wallet_classic_key: false,
			unlock_keep_secret: false,
			blacklisted_accounts: vec![],
		}
	}
}

impl AccountProvider {
	/// Creates new account provider.
	pub fn new(sstore: Box<SecretStore>, settings: AccountProviderSettings) -> Self {
		let mut hardware_store = None;
		if settings.enable_hardware_wallets {
			match HardwareWalletManager::new() {
				Ok(manager) => {
					manager.set_key_path(if settings.hardware_wallet_classic_key { KeyPath::EthereumClassic } else { KeyPath::Ethereum });
					hardware_store = Some(manager)
				},
				Err(e) => debug!("Error initializing hardware wallets: {}", e),
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
			dapps_settings: RwLock::new(DappsSettingsStore::new(&sstore.local_path())),
			sstore: sstore,
			transient_sstore: transient_sstore(),
			hardware_store: hardware_store,
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
			dapps_settings: RwLock::new(DappsSettingsStore::transient()),
			sstore: Box::new(EthStore::open(Box::new(MemoryDirectory::default())).expect("MemoryDirectory load always succeeds; qed")),
			transient_sstore: transient_sstore(),
			hardware_store: None,
			unlock_keep_secret: false,
			blacklisted_accounts: vec![],
		}
	}

	/// Creates new random account.
	pub fn new_account(&self, password: &str) -> Result<Address, Error> {
		self.new_account_and_public(password).map(|d| d.0)
	}

	/// Creates new random account and returns address and public key
	pub fn new_account_and_public(&self, password: &str) -> Result<(Address, Public), Error> {
		let acc = Random.generate().expect("secp context has generation capabilities; qed");
		let public = acc.public().clone();
		let secret = acc.secret().clone();
		let account = self.sstore.insert_account(SecretVaultRef::Root, secret, password)?;
		Ok((account.address, public))
	}

	/// Inserts new account into underlying store.
	/// Does not unlock account!
	pub fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		let account = self.sstore.insert_account(SecretVaultRef::Root, secret, password)?;
		if self.blacklisted_accounts.contains(&account.address) {
			self.sstore.remove_account(&account, password)?;
			return Err(SSError::InvalidAccount.into());
		}
		Ok(account.address)
	}

	/// Generates new derived account based on the existing one
	/// If password is not provided, account must be unlocked
	/// New account will be created with the same password (if save: true)
	pub fn derive_account(&self, address: &Address, password: Option<String>, derivation: Derivation, save: bool)
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
	pub fn import_presale(&self, presale_json: &[u8], password: &str) -> Result<Address, Error> {
		let account = self.sstore.import_presale(SecretVaultRef::Root, presale_json, password)?;
		Ok(Address::from(account.address).into())
	}

	/// Import a new presale wallet.
	pub fn import_wallet(&self, json: &[u8], password: &str) -> Result<Address, Error> {
		let account = self.sstore.import_wallet(SecretVaultRef::Root, json, password)?;
		if self.blacklisted_accounts.contains(&account.address) {
			self.sstore.remove_account(&account, password)?;
			return Err(SSError::InvalidAccount.into());
		}
		Ok(Address::from(account.address).into())
	}

	/// Checks whether an account with a given address is present.
	pub fn has_account(&self, address: Address) -> Result<bool, Error> {
		Ok(self.accounts()?.iter().any(|&a| a == address))
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

	/// Returns addresses of hardware accounts.
	pub fn hardware_accounts(&self) -> Result<Vec<Address>, Error> {
		let accounts = self.hardware_store.as_ref().map_or(Vec::new(), |h| h.list_wallets());
		Ok(accounts.into_iter().map(|a| a.address).collect())
	}

	/// Sets addresses of accounts exposed for unknown dapps.
	/// `None` means that all accounts will be visible.
	/// If not `None` or empty it will also override default account.
	pub fn set_new_dapps_addresses(&self, accounts: Option<Vec<Address>>) -> Result<(), Error> {
		let current_default = self.new_dapps_default_address()?;

		self.dapps_settings.write().set_policy(match accounts {
			None => NewDappsPolicy::AllAccounts {
				default: current_default,
			},
			Some(accounts) => NewDappsPolicy::Whitelist(accounts),
		});
		Ok(())
	}

	/// Gets addresses of accounts exposed for unknown dapps.
	/// `None` means that all accounts will be visible.
	pub fn new_dapps_addresses(&self) -> Result<Option<Vec<Address>>, Error> {
		Ok(match self.dapps_settings.read().policy() {
			NewDappsPolicy::AllAccounts { .. } => None,
			NewDappsPolicy::Whitelist(accounts) => Some(accounts),
		})
	}

	/// Sets a default account for unknown dapps.
	/// This account will always be returned as the first one.
	pub fn set_new_dapps_default_address(&self, address: Address) -> Result<(), Error> {
		if !self.valid_addresses()?.contains(&address) {
			return Err(SSError::InvalidAccount.into());
		}

		let mut settings = self.dapps_settings.write();
		let new_policy = match settings.policy() {
			NewDappsPolicy::AllAccounts { .. } => NewDappsPolicy::AllAccounts { default: address },
			NewDappsPolicy::Whitelist(list) => NewDappsPolicy::Whitelist(Self::insert_default(list, address)),
		};
		settings.set_policy(new_policy);

		Ok(())
	}

	/// Inserts given address as first in the vector, preventing duplicates.
	fn insert_default(mut addresses: Vec<Address>, default: Address) -> Vec<Address> {
		if let Some(position) = addresses.iter().position(|address| address == &default) {
			addresses.swap(0, position);
		} else {
			addresses.insert(0, default);
		}

		addresses
	}

	/// Returns a list of accounts that new dapp should see.
	/// First account is always the default account.
	fn new_dapps_addresses_list(&self) -> Result<Vec<Address>, Error> {
		match self.dapps_settings.read().policy() {
			NewDappsPolicy::AllAccounts { default } => if default.is_zero() {
				self.accounts()
			} else {
				Ok(Self::insert_default(self.accounts()?, default))
			},
			NewDappsPolicy::Whitelist(accounts) => {
				let addresses = self.filter_addresses(accounts)?;
				if addresses.is_empty() {
					Ok(vec![self.accounts()?.get(0).cloned().unwrap_or(0.into())])
				} else {
					Ok(addresses)
				}
			},
		}
	}

	/// Gets a default account for new dapps
	/// Will return zero address in case the default is not set and there are no accounts configured.
	pub fn new_dapps_default_address(&self) -> Result<Address, Error> {
		Ok(self.new_dapps_addresses_list()?
			.get(0)
			.cloned()
			.unwrap_or(0.into())
		)
	}

	/// Gets a list of dapps recently requesting accounts.
	pub fn recent_dapps(&self) -> Result<HashMap<DappId, u64>, Error> {
		Ok(self.dapps_settings.read().recent_dapps())
	}

	/// Marks dapp as recently used.
	pub fn note_dapp_used(&self, dapp: DappId) -> Result<(), Error> {
		let mut dapps = self.dapps_settings.write();
		dapps.mark_dapp_used(dapp.clone());
		Ok(())
	}

	/// Gets addresses visible for given dapp.
	pub fn dapp_addresses(&self, dapp: DappId) -> Result<Vec<Address>, Error> {
		let accounts = self.dapps_settings.read().settings().get(&dapp).map(|settings| {
			(settings.accounts.clone(), settings.default.clone())
		});

		match accounts {
			Some((Some(accounts), Some(default))) => self.filter_addresses(Self::insert_default(accounts, default)),
			Some((Some(accounts), None)) => self.filter_addresses(accounts),
			Some((None, Some(default))) => self.filter_addresses(Self::insert_default(self.new_dapps_addresses_list()?, default)),
			_ => self.new_dapps_addresses_list(),
		}
	}

	/// Returns default account for particular dapp falling back to other allowed accounts if necessary.
	pub fn dapp_default_address(&self, dapp: DappId) -> Result<Address, Error> {
		let dapp_default = self.dapp_addresses(dapp)?
			.get(0)
			.cloned();

		match dapp_default {
			Some(default) => Ok(default),
			None => self.new_dapps_default_address(),
		}
	}

	/// Sets default address for given dapp.
	/// Does not alter dapp addresses, but this account will always be returned as the first one.
	pub fn set_dapp_default_address(&self, dapp: DappId, address: Address) -> Result<(), Error> {
		if !self.valid_addresses()?.contains(&address) {
			return Err(SSError::InvalidAccount.into());
		}

		self.dapps_settings.write().set_default(dapp, address);
		Ok(())
	}

	/// Sets addresses visible for given dapp.
	/// If `None` - falls back to dapps addresses
	/// If not `None` and not empty it will also override default account.
	pub fn set_dapp_addresses(&self, dapp: DappId, addresses: Option<Vec<Address>>) -> Result<(), Error> {
		let (addresses, default) = match addresses {
			Some(addresses) => {
				let addresses = self.filter_addresses(addresses)?;
				let default = addresses.get(0).cloned();
				(Some(addresses), default)
			},
			None => (None, None),
		};

		let mut settings = self.dapps_settings.write();
		if let Some(default) = default {
			settings.set_default(dapp.clone(), default);
		}
		settings.set_accounts(dapp, addresses);
		Ok(())
	}

	fn valid_addresses(&self) -> Result<HashSet<Address>, Error> {
		Ok(self.addresses_info().into_iter()
			.map(|(address, _)| address)
			.chain(self.accounts()?)
			.collect())
	}

	/// Removes addresses that are neither accounts nor in address book.
	fn filter_addresses(&self, addresses: Vec<Address>) -> Result<Vec<Address>, Error> {
		let valid = self.valid_addresses()?;

		Ok(addresses.into_iter()
			.filter(|a| valid.contains(&a))
			.collect()
		)
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

	/// Removes and address from the addressbook
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

	/// Returns each hardware account along with name and meta.
	pub fn hardware_accounts_info(&self) -> Result<HashMap<Address, AccountMeta>, Error> {
		let r = self.hardware_accounts()?
			.into_iter()
			.map(|address| (address.clone(), self.account_meta(address).ok().unwrap_or_default()))
			.collect();
		Ok(r)
	}

	/// Returns each hardware account along with name and meta.
	pub fn is_hardware_address(&self, address: Address) -> bool {
		self.hardware_store.as_ref().and_then(|s| s.wallet_info(&address)).is_some()
	}

	/// Returns each account along with name and meta.
	pub fn account_meta(&self, address: Address) -> Result<AccountMeta, Error> {
		if let Some(info) = self.hardware_store.as_ref().and_then(|s| s.wallet_info(&address)) {
			Ok(AccountMeta {
				name: info.name,
				meta: info.manufacturer,
				uuid: None,
			})
		} else {
			let account = self.sstore.account_ref(&address)?;
			Ok(AccountMeta {
				name: self.sstore.name(&account)?,
				meta: self.sstore.meta(&account)?,
				uuid: self.sstore.uuid(&account).ok().map(Into::into),	// allowed to not have a Uuid
			})
		}
	}

	/// Returns account public key.
	pub fn account_public(&self, address: Address, password: &str) -> Result<Public, Error> {
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
	pub fn test_password(&self, address: &Address, password: &str) -> Result<bool, Error> {
		self.sstore.test_password(&self.sstore.account_ref(&address)?, password)
			.map_err(Into::into)
	}

	/// Permanently removes an account.
	pub fn kill_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		self.sstore.remove_account(&self.sstore.account_ref(&address)?, &password)?;
		Ok(())
	}

	/// Changes the password of `account` from `password` to `new_password`. Fails if incorrect `password` given.
	pub fn change_password(&self, address: &Address, password: String, new_password: String) -> Result<(), Error> {
		self.sstore.change_password(&self.sstore.account_ref(address)?, &password, &new_password)
	}

	/// Exports an account for given address.
	pub fn export_account(&self, address: &Address, password: String) -> Result<KeyFile, Error> {
		self.sstore.export_account(&self.sstore.account_ref(address)?, &password)
	}

	/// Helper method used for unlocking accounts.
	fn unlock_account(&self, address: Address, password: String, unlock: Unlock) -> Result<(), Error> {
		let account = self.sstore.account_ref(&address)?;

		// check if account is already unlocked pernamently, if it is, do nothing
		let mut unlocked = self.unlocked.write();
		if let Some(data) = unlocked.get(&account) {
			if let Unlock::Perm = data.unlock {
				return Ok(())
			}
		}

		if self.unlock_keep_secret && unlock != Unlock::OneTime {
			// verify password and get the secret
			let secret = self.sstore.raw_secret(&account, &password)?;
			self.unlocked_secrets.write().insert(account.clone(), secret);
		} else {
			// verify password by signing dump message
			// result may be discarded
			let _ = self.sstore.sign(&account, &password, &Default::default())?;
		}

		let data = AccountData {
			unlock: unlock,
			password: password,
		};

		unlocked.insert(account, data);
		Ok(())
	}

	fn password(&self, account: &StoreAccountRef) -> Result<String, SignError> {
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
		Ok(data.password.clone())
	}

	/// Unlocks account permanently.
	pub fn unlock_account_permanently(&self, account: Address, password: String) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Perm)
	}

	/// Unlocks account temporarily (for one signing).
	pub fn unlock_account_temporarily(&self, account: Address, password: String) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::OneTime)
	}

	/// Unlocks account temporarily with a timeout.
	pub fn unlock_account_timed(&self, account: Address, password: String, duration_ms: u32) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Timed(Instant::now() + Duration::from_millis(duration_ms as u64)))
	}

	/// Checks if given account is unlocked
	pub fn is_unlocked(&self, address: Address) -> bool {
		let unlocked = self.unlocked.read();
		let unlocked_secrets = self.unlocked_secrets.read();
		self.sstore.account_ref(&address)
			.map(|r| unlocked.get(&r).is_some() || unlocked_secrets.get(&r).is_some())
			.unwrap_or(false)
	}

	/// Signs the message. If password is not provided the account must be unlocked.
	pub fn sign(&self, address: Address, password: Option<String>, message: Message) -> Result<Signature, SignError> {
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
	pub fn sign_derived(&self, address: &Address, password: Option<String>, derivation: Derivation, message: Message)
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

		let new_token = random_string(16);
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

		let new_token = random_string(16);
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
	pub fn decrypt(&self, address: Address, password: Option<String>, shared_mac: &[u8], message: &[u8]) -> Result<Vec<u8>, SignError> {
		let account = self.sstore.account_ref(&address)?;
		let password = password.map(Ok).unwrap_or_else(|| self.password(&account))?;
		Ok(self.sstore.decrypt(&account, &password, shared_mac, message)?)
	}

	/// Agree on shared key.
	pub fn agree(&self, address: Address, password: Option<String>, other_public: &Public) -> Result<Secret, SignError> {
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
	pub fn create_vault(&self, name: &str, password: &str) -> Result<(), Error> {
		self.sstore.create_vault(name, password)
			.map_err(Into::into)
	}

	/// Open existing vault.
	pub fn open_vault(&self, name: &str, password: &str) -> Result<(), Error> {
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
	pub fn change_vault_password(&self, name: &str, new_password: &str) -> Result<(), Error> {
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

	/// Sign transaction with hardware wallet.
	pub fn sign_with_hardware(&self, address: Address, transaction: &[u8]) -> Result<Signature, SignError> {
		match self.hardware_store.as_ref().map(|s| s.sign_transaction(&address, transaction)) {
			None | Some(Err(HardwareError::KeyNotFound)) => Err(SignError::NotFound),
			Some(Err(e)) => Err(From::from(e)),
			Some(Ok(s)) => Ok(s),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{AccountProvider, Unlock, DappId};
	use std::time::Instant;
	use ethstore::ethkey::{Generator, Random, Address};
	use ethstore::{StoreAccountRef, Derivation};
	use util::H256;

	#[test]
	fn unlock_account_temp() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), "test").is_ok());
		assert!(ap.unlock_account_temporarily(kp.address(), "test1".into()).is_err());
		assert!(ap.unlock_account_temporarily(kp.address(), "test".into()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_err());
	}

	#[test]
	fn derived_account_nosave() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), "base").is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from(999)),
			false,
		).expect("Derivation should not fail");

		assert!(ap.unlock_account_permanently(derived_addr, "base".into()).is_err(),
			"There should be an error because account is not supposed to be saved");
	}

	#[test]
	fn derived_account_save() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), "base").is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from(999)),
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
		assert!(ap.insert_account(kp.secret().clone(), "base").is_ok());
		assert!(ap.unlock_account_permanently(kp.address(), "base".into()).is_ok());

		let derived_addr = ap.derive_account(
			&kp.address(),
			None,
			Derivation::SoftHash(H256::from(1999)),
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
			Derivation::SoftHash(H256::from(1999)),
			msg,
		).expect("Derived signing with existing unlocked account should not fail");

		assert_eq!(signed_msg1, signed_msg2,
			"Signed messages should match");
	}

	#[test]
	fn unlock_account_perm() {
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), "test").is_ok());
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
		assert!(ap.insert_account(kp.secret().clone(), "test").is_ok());
		assert!(ap.unlock_account_timed(kp.address(), "test1".into(), 60000).is_err());
		assert!(ap.unlock_account_timed(kp.address(), "test".into(), 60000).is_ok());
		assert!(ap.sign(kp.address(), None, Default::default()).is_ok());
		ap.unlocked.write().get_mut(&StoreAccountRef::root(kp.address())).unwrap().unlock = Unlock::Timed(Instant::now());
		assert!(ap.sign(kp.address(), None, Default::default()).is_err());
	}

	#[test]
	fn should_sign_and_return_token() {
		// given
		let kp = Random.generate().unwrap();
		let ap = AccountProvider::transient_provider();
		assert!(ap.insert_account(kp.secret().clone(), "test").is_ok());

		// when
		let (_signature, token) = ap.sign_with_token(kp.address(), "test".into(), Default::default()).unwrap();

		// then
		ap.sign_with_token(kp.address(), token.clone(), Default::default())
			.expect("First usage of token should be correct.");
		assert!(ap.sign_with_token(kp.address(), token, Default::default()).is_err(), "Second usage of the same token should fail.");
	}

	#[test]
	fn should_reset_dapp_addresses_to_default() {
		// given
		let ap = AccountProvider::transient_provider();
		let app = DappId("app1".into());
		// add accounts to address book
		ap.set_address_name(1.into(), "1".into());
		ap.set_address_name(2.into(), "2".into());
		// set `AllAccounts` policy
		ap.set_new_dapps_addresses(Some(vec![1.into(), 2.into()])).unwrap();
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![1.into(), 2.into()]);

		// Alter and check
		ap.set_dapp_addresses(app.clone(), Some(vec![1.into(), 3.into()])).unwrap();
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![1.into()]);

		// Reset back to default
		ap.set_dapp_addresses(app.clone(), None).unwrap();
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![1.into(), 2.into()]);
	}

	#[test]
	fn should_set_dapps_default_address() {
		// given
		let ap = AccountProvider::transient_provider();
		let app = DappId("app1".into());
		// set `AllAccounts` policy
		ap.set_new_dapps_addresses(None).unwrap();
		// add accounts to address book
		ap.set_address_name(1.into(), "1".into());
		ap.set_address_name(2.into(), "2".into());

		ap.set_dapp_addresses(app.clone(), Some(vec![1.into(), 2.into(), 3.into()])).unwrap();
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![1.into(), 2.into()]);
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), 1.into());

		// when setting empty list
		ap.set_dapp_addresses(app.clone(), Some(vec![])).unwrap();

		// then default account is intact
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![1.into()]);
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), 1.into());

		// alter default account
		ap.set_dapp_default_address("app1".into(), 2.into()).unwrap();
		assert_eq!(ap.dapp_addresses(app.clone()).unwrap(), vec![2.into()]);
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), 2.into());
	}

	#[test]
	fn should_set_dapps_policy_and_default_account() {
		// given
		let ap = AccountProvider::transient_provider();

		// default_account should be always available
		assert_eq!(ap.new_dapps_default_address().unwrap(), 0.into());

		let address = ap.new_account("test").unwrap();
		ap.set_address_name(1.into(), "1".into());

		// Default account set to first account by default
		assert_eq!(ap.new_dapps_default_address().unwrap(), address);
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), address);

		// Even when returning nothing
		ap.set_new_dapps_addresses(Some(vec![])).unwrap();
		// Default account is still returned
		assert_eq!(ap.dapp_addresses("app1".into()).unwrap(), vec![address]);

		// change to all
		ap.set_new_dapps_addresses(None).unwrap();
		assert_eq!(ap.dapp_addresses("app1".into()).unwrap(), vec![address]);

		// change to non-existent account
		ap.set_new_dapps_addresses(Some(vec![2.into()])).unwrap();
		assert_eq!(ap.dapp_addresses("app1".into()).unwrap(), vec![address]);

		// change to a addresses
		ap.set_new_dapps_addresses(Some(vec![1.into()])).unwrap();
		assert_eq!(ap.dapp_addresses("app1".into()).unwrap(), vec![1.into()]);

		// it overrides default account
		assert_eq!(ap.new_dapps_default_address().unwrap(), 1.into());
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), 1.into());

		ap.set_new_dapps_default_address(address).unwrap();
		assert_eq!(ap.new_dapps_default_address().unwrap(), address);
		assert_eq!(ap.dapp_default_address("app1".into()).unwrap(), address);
	}

	#[test]
	fn should_not_return_blacklisted_account() {
		// given
		let mut ap = AccountProvider::transient_provider();
		let acc = ap.new_account("test").unwrap();
		ap.blacklisted_accounts = vec![acc];

		// then
		assert_eq!(ap.accounts_info().unwrap().keys().cloned().collect::<Vec<Address>>(), vec![]);
		assert_eq!(ap.accounts().unwrap(), vec![]);
	}
}
