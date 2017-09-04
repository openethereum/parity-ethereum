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

use std::sync::Arc;
use transient_hashmap::TransientHashMap;
use bigint::prelude::U256;
use util::Mutex;

use ethcore::account_provider::AccountProvider;

use futures::{future, BoxFuture, Future};
use jsonrpc_core::Error;
use v1::helpers::{
	errors, oneshot,
	DefaultAccount,
	SIGNING_QUEUE_LIMIT, SigningQueue, ConfirmationPromise, ConfirmationResult, SignerService,
};
use v1::helpers::dispatch::{self, Dispatcher};
use v1::helpers::accounts::unwrap_provider;
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

/// After 60s entries that are not queried with `check_request` will get garbage collected.
const MAX_PENDING_DURATION_SEC: u32 = 60;
/// Max number of total requests pending and completed, before we start garbage collecting them.
const MAX_TOTAL_REQUESTS: usize = SIGNING_QUEUE_LIMIT;

enum DispatchResult {
	Promise(ConfirmationPromise),
	Value(RpcConfirmationResponse),
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<D> {
	signer: Arc<SignerService>,
	accounts: Option<Arc<AccountProvider>>,
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

fn collect_garbage(map: &mut TransientHashMap<U256, ConfirmationPromise>) {
	map.prune();
	if map.len() > MAX_TOTAL_REQUESTS {
		// Remove all non-waiting entries.
		let non_waiting: Vec<_> = map
			.iter()
			.filter(|&(_, val)| val.result() != ConfirmationResult::Waiting)
			.map(|(key, _)| *key)
			.collect();
		for k in non_waiting {
			map.remove(&k);
		}
	}
}

impl<D: Dispatcher + 'static> SigningQueueClient<D> {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(signer: &Arc<SignerService>, dispatcher: D, accounts: &Option<Arc<AccountProvider>>) -> Self {
		SigningQueueClient {
			signer: signer.clone(),
			accounts: accounts.clone(),
			dispatcher: dispatcher,
			pending: Arc::new(Mutex::new(TransientHashMap::new(MAX_PENDING_DURATION_SEC))),
		}
	}

	fn account_provider(&self) -> Result<Arc<AccountProvider>, Error> {
		unwrap_provider(&self.accounts)
	}

	fn dispatch(&self, payload: RpcConfirmationPayload, default_account: DefaultAccount, origin: Origin) -> BoxFuture<DispatchResult, Error> {
		let accounts = try_bf!(self.account_provider());
		let default_account = match default_account {
			DefaultAccount::Provided(acc) => acc,
			DefaultAccount::ForDapp(dapp) => accounts.dapp_default_address(dapp).ok().unwrap_or_default(),
		};

		let dispatcher = self.dispatcher.clone();
		let signer = self.signer.clone();
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

	fn compose_transaction(&self, meta: Metadata, transaction: RpcTransactionRequest) -> BoxFuture<RpcTransactionRequest, Error> {
		let accounts = try_bf!(self.account_provider());
		let default_account = accounts.dapp_default_address(meta.dapp_id().into()).ok().unwrap_or_default();
		self.dispatcher.fill_optional_fields(transaction.into(), default_account, true).map(Into::into).boxed()
	}

	fn post_sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		let pending = self.pending.clone();
		self.dispatch(
			RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()),
			DefaultAccount::Provided(address.into()),
			meta.origin
		).map(move |result| match result {
			DispatchResult::Value(v) => RpcEither::Or(v),
			DispatchResult::Promise(promise) => {
				let id = promise.id();
				let mut pending = pending.lock();
				collect_garbage(&mut pending);
				pending.insert(id, promise);

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
					let mut pending = pending.lock();
					collect_garbage(&mut pending);
					pending.insert(id, promise);

					RpcEither::Either(id.into())
				},
			})
			.boxed()
	}

	fn check_request(&self, id: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
		let id: U256 = id.into();
		match self.pending.lock().get(&id) {
			Some(ref promise) => match promise.result() {
				ConfirmationResult::Waiting => Ok(None),
				ConfirmationResult::Rejected => Err(errors::request_rejected()),
				ConfirmationResult::Confirmed(rpc_response) => rpc_response.map(Some),
			},
			_ => Err(errors::request_not_found()),
		}
	}

	fn decrypt_message(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::Decrypt((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		let (ready, p) = oneshot::oneshot();

		// when dispatch is complete
		res.then(move |res| {
			// register callback via the oneshot sender.
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::Decrypt(data)) => ready.send(Ok(data)),
					Err(e) => ready.send(Err(e)),
					e => ready.send(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p
		}).boxed()
	}
}

impl<D: Dispatcher + 'static> EthSigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		let (ready, p) = oneshot::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::Signature(sig)) => ready.send(Ok(sig)),
					Err(e) => ready.send(Err(e)),
					e => ready.send(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p
		}).boxed()
	}

	fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SendTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		let (ready, p) = oneshot::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::SendTransaction(hash)) => ready.send(Ok(hash)),
					Err(e) => ready.send(Err(e)),
					e => ready.send(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p
		}).boxed()
	}

	fn sign_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SignTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		let (ready, p) = oneshot::oneshot();

		res.then(move |res| {
			handle_dispatch(res, move |response| {
				match response {
					Ok(RpcConfirmationResponse::SignTransaction(tx)) => ready.send(Ok(tx)),
					Err(e) => ready.send(Err(e)),
					e => ready.send(Err(errors::internal("Unexpected result.", e))),
				}
			});

			p
		}).boxed()
	}
}
