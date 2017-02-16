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

//! Account management (personal) rpc implementation
use std::sync::{Arc, Weak};
use std::collections::BTreeMap;
use util::{Address};

use ethkey::{Brain, Generator, Secret};
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use v1::helpers::errors;
use v1::traits::ParityAccounts;
use v1::types::{H160 as RpcH160, H256 as RpcH256, DappId, Derive, DeriveHierarchical, DeriveHash};

/// Account management (personal) rpc implementation.
pub struct ParityAccountsClient {
	accounts: Weak<AccountProvider>,
}

impl ParityAccountsClient {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>) -> Self {
		ParityAccountsClient {
			accounts: Arc::downgrade(store),
		}
	}
}

impl ParityAccounts for ParityAccountsClient {
	fn all_accounts_info(&self) -> Result<BTreeMap<RpcH160, BTreeMap<String, String>>, Error> {
		let store = take_weak!(self.accounts);
		let info = store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = store.addresses_info();

		Ok(info
		   .into_iter()
		   .chain(other.into_iter())
		   .map(|(address, v)| {
			   let mut m = map![
				   "name".to_owned() => v.name,
				   "meta".to_owned() => v.meta
			   ];
			   if let &Some(ref uuid) = &v.uuid {
				   m.insert("uuid".to_owned(), format!("{}", uuid));
			   }
			   (address.into(), m)
		   })
		   .collect()
		)
	}

