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

//! Utilities and helpers for transaction dispatch.

pub(crate) mod light;
mod full;
mod prospective_signer;

#[cfg(any(test, feature = "accounts"))]
mod signing;
#[cfg(not(any(test, feature = "accounts")))]
mod signing {
	use super::*;
	use v1::helpers::errors;

	/// Dummy signer implementation
	#[derive(Debug, Clone)]
	pub struct Signer;

	impl Signer {
		/// Create new instance of dummy signer (accept any AccountProvider)
		pub fn new<T>(_ap: T) -> Self {
			Signer
		}
	}

	impl super::Accounts for Signer {
		fn sign_transaction(&self, _filled: FilledTransactionRequest, _chain_id: Option<u64>, _nonce: U256, _password: SignWith) -> Result<WithToken<SignedTransaction>> {
			Err(errors::account("Signing unsupported", "See #9997"))
		}

		fn sign_message(&self, _address: Address, _password: SignWith, _hash: SignMessage) -> Result<WithToken<Signature>> {
			Err(errors::account("Signing unsupported", "See #9997"))
		}

		fn decrypt(&self, _address: Address, _password: SignWith, _data: Bytes) -> Result<WithToken<Bytes>> {
			Err(errors::account("Signing unsupported", "See #9997"))
		}

		fn supports_prospective_signing(&self, _address: &Address, _password: &SignWith) -> bool {
			false
		}

		fn default_account(&self) -> Address {
			Default::default()
		}

		fn is_unlocked(&self, _address: &Address) -> bool {
			false
		}
	}
}

pub use self::light::LightDispatcher;
pub use self::full::FullDispatcher;
pub use self::signing::Signer;
pub use v1::helpers::nonce::Reservations;

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use bytes::Bytes;
use ethcore::client::BlockChainClient;
use ethcore::miner::MinerService;
use ethereum_types::{H520, H256, U256, Address};
use ethkey::{Password, Signature};
use hash::keccak;
use types::transaction::{SignedTransaction, PendingTransaction};

use jsonrpc_core::{BoxFuture, Result, Error};
use jsonrpc_core::futures::{future, Future, IntoFuture};
use v1::helpers::{TransactionRequest, FilledTransactionRequest, ConfirmationPayload};
use v1::types::{
	H520 as RpcH520, Bytes as RpcBytes,
	RichRawTransaction as RpcRichRawTransaction,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse,
	EthSignRequest as RpcEthSignRequest,
	EIP191SignRequest as RpcSignRequest,
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
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>;

	/// Sign the given transaction request without dispatching, fetching appropriate nonce.
	fn sign<P>(
		&self,
		filled: FilledTransactionRequest,
		signer: &Arc<Accounts>,
		password: SignWith,
		post_sign: P,
	) -> BoxFuture<P::Item> where
		P: PostSign + 'static,
		<P::Out as futures::future::IntoFuture>::Future: Send;

	/// Converts a `SignedTransaction` into `RichRawTransaction`
	fn enrich(&self, SignedTransaction) -> RpcRichRawTransaction;

	/// "Dispatch" a local transaction.
	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256>;
}

/// Payload to sign
pub enum SignMessage {
	/// Eth-sign kind data (requires prefixing)
	Data(Bytes),
	/// Prefixed data hash
	Hash(H256),
}

/// Abstract transaction signer.
///
/// NOTE This signer is semi-correct, it's a temporary measure to avoid moving too much code.
/// If accounts are ultimately removed all password-dealing endpoints will be wiped out.
pub trait Accounts: Send + Sync {
	/// Sign given filled transaction request for the specified chain_id.
	fn sign_transaction(&self, filled: FilledTransactionRequest, chain_id: Option<u64>, nonce: U256, password: SignWith) -> Result<WithToken<SignedTransaction>>;

	/// Sign given message.
	fn sign_message(&self, address: Address, password: SignWith, hash: SignMessage) -> Result<WithToken<Signature>>;

	/// Decrypt given message.
	fn decrypt(&self, address: Address, password: SignWith, data: Bytes) -> Result<WithToken<Bytes>>;

	/// Returns `true` if the accounts can sign multiple times.
	fn supports_prospective_signing(&self, address: &Address, password: &SignWith) -> bool;

	/// Returns default account.
	fn default_account(&self) -> Address;

	/// Returns true if account is unlocked (i.e. can sign without a password)
	fn is_unlocked(&self, address: &Address) -> bool;
}

/// action to execute after signing
/// e.g importing a transaction into the chain
pub trait PostSign: Send {
	/// item that this PostSign returns
	type Item: Send;
	/// incase you need to perform async PostSign actions
	type Out: IntoFuture<Item = Self::Item, Error = Error> + Send;
	/// perform an action with the signed transaction
	fn execute(self, signer: WithToken<SignedTransaction>) -> Self::Out;
}

