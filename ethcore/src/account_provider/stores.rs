// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
use std::collections::HashMap;
use std::path::PathBuf;

use ethstore::ethkey::Address;
use ethjson::misc::{AccountMeta, DappsSettings as JsonSettings};
use account_provider::DappId;

/// Disk-backed map from Address to String. Uses JSON.
pub struct AddressBook {
	cache: DiskMap<Address, AccountMeta>,
}

impl AddressBook {
	/// Creates new address book at given directory.
	pub fn new(path: String) -> Self {
		let mut r = AddressBook {
			cache: DiskMap::new(path, "address_book.json".into())
		};
		r.cache.revert(AccountMeta::read_address_map);
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
		self.cache.save(AccountMeta::write_address_map)
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
	pub accounts: Vec<Address>,
}

impl From<JsonSettings> for DappsSettings {
	fn from(s: JsonSettings) -> Self {
		DappsSettings {
			accounts: s.accounts.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<DappsSettings> for JsonSettings {
	fn from(s: DappsSettings) -> Self {
		JsonSettings {
			accounts: s.accounts.into_iter().map(Into::into).collect(),
		}
	}
}

/// Disk-backed map from DappId to Settings. Uses JSON.
pub struct DappsSettingsStore {
	cache: DiskMap<DappId, DappsSettings>,
}

impl DappsSettingsStore {
	/// Creates new store at given directory path.
	pub fn new(path: String) -> Self {
		let mut r = DappsSettingsStore {
			cache: DiskMap::new(path, "dapps_accounts.json".into())
		};
		r.cache.revert(JsonSettings::read_dapps_settings);
		r
	}

	/// Creates transient store (no changes are saved to disk).
	pub fn transient() -> Self {
		DappsSettingsStore {
			cache: DiskMap::transient()
		}
	}

	/// Get copy of the dapps settings
	pub fn get(&self) -> HashMap<DappId, DappsSettings> {
		self.cache.clone()
	}

	fn save(&self) {
		self.cache.save(JsonSettings::write_dapps_settings)
	}

	pub fn set_accounts(&mut self, id: DappId, accounts: Vec<Address>) {
		{
			let mut settings = self.cache.entry(id).or_insert_with(DappsSettings::default);
			settings.accounts = accounts;
		}
		self.save();
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
	pub fn new(path: String, file_name: String) -> Self {
		trace!(target: "diskmap", "new({})", path);
		let mut path: PathBuf = path.into();
		path.push(file_name);
		trace!(target: "diskmap", "path={:?}", path);
		DiskMap {
			path: path,
			cache: HashMap::new(),
			transient: false,
		}
	}

	pub fn transient() -> Self {
		let mut map = DiskMap::new(Default::default(), "diskmap.json".into());
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
	use super::{AddressBook, DappsSettingsStore, DappsSettings};
	use std::collections::HashMap;
	use ethjson::misc::AccountMeta;
	use devtools::RandomTempPath;

	#[test]
	fn should_save_and_reload_address_book() {
		let temp = RandomTempPath::create_dir();
		let path = temp.as_str().to_owned();
		let mut b = AddressBook::new(path.clone());
		b.set_name(1.into(), "One".to_owned());
		b.set_meta(1.into(), "{1:1}".to_owned());
		let b = AddressBook::new(path);
		assert_eq!(b.get(), hash_map![1.into() => AccountMeta{name: "One".to_owned(), meta: "{1:1}".to_owned(), uuid: None}]);
	}

	#[test]
	fn should_save_and_reload_dapps_settings() {
		// given
		let temp = RandomTempPath::create_dir();
		let path = temp.as_str().to_owned();
		let mut b = DappsSettingsStore::new(path.clone());

		// when
		b.set_accounts("dappOne".into(), vec![1.into(), 2.into()]);

		// then
		let b = DappsSettingsStore::new(path);
		assert_eq!(b.get(), hash_map![
			"dappOne".into() => DappsSettings {
				accounts: vec![1.into(), 2.into()],
			}
		]);
	}

	#[test]
	fn should_remove_address() {
		let temp = RandomTempPath::create_dir();
		let path = temp.as_str().to_owned();
		let mut b = AddressBook::new(path.clone());

		b.set_name(1.into(), "One".to_owned());
		b.set_name(2.into(), "Two".to_owned());
		b.set_name(3.into(), "Three".to_owned());
		b.remove(2.into());

		let b = AddressBook::new(path);
		assert_eq!(b.get(), hash_map![
			1.into() => AccountMeta{name: "One".to_owned(), meta: "{}".to_owned(), uuid: None},
			3.into() => AccountMeta{name: "Three".to_owned(), meta: "{}".to_owned(), uuid: None}
		]);
	}
}
