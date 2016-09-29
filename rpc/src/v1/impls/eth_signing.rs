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
use util::{U256, Address, H256, Mutex};
use transient_hashmap::TransientHashMap;
use ethcore::account_provider::AccountProvider;
use v1::helpers::{errors, SigningQueue, ConfirmationPromise, ConfirmationResult, ConfirmationPayload, TransactionRequest as TRequest, FilledTransactionRequest as FilledRequest, SignerService};
use v1::helpers::dispatch::{default_gas_price, sign_and_dispatch, sign, decrypt};
use v1::traits::EthSigning;
use v1::types::{TransactionRequest, H160 as RpcH160, H256 as RpcH256, U256 as RpcU256, Bytes as RpcBytes};

fn fill_optional_fields<C, M>(request: TRequest, client: &C, miner: &M) -> FilledRequest
	where C: MiningBlockChainClient, M: MinerService {
	FilledRequest {
		from: request.from,
		to: request.to,
		nonce: request.nonce,
		gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(client, miner)),
		gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
		value: request.value.unwrap_or_else(|| 0.into()),
		data: request.data.unwrap_or_else(Vec::new),
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct EthSigningQueueClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	signer: Weak<SignerService>,
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,

	pending: Mutex<TransientHashMap<U256, ConfirmationPromise>>,
}

const MAX_PENDING_DURATION: u64 = 60 * 60;

pub enum DispatchResult {
	Promise(ConfirmationPromise),
	Value(Value),
}

impl<C, M> EthSigningQueueClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(signer: &Arc<SignerService>, client: &Arc<C>, miner: &Arc<M>, accounts: &Arc<AccountProvider>) -> Self {
		EthSigningQueueClient {
			signer: Arc::downgrade(signer),
			accounts: Arc::downgrade(accounts),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			pending: Mutex::new(TransientHashMap::new(MAX_PENDING_DURATION)),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}

	fn add_to_queue<WhenUnlocked, Payload>(&self, sender: Address, when_unlocked: WhenUnlocked, payload: Payload)
		-> Result<DispatchResult, Error> where
			WhenUnlocked: Fn(&AccountProvider) -> Result<Value, Error>,
			Payload: Fn() -> ConfirmationPayload, {

		let accounts = take_weak!(self.accounts);
		if accounts.is_unlocked(sender) {
			return when_unlocked(&accounts).map(DispatchResult::Value);
		}

		take_weak!(self.signer).add_request(payload())
			.map(DispatchResult::Promise)
			.map_err(|_| errors::request_rejected_limit())
	}

	fn handle_dispatch(&self, res: Result<DispatchResult, Error>, ready: Ready) {
		match res {
			Ok(DispatchResult::Value(v)) => ready.ready(Ok(v)),
			Ok(DispatchResult::Promise(promise)) => {
				promise.wait_for_result(move |result| {
					ready.ready(result.unwrap_or_else(|| Err(errors::request_rejected())))
				})
			},
			Err(e) => ready.ready(Err(e)),
		}
	}

	fn dispatch_sign(&self, params: Params) -> Result<DispatchResult, Error> {
		from_params::<(RpcH160, RpcH256)>(params).and_then(|(address, msg)| {
			let address: Address = address.into();
			let msg: H256 = msg.into();

			self.add_to_queue(
				address,
				|accounts| sign(accounts, address, None, msg.clone()),
				|| ConfirmationPayload::Sign(address, msg.clone()),
			)
		})
	}

	fn dispatch_transaction(&self, params: Params) -> Result<DispatchResult, Error> {
		from_params::<(TransactionRequest, )>(params).and_then(|(request, )| {
			let request: TRequest = request.into();
			let (client, miner) = (take_weak!(self.client), take_weak!(self.miner));
			self.add_to_queue(
				request.from,
				|accounts| sign_and_dispatch(&*client, &*miner, accounts, request.clone(), None),
				|| {
					let request = fill_optional_fields(request.clone(), &*client, &*miner);
					ConfirmationPayload::Transaction(request)
				}
			)
		})
	}
}

impl<C, M> EthSigning for EthSigningQueueClient<C, M>
	where C: MiningBlockChainClient + 'static, M: MinerService + 'static
{

	fn post_sign(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		self.dispatch_sign(params).map(|result| match result {
			DispatchResult::Value(v) => v,
			DispatchResult::Promise(promise) => {
				let id = promise.id();
				self.pending.lock().insert(id, promise);
				to_value(&RpcU256::from(id))
			},
		})
	}

	fn post_transaction(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		self.dispatch_transaction(params).map(|result| match result {
			DispatchResult::Value(v) => v,
			DispatchResult::Promise(promise) => {
				let id = promise.id();
				self.pending.lock().insert(id, promise);
				to_value(&RpcU256::from(id))
			},
		})
	}

	fn check_request(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		let mut pending = self.pending.lock();
		from_params::<(RpcU256, )>(params).and_then(|(id, )| {
			let id: U256 = id.into();
			let res = match pending.get(&id) {
				Some(ref promise) => match promise.result() {
					ConfirmationResult::Waiting => { return Ok(Value::Null); }
					ConfirmationResult::Rejected => Err(errors::request_rejected()),
					ConfirmationResult::Confirmed(rpc_response) => rpc_response,
				},
				_ => { return Err(errors::request_not_found()); }
			};
			pending.remove(&id);
			res
		})
	}

	fn sign(&self, params: Params, ready: Ready) {
		let res = self.active().and_then(|_| self.dispatch_sign(params));
		self.handle_dispatch(res, ready);
	}

	fn send_transaction(&self, params: Params, ready: Ready) {
		let res = self.active().and_then(|_| self.dispatch_transaction(params));
		self.handle_dispatch(res, ready);
	}

	fn decrypt_message(&self, params: Params, ready: Ready) {
		let res = self.active()
			.and_then(|_| from_params::<(RpcH160, RpcBytes)>(params))
			.and_then(|(address, msg)| {
				let address: Address = address.into();

				self.add_to_queue(
					address,
					|accounts| decrypt(accounts, address, None, msg.clone().into()),
					|| ConfirmationPayload::Decrypt(address, msg.clone().into())
				)
			});

		self.handle_dispatch(res, ready);
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

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M> EthSigning for EthSigningUnsafeClient<C, M> where
	C: MiningBlockChainClient + 'static,
	M: MinerService + 'static {

	fn sign(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(RpcH160, RpcH256)>(params))
			.and_then(|(address, msg)| {
				sign(&*take_weak!(self.accounts), address.into(), None, msg.into())
			}))
	}

	fn send_transaction(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(TransactionRequest, )>(params))
			.and_then(|(request, )| {
				sign_and_dispatch(&*take_weak!(self.client), &*take_weak!(self.miner), &*take_weak!(self.accounts), request.into(), None)
			}))
	}

	fn decrypt_message(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(RpcH160, RpcBytes)>(params))
			.and_then(|(address, ciphertext)| {
				decrypt(&*take_weak!(self.accounts), address.into(), None, ciphertext.0)
			}))
	}

	fn post_sign(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn post_transaction(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn check_request(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}
}
