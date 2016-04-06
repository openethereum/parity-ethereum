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

//! Account management (personal) rpc implementation
use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use v1::traits::Personal;
use util::keys::store::*;
use util::Address;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<A> where A: AccountProvider {
	accounts: Weak<A>,
}

impl<A> PersonalClient<A> where A: AccountProvider {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<A>) -> Self {
		PersonalClient {
			accounts: Arc::downgrade(store),
		}
	}
}

impl<A> Personal for PersonalClient<A> where A: AccountProvider + 'static {
	fn accounts(&self, _: Params) -> Result<Value, Error> {
		let store = take_weak!(self.accounts);
		match store.accounts() {
			Ok(account_list) => to_value(&account_list),
			Err(_) => Err(Error::internal_error())
		}
	}

	fn new_account(&self, params: Params) -> Result<Value, Error> {
		from_params::<(String, )>(params).and_then(
			|(pass, )| {
				let store = take_weak!(self.accounts);
				match store.new_account(&pass) {
					Ok(address) => to_value(&address),
					Err(_) => Err(Error::internal_error())
				}
			}
		)
	}

	fn unlock_account(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, String, u64)>(params).and_then(
			|(account, account_pass, _)|{
				let store = take_weak!(self.accounts);
				match store.unlock_account(&account, &account_pass) {
					Ok(_) => Ok(Value::Bool(true)),
					Err(_) => Ok(Value::Bool(false)),
				}
			})
	}
}