	fn new_account_from_phrase(&self, phrase: String, pass: String) -> Result<RpcH160, Error> {
		let store = take_weak!(self.accounts);

		let brain = Brain::new(phrase).generate().unwrap();
		store.insert_account(brain.secret().clone(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_wallet(&self, json: String, pass: String) -> Result<RpcH160, Error> {
		let store = take_weak!(self.accounts);

		store.import_presale(json.as_bytes(), &pass)
			.or_else(|_| store.import_wallet(json.as_bytes(), &pass))
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_secret(&self, secret: RpcH256, pass: String) -> Result<RpcH160, Error> {
		let store = take_weak!(self.accounts);

		let secret = Secret::from_slice(&secret.0)
			.map_err(|e| errors::account("Could not create account.", e))?;
		store.insert_account(secret, &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn test_password(&self, account: RpcH160, password: String) -> Result<bool, Error> {
		let account: Address = account.into();

		take_weak!(self.accounts)
			.test_password(&account, &password)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn change_password(&self, account: RpcH160, password: String, new_password: String) -> Result<bool, Error> {
		let account: Address = account.into();
		take_weak!(self.accounts)
			.change_password(&account, password, new_password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn kill_account(&self, account: RpcH160, password: String) -> Result<bool, Error> {
		let account: Address = account.into();
		take_weak!(self.accounts)
			.kill_account(&account, &password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not delete account.", e))
	}

	fn remove_address(&self, addr: RpcH160) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.remove_address(addr);
		Ok(true)
	}

	fn set_account_name(&self, addr: RpcH160, name: String) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_name(addr.clone(), name.clone())
			.unwrap_or_else(|_| store.set_address_name(addr, name));
		Ok(true)
	}

	fn set_account_meta(&self, addr: RpcH160, meta: String) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_meta(addr.clone(), meta.clone())
			.unwrap_or_else(|_| store.set_address_meta(addr, meta));
		Ok(true)
	}

	fn set_dapp_addresses(&self, dapp: DappId, addresses: Option<Vec<RpcH160>>) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store.set_dapp_addresses(dapp.into(), addresses.map(into_vec))
			.map_err(|e| errors::account("Couldn't set dapp addresses.", e))
			.map(|_| true)
	}

	fn dapp_addresses(&self, dapp: DappId) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		store.dapp_addresses(dapp.into())
			.map_err(|e| errors::account("Couldn't get dapp addresses.", e))
			.map(into_vec)
	}

	fn set_dapp_default_address(&self, dapp: DappId, address: RpcH160) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store.set_dapp_default_address(dapp.into(), address.into())
			.map_err(|e| errors::account("Couldn't set dapp default address.", e))
			.map(|_| true)
	}

	fn dapp_default_address(&self, dapp: DappId) -> Result<RpcH160, Error> {
		let store = take_weak!(self.accounts);

		store.dapp_default_address(dapp.into())
			.map_err(|e| errors::account("Couldn't get dapp default address.", e))
			.map(Into::into)
	}

	fn set_new_dapps_addresses(&self, addresses: Option<Vec<RpcH160>>) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store
			.set_new_dapps_addresses(addresses.map(into_vec))
			.map_err(|e| errors::account("Couldn't set dapps addresses.", e))
			.map(|_| true)
	}

	fn new_dapps_addresses(&self) -> Result<Option<Vec<RpcH160>>, Error> {
		let store = take_weak!(self.accounts);

		store.new_dapps_addresses()
			.map_err(|e| errors::account("Couldn't get dapps addresses.", e))
			.map(|accounts| accounts.map(into_vec))
	}

	fn set_new_dapps_default_address(&self, address: RpcH160) -> Result<bool, Error> {
		let store = take_weak!(self.accounts);

		store.set_new_dapps_default_address(address.into())
			.map_err(|e| errors::account("Couldn't set new dapps default address.", e))
			.map(|_| true)
	}

	fn new_dapps_default_address(&self) -> Result<RpcH160, Error> {
		let store = take_weak!(self.accounts);

		store.new_dapps_default_address()
			.map_err(|e| errors::account("Couldn't get new dapps default address.", e))
			.map(Into::into)
	}

	fn recent_dapps(&self) -> Result<BTreeMap<DappId, u64>, Error> {
		let store = take_weak!(self.accounts);

		store.recent_dapps()
			.map_err(|e| errors::account("Couldn't get recent dapps.", e))
			.map(|map| map.into_iter().map(|(k, v)| (k.into(), v)).collect())
	}

	fn import_geth_accounts(&self, addresses: Vec<RpcH160>) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		store
			.import_geth_accounts(into_vec(addresses), false)
			.map(into_vec)
			.map_err(|e| errors::account("Couldn't import Geth accounts", e))
	}

	fn geth_accounts(&self) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		Ok(into_vec(store.list_geth_accounts(false)))
	}

	fn create_vault(&self, name: String, password: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.create_vault(&name, &password)
			.map_err(|e| errors::account("Could not create vault.", e))
			.map(|_| true)
	}

	fn open_vault(&self, name: String, password: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.open_vault(&name, &password)
			.map_err(|e| errors::account("Could not open vault.", e))
			.map(|_| true)
	}

	fn close_vault(&self, name: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.close_vault(&name)
			.map_err(|e| errors::account("Could not close vault.", e))
			.map(|_| true)
	}

	fn list_vaults(&self) -> Result<Vec<String>, Error> {
		take_weak!(self.accounts)
			.list_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn list_opened_vaults(&self) -> Result<Vec<String>, Error> {
		take_weak!(self.accounts)
			.list_opened_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn change_vault_password(&self, name: String, new_password: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.change_vault_password(&name, &new_password)
			.map_err(|e| errors::account("Could not change vault password.", e))
			.map(|_| true)
	}

	fn change_vault(&self, address: RpcH160, new_vault: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.change_vault(address.into(), &new_vault)
			.map_err(|e| errors::account("Could not change vault.", e))
			.map(|_| true)
	}

	fn get_vault_meta(&self, name: String) -> Result<String, Error> {
		take_weak!(self.accounts)
			.get_vault_meta(&name)
			.map_err(|e| errors::account("Could not get vault metadata.", e))
	}

	fn set_vault_meta(&self, name: String, meta: String) -> Result<bool, Error> {
		take_weak!(self.accounts)
			.set_vault_meta(&name, &meta)
			.map_err(|e| errors::account("Could not update vault metadata.", e))
			.map(|_| true)
	}

	fn derive_key_index(&self, addr: RpcH160, password: String, derivation: DeriveHierarchical, save_as_account: bool) -> Result<RpcH160, Error> {
		let addr: Address = addr.into();
		take_weak!(self.accounts)
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}

	fn derive_key_hash(&self, addr: RpcH160, password: String, derivation: DeriveHash, save_as_account: bool) -> Result<RpcH160, Error> {
		let addr: Address = addr.into();
		take_weak!(self.accounts)
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}
}

fn into_vec<A, B>(a: Vec<A>) -> Vec<B> where
	A: Into<B>
{
	a.into_iter().map(Into::into).collect()
}
