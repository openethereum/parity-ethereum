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
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use util::numbers::*;
use ethcore::account_provider::AccountProvider;
use v1::helpers::{SigningQueue, ConfirmationsQueue};
use v1::traits::EthSigning;
use v1::types::{TransactionRequest, Bytes};
use v1::impls::{default_gas_price, sign_and_dispatch};

fn fill_optional_fields<C, M>(request: &mut TransactionRequest, client: &C, miner: &M)
	where C: MiningBlockChainClient, M: MinerService {
	if request.gas.is_none() {
		request.gas = Some(miner.sensible_gas_limit());
	}
	if request.gas_price.is_none() {
		request.gas_price = Some(default_gas_price(client, miner));
	}
	if request.data.is_none() {
		request.data = Some(Bytes::new(Vec::new()));
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningQueueClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	queue: Weak<ConfirmationsQueue>,
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C, M> EthSigningQueueClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(queue: &Arc<ConfirmationsQueue>, client: &Arc<C>, miner: &Arc<M>, accounts: &Arc<AccountProvider>) -> Self {
		EthSigningQueueClient {
			queue: Arc::downgrade(queue),
			accounts: Arc::downgrade(accounts),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}
}

impl<C, M> EthSigning for EthSigningQueueClient<C, M>
	where C: MiningBlockChainClient + 'static, M: MinerService + 'static
{

	fn sign(&self, _params: Params) -> Result<Value, Error> {
		warn!("Invoking eth_sign is not yet supported with signer enabled.");
		// TODO [ToDr] Implement sign when rest of the signing queue is ready.
		rpc_unimplemented!()
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(mut request, )| {
				let accounts = take_weak!(self.accounts);
				let (client, miner) = (take_weak!(self.client), take_weak!(self.miner));

				if accounts.is_unlocked(request.from) {
					let sender = request.from;
					return match sign_and_dispatch(&*client, &*miner, request, &*accounts, sender) {
						Ok(hash) => to_value(&hash),
						_ => to_value(&H256::zero()),
					}
				}

				let queue = take_weak!(self.queue);
				fill_optional_fields(&mut request, &*client, &*miner);
				let id = queue.add_request(request);
				let result = id.wait_with_timeout();
				result.unwrap_or_else(|| to_value(&H256::new()))
		})
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService {
	client: Weak<C>,
	accounts: Weak<AccountProvider>,
	miner: Weak<M>,
}

impl<C, M> EthSigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService {

	/// Creates new EthClient.
	pub fn new(client: &Arc<C>, accounts: &Arc<AccountProvider>, miner: &Arc<M>)
		-> Self {
		EthSigningUnsafeClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
		}
	}
}

impl<C, M> EthSigning for EthSigningUnsafeClient<C, M> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static {

	fn sign(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, H256)>(params).and_then(|(addr, msg)| {
			to_value(&take_weak!(self.accounts).sign(addr, msg).unwrap_or(H520::zero()))
		})
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let sender = request.from;
				match sign_and_dispatch(&*take_weak!(self.client), &*take_weak!(self.miner), request, &*take_weak!(self.accounts), sender) {
					Ok(hash) => to_value(&hash),
					_ => to_value(&H256::zero()),
				}
		})
	}
}
