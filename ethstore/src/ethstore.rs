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
use ethkey::KeyPair;
use crypto::KEY_ITERATIONS;
use random::Random;
use ethkey::{Signature, Address, Message, Secret};
use dir::KeyDirectory;
use account::SafeAccount;
use {Error, SecretStore};

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
		try!(self.dir.insert(account.clone()));

		// update cache
		let mut cache = self.cache.write().unwrap();
		cache.insert(account.address.clone(), account);
		Ok(())
	}
}

impl SecretStore for EthStore {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error> {
		let keypair = try!(KeyPair::from_secret(secret).map_err(|_| Error::CreationFailed));
		let id: [u8; 16] = Random::random();
		let account = SafeAccount::create(&keypair, id, password, self.iterations);
		let address = account.address.clone();
		try!(self.save(account));
		Ok(address)
	}

	fn accounts(&self) -> Vec<Address> {
		self.cache.read().unwrap().keys().cloned().collect()
	}

	fn change_password(&self, address: &Address, old_password: &str, new_password: &str) -> Result<(), Error> {
		// change password
		let account = {
			let cache = self.cache.read().unwrap();
			let account = try!(cache.get(address).ok_or(Error::InvalidAccount));
			try!(account.change_password(old_password, new_password, self.iterations))
		};

		// save to file
		self.save(account)
	}

	fn remove_account(&self, address: &Address, password: &str) -> Result<(), Error> {
		let can_remove = {
			let cache = self.cache.read().unwrap();
			let account = try!(cache.get(address).ok_or(Error::InvalidAccount));
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

	fn sign(&self, account: &Address, password: &str, message: &Message) -> Result<Signature, Error> {
		let cache = self.cache.read().unwrap();
		let account = try!(cache.get(account).ok_or(Error::InvalidAccount));
		account.sign(password, message)
	}
}
