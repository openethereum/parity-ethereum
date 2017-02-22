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

//! Unsafe Signing RPC implementation.

use std::sync::{Arc, Weak};

use ethcore::account_provider::AccountProvider;

use futures::{future, BoxFuture, Future};
use jsonrpc_core::Error;
use v1::helpers::{errors, DefaultAccount};
use v1::helpers::dispatch::{self, Dispatcher};
use v1::metadata::Metadata;
use v1::traits::{EthSigning, ParitySigning};
use v1::types::{
	U256 as RpcU256,
	H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes,
	Either as RpcEither,
	RichRawTransaction as RpcRichRawTransaction,
	TransactionRequest as RpcTransactionRequest,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse,
};

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningUnsafeClient<D> {
	accounts: Weak<AccountProvider>,
	dispatcher: D,
}

impl<D: Dispatcher + 'static> SigningUnsafeClient<D> {
	/// Creates new SigningUnsafeClient.
	pub fn new(accounts: &Arc<AccountProvider>, dispatcher: D) -> Self {
		SigningUnsafeClient {
			accounts: Arc::downgrade(accounts),
			dispatcher: dispatcher,
		}
	}

	fn handle(&self, payload: RpcConfirmationPayload, account: DefaultAccount) -> BoxFuture<RpcConfirmationResponse, Error> {
		let accounts = take_weakf!(self.accounts);
		let default = match account {
			DefaultAccount::Provided(acc) => acc,
			DefaultAccount::ForDapp(dapp) => accounts.dapp_default_address(dapp).ok().unwrap_or_default(),
		};

		let dis = self.dispatcher.clone();
		dispatch::from_rpc(payload, default, &dis)
			.and_then(move |payload| {
				dispatch::execute(dis, accounts, payload, dispatch::SignWith::Nothing)
			})
			.map(|v| v.into_value())
			.boxed()
	}
}

impl<D: Dispatcher + 'static> EthSigning for SigningUnsafeClient<D>
{
	type Metadata = Metadata;

	fn sign(&self, _: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520, Error> {
		self.handle(RpcConfirmationPayload::Signature((address.clone(), data).into()), address.into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
			.boxed()
	}

	fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		self.handle(RpcConfirmationPayload::SendTransaction(request), meta.dapp_id().into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::SendTransaction(hash)) => Ok(hash),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
			.boxed()
	}

	fn sign_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		self.handle(RpcConfirmationPayload::SignTransaction(request), meta.dapp_id().into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::SignTransaction(tx)) => Ok(tx),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
			.boxed()
	}
}

impl<D: Dispatcher + 'static> ParitySigning for SigningUnsafeClient<D> {
	type Metadata = Metadata;

	fn decrypt_message(&self, _: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes, Error> {
		self.handle(RpcConfirmationPayload::Decrypt((address.clone(), data).into()), address.into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Decrypt(data)) => Ok(data),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
			.boxed()
	}

	fn post_sign(&self, _: Metadata,  _: RpcH160, _: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		future::err(errors::signer_disabled()).boxed()
	}

	fn post_transaction(&self, _: Metadata, _: RpcTransactionRequest) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		future::err((errors::signer_disabled())).boxed()
	}

	fn check_request(&self, _: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}
}
