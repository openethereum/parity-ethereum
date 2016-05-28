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
use v1::types::TransactionRequest;
use v1::impls::sign_and_dispatch;
use util::keys::store::AccountProvider;
use util::numbers::*;
use ethcore::client::BlockChainClient;
use ethminer::MinerService;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {
	accounts: Weak<A>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<A, C, M> PersonalClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<A>, client: &Arc<C>, miner: &Arc<M>) -> Self {
		PersonalClient {
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}
}

impl<A: 'static, C: 'static, M: 'static> Personal for PersonalClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {
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
				match store.unlock_account_temp(&account, &account_pass) {
					Ok(_) => Ok(Value::Bool(true)),
					Err(_) => Ok(Value::Bool(false)),
				}
			})
	}

	fn sign_and_send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, String)>(params)
			.and_then(|(request, password)| {
				let accounts = take_weak!(self.accounts);
				match accounts.locked_account_secret(&request.from, &password) {
					Ok(secret) => sign_and_dispatch(&self.client, &self.miner, request, secret),
					Err(_) => to_value(&H256::zero()),
				}
		})
	}
}
