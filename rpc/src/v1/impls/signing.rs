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
use ethereum_types::U256;
use parking_lot::Mutex;

use ethcore::account_provider::AccountProvider;

use jsonrpc_core::{BoxFuture, Result, Error};
use jsonrpc_core::futures::{future, Future, Poll, Async};
use jsonrpc_core::futures::future::Either;
use v1::helpers::{
	errors, DefaultAccount, SignerService, SigningQueue,
	ConfirmationReceiver as RpcConfirmationReceiver,
	ConfirmationResult as RpcConfirmationResult,
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

use parity_reactor::Remote;

/// After 60s entries that are not queried with `check_request` will get garbage collected.
const MAX_PENDING_DURATION_SEC: u32 = 60;

#[must_use = "futures do nothing unless polled"]
enum DispatchResult {
	Future(U256, RpcConfirmationReceiver),
	Value(RpcConfirmationResponse),
}

impl Future for DispatchResult {
	type Item = RpcConfirmationResponse;
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		match *self {
			DispatchResult::Value(ref response) => Ok(Async::Ready(response.clone())),
			DispatchResult::Future(_uid, ref mut future) => try_ready!(future.poll()).map(Async::Ready),
		}
	}
}

fn schedule(remote: Remote,
	confirmations: Arc<Mutex<TransientHashMap<U256, Option<RpcConfirmationResult>>>>,
	id: U256,
	future: RpcConfirmationReceiver) {
	{
		let mut confirmations = confirmations.lock();
		confirmations.insert(id.clone(), None);
	}

	let future = future.then(move |result| {
		let mut confirmations = confirmations.lock();
		confirmations.prune();
		let result = result.and_then(|response| response);
		confirmations.insert(id, Some(result));
		Ok(())
	});
	remote.spawn(future);
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<D> {
	signer: Arc<SignerService>,
	accounts: Arc<AccountProvider>,
	dispatcher: D,
	remote: Remote,
	// None here means that the request hasn't yet been confirmed
	confirmations: Arc<Mutex<TransientHashMap<U256, Option<RpcConfirmationResult>>>>,
}

impl<D: Dispatcher + 'static> SigningQueueClient<D> {
	/// Creates a new signing queue client given shared signing queue.
	pub fn new(signer: &Arc<SignerService>, dispatcher: D, remote: Remote, accounts: &Arc<AccountProvider>) -> Self {
		SigningQueueClient {
			signer: signer.clone(),
			accounts: accounts.clone(),
			dispatcher,
			remote,
			confirmations: Arc::new(Mutex::new(TransientHashMap::new(MAX_PENDING_DURATION_SEC))),
		}
	}

	fn account_provider(&self) -> Result<Arc<AccountProvider>> {
		Ok(self.accounts.clone())
	}

	fn dispatch(&self, payload: RpcConfirmationPayload, default_account: DefaultAccount, origin: Origin) -> BoxFuture<DispatchResult> {
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
				if accounts.is_unlocked(&sender) {
					Either::A(dispatch::execute(dispatcher, accounts, payload, dispatch::SignWith::Nothing)
						.map(|v| v.into_value())
						.map(DispatchResult::Value))
				} else {
					Either::B(future::done(
						signer.add_request(payload, origin)
							.map(|(id, future)| DispatchResult::Future(id, future))
							.map_err(|_| errors::request_rejected_limit())
					))
				}
			}))
	}
}

impl<D: Dispatcher + 'static> ParitySigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn compose_transaction(&self, meta: Metadata, transaction: RpcTransactionRequest) -> BoxFuture<RpcTransactionRequest> {
		let accounts = try_bf!(self.account_provider());
		let default_account = accounts.dapp_default_address(meta.dapp_id().into()).ok().unwrap_or_default();
		Box::new(self.dispatcher.fill_optional_fields(transaction.into(), default_account, true).map(Into::into))
	}

	fn post_sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>> {
		let remote = self.remote.clone();
		let confirmations = self.confirmations.clone();

		Box::new(self.dispatch(
			RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()),
			DefaultAccount::Provided(address.into()),
			meta.origin
		).map(move |result| match result {
			DispatchResult::Value(v) => RpcEither::Or(v),
			DispatchResult::Future(id, future) => {
				schedule(remote, confirmations, id, future);
				RpcEither::Either(id.into())
			},
		}))
	}

	fn post_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>> {
		let remote = self.remote.clone();
		let confirmations = self.confirmations.clone();

		Box::new(self.dispatch(RpcConfirmationPayload::SendTransaction(request), meta.dapp_id().into(), meta.origin)
			.map(|result| match result {
				DispatchResult::Value(v) => RpcEither::Or(v),
				DispatchResult::Future(id, future) => {
					schedule(remote, confirmations, id, future);
					RpcEither::Either(id.into())
				},
			}))
	}

	fn check_request(&self, id: RpcU256) -> Result<Option<RpcConfirmationResponse>> {
		let id: U256 = id.into();
		match self.confirmations.lock().get(&id) {
			None => Err(errors::request_not_found()), // Request info has been dropped, or even never been there
			Some(&None) => Ok(None), // No confirmation yet, request is known, confirmation is pending
			Some(&Some(ref confirmation)) => confirmation.clone().map(Some), // Confirmation is there
		}
	}

	fn decrypt_message(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes> {
		let res = self.dispatch(
			RpcConfirmationPayload::Decrypt((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		// when dispatch is complete - wait for result and then
		Box::new(res.flatten().and_then(move |response| {
			match response {
				RpcConfirmationResponse::Decrypt(data) => Ok(data),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}
}

impl<D: Dispatcher + 'static> EthSigning for SigningQueueClient<D> {
	type Metadata = Metadata;

	fn sign(&self, meta: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520> {
		let res = self.dispatch(
			RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()),
			address.into(),
			meta.origin,
		);

		Box::new(res.flatten().and_then(move |response| {
			match response {
				RpcConfirmationResponse::Signature(sig) => Ok(sig),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}

	fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256> {
		let res = self.dispatch(
			RpcConfirmationPayload::SendTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		Box::new(res.flatten().and_then(move |response| {
			match response {
				RpcConfirmationResponse::SendTransaction(hash) => Ok(hash),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}

	fn sign_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction> {
		let res = self.dispatch(
			RpcConfirmationPayload::SignTransaction(request),
			meta.dapp_id().into(),
			meta.origin,
		);

		Box::new(res.flatten().and_then(move |response| {
			match response {
				RpcConfirmationResponse::SignTransaction(tx) => Ok(tx),
				e => Err(errors::internal("Unexpected result.", e)),
			}
		}))
	}
}
