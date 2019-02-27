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

//! Address Book Store

use std::{fs, fmt, hash, ops};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ethkey::Address;
use log::{trace, warn};

use crate::AccountMeta;

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
			let x = self.cache.entry(a)
				.or_insert_with(|| AccountMeta {name: Default::default(), meta: "{}".to_owned(), uuid: None});
			x.name = name;
		}
		self.save();
	}

	/// Sets new meta for given address.
	pub fn set_meta(&mut self, a: Address, meta: String) {
		{
			let x = self.cache.entry(a)
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
	use super::AddressBook;
	use std::collections::HashMap;
	use tempdir::TempDir;
	use crate::account_data::AccountMeta;

	#[test]
	fn should_save_and_reload_address_book() {
		let tempdir = TempDir::new("").unwrap();
		let mut b = AddressBook::new(tempdir.path());
		b.set_name(1.into(), "One".to_owned());
		b.set_meta(1.into(), "{1:1}".to_owned());
		let b = AddressBook::new(tempdir.path());
		assert_eq!(b.get(), vec![
		   (1, AccountMeta {name: "One".to_owned(), meta: "{1:1}".to_owned(), uuid: None})
		].into_iter().map(|(a, b)| (a.into(), b)).collect::<HashMap<_, _>>());
	}

	#[test]
	fn should_remove_address() {
		let tempdir = TempDir::new("").unwrap();
		let mut b = AddressBook::new(tempdir.path());

		b.set_name(1.into(), "One".to_owned());
		b.set_name(2.into(), "Two".to_owned());
		b.set_name(3.into(), "Three".to_owned());
		b.remove(2.into());

		let b = AddressBook::new(tempdir.path());
		assert_eq!(b.get(), vec![
			(1, AccountMeta{name: "One".to_owned(), meta: "{}".to_owned(), uuid: None}),
			(3, AccountMeta{name: "Three".to_owned(), meta: "{}".to_owned(), uuid: None}),
		].into_iter().map(|(a, b)| (a.into(), b)).collect::<HashMap<_, _>>());
	}
}
