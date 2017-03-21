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

//! Address Book and Dapps Settings Store

use std::{fs, fmt, hash, ops};
use std::sync::atomic::{self, AtomicUsize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ethstore::ethkey::Address;
use ethjson::misc::{
	AccountMeta,
	DappsSettings as JsonSettings,
	DappsHistory as JsonDappsHistory,
	NewDappsPolicy as JsonNewDappsPolicy,
};
use account_provider::DappId;

/// Disk-backed map from Address to String. Uses JSON.
pub struct AddressBook {
	cache: DiskMap<Address, AccountMeta>,
}

impl AddressBook {
	/// Creates new address book at given directory.
	pub fn new(path: &Path) -> Self {
		let mut r = AddressBook {
			cache: DiskMap::new(path, "address_book.json")
		};
		r.cache.revert(AccountMeta::read);
		r
	}

	/// Creates transient address book (no changes are saved to disk).
	pub fn transient() -> Self {
		AddressBook {
			cache: DiskMap::transient()
		}
	}

	/// Get the address book.
	pub fn get(&self) -> HashMap<Address, AccountMeta> {
		self.cache.clone()
	}

	fn save(&self) {
		self.cache.save(AccountMeta::write)
	}

	/// Sets new name for given address.
	pub fn set_name(&mut self, a: Address, name: String) {
		{
			let mut x = self.cache.entry(a)
				.or_insert_with(|| AccountMeta {name: Default::default(), meta: "{}".to_owned(), uuid: None});
			x.name = name;
		}
		self.save();
	}

	/// Sets new meta for given address.
	pub fn set_meta(&mut self, a: Address, meta: String) {
		{
			let mut x = self.cache.entry(a)
				.or_insert_with(|| AccountMeta {name: "Anonymous".to_owned(), meta: Default::default(), uuid: None});
			x.meta = meta;
		}
		self.save();
	}

	/// Removes an entry
	pub fn remove(&mut self, a: Address) {
		self.cache.remove(&a);
		self.save();
	}
}

/// Dapps user settings
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct DappsSettings {
	/// A list of visible accounts
	pub accounts: Option<Vec<Address>>,
	/// Default account
	pub default: Option<Address>,
}

impl From<JsonSettings> for DappsSettings {
	fn from(s: JsonSettings) -> Self {
		DappsSettings {
			accounts: s.accounts.map(|accounts| accounts.into_iter().map(Into::into).collect()),
			default: s.default.map(Into::into),
		}
	}
}

impl From<DappsSettings> for JsonSettings {
	fn from(s: DappsSettings) -> Self {
		JsonSettings {
			accounts: s.accounts.map(|accounts| accounts.into_iter().map(Into::into).collect()),
			default: s.default.map(Into::into),
		}
	}
}

/// Dapps user settings
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NewDappsPolicy {
	AllAccounts {
		default: Address,
	},
	Whitelist(Vec<Address>),
}

impl From<JsonNewDappsPolicy> for NewDappsPolicy {
	fn from(s: JsonNewDappsPolicy) -> Self {
		match s {
			JsonNewDappsPolicy::AllAccounts { default } => NewDappsPolicy::AllAccounts {
				default: default.into(),
			},
			JsonNewDappsPolicy::Whitelist(accounts) => NewDappsPolicy::Whitelist(
				accounts.into_iter().map(Into::into).collect()
			),
		}
	}
}

impl From<NewDappsPolicy> for JsonNewDappsPolicy {
	fn from(s: NewDappsPolicy) -> Self {
		match s {
			NewDappsPolicy::AllAccounts { default } => JsonNewDappsPolicy::AllAccounts {
				default: default.into(),
			},
			NewDappsPolicy::Whitelist(accounts) => JsonNewDappsPolicy::Whitelist(
				accounts.into_iter().map(Into::into).collect()
			),
		}
	}
}

/// Transient dapps data
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct TransientDappsData {
	/// Timestamp of last access
	pub last_accessed: u64,
}

impl From<JsonDappsHistory> for TransientDappsData {
	fn from(s: JsonDappsHistory) -> Self {
		TransientDappsData {
			last_accessed: s.last_accessed,
		}
	}
}

impl From<TransientDappsData> for JsonDappsHistory {
	fn from(s: TransientDappsData) -> Self {
		JsonDappsHistory {
			last_accessed: s.last_accessed,
		}
	}
}

enum TimeProvider {
	Clock,
	Incremenetal(AtomicUsize)
}

impl TimeProvider {
	fn get(&self) -> u64 {
		match *self {
			TimeProvider::Clock => {
				::std::time::UNIX_EPOCH.elapsed()
					.expect("Correct time is required to be set")
					.as_secs()

			},
			TimeProvider::Incremenetal(ref time) => {
				time.fetch_add(1, atomic::Ordering::SeqCst) as u64
			},
		}
	}
}

const MAX_RECENT_DAPPS: usize = 50;

/// Disk-backed map from DappId to Settings. Uses JSON.
pub struct DappsSettingsStore {
	/// Dapps Settings
	settings: DiskMap<DappId, DappsSettings>,
	/// New Dapps Policy
	policy: DiskMap<String, NewDappsPolicy>,
	/// Transient Data of recently Accessed Dapps
	history: DiskMap<DappId, TransientDappsData>,
	/// Time
	time: TimeProvider,
}

impl DappsSettingsStore {
	/// Creates new store at given directory path.
	pub fn new(path: &Path) -> Self {
		let mut r = DappsSettingsStore {
			settings: DiskMap::new(path, "dapps_accounts.json".into()),
			policy: DiskMap::new(path, "dapps_policy.json".into()),
			history: DiskMap::new(path, "dapps_history.json".into()),
			time: TimeProvider::Clock,
		};
		r.settings.revert(JsonSettings::read);
		r.policy.revert(JsonNewDappsPolicy::read);
		r.history.revert(JsonDappsHistory::read);
		r
	}

	/// Creates transient store (no changes are saved to disk).
	pub fn transient() -> Self {
		DappsSettingsStore {
			settings: DiskMap::transient(),
			policy: DiskMap::transient(),
			history: DiskMap::transient(),
			time: TimeProvider::Incremenetal(AtomicUsize::new(1)),
		}
	}

	/// Get copy of the dapps settings
	pub fn settings(&self) -> HashMap<DappId, DappsSettings> {
		self.settings.clone()
	}

	/// Returns current new dapps policy
	pub fn policy(&self) -> NewDappsPolicy {
		self.policy.get("default").cloned().unwrap_or(NewDappsPolicy::AllAccounts {
			default: 0.into(),
		})
	}

	/// Returns recent dapps with last accessed timestamp
	pub fn recent_dapps(&self) -> HashMap<DappId, u64> {
		self.history.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect()
	}

	/// Marks recent dapp as used
	pub fn mark_dapp_used(&mut self, dapp: DappId) {
		{
			let mut entry = self.history.entry(dapp).or_insert_with(|| Default::default());
			entry.last_accessed = self.time.get();
		}
		// Clear extraneous entries
		while self.history.len() > MAX_RECENT_DAPPS {
			let min = self.history.iter()
				.min_by_key(|&(_, ref v)| v.last_accessed)
				.map(|(ref k, _)| k.clone())
				.cloned();

			match min {
				Some(k) => self.history.remove(&k),
				None => break,
			};
		}
		self.history.save(JsonDappsHistory::write);
	}

	/// Sets current new dapps policy
	pub fn set_policy(&mut self, policy: NewDappsPolicy) {
		self.policy.insert("default".into(), policy);
		self.policy.save(JsonNewDappsPolicy::write);
	}

	/// Sets accounts for specific dapp.
	pub fn set_accounts(&mut self, id: DappId, accounts: Option<Vec<Address>>) {
		{
			let mut settings = self.settings.entry(id).or_insert_with(DappsSettings::default);
			settings.accounts = accounts;
		}
		self.settings.save(JsonSettings::write);
	}

	/// Sets a default account for specific dapp.
	pub fn set_default(&mut self, id: DappId, default: Address) {
		{
			let mut settings = self.settings.entry(id).or_insert_with(DappsSettings::default);
			settings.default = Some(default);
		}
		self.settings.save(JsonSettings::write);
	}
}

/// Disk-serializable HashMap
#[derive(Debug)]
struct DiskMap<K: hash::Hash + Eq, V> {
	path: PathBuf,
	cache: HashMap<K, V>,
	transient: bool,
}

impl<K: hash::Hash + Eq, V> ops::Deref for DiskMap<K, V> {
	type Target = HashMap<K, V>;
	fn deref(&self) -> &Self::Target {
		&self.cache
	}
}

impl<K: hash::Hash + Eq, V> ops::DerefMut for DiskMap<K, V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.cache
	}
}

