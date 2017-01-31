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
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{SignedTransaction, PendingTransaction};
use ethcore::miner::MinerService;

use jsonrpc_core::Error;
use v1::helpers::{errors, SignerService, SigningQueue, ConfirmationPayload};
use v1::helpers::dispatch::{self, dispatch_transaction, WithToken};
use v1::traits::Signer;
use v1::types::{TransactionModification, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken, U256, Bytes};

/// Transactions confirmation (personal) rpc implementation.
pub struct SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {
	signer: Weak<SignerService>,
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C: 'static, M: 'static> SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	/// Create new instance of signer client.
	pub fn new(
		store: &Arc<AccountProvider>,
		client: &Arc<C>,
		miner: &Arc<M>,
		signer: &Arc<SignerService>,
	) -> Self {
		SignerClient {
			signer: Arc::downgrade(signer),
			accounts: Arc::downgrade(store),
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}

	fn confirm_internal<F>(&self, id: U256, modification: TransactionModification, f: F) -> Result<WithToken<ConfirmationResponse>, Error> where
		F: FnOnce(&C, &M, &AccountProvider, ConfirmationPayload) -> Result<WithToken<ConfirmationResponse>, Error>,
	{
		self.active()?;

		let id = id.into();
		let accounts = take_weak!(self.accounts);
		let signer = take_weak!(self.signer);
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);

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
				if let Some(ref min_block) = modification.min_block {
					request.min_block = min_block.as_ref().and_then(|b| b.to_min_block_num());
				}
			}
			let result = f(&*client, &*miner, &*accounts, payload);
			// Execute
			if let Ok(ref response) = result {
				signer.request_confirmed(id, Ok((*response).clone()));
			}
			result
		}).unwrap_or_else(|| Err(errors::invalid_params("Unknown RequestID", id)))
	}
}

impl<C: 'static, M: 'static> Signer for SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>, Error> {
		self.active()?;
		let signer = take_weak!(self.signer);

		Ok(signer.requests()
			.into_iter()
			.map(Into::into)
			.collect()
		)
	}

	// TODO [ToDr] TransactionModification is redundant for some calls
	// might be better to replace it in future
	fn confirm_request(&self, id: U256, modification: TransactionModification, pass: String) -> Result<ConfirmationResponse, Error> {
		self.confirm_internal(id, modification, move |client, miner, accounts, payload| {
			dispatch::execute(client, miner, accounts, payload, dispatch::SignWith::Password(pass))
		}).map(|v| v.into_value())
	}

	fn confirm_request_with_token(&self, id: U256, modification: TransactionModification, token: String) -> Result<ConfirmationResponseWithToken, Error> {
		self.confirm_internal(id, modification, move |client, miner, accounts, payload| {
			dispatch::execute(client, miner, accounts, payload, dispatch::SignWith::Token(token))
		}).and_then(|v| match v {
			WithToken::No(_) => Err(errors::internal("Unexpected response without token.", "")),
			WithToken::Yes(response, token) => Ok(ConfirmationResponseWithToken {
				result: response,
				token: token,
			}),
		})
	}

	fn confirm_request_raw(&self, id: U256, bytes: Bytes) -> Result<ConfirmationResponse, Error> {
		self.active()?;

		let id = id.into();
		let signer = take_weak!(self.signer);
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);

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
						let pending_transaction = PendingTransaction::new(signed_transaction, request.min_block);
						dispatch_transaction(&*client, &*miner, pending_transaction)
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
		self.active()?;
		let signer = take_weak!(self.signer);

		let res = signer.request_rejected(id.into());
		Ok(res.is_some())
	}

	fn generate_token(&self) -> Result<String, Error> {
		self.active()?;
		let signer = take_weak!(self.signer);

		signer.generate_token()
			.map_err(|e| errors::token(e))
	}

	fn generate_web_proxy_token(&self) -> Result<String, Error> {
		try!(self.active());
		let signer = take_weak!(self.signer);

		Ok(signer.generate_web_proxy_access_token())
	}
}

