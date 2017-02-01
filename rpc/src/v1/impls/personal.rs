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

use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;
use ethcore::miner::MinerService;
use util::{Address, U128, Uint};

use futures::{self, Future, BoxFuture};
use jsonrpc_core::Error;
use v1::helpers::errors;
use v1::helpers::dispatch::{self, sign_and_dispatch};
use v1::traits::Personal;
use v1::types::{H160 as RpcH160, H256 as RpcH256, U128 as RpcU128, TransactionRequest};
use v1::metadata::Metadata;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
	allow_perm_unlock: bool,
}

impl<C, M> PersonalClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>, miner: &Arc<M>, allow_perm_unlock: bool) -> Self {
		PersonalClient {
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

impl<C, M> Personal for PersonalClient<C, M> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static,
{
	type Metadata = Metadata;

	fn accounts(&self) -> Result<Vec<RpcH160>, Error> {
		self.active()?;

		let store = take_weak!(self.accounts);
		let accounts = store.accounts().map_err(|e| errors::account("Could not fetch accounts.", e))?;
		Ok(accounts.into_iter().map(Into::into).collect::<Vec<RpcH160>>())
	}

	fn new_account(&self, pass: String) -> Result<RpcH160, Error> {
		self.active()?;
		let store = take_weak!(self.accounts);

		store.new_account(&pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn unlock_account(&self, account: RpcH160, account_pass: String, duration: Option<RpcU128>) -> Result<bool, Error> {
		self.active()?;
		let account: Address = account.into();
		let store = take_weak!(self.accounts);
		let duration = match duration {
			None => None,
			Some(duration) => {
				let duration: U128 = duration.into();
				let v = duration.low_u64() as u32;
				if duration != v.into() {
					return Err(errors::invalid_params("Duration", "Invalid Number"));
				} else {
					Some(v)
				}
			},
		};

		let r = match (self.allow_perm_unlock, duration) {
			(false, _) => store.unlock_account_temporarily(account, account_pass),
			(true, Some(0)) => store.unlock_account_permanently(account, account_pass),
			(true, Some(d)) => store.unlock_account_timed(account, account_pass, d * 1000),
			(true, None) => store.unlock_account_timed(account, account_pass, 300_000),
		};
		match r {
			Ok(_) => Ok(true),
			// TODO [ToDr] Proper error here?
			Err(_) => Ok(false),
		}
	}

	fn send_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcH256, Error> {
		let sign_and_send = move || {
			self.active()?;
			let client = take_weak!(self.client);
			let miner = take_weak!(self.miner);
			let accounts = take_weak!(self.accounts);

			let default_account = match request.from {
				Some(ref account) => account.clone().into(),
				None => accounts
					.default_address(meta.dapp_id.unwrap_or_default().into())
					.map_err(|e| errors::account("Cannot find default account.", e))?,
			};

			let request = dispatch::fill_optional_fields(request.into(), default_account, &*client, &*miner);
			sign_and_dispatch(
				&*client,
				&*miner,
				&*accounts,
				request,
				dispatch::SignWith::Password(password)
			).map(|v| v.into_value().into())
		};

		futures::done(sign_and_send()).boxed()
	}
}
