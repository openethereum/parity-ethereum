// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Account management (personal) rpc implementation
use std::sync::Arc;
use std::time::Duration;

use bytes::{Bytes, ToPretty};
use ethcore::account_provider::AccountProvider;
use transaction::PendingTransaction;
use ethereum_types::{H520, U128, Address};
use ethkey::{public_to_address, recover, Signature};

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::{future, Future};
use v1::helpers::errors;
use v1::helpers::dispatch::{self, eth_data_hash, Dispatcher, SignWith};
use v1::traits::Personal;
use v1::types::{
	H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, U128 as RpcU128,
	Bytes as RpcBytes,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse,
	TransactionRequest,
	RichRawTransaction as RpcRichRawTransaction,
};
use v1::metadata::Metadata;

/// Account management (personal) rpc implementation.
pub struct PersonalClient<D: Dispatcher> {
	accounts: Arc<AccountProvider>,
	dispatcher: D,
	allow_perm_unlock: bool,
}

impl<D: Dispatcher> PersonalClient<D> {
	/// Creates new PersonalClient
	pub fn new(accounts: &Arc<AccountProvider>, dispatcher: D, allow_perm_unlock: bool) -> Self {
		PersonalClient {
			accounts: accounts.clone(),
			dispatcher,
			allow_perm_unlock,
		}
	}
}

impl<D: Dispatcher + 'static> PersonalClient<D> {
	fn do_sign_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<(PendingTransaction, D)> {
		let dispatcher = self.dispatcher.clone();
		let accounts = self.accounts.clone();

		let default = match request.from.as_ref() {
			Some(account) => Ok(account.clone().into()),
			None => accounts
				.dapp_default_address(meta.dapp_id().into())
				.map_err(|e| errors::account("Cannot find default account.", e)),
		};

		let default = match default {
			Ok(default) => default,
			Err(e) => return Box::new(future::err(e)),
		};

		Box::new(dispatcher.fill_optional_fields(request.into(), default, false)
			.and_then(move |filled| {
				let condition = filled.condition.clone().map(Into::into);
				dispatcher.sign(accounts, filled, SignWith::Password(password))
					.map(|tx| tx.into_value())
					.map(move |tx| PendingTransaction::new(tx, condition))
					.map(move |tx| (tx, dispatcher))
			})
		)
	}
}

impl<D: Dispatcher + 'static> Personal for PersonalClient<D> {
	type Metadata = Metadata;

	fn accounts(&self) -> Result<Vec<RpcH160>> {
		let accounts = self.accounts.accounts().map_err(|e| errors::account("Could not fetch accounts.", e))?;
		Ok(accounts.into_iter().map(Into::into).collect::<Vec<RpcH160>>())
	}

	fn new_account(&self, pass: String) -> Result<RpcH160> {
		self.accounts.new_account(&pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn unlock_account(&self, account: RpcH160, account_pass: String, duration: Option<RpcU128>) -> Result<bool> {
		let account: Address = account.into();
		let store = self.accounts.clone();
		let duration = match duration {
			None => None,
			Some(duration) => {
				let duration: U128 = duration.into();
				let v = duration.low_u64() as u32;
				if duration != v.into() {
					return Err(errors::invalid_params("Duration", "Invalid Number"));
				} else {
					Some(v)
				}
			},
		};

		let r = match (self.allow_perm_unlock, duration) {
			(false, None) => store.unlock_account_temporarily(account, account_pass),
			(false, _) => return Err(errors::unsupported(
				"Time-unlocking is only supported in --geth compatibility mode.",
				Some("Restart your client with --geth flag or use personal_sendTransaction instead."),
			)),
			(true, Some(0)) => store.unlock_account_permanently(account, account_pass),
			(true, Some(d)) => store.unlock_account_timed(account, account_pass, Duration::from_secs(d.into())),
			(true, None) => store.unlock_account_timed(account, account_pass, Duration::from_secs(300)),
		};
		match r {
			Ok(_) => Ok(true),
			Err(err) => Err(errors::account("Unable to unlock the account.", err)),
		}
	}

	fn sign(&self, data: RpcBytes, account: RpcH160, password: String) -> BoxFuture<RpcH520> {
		let dispatcher = self.dispatcher.clone();
		let accounts = self.accounts.clone();

		let payload = RpcConfirmationPayload::EthSignMessage((account.clone(), data).into());

		Box::new(dispatch::from_rpc(payload, account.into(), &dispatcher)
				 .and_then(|payload| {
					 dispatch::execute(dispatcher, accounts, payload, dispatch::SignWith::Password(password))
				 })
				 .map(|v| v.into_value())
				 .then(|res| match res {
					 Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
					 Err(e) => Err(e),
					 e => Err(errors::internal("Unexpected result", e)),
				 }))
	}

	fn ec_recover(&self, data: RpcBytes, signature: RpcH520) -> BoxFuture<RpcH160> {
		let signature: H520 = signature.into();
		let signature = Signature::from_electrum(&signature);
		let data: Bytes = data.into();

		let hash = eth_data_hash(data);
		let account = recover(&signature.into(), &hash)
			.map_err(errors::encryption)
			.map(|public| {
				public_to_address(&public).into()
			});

		Box::new(future::done(account))
	}

	fn sign_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcRichRawTransaction> {
		Box::new(self.do_sign_transaction(meta, request, password)
			.map(|(pending_tx, dispatcher)| dispatcher.enrich(pending_tx.transaction)))
	}

	fn send_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcH256> {
		Box::new(self.do_sign_transaction(meta, request, password)
			.and_then(|(pending_tx, dispatcher)| {
				let chain_id = pending_tx.chain_id();
				trace!(target: "miner", "send_transaction: dispatching tx: {} for chain ID {:?}",
					::rlp::encode(&*pending_tx).into_vec().pretty(), chain_id);

				dispatcher.dispatch_transaction(pending_tx).map(Into::into)
			})
		)
	}

	fn sign_and_send_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcH256> {
		warn!("Using deprecated personal_signAndSendTransaction, use personal_sendTransaction instead.");
		self.send_transaction(meta, request, password)
	}
}