impl<K: hash::Hash + Eq, V> DiskMap<K, V> {
	pub fn new(path: &Path, file_name: &str) -> Self {
		let mut path = path.to_owned();
		path.push(file_name);
		trace!(target: "diskmap", "path={:?}", path);
		DiskMap {
			path: path,
			cache: HashMap::new(),
			transient: false,
		}
	}

	pub fn transient() -> Self {
		let mut map = DiskMap::new(&PathBuf::new(), "diskmap.json".into());
		map.transient = true;
		map
	}

	fn revert<F, E>(&mut self, read: F) where
		F: Fn(fs::File) -> Result<HashMap<K, V>, E>,
		E: fmt::Display,
	{
		if self.transient { return; }
		trace!(target: "diskmap", "revert {:?}", self.path);
		let _ = fs::File::open(self.path.clone())
			.map_err(|e| trace!(target: "diskmap", "Couldn't open disk map: {}", e))
			.and_then(|f| read(f).map_err(|e| warn!(target: "diskmap", "Couldn't read disk map: {}", e)))
			.and_then(|m| {
				self.cache = m;
				Ok(())
			});
	}

	fn save<F, E>(&self, write: F) where
		F: Fn(&HashMap<K, V>, &mut fs::File) -> Result<(), E>,
		E: fmt::Display,
	{
		if self.transient { return; }
		trace!(target: "diskmap", "save {:?}", self.path);
		let _ = fs::File::create(self.path.clone())
			.map_err(|e| warn!(target: "diskmap", "Couldn't open disk map for writing: {}", e))
			.and_then(|mut f| {
				write(&self.cache, &mut f).map_err(|e| warn!(target: "diskmap", "Couldn't write to disk map: {}", e))
			});
	}
}

