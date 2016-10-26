// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Transactions Confirmations (personal) rpc implementation

use std::sync::{Arc, Weak};
use jsonrpc_core::*;
use ethcore::account_provider::AccountProvider;
use ethcore::client::MiningBlockChainClient;
use ethcore::miner::MinerService;
use v1::traits::PersonalSigner;
use v1::types::{TransactionModification, ConfirmationRequest, U256};
use v1::helpers::{errors, SignerService, SigningQueue, ConfirmationPayload};
use v1::helpers::params::expect_no_params;
use v1::helpers::dispatch::{sign_and_dispatch, sign, decrypt};

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
}

impl<C: 'static, M: 'static> PersonalSigner for SignerClient<C, M> where C: MiningBlockChainClient, M: MinerService {

	fn requests_to_confirm(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let signer = take_weak!(self.signer);
		Ok(to_value(&signer.requests().into_iter().map(From::from).collect::<Vec<ConfirmationRequest>>()))
	}

	fn confirm_request(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		// TODO [ToDr] TransactionModification is redundant for some calls
		// might be better to replace it in future
		from_params::<(U256, TransactionModification, String)>(params).and_then(
			|(id, modification, pass)| {
				let id = id.into();
				let accounts = take_weak!(self.accounts);
				let signer = take_weak!(self.signer);
				let client = take_weak!(self.client);
				let miner = take_weak!(self.miner);

				signer.peek(&id).map(|confirmation| {
					let result = match confirmation.payload {
						ConfirmationPayload::Transaction(mut request) => {
							// apply modification
							if let Some(gas_price) = modification.gas_price {
								request.gas_price = gas_price.into();
							}
							sign_and_dispatch(&*client, &*miner, &*accounts, request.into(), Some(pass))
						},
						ConfirmationPayload::Sign(address, hash) => {
							sign(&*accounts, address, Some(pass), hash)
						},
						ConfirmationPayload::Decrypt(address, msg) => {
							decrypt(&*accounts, address, Some(pass), msg)
						},
					};
					if let Ok(ref response) = result {
						signer.request_confirmed(id, Ok(response.clone()));
					}
					result
				}).unwrap_or_else(|| Err(errors::invalid_params("Unknown RequestID", id)))
			}
		)
	}

	fn reject_request(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(U256, )>(params).and_then(
			|(id, )| {
				let signer = take_weak!(self.signer);
				let res = signer.request_rejected(id.into());
				Ok(to_value(&res.is_some()))
			}
		)
	}

	fn generate_token(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));
		let signer = take_weak!(self.signer);
		signer.generate_token()
			.map(|token| to_value(&token))
			.map_err(|e| errors::token(e))
	}
}

