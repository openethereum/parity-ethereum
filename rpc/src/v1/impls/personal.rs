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
use std::collections::{BTreeMap};
use jsonrpc_core::*;
use v1::traits::Personal;
use v1::types::{H160 as RpcH160};
use v1::helpers::errors;
use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<C> where C: MiningBlockChainClient {
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
}

impl<C> PersonalClient<C> where C: MiningBlockChainClient {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>) -> Self {
		PersonalClient {
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C: 'static> Personal for PersonalClient<C> where C: MiningBlockChainClient {

	fn accounts(&self) -> Result<Vec<RpcH160>, Error> {
		try!(self.active());

		let store = take_weak!(self.accounts);
		let accounts = try!(store.accounts().map_err(|e| errors::account("Could not fetch accounts.", e)));
		Ok(accounts.into_iter().map(Into::into).collect::<Vec<RpcH160>>())
	}

	fn accounts_info(&self) -> Result<BTreeMap<String, Value>, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);
		let info = try!(store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e)));
		let other = store.addresses_info().expect("addresses_info always returns Ok; qed");

		Ok(info.into_iter().chain(other.into_iter()).map(|(a, v)| {
			let m = map![
				"name".to_owned() => to_value(&v.name),
				"meta".to_owned() => to_value(&v.meta),
				"uuid".to_owned() => if let &Some(ref uuid) = &v.uuid {
					to_value(uuid)
				} else {
					Value::Null
				}
			];
			(format!("0x{}", a.hex()), Value::Object(m))
		}).collect())
	}
}
