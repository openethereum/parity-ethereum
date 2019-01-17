// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Unsafe Signing RPC implementation.

use std::sync::Arc;

use ethereum_types::Address;

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::{future, Future};
use v1::helpers::{errors};
use v1::helpers::deprecated::{self, DeprecationNotice};
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
	accounts: Arc<dispatch::Accounts>,
	dispatcher: D,
	deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher + 'static> SigningUnsafeClient<D> {
	/// Creates new SigningUnsafeClient.
	pub fn new(accounts: &Arc<dispatch::Accounts>, dispatcher: D) -> Self {
		SigningUnsafeClient {
			accounts: accounts.clone(),
			dispatcher,
			deprecation_notice: Default::default(),
		}
	}

	fn handle(&self, payload: RpcConfirmationPayload, account: Address) -> BoxFuture<RpcConfirmationResponse> {
		let accounts = self.accounts.clone();

		let dis = self.dispatcher.clone();
		Box::new(dispatch::from_rpc(payload, account, &dis)
			.and_then(move |payload| {
				dispatch::execute(dis, &accounts, payload, dispatch::SignWith::Nothing)
			})
			.map(|v| v.into_value()))
	}
}

impl<D: Dispatcher + 'static> EthSigning for SigningUnsafeClient<D>
{
	type Metadata = Metadata;

	fn sign(&self, _: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520> {
		self.deprecation_notice.print("eth_sign", deprecated::msgs::ACCOUNTS);

		Box::new(self.handle(RpcConfirmationPayload::EthSignMessage((address.clone(), data).into()), address.into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			}))
	}

	fn send_transaction(&self, _meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcH256> {
		self.deprecation_notice.print("eth_sendTransaction", deprecated::msgs::ACCOUNTS);

		Box::new(self.handle(RpcConfirmationPayload::SendTransaction(request), self.accounts.default_account())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::SendTransaction(hash)) => Ok(hash),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			}))
	}

	fn sign_transaction(&self, _meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction> {
		self.deprecation_notice.print("eth_signTransaction", deprecated::msgs::ACCOUNTS);

		Box::new(self.handle(RpcConfirmationPayload::SignTransaction(request), self.accounts.default_account())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::SignTransaction(tx)) => Ok(tx),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			}))
	}
}

impl<D: Dispatcher + 'static> ParitySigning for SigningUnsafeClient<D> {
	type Metadata = Metadata;

	fn compose_transaction(&self, _meta: Metadata, transaction: RpcTransactionRequest) -> BoxFuture<RpcTransactionRequest> {
		let accounts = self.accounts.clone();
		let default_account = accounts.default_account();
		Box::new(self.dispatcher.fill_optional_fields(transaction.into(), default_account, true).map(Into::into))
	}

	fn decrypt_message(&self, _: Metadata, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes> {
		self.deprecation_notice.print("parity_decryptMessage", deprecated::msgs::ACCOUNTS);

		Box::new(self.handle(RpcConfirmationPayload::Decrypt((address.clone(), data).into()), address.into())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Decrypt(data)) => Ok(data),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			}))
	}

	fn post_sign(&self, _: Metadata,  _: RpcH160, _: RpcBytes) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>> {
		// We don't support this in non-signer mode.
		Box::new(future::err(errors::signer_disabled()))
	}

	fn post_transaction(&self, _: Metadata, _: RpcTransactionRequest) -> BoxFuture<RpcEither<RpcU256, RpcConfirmationResponse>> {
		// We don't support this in non-signer mode.
		Box::new(future::err(errors::signer_disabled()))
	}

	fn check_request(&self, _: RpcU256) -> Result<Option<RpcConfirmationResponse>> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}
}
