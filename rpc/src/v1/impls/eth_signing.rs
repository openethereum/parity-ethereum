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

//! Eth Signing RPC implementation.

use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use ethminer::MinerService;
use ethcore::client::BlockChainClient;
use util::numbers::*;
use util::keys::store::AccountProvider;
use v1::helpers::{SigningQueue, ConfirmationsQueue};
use v1::traits::EthSigning;
use v1::types::TransactionRequest;
use v1::impls::sign_and_dispatch;


/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningQueueClient {
	queue: Weak<ConfirmationsQueue>,
}

impl EthSigningQueueClient {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(queue: &Arc<ConfirmationsQueue>) -> Self {
		EthSigningQueueClient {
			queue: Arc::downgrade(queue),
		}
	}
}

impl EthSigning for EthSigningQueueClient  {

	fn sign(&self, _params: Params) -> Result<Value, Error> {
		// TODO [ToDr] Implement sign when rest of the signing queue is ready.
		rpc_unimplemented!()
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let queue = take_weak!(self.queue);
				queue.add_request(request);
				// TODO [ToDr] Block and wait for confirmation?
				to_value(&H256::zero())
		})
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningUnsafeClient<C, A, M> where
	C: BlockChainClient,
	A: AccountProvider,
	M: MinerService {
	client: Weak<C>,
	accounts: Weak<A>,
	miner: Weak<M>,
}

impl<C, A, M> EthSigningUnsafeClient<C, A, M> where
	C: BlockChainClient,
	A: AccountProvider,
	M: MinerService {

	/// Creates new EthClient.
	pub fn new(client: &Arc<C>, accounts: &Arc<A>, miner: &Arc<M>)
		-> Self {
		EthSigningUnsafeClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
		}
	}
}

impl<C, A, M> EthSigning for EthSigningUnsafeClient<C, A, M> where
	C: BlockChainClient + 'static,
	A: AccountProvider + 'static,
	M: MinerService + 'static {

	fn sign(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, H256)>(params).and_then(|(addr, msg)| {
			to_value(&take_weak!(self.accounts).sign(&addr, &msg).unwrap_or(H520::zero()))
		})
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let accounts = take_weak!(self.accounts);
				match accounts.account_secret(&request.from) {
					Ok(secret) => sign_and_dispatch(&self.client, &self.miner, request, secret),
					Err(_) => to_value(&H256::zero())
				}
		})
	}

}
