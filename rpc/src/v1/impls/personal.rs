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

//! Account management (personal) rpc implementation
use std::sync::Arc;
use std::time::Duration;

use accounts::AccountProvider;
use bytes::Bytes;
use eip_712::{EIP712, hash_structured_data};
use ethereum_types::{H520, U128, Address};
use ethkey::{public_to_address, recover, Signature};
use types::transaction::{PendingTransaction, SignedTransaction};

use jsonrpc_core::futures::{future, Future};
use jsonrpc_core::types::Value;
use jsonrpc_core::{BoxFuture, Result};
use v1::helpers::deprecated::{self, DeprecationNotice};
use v1::helpers::dispatch::{self, eth_data_hash, Dispatcher, SignWith, PostSign, WithToken};
use v1::helpers::{errors, eip191};
use v1::metadata::Metadata;
use v1::traits::Personal;
use v1::types::{
	H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, U128 as RpcU128,
	Bytes as RpcBytes,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse,
	TransactionRequest,
	RichRawTransaction as RpcRichRawTransaction,
	EIP191Version,
};

/// Account management (personal) rpc implementation.
pub struct PersonalClient<D: Dispatcher> {
	accounts: Arc<AccountProvider>,
	dispatcher: D,
	allow_perm_unlock: bool,
	allow_experimental_rpcs: bool,
	deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher> PersonalClient<D> {
	/// Creates new PersonalClient
	pub fn new(
		accounts: &Arc<AccountProvider>,
		dispatcher: D,
		allow_perm_unlock: bool,
		allow_experimental_rpcs: bool,
	) -> Self {
		PersonalClient {
			accounts: accounts.clone(),
			dispatcher,
			allow_perm_unlock,
			allow_experimental_rpcs,
			deprecation_notice: DeprecationNotice::default(),
		}
	}
}

impl<D: Dispatcher + 'static> PersonalClient<D> {
	fn do_sign_transaction<P>(
		&self,
		_meta: Metadata,
		request: TransactionRequest,
		password: String,
		post_sign: P
 	) -> BoxFuture<P::Item>
		where P: PostSign + 'static,
 		      <P::Out as futures::future::IntoFuture>::Future: Send
	{
		let dispatcher = self.dispatcher.clone();
		let accounts = self.accounts.clone();

		let default = match request.from.as_ref() {
			Some(account) => Ok(account.clone().into()),
			None => accounts
				.default_account()
				.map_err(|e| errors::account("Cannot find default account.", e)),
		};

		let default = match default {
			Ok(default) => default,
			Err(e) => return Box::new(future::err(e)),
		};

		let accounts = Arc::new(dispatch::Signer::new(accounts)) as _;
		Box::new(dispatcher.fill_optional_fields(request.into(), default, false)
			.and_then(move |filled| {
				dispatcher.sign(filled, &accounts, SignWith::Password(password.into()), post_sign)
			})
		)
	}
}

