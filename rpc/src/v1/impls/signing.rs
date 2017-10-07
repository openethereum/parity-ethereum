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
use parking_lot::Mutex;

use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use jsonrpc_core::futures::{future, BoxFuture, Future, Poll, Async};
use jsonrpc_core::futures::future::Either;
use v1::helpers::{
	errors, DefaultAccount,
	SIGNING_QUEUE_LIMIT, SigningQueue, ConfirmationReceiver, ConfirmationResult as RpcConfirmationResult, SignerService,
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
	Future(ConfirmationReceiver),
	Value(RpcConfirmationResponse),
}

impl Future for DispatchResult {
	type Item=RpcConfirmationResponse;
	type Error=Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		match self {
			&mut DispatchResult::Value(ref response) => Ok(Async::Ready(response.clone())),
			&mut DispatchResult::Future(ref mut future) => future.poll().and_then(|async|
				match async {
					Async::NotReady => Ok(Async::NotReady),
					Async::Ready(RpcConfirmationResult::Rejected) => Err(errors::request_rejected()),
					Async::Ready(RpcConfirmationResult::Confirmed(Ok(response))) => Ok(Async::Ready(response)),
					Async::Ready(RpcConfirmationResult::Confirmed(Err(error))) => Err(error)
				}
			)
		}
	}
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<D> {
	signer: Arc<SignerService>,
	accounts: Option<Arc<AccountProvider>>,
	dispatcher: D,
	pending: Arc<Mutex<TransientHashMap<U256, ConfirmationReceiver>>>,
}

fn collect_garbage(map: &mut TransientHashMap<U256, ConfirmationReceiver>) {
	map.prune();
	if map.len() > MAX_TOTAL_REQUESTS {
		map.retain(|_, ref mut val| val.poll().map(|async| async.is_not_ready()).unwrap_or(false));
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
		Box::new(dispatch::from_rpc(payload, default_account, &dispatcher)
			.and_then(move |payload| {
				let sender = payload.sender();
				if accounts.is_unlocked(sender) {
					Either::A(dispatch::execute(dispatcher, accounts, payload, dispatch::SignWith::Nothing)
						.map(|v| v.into_value())
						.map(DispatchResult::Value))
				} else {
					Either::B(future::done(
						signer.add_request(payload, origin)
							.map(DispatchResult::Future)
							.map_err(|_| errors::request_rejected_limit())
					))
				}
			}))
	}
}

impl<D: Dispatcher + 'static> ParitySigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn compose_transaction(&self, meta: Metadata, transaction: RpcTransactionRequest) -> BoxFuture<RpcTransactionRequest, Error> {
		let accounts = try_bf!(self.account_provider());
		let default_account = accounts.dapp_default_address(meta.dapp_id().into()).ok().unwrap_or_default();
		Box::new(self.dispatcher.fill_optional_fields(transaction.into(), default_account, true).map(Into::into))
	}

	fn post_sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		let pending = self.pending.clone();
		Box::new(self.dispatch(
			RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()),
			DefaultAccount::Provided(address.into()),
			meta.origin
		).map(move |result| match result {
			DispatchResult::Value(v) => RpcEither::Or(v),
			DispatchResult::Future(future) => {
				let id = future.id();
				let mut pending = pending.lock();
				collect_garbage(&mut pending);
				pending.insert(id, future);

				RpcEither::Either(id.into())
			},
		}))
	}

	fn post_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		let pending = self.pending.clone();
		Box::new(self.dispatch(RpcConfirmationPayload::SendTransaction(request), meta.dapp_id().into(), meta.origin)
			.map(move |result| match result {
				DispatchResult::Value(v) => RpcEither::Or(v),
				DispatchResult::Future(future) => {
					let id = future.id();
					let mut pending = pending.lock();
					collect_garbage(&mut pending);
					pending.insert(id, future);

					RpcEither::Either(id.into())
				},
			}))
	}

	fn check_request(&self, id: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
		let id: U256 = id.into();
		match self.pending.lock().get_mut(&id) {
			Some(ref mut future) => match future.poll() {
				Ok(Async::NotReady) => Ok(None),
				Ok(Async::Ready(status)) => { 
					match status {
						RpcConfirmationResult::Rejected => Err(errors::request_rejected()),
						RpcConfirmationResult::Confirmed(rpc_response) => rpc_response.clone().map(Some),
					}
				},
				Err(error) => Err(error)
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

		// when dispatch is complete - wait for result and then
		Box::new(res.map(|result| result.wait()).and_then(move |response| {
			match response {
				Ok(RpcConfirmationResponse::Decrypt(data)) => Ok(data),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
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

		Box::new(res.map(|result| result.wait()).and_then(move |response| {
			match response {
				Ok(RpcConfirmationResponse::Signature(sig)) => Ok(sig),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}

	fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SendTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		Box::new(res.map(|result| result.wait()).and_then(move |response| {
			match response {
				Ok(RpcConfirmationResponse::SendTransaction(hash)) => Ok(hash),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}

	fn sign_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		let res = self.dispatch(
			RpcConfirmationPayload::SignTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		Box::new(res.map(|result| result.wait()).and_then(move |response| {
			match response {
				Ok(RpcConfirmationResponse::SignTransaction(tx)) => Ok(tx),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}
}
