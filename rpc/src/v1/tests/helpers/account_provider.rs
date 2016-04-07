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

//! Test implementation of account provider.

use std::sync::RwLock;
use std::collections::HashMap;
use std::io;
use util::hash::{Address, H256, FixedHash};
use util::crypto::{Secret, Signature, KeyPair};
use util::keys::store::{AccountProvider, SigningError, EncryptedHashMapError};

/// Account mock.
#[derive(Clone)]
pub struct TestAccount {
	/// True if account is unlocked.
	pub unlocked: bool,
	/// Account's password.
	pub password: String,
	/// Account's secret.
	pub secret: Secret,
}

impl TestAccount {
	/// Creates new test account.
	pub fn new(password: &str) -> Self {
		let pair = KeyPair::create().unwrap();
		TestAccount {
			unlocked: false,
			password: password.to_owned(),
			secret: pair.secret().clone()
		}
	}

	/// Returns account address.
	pub fn address(&self) -> Address {
		KeyPair::from_secret(self.secret.clone()).unwrap().address()
	}
}

/// Test account provider.
pub struct TestAccountProvider {
	/// Test provider accounts.
	pub accounts: RwLock<HashMap<Address, TestAccount>>,
}

impl TestAccountProvider {
	/// Basic constructor.
	pub fn new(accounts: HashMap<Address, TestAccount>) -> Self {
		TestAccountProvider {
			accounts: RwLock::new(accounts),
		}
	}
}

impl AccountProvider for TestAccountProvider {
	fn accounts(&self) -> Result<Vec<Address>, io::Error> {
		Ok(self.accounts.read().unwrap().keys().cloned().collect())
	}

	fn unlock_account(&self, account: &Address, pass: &str) -> Result<(), EncryptedHashMapError> {
		match self.accounts.write().unwrap().get_mut(account) {
			Some(ref mut acc) if acc.password == pass => {
				acc.unlocked = true;
				Ok(())
			},
			Some(_) => Err(EncryptedHashMapError::InvalidPassword),
			None => Err(EncryptedHashMapError::UnknownIdentifier),
		}
	}

	fn new_account(&self, pass: &str) -> Result<Address, io::Error> {
		let account = TestAccount::new(pass);
		let address = KeyPair::from_secret(account.secret.clone()).unwrap().address();
		self.accounts.write().unwrap().insert(address.clone(), account);
		Ok(address)
	}

	fn account_secret(&self, address: &Address) -> Result<Secret, SigningError> {
		// todo: consider checking if account is unlock. some test may need alteration then.
		self.accounts
			.read()
			.unwrap()
			.get(address)
			.ok_or(SigningError::NoAccount)
			.map(|acc| acc.secret.clone())
	}

	fn sign(&self, _account: &Address, _message: &H256) -> Result<Signature, SigningError> {
		unimplemented!()
	}

}