#[cfg(test)]
mod tests {
	use super::{AddressBook, DappsSettingsStore, DappsSettings, NewDappsPolicy};
	use account_provider::DappId;
	use std::collections::HashMap;
	use ethjson::misc::AccountMeta;
	use devtools::RandomTempPath;

	#[test]
	fn should_save_and_reload_address_book() {
		let path = RandomTempPath::create_dir();
		let mut b = AddressBook::new(&path);
		b.set_name(1.into(), "One".to_owned());
		b.set_meta(1.into(), "{1:1}".to_owned());
		let b = AddressBook::new(&path);
		assert_eq!(b.get(), hash_map![1.into() => AccountMeta{name: "One".to_owned(), meta: "{1:1}".to_owned(), uuid: None}]);
	}

	#[test]
	fn should_remove_address() {
		let path = RandomTempPath::create_dir();
		let mut b = AddressBook::new(&path);

		b.set_name(1.into(), "One".to_owned());
		b.set_name(2.into(), "Two".to_owned());
		b.set_name(3.into(), "Three".to_owned());
		b.remove(2.into());

		let b = AddressBook::new(&path);
		assert_eq!(b.get(), hash_map![
			1.into() => AccountMeta{name: "One".to_owned(), meta: "{}".to_owned(), uuid: None},
			3.into() => AccountMeta{name: "Three".to_owned(), meta: "{}".to_owned(), uuid: None}
		]);
	}

	#[test]
	fn should_save_and_reload_dapps_settings() {
		// given
		let path = RandomTempPath::create_dir();
		let mut b = DappsSettingsStore::new(&path);

		// when
		b.set_accounts("dappOne".into(), Some(vec![1.into(), 2.into()]));

		// then
		let b = DappsSettingsStore::new(&path);
		assert_eq!(b.settings(), hash_map![
			"dappOne".into() => DappsSettings {
				accounts: Some(vec![1.into(), 2.into()]),
				default: None,
			}
		]);
	}

	#[test]
	fn should_maintain_a_map_of_recent_dapps() {
		let mut store = DappsSettingsStore::transient();
		assert!(store.recent_dapps().is_empty(), "Initially recent dapps should be empty.");

		let dapp1: DappId = "dapp1".into();
		let dapp2: DappId = "dapp2".into();
		store.mark_dapp_used(dapp1.clone());
		let recent = store.recent_dapps();
		assert_eq!(recent.len(), 1);
		assert_eq!(recent.get(&dapp1), Some(&1));

		store.mark_dapp_used(dapp2.clone());
		let recent = store.recent_dapps();
		assert_eq!(recent.len(), 2);
		assert_eq!(recent.get(&dapp1), Some(&1));
		assert_eq!(recent.get(&dapp2), Some(&2));
	}

	#[test]
	fn should_store_dapps_policy() {
		// given
		let path = RandomTempPath::create_dir();
		let mut store = DappsSettingsStore::new(&path);

		// Test default policy
		assert_eq!(store.policy(), NewDappsPolicy::AllAccounts {
			default: 0.into(),
		});

		// when
		store.set_policy(NewDappsPolicy::Whitelist(vec![1.into(), 2.into()]));

		// then
		let store = DappsSettingsStore::new(&path);
		assert_eq!(store.policy.clone(), hash_map![
			"default".into() => NewDappsPolicy::Whitelist(vec![1.into(), 2.into()])
		]);
	}
}
