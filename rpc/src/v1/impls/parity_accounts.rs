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

//! Account management (personal) rpc implementation
use std::sync::{Arc, Weak};
use std::collections::BTreeMap;
use util::{Address};

use ethkey::{Brain, Generator, Secret};
use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;

use jsonrpc_core::Error;
use v1::traits::ParityAccounts;
use v1::types::{H160 as RpcH160, H256 as RpcH256, DappId};
use v1::helpers::errors;

/// Account management (personal) rpc implementation.
pub struct ParityAccountsClient<C> where C: MiningBlockChainClient {
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
}

impl<C> ParityAccountsClient<C> where C: MiningBlockChainClient {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>) -> Self {
		ParityAccountsClient {
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

impl<C: 'static> ParityAccounts for ParityAccountsClient<C> where C: MiningBlockChainClient {
	fn all_accounts_info(&self) -> Result<BTreeMap<String, BTreeMap<String, String>>, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);
		let info = store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = store.addresses_info().expect("addresses_info always returns Ok; qed");

		Ok(info.into_iter().chain(other.into_iter()).map(|(a, v)| {
			let mut m = map![
				"name".to_owned() => v.name,
				"meta".to_owned() => v.meta
			];
			if let &Some(ref uuid) = &v.uuid {
				m.insert("uuid".to_owned(), format!("{}", uuid));
			}
			(format!("0x{}", a.hex()), m)
		}).collect())
	}

	fn new_account_from_phrase(&self, phrase: String, pass: String) -> Result<RpcH160, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);

		let brain = Brain::new(phrase).generate().unwrap();
		store.insert_account(brain.secret().clone(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_wallet(&self, json: String, pass: String) -> Result<RpcH160, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);

		store.import_presale(json.as_bytes(), &pass)
			.or_else(|_| store.import_wallet(json.as_bytes(), &pass))
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_secret(&self, secret: RpcH256, pass: String) -> Result<RpcH160, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);

		let secret = Secret::from_slice(&secret.0)
			.map_err(|e| errors::account("Could not create account.", e))?;
		store.insert_account(secret, &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn test_password(&self, account: RpcH160, password: String) -> Result<bool, Error> {
		self.active()?;
		let account: Address = account.into();

		take_weak!(self.accounts)
			.test_password(&account, &password)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn change_password(&self, account: RpcH160, password: String, new_password: String) -> Result<bool, Error> {
		self.active()?;
		let account: Address = account.into();
		take_weak!(self.accounts)
			.change_password(&account, password, new_password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn kill_account(&self, account: RpcH160, password: String) -> Result<bool, Error> {
		self.active()?;
		let account: Address = account.into();
		take_weak!(self.accounts)
			.kill_account(&account, &password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not delete account.", e))
	}

	fn remove_address(&self, addr: RpcH160) -> Result<bool, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.remove_address(addr)
			.expect("remove_address always returns Ok; qed");
		Ok(true)
	}

	fn set_account_name(&self, addr: RpcH160, name: String) -> Result<bool, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_name(addr.clone(), name.clone())
			.or_else(|_| store.set_address_name(addr, name))
			.expect("set_address_name always returns Ok; qed");
		Ok(true)
	}

	fn set_account_meta(&self, addr: RpcH160, meta: String) -> Result<bool, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_meta(addr.clone(), meta.clone())
			.or_else(|_| store.set_address_meta(addr, meta))
			.expect("set_address_meta always returns Ok; qed");
		Ok(true)
	}

	fn set_account_visibility(&self, _address: RpcH160, _dapp: RpcH256, _visible: bool) -> Result<bool, Error> {
		Ok(false)
	}

	fn set_dapps_addresses(&self, dapp: DappId, addresses: Vec<RpcH160>) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store.set_dapps_addresses(dapp.into(), into_vec(addresses))
			.map_err(|e| errors::account("Couldn't set dapps addresses.", e))
			.map(|_| true)
	}

	fn dapps_addresses(&self, dapp: DappId) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		store.dapps_addresses(dapp.into())
			.map_err(|e| errors::account("Couldn't get dapps addresses.", e))
			.map(into_vec)
	}

	fn set_new_dapps_whitelist(&self, whitelist: Option<Vec<RpcH160>>) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store
			.set_new_dapps_whitelist(whitelist.map(into_vec))
			.map_err(|e| errors::account("Couldn't set dapps whitelist.", e))
			.map(|_| true)
	}

	fn new_dapps_whitelist(&self) -> Result<Option<Vec<RpcH160>>, Error> {
		let store = take_weak!(self.accounts);

		store.new_dapps_whitelist()
			.map_err(|e| errors::account("Couldn't get dapps whitelist.", e))
			.map(|accounts| accounts.map(into_vec))
	}

	fn recent_dapps(&self) -> Result<Vec<DappId>, Error> {
		let store = take_weak!(self.accounts);

		store.recent_dapps()
			.map_err(|e| errors::account("Couldn't get recent dapps.", e))
			.map(into_vec)
	}

	fn import_geth_accounts(&self, addresses: Vec<RpcH160>) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		store
			.import_geth_accounts(into_vec(addresses), false)
			.map(into_vec)
			.map_err(|e| errors::account("Couldn't import Geth accounts", e))
	}

	fn geth_accounts(&self) -> Result<Vec<RpcH160>, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);

		Ok(into_vec(store.list_geth_accounts(false)))
	}
}

fn into_vec<A, B>(a: Vec<A>) -> Vec<B> where
	A: Into<B>
{
	a.into_iter().map(Into::into).collect()
}
