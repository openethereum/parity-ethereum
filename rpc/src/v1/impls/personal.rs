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
use util::bytes::ToPretty;
use util::keys::store::*;
use util::numbers::*;
use util::rlp::encode;
use ethcore::client::BlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, Transaction as EthTransaction};
use ethminer::{AccountDetails, MinerService};

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

	fn dispatch_transaction(&self, signed_transaction: SignedTransaction) -> Result<Value, Error> {
		let hash = signed_transaction.hash();

		let import = {
			let client = take_weak!(self.client);
			let miner = take_weak!(self.miner);

			miner.import_own_transaction(&*client, signed_transaction, |a: &Address| {
				AccountDetails {
					nonce: client.nonce(&a),
					balance: client.balance(&a),
				}
			})
		};

		match import {
			Ok(_) => to_value(&hash),
			Err(e) => {
				warn!("Error sending transaction: {:?}", e);
				to_value(&H256::zero())
			}
		}
	}

	fn sign_and_dispatch(&self, request: TransactionRequest, secret: H256) -> Result<Value, Error> {
		let signed_transaction = {
			let client = take_weak!(self.client);
			let miner = take_weak!(self.miner);
			EthTransaction {
				nonce: request.nonce
					.or_else(|| miner
							 .last_nonce(&request.from)
							 .map(|nonce| nonce + U256::one()))
					.unwrap_or_else(|| client.nonce(&request.from)),
					action: request.to.map_or(Action::Create, Action::Call),
					gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
					gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
					value: request.value.unwrap_or_else(U256::zero),
					data: request.data.map_or_else(Vec::new, |b| b.to_vec()),
			}.sign(&secret)
		};
		trace!(target: "miner", "send_transaction: dispatching tx: {}", encode(&signed_transaction).to_vec().pretty());
		self.dispatch_transaction(signed_transaction)
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
					Ok(secret) => self.sign_and_dispatch(request, secret),
					Err(_) => to_value(&H256::zero()),
				}
		})
	}
}
