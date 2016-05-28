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
use v1::traits::SignerPersonal;
use v1::types::TransactionRequest;
use v1::impls::sign_and_dispatch;
use v1::helpers::SigningQueue;
use util::keys::store::AccountProvider;
use util::numbers::*;
use ethcore::client::BlockChainClient;
use ethminer::MinerService;

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {
	queue: Weak<SigningQueue>,
	accounts: Weak<A>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<A: 'static, C: 'static, M: 'static> SignerClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {

	/// Create new instance of signer client.
	pub fn new(store: &Arc<A>, client: &Arc<C>, miner: &Arc<M>, queue: &Arc<SigningQueue>) -> Self {
		SignerClient {
			queue: Arc::downgrade(queue),
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}
}

impl<A: 'static, C: 'static, M: 'static> SignerPersonal for SignerClient<A, C, M>
	where A: AccountProvider, C: BlockChainClient, M: MinerService {

	fn transactions_to_confirm(&self, params: Params) -> Result<Value, Error> {
		let queue = take_weak!(self.queue);
		to_value(&queue.requests())
	}

	fn confirm_transaction(&self, params: Params) -> Result<Value, Error> {
		Err(Error::internal_error())
	}

	fn reject_transaction(&self, params: Params) -> Result<Value, Error> {
		Err(Error::internal_error())
	}
}

