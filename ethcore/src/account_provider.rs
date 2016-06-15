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

//! Account management.

use std::fmt;
use std::sync::RwLock;
use std::collections::HashMap;
use util::{Address, H256, H520};
use ethstore::{SecretStore, Error as SSError};
use ethstore::ethkey::{Address as SSAddress, Message as SSMessage, Signature as SSSignature, Secret as SSSecret};

/// Helper trait, works the same as Into.
/// Used instead of into, cause we cannot implement into,
/// if both types are from foreign crates.
trait IntoSS<T> {
	fn into(self) -> T;
}

impl IntoSS<SSAddress> for Address {
	fn into(self) -> SSAddress {
		SSAddress::from(self.0)
	}
}

impl IntoSS<SSMessage> for H256 {
	fn into(self) -> SSMessage {
		SSMessage::from(self.0)
	}
}

impl IntoSS<SSSecret> for H256 {
	fn into(self) -> SSSecret {
		SSSecret::from(self.0)
	}
}

trait FromSS<T> {
	fn from(T) -> Self where Self: Sized;
}

impl FromSS<SSSignature> for H520 {
	fn from(s: SSSignature) -> Self {
		let bare: [u8; 65] = s.into();
		From::from(bare)
	}
}

impl FromSS<SSAddress> for Address {
	fn from(a: SSAddress) -> Self {
		let bare: [u8; 20] = a.into();
		From::from(bare)
	}
}

/// Type of unlock.
enum Unlock {
	/// If account is unlocked temporarily, it should be locked after first usage.
	Temp,
	/// Account unlocked permantently can always sign message.
	/// Use with caution.
	Perm,
}

/// Data associated with account.
struct AccountData {
	unlock: Unlock,
	password: String,
}

#[derive(Debug)]
pub enum Error {
	NotUnlocked,
	SStore(SSError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::NotUnlocked => write!(f, "Account is locked"),
			Error::SStore(ref e) => write!(f, "{}", e),
		}
	}
}

impl From<SSError> for Error {
	fn from(e: SSError) -> Self {
		Error::SStore(e)
	}
}

/// Account management.
/// Responsible for unlocking accounts.
pub struct AccountProvider {
	unlocked: RwLock<HashMap<Address, AccountData>>,
	sstore: Box<SecretStore>,
}

impl AccountProvider {
	/// Creates new account provider.
	pub fn new(sstore: Box<SecretStore>) -> Self {
		AccountProvider {
			unlocked: RwLock::new(HashMap::new()),
			sstore: sstore,
		}
	}

	/// Inserts new account into underlying store.
	/// Does not unlock account!
	pub fn insert_account(&self, secret: H256, password: &str) -> Result<(), Error> {
		let _ = try!(self.sstore.insert_account(IntoSS::into(secret), password));
		Ok(())
	}

	/// Returns addresses of all accounts.
	pub fn accounts(&self) -> Vec<Address> {
		self.sstore.accounts().into_iter().map(FromSS::from).collect()
	}

	/// Helper method used for unlocking accounts.
	fn unlock_account(&self, account: Address, password: String, unlock: Unlock) -> Result<(), Error> {
		// verify password by signing dump message
		// result may be discarded
		let _ = try!(self.sstore.sign(&IntoSS::into(account), &password, &Default::default()));

		let data = AccountData {
			unlock: unlock,
			password: password,
		};
		let mut unlocked = self.unlocked.write().unwrap();
		unlocked.insert(account, data);
		Ok(())
	}

	/// Unlocks account permanently.
	pub fn unlock_account_permanently(&self, account: Address, password: String) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Perm)
	}

	/// Unlocks account temporarily (for one signing).
	pub fn unlock_account_temporarily(&self, account: Address, password: String) -> Result<(), Error> {
		self.unlock_account(account, password, Unlock::Temp)
	}

	/// Signs the message. Account must be unlocked.
	pub fn sign(&self, account: &Address, message: &H256) -> Result<H520, Error> {
		let password = {
			let unlocked = self.unlocked.read().unwrap();
			let data = try!(unlocked.get(account).ok_or(Error::NotUnlocked));
			data.password.clone()
		};

		let signature = try!(self.sstore.sign(&IntoSS::into(*account), &password, &IntoSS::into(*message)));
		Ok(FromSS::from(signature))
	}
}

#[cfg(test)]
mod tests {
	use super::AccountProvider;
	use ethstore::{SecretStore, SafeAccount, Error, EthStore};
	use ethstore::dir::KeyDirectory;
	use ethstore::ethkey::{Address, Generator, Random, Secret};

	struct NullDir;

	impl KeyDirectory for NullDir {
		fn load(&self) -> Result<Vec<SafeAccount>, Error> {
			Ok(vec![])
		}

		fn insert(&self, account: SafeAccount) -> Result<(), Error> {
			Ok(())
		}

		fn remove(&self, address: &Address) -> Result<(), Error> {
			Ok(())
		}
	}

	fn account_provider() -> AccountProvider {
		AccountProvider::new(Box::new(EthStore::open(Box::new(NullDir)).unwrap()))
	}

	fn random_secret() -> Secret {
		Random.generate().unwrap().secret().clone()
	}

	#[test]
	fn unlock_account_temp() {
		let ap = account_provider();
	}
}
