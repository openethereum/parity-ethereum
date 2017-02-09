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

//! Utilities and helpers for transaction dispatch.

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Weak;

use futures::{future, Future, BoxFuture};
use util::{Address, H520, H256, U256, Uint, Bytes};
use util::sha3::Hashable;

use ethkey::Signature;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, PendingTransaction, Transaction};
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use v1::helpers::{errors, TransactionRequest, FilledTransactionRequest, ConfirmationPayload};
use v1::types::{
	H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes,
	RichRawTransaction as RpcRichRawTransaction,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse,
	SignRequest as RpcSignRequest,
	DecryptRequest as RpcDecryptRequest,
};

/// Has the capability to dispatch, sign, and decrypt.
///
/// Requires a clone implementation, with the implication that it be cheap;
/// usually just bumping a reference count or two.
pub trait Dispatcher: Send + Sync + Clone {
	// TODO: when ATC exist, use zero-cost
	// type Out<T>: IntoFuture<T, Error>

	/// Fill optional fields of a transaction request, fetching gas price but not nonce.
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address)
		-> BoxFuture<FilledTransactionRequest, Error>;

	/// Sign the given transaction request without dispatching, fetching appropriate nonce.
	fn sign(&self, accounts: &AccountProvider, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>, Error>;

	/// "Dispatch" a local transaction.
	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256, Error>;
}

/// A dispatcher which uses references to a client and miner in order to sign
/// requests locally.
#[derive(Debug)]
pub struct FullDispatcher<C, M> {
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C, M> FullDispatcher<C, M> {
	/// Create a `FullDispatcher` from weak references to a client and miner.
	pub fn new(client: Weak<C>, miner: Weak<M>) -> Self {
		FullDispatcher {
			client: client,
			miner: miner,
		}
	}
}

impl<C, M> Clone for FullDispatcher<C, M> {
	fn clone(&self) -> Self {
		FullDispatcher {
			client: self.client.clone(),
			miner: self.miner.clone(),
		}
	}
}

impl<C: MiningBlockChainClient, M: MinerService> Dispatcher for FullDispatcher<C, M> {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address)
		-> BoxFuture<FilledTransactionRequest, Error>
	{
		let (client, miner) = (take_weakf!(self.client), take_weakf!(self.miner));
		let request = request;
		future::ok(FilledTransactionRequest {
			from: request.from.unwrap_or(default_sender),
			used_default_from: request.from.is_none(),
			to: request.to,
			nonce: request.nonce,
			gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(&*client, &*miner)),
			gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
			value: request.value.unwrap_or_else(|| 0.into()),
			data: request.data.unwrap_or_else(Vec::new),
			condition: request.condition,
		}).boxed()
	}

	fn sign(&self, accounts: &AccountProvider, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>, Error>
	{
		let (client, miner) = (take_weakf!(self.client), take_weakf!(self.miner));
		let network_id = client.signing_network_id();
		let address = filled.from;
		future::ok({
			let t = Transaction {
				nonce: filled.nonce
					.or_else(|| miner
						.last_nonce(&filled.from)
						.map(|nonce| nonce + U256::one()))
					.unwrap_or_else(|| client.latest_nonce(&filled.from)),

				action: filled.to.map_or(Action::Create, Action::Call),
				gas: filled.gas,
				gas_price: filled.gas_price,
				value: filled.value,
				data: filled.data,
			};

			let hash = t.hash(network_id);
			let signature = try_bf!(signature(accounts, address, hash, password));

			signature.map(|sig| {
				SignedTransaction::new(t.with_signature(sig, network_id))
					.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
			})
		}).boxed()
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256, Error> {
		let hash = signed_transaction.transaction.hash();

		take_weak!(self.miner).import_own_transaction(&*take_weak!(self.client), signed_transaction)
			.map_err(errors::from_transaction_error)
			.map(|_| hash)
	}
}

/// default MAC to use.
pub const DEFAULT_MAC: [u8; 2] = [0, 0];

/// Single-use account token.
pub type AccountToken = String;

/// Values used to unlock accounts for signing.
#[derive(Debug, Clone, PartialEq)]
pub enum SignWith {
	/// Nothing -- implies the account is already unlocked.
	Nothing,
	/// Unlock with password.
	Password(String),
	/// Unlock with single-use token.
	Token(AccountToken),
}

/// A value, potentially accompanied by a signing token.
#[derive(Debug)]
pub enum WithToken<T: Debug> {
	/// No token.
	No(T),
	/// With token.
	Yes(T, AccountToken),
}

impl<T: Debug> Deref for WithToken<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match *self {
			WithToken::No(ref v) => v,
			WithToken::Yes(ref v, _) => v,
		}
	}
}

impl<T: Debug> WithToken<T> {
	/// Map the value with the given closure, preserving the token.
	pub fn map<S, F>(self, f: F) -> WithToken<S> where
		S: Debug,
		F: FnOnce(T) -> S,
	{
		match self {
			WithToken::No(v) => WithToken::No(f(v)),
			WithToken::Yes(v, token) => WithToken::Yes(f(v), token),
		}
	}

	/// Convert into inner value, ignoring possible token.
	pub fn into_value(self) -> T {
		match self {
			WithToken::No(v) => v,
			WithToken::Yes(v, _) => v,
		}
	}

