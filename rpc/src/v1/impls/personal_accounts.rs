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
use util::{Address};
use jsonrpc_core::*;
use ethkey::{Brain, Generator};
use v1::traits::PersonalAccounts;
use v1::types::{H160 as RpcH160, H256 as RpcH256, TransactionRequest};
use v1::helpers::errors;
use v1::helpers::dispatch::sign_and_dispatch;
use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;
use ethcore::miner::MinerService;

/// Account management (personal) rpc implementation.
pub struct PersonalAccountsClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
	allow_perm_unlock: bool,
}

impl<C, M> PersonalAccountsClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>, miner: &Arc<M>, allow_perm_unlock: bool) -> Self {
		PersonalAccountsClient {
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			allow_perm_unlock: allow_perm_unlock,
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C: 'static, M: 'static> PersonalAccounts for PersonalAccountsClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	fn new_account(&self, pass: String) -> Result<RpcH160, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);

		store.new_account(&pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_phrase(&self, phrase: String, pass: String) -> Result<RpcH160, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);

		store.insert_account(*Brain::new(phrase).generate().unwrap().secret(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_wallet(&self, json: String, pass: String) -> Result<RpcH160, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);

		store.import_presale(json.as_bytes(), &pass)
			.or_else(|_| store.import_wallet(json.as_bytes(), &pass))
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_secret(&self, secret: RpcH256, pass: String) -> Result<RpcH160, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);

		store.insert_account(secret.into(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn unlock_account(&self, account: RpcH160, account_pass: String, duration: Option<u64>) -> Result<bool, Error> {
		try!(self.active());
		let account: Address = account.into();
		let store = take_weak!(self.accounts);

		let r = match (self.allow_perm_unlock, duration) {
			(false, _) => store.unlock_account_temporarily(account, account_pass),
			(true, Some(0)) => store.unlock_account_permanently(account, account_pass),
			(true, Some(d)) => store.unlock_account_timed(account, account_pass, d as u32 * 1000),
			(true, None) => store.unlock_account_timed(account, account_pass, 300_000),
		};
		match r {
			Ok(_) => Ok(true),
			// TODO [ToDr] Proper error here?
			Err(_) => Ok(false),
		}
	}

	fn test_password(&self, account: RpcH160, password: String) -> Result<bool, Error> {
		try!(self.active());
		let account: Address = account.into();

		take_weak!(self.accounts)
			.test_password(&account, password)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn change_password(&self, account: RpcH160, password: String, new_password: String) -> Result<bool, Error> {
		try!(self.active());
		let account: Address = account.into();
		take_weak!(self.accounts)
			.change_password(&account, password, new_password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn sign_and_send_transaction(&self, request: TransactionRequest, password: String) -> Result<RpcH256, Error> {
		try!(self.active());

		sign_and_dispatch(
			&*take_weak!(self.client),
			&*take_weak!(self.miner),
			&*take_weak!(self.accounts),
			request.into(),
			Some(password)
		)
	}

	fn set_account_name(&self, addr: RpcH160, name: String) -> Result<bool, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_name(addr.clone(), name.clone())
			.or_else(|_| store.set_address_name(addr, name))
			.expect("set_address_name always returns Ok; qed");
		Ok(true)
	}

	fn set_account_meta(&self, addr: RpcH160, meta: String) -> Result<bool, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);
		let addr: Address = addr.into();

		store.set_account_meta(addr.clone(), meta.clone())
			.or_else(|_| store.set_address_meta(addr, meta))
			.expect("set_address_meta always returns Ok; qed");
		Ok(true)
	}

	fn import_geth_accounts(&self, addresses: Vec<RpcH160>) -> Result<Vec<RpcH160>, Error> {
		let store = take_weak!(self.accounts);

		store
			.import_geth_accounts(addresses.into_iter().map(Into::into).collect(), false)
			.map(|imported| imported.into_iter().map(Into::into).collect())
			.map_err(|e| errors::account("Couldn't import Geth accounts", e))
	}

	fn geth_accounts(&self) -> Result<Vec<RpcH160>, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);

		Ok(store.list_geth_accounts(false)
			.into_iter()
			.map(Into::into)
			.collect()
		)
	}
}
