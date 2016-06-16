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
use util::keys::store::AccountProvider;
use v1::helpers::{SigningQueue, ConfirmationsQueue};
use v1::traits::EthSigning;
use v1::types::{TransactionRequest, Bytes};
use v1::impls::{sign_and_dispatch, error_codes};


/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningQueueClient<M: MinerService> {
	queue: Weak<ConfirmationsQueue>,
	miner: Weak<M>,
}

impl<M: MinerService> EthSigningQueueClient<M> {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(queue: &Arc<ConfirmationsQueue>, miner: &Arc<M>) -> Self {
		EthSigningQueueClient {
			queue: Arc::downgrade(queue),
			miner: Arc::downgrade(miner),
		}
	}

	fn fill_optional_fields(&self, miner: Arc<M>, mut request: TransactionRequest) -> TransactionRequest {
		if let None = request.gas {
			request.gas = Some(miner.sensible_gas_limit());
		}
		if let None = request.gas_price {
			request.gas_price = Some(miner.sensible_gas_price());
		}
		if let None = request.data {
			request.data = Some(Bytes::new(Vec::new()));
		}
		request
	}
}

impl<M: MinerService + 'static> EthSigning for EthSigningQueueClient<M>  {

	fn sign(&self, _params: Params) -> Result<Value, Error> {
		warn!("Invoking eth_sign is not yet supported with signer enabled.");
		// TODO [ToDr] Implement sign when rest of the signing queue is ready.
		rpc_unimplemented!()
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let queue = take_weak!(self.queue);
				let miner = take_weak!(self.miner);
				let request = self.fill_optional_fields(miner, request);
				let id = queue.add_request(request);
				let result = id.wait_with_timeout();
				result.unwrap_or_else(|| to_value(&H256::new()))
		})
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningUnsafeClient<C, A, M> where
	C: MiningBlockChainClient,
	A: AccountProvider,
	M: MinerService {
	client: Weak<C>,
	accounts: Weak<A>,
	miner: Weak<M>,
}

impl<C, A, M> EthSigningUnsafeClient<C, A, M> where
	C: MiningBlockChainClient,
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
	C: MiningBlockChainClient + 'static,
	A: AccountProvider + 'static,
	M: MinerService + 'static {

	fn sign(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, H256)>(params).and_then(|(addr, msg)| {
			take_weak!(self.accounts).sign(&addr, &msg)
				.map(|v| to_value(&v))
				.unwrap_or_else(|e| Err(account_locked(format!("Error: {:?}", e))))
		})
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let accounts = take_weak!(self.accounts);
				match accounts.account_secret(&request.from) {
					Ok(secret) => sign_and_dispatch(&*take_weak!(self.client), &*take_weak!(self.miner), request, secret),
					Err(e) => Err(account_locked(format!("Error: {:?}", e))),
				}
		})
	}
}

fn account_locked(data: String) -> Error {
	Error {
		code: ErrorCode::ServerError(error_codes::ACCOUNT_LOCKED),
		message: "Your account is locked. Unlock the account via CLI, personal_unlockAccount or use Trusted Signer.".into(),
		data: Some(Value::String(data)),
	}
}
