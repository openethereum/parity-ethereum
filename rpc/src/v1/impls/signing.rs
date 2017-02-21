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

//! Signing RPC implementation.

use std::sync::{Arc, Weak};
use transient_hashmap::TransientHashMap;
use util::{U256, Mutex};

use ethcore::account_provider::AccountProvider;

use futures::{self, future, BoxFuture, Future};
use jsonrpc_core::Error;
use v1::helpers::{
	errors,
	DefaultAccount,
	SigningQueue, ConfirmationPromise, ConfirmationResult, SignerService
};
use v1::helpers::dispatch::{self, Dispatcher};
use v1::metadata::Metadata;
use v1::traits::{EthSigning, ParitySigning};
use v1::types::{
	H160 as RpcH160, H256 as RpcH256, U256 as RpcU256, Bytes as RpcBytes, H520 as RpcH520,
	Either as RpcEither,
	RichRawTransaction as RpcRichRawTransaction,
	TransactionRequest as RpcTransactionRequest,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse,
	Origin,
};

const MAX_PENDING_DURATION: u64 = 60 * 60;

enum DispatchResult {
	Promise(ConfirmationPromise),
	Value(RpcConfirmationResponse),
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<D> {
	signer: Weak<SignerService>,
	accounts: Weak<AccountProvider>,
	dispatcher: D,
	pending: Arc<Mutex<TransientHashMap<U256, ConfirmationPromise>>>,
}

fn handle_dispatch<OnResponse>(res: Result<DispatchResult, Error>, on_response: OnResponse)
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

impl<D: Dispatcher + 'static> SigningQueueClient<D> {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(signer: &Arc<SignerService>, dispatcher: D, accounts: &Arc<AccountProvider>) -> Self {
		SigningQueueClient {
			signer: Arc::downgrade(signer),
			accounts: Arc::downgrade(accounts),
			dispatcher: dispatcher,
			pending: Arc::new(Mutex::new(TransientHashMap::new(MAX_PENDING_DURATION))),
		}
	}

	fn dispatch(&self, payload: RpcConfirmationPayload, default_account: DefaultAccount, origin: Origin) -> BoxFuture<DispatchResult, Error> {
		let accounts = take_weakf!(self.accounts);
		let default_account = match default_account {
			DefaultAccount::Provided(acc) => acc,
			DefaultAccount::ForDapp(dapp) => accounts.dapp_default_address(dapp).ok().unwrap_or_default(),
		};

		let dispatcher = self.dispatcher.clone();
		let signer = take_weakf!(self.signer);
		dispatch::from_rpc(payload, default_account, &dispatcher)
			.and_then(move |payload| {
				let sender = payload.sender();
				if accounts.is_unlocked(sender) {
					dispatch::execute(dispatcher, accounts, payload, dispatch::SignWith::Nothing)
						.map(|v| v.into_value())
						.map(DispatchResult::Value)
						.boxed()
				} else {
					future::done(
						signer.add_request(payload, origin)
							.map(DispatchResult::Promise)
							.map_err(|_| errors::request_rejected_limit())
					).boxed()
				}
			})
			.boxed()
	}
}

impl<D: Dispatcher + 'static> ParitySigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn post_sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		let pending = self.pending.clone();
		self.dispatch(
			RpcConfirmationPayload::Signature((address.clone(), data).into()),
			DefaultAccount::Provided(address.into()),
			meta.origin
		).map(move |result| match result {
			DispatchResult::Value(v) => RpcEither::Or(v),
			DispatchResult::Promise(promise) => {
				let id = promise.id();
				pending.lock().insert(id, promise);
				RpcEither::Either(id.into())
			},
		})
		.boxed()
	}

	fn post_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		let pending = self.pending.clone();
		self.dispatch(RpcConfirmationPayload::SendTransaction(request), meta.dapp_id().into(), meta.origin)
			.map(move |result| match result {
				DispatchResult::Value(v) => RpcEither::Or(v),
				DispatchResult::Promise(promise) => {
					let id = promise.id();
					pending.lock().insert(id, promise);
					RpcEither::Either(id.into())
				},
			})
			.boxed()
	}

	fn check_request(&self, id: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
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

	fn decrypt_message(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::Decrypt((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		let (ready, p) = futures::oneshot();

		// when dispatch is complete
		res.then(move |res| {
			// register callback via the oneshot sender.
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::Decrypt(data)) => ready.complete(Ok(data)),
					Err(e) => ready.complete(Err(e)),
					e => ready.complete(Err(errors::internal("Unexpected result.", e))),
				}
			});

			// and wait for that to resolve.
			p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled.")))
		}).boxed()
	}
}

impl<D: Dispatcher + 'static> EthSigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::Signature((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		let (ready, p) = futures::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::Signature(sig)) => ready.complete(Ok(sig)),
					Err(e) => ready.complete(Err(e)),
					e => ready.complete(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled.")))
		}).boxed()
	}

	fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SendTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		let (ready, p) = futures::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::SendTransaction(hash)) => ready.complete(Ok(hash)),
					Err(e) => ready.complete(Err(e)),
					e => ready.complete(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled.")))
		}).boxed()
	}

	fn sign_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SignTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		let (ready, p) = futures::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::SignTransaction(tx)) => ready.complete(Ok(tx)),
					Err(e) => ready.complete(Err(e)),
					e => ready.complete(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p.then(|result| futures::done(result.expect("Ready is never dropped nor canceled.")))
		}).boxed()
	}
}
