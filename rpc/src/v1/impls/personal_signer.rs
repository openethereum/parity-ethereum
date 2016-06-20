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

//! Transactions Confirmations (personal) rpc implementation

use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use v1::traits::PersonalSigner;
use v1::types::TransactionModification;
use v1::impls::unlock_sign_and_dispatch;
use v1::helpers::{SigningQueue, ConfirmationsQueue};
use ethcore::account_provider::AccountProvider;
use util::numbers::*;
use ethcore::client::MiningBlockChainClient;
use ethcore::miner::MinerService;

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	queue: Weak<ConfirmationsQueue>,
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C: 'static, M: 'static> SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	/// Create new instance of signer client.
	pub fn new(store: &Arc<AccountProvider>, client: &Arc<C>, miner: &Arc<M>, queue: &Arc<ConfirmationsQueue>) -> Self {
		SignerClient {
			queue: Arc::downgrade(queue),
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}
}

impl<C: 'static, M: 'static> PersonalSigner for SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	fn transactions_to_confirm(&self, _params: Params) -> Result<Value, Error> {
		let queue = take_weak!(self.queue);
		to_value(&queue.requests())
	}

	fn confirm_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(U256, TransactionModification, String)>(params).and_then(
			|(id, modification, pass)| {
				let accounts = take_weak!(self.accounts);
				let queue = take_weak!(self.queue);
				let client = take_weak!(self.client);
				let miner = take_weak!(self.miner);
				queue.peek(&id).and_then(|confirmation| {
						let mut request = confirmation.transaction;
						// apply modification
						if let Some(gas_price) = modification.gas_price {
							request.gas_price = Some(gas_price);
						}

						let sender = request.from;

						match unlock_sign_and_dispatch(&*client, &*miner, request, &*accounts, sender, pass) {
							Ok(hash) => {
								queue.request_confirmed(id, Ok(hash.clone()));
								Some(to_value(&hash))
							},
							_ => None
						}
					})
					.unwrap_or_else(|| {
						to_value(&false)
					})
			}
		)
	}

	fn reject_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(U256, )>(params).and_then(
			|(id, )| {
				let queue = take_weak!(self.queue);
				let res = queue.request_rejected(id);
				to_value(&res.is_some())
			}
		)
	}
}

