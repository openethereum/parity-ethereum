// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Signing RPC implementation.

use std::sync::{Arc, Weak};
use transient_hashmap::TransientHashMap;
use util::{U256, Mutex, Hashable};

use ethcore::account_provider::AccountProvider;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;

use futures::{self, BoxFuture, Future};
use jsonrpc_core::Error;
use v1::helpers::{
	errors, dispatch,
	SigningQueue, ConfirmationPromise, ConfirmationResult, ConfirmationPayload, SignerService
};
use v1::traits::{EthSigning, ParitySigning};
use v1::types::{
	H160 as RpcH160, H256 as RpcH256, U256 as RpcU256, Bytes as RpcBytes, H520 as RpcH520,
	Either as RpcEither,
	RichRawTransaction as RpcRichRawTransaction,
	TransactionRequest as RpcTransactionRequest,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse
};

const MAX_PENDING_DURATION: u64 = 60 * 60;

pub enum DispatchResult {
	Promise(ConfirmationPromise),
	Value(RpcConfirmationResponse),
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	signer: Weak<SignerService>,
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,

	pending: Mutex<TransientHashMap<U256, ConfirmationPromise>>,
}

impl<C, M> SigningQueueClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(signer: &Arc<SignerService>, client: &Arc<C>, miner: &Arc<M>, accounts: &Arc<AccountProvider>) -> Self {
		SigningQueueClient {
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

	fn handle_dispatch<OnResponse>(&self, res: Result<DispatchResult, Error>, on_response: OnResponse)
		where OnResponse: FnOnce(Result<RpcConfirmationResponse, Error>) + Send + 'static
	{
		match res {
			Ok(DispatchResult::Value(result)) => on_response(Ok(result)),
			Ok(DispatchResult::Promise(promise)) => {
				promise.wait_for_result(move |result| {
					on_response(result.unwrap_or_else(|| Err(errors::request_rejected())))
				})
			},
			Err(e) => on_response(Err(e)),
		}
	}

	fn add_to_queue(&self, payload: ConfirmationPayload) -> Result<DispatchResult, Error> {
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);
		let accounts = take_weak!(self.accounts);

		let sender = payload.sender();
		if accounts.is_unlocked(sender) {
			return dispatch::execute(&*client, &*miner, &*accounts, payload, dispatch::SignWith::Nothing)
				.map(|v| v.into_value())
				.map(DispatchResult::Value);
		}

		take_weak!(self.signer).add_request(payload)
			.map(DispatchResult::Promise)
			.map_err(|_| errors::request_rejected_limit())
	}

	fn dispatch(&self, payload: RpcConfirmationPayload) -> Result<DispatchResult, Error> {
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);

		let payload = dispatch::from_rpc(payload, &*client, &*miner);
		self.add_to_queue(payload)
	}
}

impl<C: 'static, M: 'static> ParitySigning for SigningQueueClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn post_sign(&self, address: RpcH160, hash: RpcH256) -> Result<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		self.active()?;
		self.dispatch(RpcConfirmationPayload::Signature((address, hash).into()))
			.map(|result| match result {
				DispatchResult::Value(v) => RpcEither::Or(v),
				DispatchResult::Promise(promise) => {
					let id = promise.id();
					self.pending.lock().insert(id, promise);
					RpcEither::Either(id.into())
				},
			})
	}

	fn post_transaction(&self, request: RpcTransactionRequest) -> Result<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		self.active()?;
		self.dispatch(RpcConfirmationPayload::SendTransaction(request))
			.map(|result| match result {
				DispatchResult::Value(v) => RpcEither::Or(v),
				DispatchResult::Promise(promise) => {
					let id = promise.id();
					self.pending.lock().insert(id, promise);
					RpcEither::Either(id.into())
				},
			})
	}

	fn check_request(&self, id: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
		self.active()?;
		let mut pending = self.pending.lock();
		let id: U256 = id.into();
		let res = match pending.get(&id) {
			Some(ref promise) => match promise.result() {
				ConfirmationResult::Waiting => { return Ok(None); }
				ConfirmationResult::Rejected => Err(errors::request_rejected()),
				ConfirmationResult::Confirmed(rpc_response) => rpc_response.map(Some),
			},
			_ => { return Err(errors::request_not_found()); }
		};
		pending.remove(&id);
		res
	}

	fn decrypt_message(&self, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes, Error> {
		let res = self.active()
			.and_then(|_| self.dispatch(RpcConfirmationPayload::Decrypt((address, data).into())));

		let (ready, p) = futures::oneshot();
		// TODO [todr] typed handle_dispatch
		self.handle_dispatch(res, |response| {
			match response {
				Ok(RpcConfirmationResponse::Decrypt(data)) => ready.complete(Ok(data)),
				Err(e) => ready.complete(Err(e)),
				e => ready.complete(Err(errors::internal("Unexpected result.", e))),
			}
		});

		p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled."))).boxed()
	}
}

impl<C: 'static, M: 'static> EthSigning for SigningQueueClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn sign(&self, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520, Error> {
		let hash = data.0.sha3().into();
		let res = self.active().and_then(|_| self.dispatch(RpcConfirmationPayload::Signature((address, hash).into())));

		let (ready, p) = futures::oneshot();
		self.handle_dispatch(res, |response| {
			match response {
				Ok(RpcConfirmationResponse::Signature(signature)) => ready.complete(Ok(signature)),
				Err(e) => ready.complete(Err(e)),
				e => ready.complete(Err(errors::internal("Unexpected result.", e))),
			}
		});

		p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled."))).boxed()
	}

	fn send_transaction(&self, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		let res = self.active().and_then(|_| self.dispatch(RpcConfirmationPayload::SendTransaction(request)));

		let (ready, p) = futures::oneshot();
		self.handle_dispatch(res, |response| {
			match response {
				Ok(RpcConfirmationResponse::SendTransaction(hash)) => ready.complete(Ok(hash)),
				Err(e) => ready.complete(Err(e)),
				e => ready.complete(Err(errors::internal("Unexpected result.", e))),
			}
		});

		p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled."))).boxed()
	}

	fn sign_transaction(&self, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		let res = self.active().and_then(|_| self.dispatch(RpcConfirmationPayload::SignTransaction(request)));

		let (ready, p) = futures::oneshot();
		self.handle_dispatch(res, |response| {
			match response {
				Ok(RpcConfirmationResponse::SignTransaction(tx)) => ready.complete(Ok(tx)),
				Err(e) => ready.complete(Err(e)),
				e => ready.complete(Err(errors::internal("Unexpected result.", e))),
			}
		});

		p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled."))).boxed()
	}
}