impl PostSign for () {
	type Item = WithToken<SignedTransaction>;
	type Out = Result<Self::Item>;
	fn execute(self, signed: WithToken<SignedTransaction>) -> Self::Out {
		Ok(signed)
	}
}

impl<F: Send, T: Send> PostSign for F
	where F: FnOnce(WithToken<SignedTransaction>) -> Result<T>
{
	type Item = T;
	type Out = Result<Self::Item>;
	fn execute(self, signed: WithToken<SignedTransaction>) -> Self::Out {
		(self)(signed)
	}
}

/// Single-use account token.
pub type AccountToken = Password;

/// Values used to unlock accounts for signing.
#[derive(Clone, PartialEq)]
pub enum SignWith {
	/// Nothing -- implies the account is already unlocked.
	Nothing,
	/// Unlock with password.
	Password(Password),
	/// Unlock with single-use token.
	Token(AccountToken),
}

impl SignWith {
	#[cfg(any(test, feature = "accounts"))]
	fn is_password(&self) -> bool {
		if let SignWith::Password(_) = *self {
			true
		} else {
			false
		}
	}
}

/// A value, potentially accompanied by a signing token.
pub enum WithToken<T> {
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
	signer: &Arc<Accounts>,
	payload: ConfirmationPayload,
	pass: SignWith
) -> BoxFuture<WithToken<ConfirmationResponse>> {
	match payload {
		ConfirmationPayload::SendTransaction(request) => {
			let condition = request.condition.clone().map(Into::into);
			let cloned_dispatcher = dispatcher.clone();
			let post_sign = move |with_token_signed: WithToken<SignedTransaction>| {
				let (signed, token) = with_token_signed.into_tuple();
				let signed_transaction = PendingTransaction::new(signed, condition);
				cloned_dispatcher.dispatch_transaction(signed_transaction)
					.map(|hash| (hash, token))
			};

			Box::new(
				dispatcher.sign(request, &signer, pass, post_sign).map(|(hash, token)| {
					WithToken::from((ConfirmationResponse::SendTransaction(hash.into()), token))
				})
			)
		},
		ConfirmationPayload::SignTransaction(request) => {
			Box::new(dispatcher.sign(request, &signer, pass, ())
				.map(move |result| result
					.map(move |tx| dispatcher.enrich(tx))
					.map(ConfirmationResponse::SignTransaction)
				))
		},
		ConfirmationPayload::EthSignMessage(address, data) => {
			let res = signer.sign_message(address, pass, SignMessage::Data(data))
				.map(|result| result
					.map(|s| H520(s.into_electrum()))
			 		.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				);

			Box::new(future::done(res))
		},
		ConfirmationPayload::SignMessage(address, data) => {
			let res = signer.sign_message(address, pass, SignMessage::Hash(data))
				.map(|result| result
					.map(|rsv| H520(rsv.into_electrum()))
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				);

			Box::new(future::done(res))
		},
		ConfirmationPayload::Decrypt(address, data) => {
			let res = signer.decrypt(address, pass, data)
				.map(|result| result
					.map(RpcBytes)
					.map(ConfirmationResponse::Decrypt)
				);
			Box::new(future::done(res))
		},
	}
}

/// Returns a eth_sign-compatible hash of data to sign.
/// The data is prepended with special message to prevent
/// malicious DApps from using the function to sign forged transactions.
pub fn eth_data_hash(mut data: Bytes) -> H256 {
	let mut message_data =
		format!("\x19Ethereum Signed Message:\n{}", data.len())
		.into_bytes();
	message_data.append(&mut data);
	keccak(message_data)
}

/// Extract the default gas price from a client and miner.
pub fn default_gas_price<C, M>(client: &C, miner: &M, percentile: usize) -> U256 where
	C: BlockChainClient,
	M: MinerService,
{
	client.gas_price_corpus(100).percentile(percentile).cloned().unwrap_or_else(|| miner.sensible_gas_price())
}

/// Convert RPC confirmation payload to signer confirmation payload.
/// May need to resolve in the future to fetch things like gas price.
pub fn from_rpc<D>(payload: RpcConfirmationPayload, default_account: Address, dispatcher: &D) -> BoxFuture<ConfirmationPayload>
	where D: Dispatcher
{
	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			Box::new(dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SendTransaction))
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			Box::new(dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SignTransaction))
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			Box::new(future::ok(ConfirmationPayload::Decrypt(address.into(), msg.into())))
		},
		RpcConfirmationPayload::EthSignMessage(RpcEthSignRequest { address, data }) => {
			Box::new(future::ok(ConfirmationPayload::EthSignMessage(address.into(), data.into())))
		},
		RpcConfirmationPayload::EIP191SignMessage(RpcSignRequest { address, data }) => {
			Box::new(future::ok(ConfirmationPayload::SignMessage(address.into(), data.into())))
		},
	}
}