impl<D: Dispatcher + 'static> Personal for PersonalClient<D> {
	type Metadata = Metadata;

	fn accounts(&self) -> Result<Vec<RpcH160>> {
		self.deprecation_notice.print("personal_accounts", deprecated::msgs::ACCOUNTS);

		let accounts = self.accounts.accounts().map_err(|e| errors::account("Could not fetch accounts.", e))?;
		Ok(accounts.into_iter().map(Into::into).collect::<Vec<RpcH160>>())
	}

	fn new_account(&self, pass: String) -> Result<RpcH160> {
		self.deprecation_notice.print("personal_newAccount", deprecated::msgs::ACCOUNTS);

		self.accounts.new_account(&pass.into())
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn unlock_account(&self, account: RpcH160, account_pass: String, duration: Option<RpcU128>) -> Result<bool> {
		self.deprecation_notice.print("personal_unlockAccount", deprecated::msgs::ACCOUNTS);

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
			(false, None) => store.unlock_account_temporarily(account, account_pass.into()),
			(false, _) => return Err(errors::unsupported(
				"Time-unlocking is not supported when permanent unlock is disabled.",
				Some("Use personal_sendTransaction or enable permanent unlocking, instead."),
			)),
			(true, Some(0)) => store.unlock_account_permanently(account, account_pass.into()),
			(true, Some(d)) => store.unlock_account_timed(account, account_pass.into(), Duration::from_secs(d.into())),
			(true, None) => store.unlock_account_timed(account, account_pass.into(), Duration::from_secs(300)),
		};
		match r {
			Ok(_) => Ok(true),
			Err(err) => Err(errors::account("Unable to unlock the account.", err)),
		}
	}

	fn sign(&self, data: RpcBytes, account: RpcH160, password: String) -> BoxFuture<RpcH520> {
		self.deprecation_notice.print("personal_sign", deprecated::msgs::ACCOUNTS);

		let dispatcher = self.dispatcher.clone();
		let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

		let payload = RpcConfirmationPayload::EthSignMessage((account.clone(), data).into());

		Box::new(dispatch::from_rpc(payload, account.into(), &dispatcher)
				 .and_then(move |payload| {
					 dispatch::execute(dispatcher, &accounts, payload, dispatch::SignWith::Password(password.into()))
				 })
				 .map(|v| v.into_value())
				 .then(|res| match res {
					 Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
					 Err(e) => Err(e),
					 e => Err(errors::internal("Unexpected result", e)),
				 }))
	}

	fn sign_191(&self, version: EIP191Version, data: Value, account: RpcH160, password: String) -> BoxFuture<RpcH520> {
		self.deprecation_notice.print("personal_sign191", deprecated::msgs::ACCOUNTS);

		try_bf!(errors::require_experimental(self.allow_experimental_rpcs, "191"));

		let data = try_bf!(eip191::hash_message(version, data));
		let dispatcher = self.dispatcher.clone();
		let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

		let payload = RpcConfirmationPayload::EIP191SignMessage((account.clone(), data.into()).into());

		Box::new(dispatch::from_rpc(payload, account.into(), &dispatcher)
			.and_then(move |payload| {
				dispatch::execute(dispatcher, &accounts, payload, dispatch::SignWith::Password(password.into()))
			})
			.map(|v| v.into_value())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
		)
	}

	fn sign_typed_data(&self, typed_data: EIP712, account: RpcH160, password: String) -> BoxFuture<RpcH520> {
		self.deprecation_notice.print("personal_signTypedData", deprecated::msgs::ACCOUNTS);

		try_bf!(errors::require_experimental(self.allow_experimental_rpcs, "712"));

		let data = match hash_structured_data(typed_data) {
			Ok(d) => d,
			Err(err) => return Box::new(future::err(errors::invalid_call_data(err.kind()))),
		};
		let dispatcher = self.dispatcher.clone();
		let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

		let payload = RpcConfirmationPayload::EIP191SignMessage((account.clone(), data.into()).into());

		Box::new(dispatch::from_rpc(payload, account.into(), &dispatcher)
			.and_then(move |payload| {
				dispatch::execute(dispatcher, &accounts, payload, dispatch::SignWith::Password(password.into()))
			})
			.map(|v| v.into_value())
			.then(|res| match res {
				Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
				Err(e) => Err(e),
				e => Err(errors::internal("Unexpected result", e)),
			})
		)
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
		self.deprecation_notice.print("personal_signTransaction", deprecated::msgs::ACCOUNTS);

		let condition = request.condition.clone().map(Into::into);
		let dispatcher = self.dispatcher.clone();
		Box::new(self.do_sign_transaction(meta, request, password, ())
			.map(move |tx| PendingTransaction::new(tx.into_value(), condition))
			.map(move |pending_tx| dispatcher.enrich(pending_tx.transaction)))
	}

	fn send_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcH256> {
		self.deprecation_notice.print("personal_sendTransaction", deprecated::msgs::ACCOUNTS);

		let condition = request.condition.clone().map(Into::into);
		let dispatcher = self.dispatcher.clone();
		Box::new(
			self.do_sign_transaction(meta, request, password,  move |signed: WithToken<SignedTransaction>| {
				dispatcher.dispatch_transaction(
					PendingTransaction::new(
						signed.into_value(),
						condition
					)
				)
			}).and_then(|hash| {
				Ok(RpcH256::from(hash))
			})
		)
	}

	fn sign_and_send_transaction(&self, meta: Metadata, request: TransactionRequest, password: String) -> BoxFuture<RpcH256> {
		self.deprecation_notice.print("personal_signAndSendTransaction", Some("use personal_sendTransaction instead."));

		self.send_transaction(meta, request, password)
	}
}
