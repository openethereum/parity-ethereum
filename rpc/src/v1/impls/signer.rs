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

use std::sync::Arc;

use ethcore::account_provider::AccountProvider;
use ethcore::transaction::{SignedTransaction, PendingTransaction};
use ethkey;
use futures::{future, BoxFuture, Future, IntoFuture};
use parity_reactor::Remote;
use rlp::UntrustedRlp;
use parking_lot::Mutex;

use jsonrpc_core::{futures, Error};
use jsonrpc_pubsub::SubscriptionId;
use jsonrpc_macros::pubsub::{Sink, Subscriber};
use v1::helpers::accounts::unwrap_provider;
use v1::helpers::dispatch::{self, Dispatcher, WithToken, eth_data_hash};
use v1::helpers::{errors, SignerService, SigningQueue, ConfirmationPayload, FilledTransactionRequest, Subscribers};
use v1::metadata::Metadata;
use v1::traits::Signer;
use v1::types::{TransactionModification, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken, U256, Bytes};

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<D: Dispatcher> {
	signer: Arc<SignerService>,
	accounts: Option<Arc<AccountProvider>>,
	dispatcher: D,
	subscribers: Arc<Mutex<Subscribers<Sink<Vec<ConfirmationRequest>>>>>,
}

impl<D: Dispatcher + 'static> SignerClient<D> {
	/// Create new instance of signer client.
	pub fn new(
		store: &Option<Arc<AccountProvider>>,
		dispatcher: D,
		signer: &Arc<SignerService>,
		remote: Remote,
	) -> Self {
		let subscribers = Arc::new(Mutex::new(Subscribers::default()));
		let subs = Arc::downgrade(&subscribers);
		let s = Arc::downgrade(signer);
		signer.queue().on_event(move |_event| {
			if let (Some(s), Some(subs)) = (s.upgrade(), subs.upgrade()) {
				let requests = s.requests().into_iter().map(Into::into).collect::<Vec<ConfirmationRequest>>();
				for subscription in subs.lock().values() {
					let subscription: &Sink<_> = subscription;
					remote.spawn(subscription
						.notify(Ok(requests.clone()))
						.map(|_| ())
						.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
					);
				}
			}
		});

		SignerClient {
			signer: signer.clone(),
			accounts: store.clone(),
			dispatcher: dispatcher,
			subscribers: subscribers,
		}
	}

	fn account_provider(&self) -> Result<Arc<AccountProvider>, Error> {
		unwrap_provider(&self.accounts)
	}

	fn confirm_internal<F, T>(&self, id: U256, modification: TransactionModification, f: F) -> BoxFuture<WithToken<ConfirmationResponse>, Error> where
		F: FnOnce(D, Arc<AccountProvider>, ConfirmationPayload) -> T,
		T: IntoFuture<Item=WithToken<ConfirmationResponse>, Error=Error>,
		T::Future: Send + 'static
	{
		let id = id.into();
		let dispatcher = self.dispatcher.clone();

		let setup = || {
			Ok((self.account_provider()?, self.signer.clone()))
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

	fn verify_transaction<F>(bytes: Bytes, request: FilledTransactionRequest, process: F) -> Result<ConfirmationResponse, Error> where
		F: FnOnce(PendingTransaction) -> Result<ConfirmationResponse, Error>,
	{
		let signed_transaction = UntrustedRlp::new(&bytes.0).as_val().map_err(errors::rlp)?;
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
			process(pending_transaction)
		} else {
			let mut error = Vec::new();
			if !sender_matches { error.push("from") }
			if !data_matches { error.push("data") }
			if !value_matches { error.push("value") }
			if !nonce_matches { error.push("nonce") }

			Err(errors::invalid_params("Sent transaction does not match the request.", error))
		}
	}
}

impl<D: Dispatcher + 'static> Signer for SignerClient<D> {
	type Metadata = Metadata;

	fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>, Error> {
		Ok(self.signer.requests()
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

		self.signer.peek(&id).map(|confirmation| {
			let result = match confirmation.payload {
				ConfirmationPayload::SendTransaction(request) => {
					Self::verify_transaction(bytes, request, |pending_transaction| {
						self.dispatcher.dispatch_transaction(pending_transaction)
							.map(Into::into)
							.map(ConfirmationResponse::SendTransaction)
					})
				},
				ConfirmationPayload::SignTransaction(request) => {
					Self::verify_transaction(bytes, request, |pending_transaction| {
						Ok(ConfirmationResponse::SignTransaction(pending_transaction.transaction.into()))
					})
				},
				ConfirmationPayload::EthSignMessage(address, data) => {
					let expected_hash = eth_data_hash(data);
					let signature = ethkey::Signature::from_electrum(&bytes.0);
					match ethkey::verify_address(&address, &signature, &expected_hash) {
						Ok(true) => Ok(ConfirmationResponse::Signature(bytes.0.as_slice().into())),
						Ok(false) => Err(errors::invalid_params("Sender address does not match the signature.", ())),
						Err(err) => Err(errors::invalid_params("Invalid signature received.", err)),
					}
				},
				ConfirmationPayload::Decrypt(_address, _data) => {
					// TODO [ToDr]: Decrypt can we verify if the answer is correct?
					Ok(ConfirmationResponse::Decrypt(bytes))
				},
			};
			if let Ok(ref response) = result {
				self.signer.request_confirmed(id, Ok(response.clone()));
			}
			result
		}).unwrap_or_else(|| Err(errors::invalid_params("Unknown RequestID", id)))
	}

	fn reject_request(&self, id: U256) -> Result<bool, Error> {
		let res = self.signer.request_rejected(id.into());
		Ok(res.is_some())
	}

	fn generate_token(&self) -> Result<String, Error> {
		self.signer.generate_token()
			.map_err(|e| errors::token(e))
	}

	fn generate_web_proxy_token(&self, domain: String) -> Result<String, Error> {
		Ok(self.signer.generate_web_proxy_access_token(domain.into()))
	}

	fn subscribe_pending(&self, _meta: Self::Metadata, sub: Subscriber<Vec<ConfirmationRequest>>) {
		self.subscribers.lock().push(sub)
	}

	fn unsubscribe_pending(&self, id: SubscriptionId) -> BoxFuture<bool, Error> {
		let res = self.subscribers.lock().remove(&id).is_some();
		futures::future::ok(res).boxed()
	}
}
