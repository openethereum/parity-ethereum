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

//! Net rpc implementation.
use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use v1::traits::Personal;
use util::keys::store::*;
use util::{Bytes, Address};
use std::sync::RwLock;

/// Net rpc implementation.
pub struct PersonalClient {
	secret_store: Weak<SecretStore>,
	unlocked_account: Arc<RwLock<Option<Address>>>,
	unlocked_secret: Arc<RwLock<Option<Bytes>>>,
}

impl PersonalClient {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<SecretStore>) -> Self {
		PersonalClient {
			secret_store: Arc::downgrade(store),
			unlocked_account: Arc::new(RwLock::new(None)),
			unlocked_secret: Arc::new(RwLock::new(None)),
		}
	}
}

impl Personal for PersonalClient {
	fn accounts(&self, _: Params) -> Result<Value, Error> {
		let store = take_weak!(self.secret_store);
		match store.accounts() {
			Ok(account_list) => {
				Ok(Value::Array(account_list.iter()
					.map(|&(account, _)| Value::String(format!("{:?}", account)))
					.collect::<Vec<Value>>())
				)
			}
			Err(_) => Err(Error::internal_error())
		}
	}

	fn new_account(&self, _: Params) -> Result<Value, Error> {
		Err(Error::internal_error())
	}

	fn unlock_account(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, String, u64)>(params).and_then(
			|(account, account_pass, _)|{
				let store = take_weak!(self.secret_store);
				let secret_id = match store.account(&account)  {
					None => { return Ok(Value::Bool(false)); }
					Some(id) => id
				};
				match store.get(&secret_id, &account_pass) {
					Ok(secret) => {
						*self.unlocked_account.write().unwrap() = Some(account);
						*self.unlocked_secret.write().unwrap() = Some(secret);
						Ok(Value::Bool(true))
					},
					Err(_) => {
						Ok(Value::Bool(false))
					}
				}
			})
	}
}
