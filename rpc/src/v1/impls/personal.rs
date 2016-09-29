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
use util::{Address};
use jsonrpc_core::*;
use ethkey::{Brain, Generator};
use v1::traits::Personal;
use v1::types::{H160 as RpcH160, TransactionRequest};
use v1::helpers::errors;
use v1::helpers::params::expect_no_params;
use v1::helpers::dispatch::sign_and_dispatch;
use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;
use ethcore::miner::MinerService;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
	signer_port: Option<u16>,
	allow_perm_unlock: bool,
}

impl<C, M> PersonalClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>, miner: &Arc<M>, signer_port: Option<u16>, allow_perm_unlock: bool) -> Self {
		PersonalClient {
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			signer_port: signer_port,
			allow_perm_unlock: allow_perm_unlock,
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C: 'static, M: 'static> Personal for PersonalClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	fn signer_enabled(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		Ok(self.signer_port
			.map(|v| to_value(&v))
			.unwrap_or_else(|| to_value(&false)))
	}

	fn accounts(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		let store = take_weak!(self.accounts);
		let accounts = try!(store.accounts().map_err(|e| errors::internal("Could not fetch accounts.", e)));
		Ok(to_value(&accounts.into_iter().map(Into::into).collect::<Vec<RpcH160>>()))
	}

	fn new_account(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(String, )>(params).and_then(
			|(pass, )| {
				let store = take_weak!(self.accounts);
				match store.new_account(&pass) {
					Ok(address) => Ok(to_value(&RpcH160::from(address))),
					Err(e) => Err(errors::account("Could not create account.", e)),
				}
			}
		)
	}

	fn new_account_from_phrase(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(String, String, )>(params).and_then(
			|(phrase, pass, )| {
				let store = take_weak!(self.accounts);
				match store.insert_account(*Brain::new(phrase).generate().unwrap().secret(), &pass) {
					Ok(address) => Ok(to_value(&RpcH160::from(address))),
					Err(e) => Err(errors::account("Could not create account.", e)),
				}
			}
		)
	}

	fn new_account_from_wallet(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(String, String, )>(params).and_then(
			|(json, pass, )| {
				let store = take_weak!(self.accounts);
				match store.import_presale(json.as_bytes(), &pass).or_else(|_| store.import_wallet(json.as_bytes(), &pass)) {
					Ok(address) => Ok(to_value(&RpcH160::from(address))),
					Err(e) => Err(errors::account("Could not create account.", e)),
				}
			}
		)
	}

	fn unlock_account(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(RpcH160, String, Option<u64>)>(params).and_then(
			|(account, account_pass, duration)|{
				let account: Address = account.into();
				let store = take_weak!(self.accounts);
				let r = match (self.allow_perm_unlock, duration) {
					(false, _) => store.unlock_account_temporarily(account, account_pass),
					(true, Some(0)) => store.unlock_account_permanently(account, account_pass),
					(true, Some(d)) => store.unlock_account_timed(account, account_pass, d as u32 * 1000),
					(true, None) => store.unlock_account_timed(account, account_pass, 300_000),
				};
				match r {
					Ok(_) => Ok(Value::Bool(true)),
					Err(_) => Ok(Value::Bool(false)),
				}
			})
	}

	fn sign_and_send_transaction(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(TransactionRequest, String)>(params)
			.and_then(|(request, password)| {
				sign_and_dispatch(
					&*take_weak!(self.client),
					&*take_weak!(self.miner),
					&*take_weak!(self.accounts),
					request.into(),
					Some(password)
				)
			})
	}

	fn set_account_name(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);
		from_params::<(RpcH160, String)>(params).and_then(|(addr, name)| {
			let addr: Address = addr.into();
			store.set_account_name(addr.clone(), name.clone()).or_else(|_| store.set_address_name(addr, name)).expect("set_address_name always returns Ok; qed");
			Ok(Value::Null)
		})
	}

	fn set_account_meta(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		let store = take_weak!(self.accounts);
		from_params::<(RpcH160, String)>(params).and_then(|(addr, meta)| {
			let addr: Address = addr.into();
			store.set_account_meta(addr.clone(), meta.clone()).or_else(|_| store.set_address_meta(addr, meta)).expect("set_address_meta always returns Ok; qed");
			Ok(Value::Null)
		})
	}

	fn accounts_info(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let store = take_weak!(self.accounts);
		let info = try!(store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e)));
		let other = store.addresses_info().expect("addresses_info always returns Ok; qed");
		Ok(Value::Object(info.into_iter().chain(other.into_iter()).map(|(a, v)| {
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
		}).collect::<BTreeMap<_, _>>()))
	}

	fn geth_accounts(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let store = take_weak!(self.accounts);
		Ok(to_value(&store.list_geth_accounts(false).into_iter().map(Into::into).collect::<Vec<RpcH160>>()))
	}

	fn import_geth_accounts(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Vec<RpcH160>,)>(params).and_then(|(addresses,)| {
			let store = take_weak!(self.accounts);
			Ok(to_value(&try!(store
				.import_geth_accounts(addresses.into_iter().map(Into::into).collect(), false)
				.map_err(|e| errors::account("Couldn't import Geth accounts", e))
			).into_iter().map(Into::into).collect::<Vec<RpcH160>>()))
		})
	}
}
