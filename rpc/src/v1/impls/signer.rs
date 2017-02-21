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

//! Transactions Confirmations rpc implementation

use std::sync::{Arc, Weak};

use rlp::{UntrustedRlp, View};
use ethcore::account_provider::AccountProvider;
use ethcore::transaction::{SignedTransaction, PendingTransaction};
use futures::{future, BoxFuture, Future, IntoFuture};

use jsonrpc_core::Error;
use v1::helpers::{errors, SignerService, SigningQueue, ConfirmationPayload};
use v1::helpers::dispatch::{self, Dispatcher, WithToken};
use v1::traits::Signer;
use v1::types::{TransactionModification, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken, U256, Bytes};

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<D: Dispatcher> {
	signer: Weak<SignerService>,
	accounts: Weak<AccountProvider>,
	dispatcher: D
}

impl<D: Dispatcher + 'static> SignerClient<D> {

	/// Create new instance of signer client.
	pub fn new(
		store: &Arc<AccountProvider>,
		dispatcher: D,
		signer: &Arc<SignerService>,
	) -> Self {
		SignerClient {
			signer: Arc::downgrade(signer),
			accounts: Arc::downgrade(store),
			dispatcher: dispatcher,
		}
	}

	fn confirm_internal<F, T>(&self, id: U256, modification: TransactionModification, f: F) -> BoxFuture<WithToken<ConfirmationResponse>, Error> where
		F: FnOnce(D, Arc<AccountProvider>, ConfirmationPayload) -> T,
		T: IntoFuture<Item=WithToken<ConfirmationResponse>, Error=Error>,
		T::Future: Send + 'static
	{
		let id = id.into();
		let dispatcher = self.dispatcher.clone();

		let setup = || {
			Ok((take_weak!(self.accounts), take_weak!(self.signer)))
		};

		let (accounts, signer) = match setup() {
			Ok(x) => x,
			Err(e) => return future::err(e).boxed(),
		};

		signer.peek(&id).map(|confirmation| {
			let mut payload = confirmation.payload.clone();
			// Modify payload
			if let ConfirmationPayload::SendTransaction(ref mut request) = payload {
				if let Some(sender) = modification.sender.clone() {
					request.from = sender.into();
					// Altering sender should always reset the nonce.
					request.nonce = None;
				}
				if let Some(gas_price) = modification.gas_price {
					request.gas_price = gas_price.into();
				}
				if let Some(gas) = modification.gas {
					request.gas = gas.into();
				}
				if let Some(ref condition) = modification.condition {
					request.condition = condition.clone().map(Into::into);
				}
			}
			let fut = f(dispatcher, accounts, payload);
			fut.into_future().then(move |result| {
				// Execute
				if let Ok(ref response) = result {
					signer.request_confirmed(id, Ok((*response).clone()));
				}

				result
			}).boxed()
		})
		.unwrap_or_else(|| future::err(errors::invalid_params("Unknown RequestID", id)).boxed())
	}
}

impl<D: Dispatcher + 'static> Signer for SignerClient<D> {

	fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>, Error> {
		let signer = take_weak!(self.signer);

		Ok(signer.requests()
			.into_iter()
			.map(Into::into)
			.collect()
		)
	}

	// TODO [ToDr] TransactionModification is redundant for some calls
	// might be better to replace it in future
	fn confirm_request(&self, id: U256, modification: TransactionModification, pass: String)
		-> BoxFuture<ConfirmationResponse, Error>
	{
		self.confirm_internal(id, modification, move |dis, accounts, payload| {
			dispatch::execute(dis, accounts, payload, dispatch::SignWith::Password(pass))
		}).map(|v| v.into_value()).boxed()
	}

	fn confirm_request_with_token(&self, id: U256, modification: TransactionModification, token: String)
		-> BoxFuture<ConfirmationResponseWithToken, Error>
	{
		self.confirm_internal(id, modification, move |dis, accounts, payload| {
			dispatch::execute(dis, accounts, payload, dispatch::SignWith::Token(token))
		}).and_then(|v| match v {
			WithToken::No(_) => Err(errors::internal("Unexpected response without token.", "")),
			WithToken::Yes(response, token) => Ok(ConfirmationResponseWithToken {
				result: response,
				token: token,
			}),
		}).boxed()
	}

	fn confirm_request_raw(&self, id: U256, bytes: Bytes) -> Result<ConfirmationResponse, Error> {
		let id = id.into();
		let signer = take_weak!(self.signer);

		signer.peek(&id).map(|confirmation| {
			let result = match confirmation.payload {
				ConfirmationPayload::SendTransaction(request) => {
					let signed_transaction = UntrustedRlp::new(&bytes.0).as_val().map_err(errors::from_rlp_error)?;
					let signed_transaction = SignedTransaction::new(signed_transaction).map_err(|e| errors::invalid_params("Invalid signature.", e))?;
					let sender = signed_transaction.sender();

					// Verification
					let sender_matches = sender == request.from;
					let data_matches = signed_transaction.data == request.data;
					let value_matches = signed_transaction.value == request.value;
					let nonce_matches = match request.nonce {
						Some(nonce) => signed_transaction.nonce == nonce,
						None => true,
					};

					// Dispatch if everything is ok
					if sender_matches && data_matches && value_matches && nonce_matches {
						let pending_transaction = PendingTransaction::new(signed_transaction, request.condition.map(Into::into));
						self.dispatcher.dispatch_transaction(pending_transaction)
							.map(Into::into)
							.map(ConfirmationResponse::SendTransaction)
					} else {
						let mut error = Vec::new();
						if !sender_matches { error.push("from") }
						if !data_matches { error.push("data") }
						if !value_matches { error.push("value") }
						if !nonce_matches { error.push("nonce") }

						Err(errors::invalid_params("Sent transaction does not match the request.", error))
					}
				},
				// TODO [ToDr]:
				// 1. Sign - verify signature
				// 2. Decrypt - pass through?
				_ => Err(errors::unimplemented(Some("Non-transaction requests does not support RAW signing yet.".into()))),
			};
			if let Ok(ref response) = result {
				signer.request_confirmed(id, Ok(response.clone()));
			}
			result
		}).unwrap_or_else(|| Err(errors::invalid_params("Unknown RequestID", id)))
	}

	fn reject_request(&self, id: U256) -> Result<bool, Error> {
		let signer = take_weak!(self.signer);

		let res = signer.request_rejected(id.into());
		Ok(res.is_some())
	}

	fn generate_token(&self) -> Result<String, Error> {
		let signer = take_weak!(self.signer);

		signer.generate_token()
			.map_err(|e| errors::token(e))
	}

	fn generate_web_proxy_token(&self) -> Result<String, Error> {
		let signer = take_weak!(self.signer);

		Ok(signer.generate_web_proxy_access_token())
	}
}