	/// Convert the `WithToken` into a tuple.
	pub fn into_tuple(self) -> (T, Option<AccountToken>) {
		match self {
			WithToken::No(v) => (v, None),
			WithToken::Yes(v, token) => (v, Some(token))
		}
	}
}

impl<T: Debug> From<(T, AccountToken)> for WithToken<T> {
	fn from(tuple: (T, AccountToken)) -> Self {
		WithToken::Yes(tuple.0, tuple.1)
	}
}

impl<T: Debug> From<(T, Option<AccountToken>)> for WithToken<T> {
	fn from(tuple: (T, Option<AccountToken>)) -> Self {
		match tuple.1 {
			Some(token) => WithToken::Yes(tuple.0, token),
			None => WithToken::No(tuple.0),
		}
	}
}

/// Execute a confirmation payload.
pub fn execute<D: Dispatcher + 'static>(
	dispatcher: D,
	accounts: &AccountProvider,
	payload: ConfirmationPayload,
	pass: SignWith
) -> BoxFuture<WithToken<ConfirmationResponse>, Error> {
	match payload {
		ConfirmationPayload::SendTransaction(request) => {
			let condition = request.condition.clone().map(Into::into);
			dispatcher.sign(accounts, request, pass)
				.map(move |v| v.map(move |tx| PendingTransaction::new(tx, condition)))
				.map(WithToken::into_tuple)
				.map(|(tx, token)| (tx, token, dispatcher))
				.and_then(|(tx, tok, dispatcher)| {
					dispatcher.dispatch_transaction(tx)
						.map(RpcH256::from)
						.map(ConfirmationResponse::SendTransaction)
						.map(move |h| WithToken::from((h, tok)))
				}).boxed()
		},
		ConfirmationPayload::SignTransaction(request) => {
			dispatcher.sign(accounts, request, pass)
				.map(|result| result
					.map(RpcRichRawTransaction::from)
					.map(ConfirmationResponse::SignTransaction)
				).boxed()
		},
		ConfirmationPayload::Signature(address, mut data) => {
			let mut message_data =
				format!("\x19Ethereum Signed Message:\n{}", data.len())
				.into_bytes();
			message_data.append(&mut data);
			let res = signature(accounts, address, message_data.sha3(), pass)
				.map(|result| result
					.map(|rsv| {
						let mut vrs = [0u8; 65];
						let rsv = rsv.as_ref();
						vrs[0] = rsv[64] + 27;
						vrs[1..33].copy_from_slice(&rsv[0..32]);
						vrs[33..65].copy_from_slice(&rsv[32..64]);
						H520(vrs)
					})
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				);
			future::done(res).boxed()
		},
		ConfirmationPayload::Decrypt(address, data) => {
			let res = decrypt(accounts, address, data, pass)
				.map(|result| result
					.map(RpcBytes)
					.map(ConfirmationResponse::Decrypt)
				);
			future::done(res).boxed()
		},
	}
}

fn signature(accounts: &AccountProvider, address: Address, hash: H256, password: SignWith) -> Result<WithToken<Signature>, Error> {
	match password.clone() {
		SignWith::Nothing => accounts.sign(address, None, hash).map(WithToken::No),
		SignWith::Password(pass) => accounts.sign(address, Some(pass), hash).map(WithToken::No),
		SignWith::Token(token) => accounts.sign_with_token(address, token, hash).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::from_signing_error(e),
		_ => errors::from_password_error(e),
	})
}

fn decrypt(accounts: &AccountProvider, address: Address, msg: Bytes, password: SignWith) -> Result<WithToken<Bytes>, Error> {
	match password.clone() {
		SignWith::Nothing => accounts.decrypt(address, None, &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Password(pass) => accounts.decrypt(address, Some(pass), &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Token(token) => accounts.decrypt_with_token(address, token, &DEFAULT_MAC, &msg).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::from_signing_error(e),
		_ => errors::from_password_error(e),
	})
}

/// Extract default gas price from a client and miner.
pub fn default_gas_price<C, M>(client: &C, miner: &M) -> U256
	where C: MiningBlockChainClient, M: MinerService
{
	client.gas_price_median(100).unwrap_or_else(|| miner.sensible_gas_price())
}

/// Convert RPC confirmation payload to signer confirmation payload.
/// May need to resolve in the future to fetch things like gas price.
pub fn from_rpc<D>(payload: RpcConfirmationPayload, default_account: Address, dispatcher: &D) -> BoxFuture<ConfirmationPayload, Error>
	where D: Dispatcher
{
	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			dispatcher.fill_optional_fields(request.into(), default_account)
				.map(ConfirmationPayload::SendTransaction)
				.boxed()
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			dispatcher.fill_optional_fields(request.into(), default_account)
				.map(ConfirmationPayload::SignTransaction)
				.boxed()
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			future::ok(ConfirmationPayload::Decrypt(address.into(), msg.into())).boxed()
		},
		RpcConfirmationPayload::Signature(RpcSignRequest { address, data }) => {
			future::ok(ConfirmationPayload::Signature(address.into(), data.into())).boxed()
		},
	}
}
