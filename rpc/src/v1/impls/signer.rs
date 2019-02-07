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

//! Transactions Confirmations rpc implementation

use std::sync::Arc;

use ethkey;
use parity_runtime::Executor;
use parking_lot::Mutex;
use rlp::Rlp;
use types::transaction::{SignedTransaction, PendingTransaction};

use jsonrpc_core::{Result, BoxFuture, Error};
use jsonrpc_core::futures::{future, Future, IntoFuture};
use jsonrpc_core::futures::future::Either;
use jsonrpc_pubsub::{SubscriptionId, typed::{Sink, Subscriber}};
use v1::helpers::deprecated::{self, DeprecationNotice};
use v1::helpers::dispatch::{self, Dispatcher, WithToken, eth_data_hash};
use v1::helpers::{errors, ConfirmationPayload, FilledTransactionRequest, Subscribers};
use v1::helpers::external_signer::{SigningQueue, SignerService};
use v1::metadata::Metadata;
use v1::traits::Signer;
use v1::types::{TransactionModification, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken, U256, Bytes};

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<D: Dispatcher> {
	signer: Arc<SignerService>,
	accounts: Arc<dispatch::Accounts>,
	dispatcher: D,
	subscribers: Arc<Mutex<Subscribers<Sink<Vec<ConfirmationRequest>>>>>,
	deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher + 'static> SignerClient<D> {
	/// Create new instance of signer client.
	pub fn new(
		accounts: Arc<dispatch::Accounts>,
		dispatcher: D,
		signer: &Arc<SignerService>,
		executor: Executor,
	) -> Self {
		let subscribers = Arc::new(Mutex::new(Subscribers::default()));
		let subs = Arc::downgrade(&subscribers);
		let s = Arc::downgrade(signer);
		signer.queue().on_event(move |_event| {
			if let (Some(s), Some(subs)) = (s.upgrade(), subs.upgrade()) {
				let requests = s.requests().into_iter().map(Into::into).collect::<Vec<ConfirmationRequest>>();
				for subscription in subs.lock().values() {
					let subscription: &Sink<_> = subscription;
					executor.spawn(subscription
						.notify(Ok(requests.clone()))
						.map(|_| ())
						.map_err(|e| warn!(target: "rpc", "Unable to send notification: {}", e))
					);
				}
			}
		});

		SignerClient {
			signer: signer.clone(),
			accounts: accounts.clone(),
			dispatcher,
			subscribers,
			deprecation_notice: Default::default(),
		}
	}

	fn confirm_internal<F, T>(&self, id: U256, modification: TransactionModification, f: F) -> BoxFuture<WithToken<ConfirmationResponse>> where
		F: FnOnce(D, &Arc<dispatch::Accounts>, ConfirmationPayload) -> T,
		T: IntoFuture<Item=WithToken<ConfirmationResponse>, Error=Error>,
		T::Future: Send + 'static
	{
		let id = id.into();
		let dispatcher = self.dispatcher.clone();
		let signer = self.signer.clone();

		Box::new(signer.take(&id).map(|sender| {
			let mut payload = sender.request.payload.clone();
			// Modify payload
			if let ConfirmationPayload::SendTransaction(ref mut request) = payload {
				if let Some(sender) = modification.sender {
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
			let fut = f(dispatcher, &self.accounts, payload);
			Either::A(fut.into_future().then(move |result| {
				// Execute
				if let Ok(ref response) = result {
					signer.request_confirmed(sender, Ok((*response).clone()));
				} else {
					signer.request_untouched(sender);
				}

				result
			}))
		})
		.unwrap_or_else(|| Either::B(future::err(errors::invalid_params("Unknown RequestID", id)))))
	}

	fn verify_transaction<F>(bytes: Bytes, request: FilledTransactionRequest, process: F) -> Result<ConfirmationResponse> where
		F: FnOnce(PendingTransaction) -> Result<ConfirmationResponse>,
	{
		let signed_transaction = Rlp::new(&bytes.0).as_val().map_err(errors::rlp)?;
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

	fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>> {
		self.deprecation_notice.print("signer_requestsToConfirm", deprecated::msgs::ACCOUNTS);

		Ok(self.signer.requests()
			.into_iter()
			.map(Into::into)
			.collect()
		)
	}

	// TODO [ToDr] TransactionModification is redundant for some calls
	// might be better to replace it in future
	fn confirm_request(&self, id: U256, modification: TransactionModification, pass: String)
		-> BoxFuture<ConfirmationResponse>
	{
		self.deprecation_notice.print("signer_confirmRequest", deprecated::msgs::ACCOUNTS);

		Box::new(self.confirm_internal(id, modification, move |dis, accounts, payload| {
			dispatch::execute(dis, accounts, payload, dispatch::SignWith::Password(pass.into()))
		}).map(|v| v.into_value()))
	}

	fn confirm_request_with_token(&self, id: U256, modification: TransactionModification, token: String)
		-> BoxFuture<ConfirmationResponseWithToken>
	{
		self.deprecation_notice.print("signer_confirmRequestWithToken", deprecated::msgs::ACCOUNTS);

		Box::new(self.confirm_internal(id, modification, move |dis, accounts, payload| {
			dispatch::execute(dis, accounts, payload, dispatch::SignWith::Token(token.into()))
		}).and_then(|v| match v {
			WithToken::No(_) => Err(errors::internal("Unexpected response without token.", "")),
			WithToken::Yes(response, token) => Ok(ConfirmationResponseWithToken {
				result: response,
				token: token,
			}),
		}))
	}

	fn confirm_request_raw(&self, id: U256, bytes: Bytes) -> Result<ConfirmationResponse> {
		self.deprecation_notice.print("signer_confirmRequestRaw", deprecated::msgs::ACCOUNTS);

		let id = id.into();

		self.signer.take(&id).map(|sender| {
			let payload = sender.request.payload.clone();
			let result = match payload {
				ConfirmationPayload::SendTransaction(request) => {
					Self::verify_transaction(bytes, request, |pending_transaction| {
						self.dispatcher.dispatch_transaction(pending_transaction)
							.map(Into::into)
							.map(ConfirmationResponse::SendTransaction)
					})
				},
				ConfirmationPayload::SignTransaction(request) => {
					Self::verify_transaction(bytes, request, |pending_transaction| {
						let rich = self.dispatcher.enrich(pending_transaction.transaction);
						Ok(ConfirmationResponse::SignTransaction(rich))
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
				ConfirmationPayload::SignMessage(address, hash) => {
					let signature = ethkey::Signature::from_electrum(&bytes.0);
					match ethkey::verify_address(&address, &signature, &hash) {
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
				self.signer.request_confirmed(sender, Ok(response.clone()));
			} else {
				self.signer.request_untouched(sender);
			}
			result
		}).unwrap_or_else(|| Err(errors::invalid_params("Unknown RequestID", id)))
	}

	fn reject_request(&self, id: U256) -> Result<bool> {
		self.deprecation_notice.print("signer_rejectRequest", deprecated::msgs::ACCOUNTS);

		let res = self.signer.take(&id.into()).map(|sender| self.signer.request_rejected(sender));
		Ok(res.is_some())
	}

	fn generate_token(&self) -> Result<String> {
		self.deprecation_notice.print("signer_generateAuthorizationToken", deprecated::msgs::ACCOUNTS);

		self.signer.generate_token()
			.map_err(|e| errors::token(e))
	}

	fn subscribe_pending(&self, _meta: Self::Metadata, sub: Subscriber<Vec<ConfirmationRequest>>) {
		self.deprecation_notice.print("signer_subscribePending", deprecated::msgs::ACCOUNTS);

		self.subscribers.lock().push(sub)
	}

	fn unsubscribe_pending(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
		let res = self.subscribers.lock().remove(&id).is_some();
		Ok(res)
	}
}
