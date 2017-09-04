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
use std::sync::Arc;

use futures::{future, Future, BoxFuture};
use light::cache::Cache as LightDataCache;
use light::client::LightChainClient;
use light::on_demand::{request, OnDemand};
use light::TransactionQueue as LightTransactionQueue;
use rlp;
use hash::keccak;
use util::{Address, H520, H256, U256, Bytes};
use parking_lot::{Mutex, RwLock};
use stats::Corpus;

use ethkey::Signature;
use ethsync::LightSync;
use ethcore::ids::BlockId;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, PendingTransaction, Transaction};
use ethcore::account_provider::AccountProvider;
use crypto::DEFAULT_MAC;

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
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest, Error>;

	/// Sign the given transaction request without dispatching, fetching appropriate nonce.
	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>, Error>;

	/// "Dispatch" a local transaction.
	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256, Error>;
}

/// A dispatcher which uses references to a client and miner in order to sign
/// requests locally.
#[derive(Debug)]
pub struct FullDispatcher<C, M> {
	client: Arc<C>,
	miner: Arc<M>,
}

impl<C, M> FullDispatcher<C, M> {
	/// Create a `FullDispatcher` from Arc references to a client and miner.
	pub fn new(client: Arc<C>, miner: Arc<M>) -> Self {
		FullDispatcher {
			client,
			miner,
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

impl<C: MiningBlockChainClient, M: MinerService> FullDispatcher<C, M> {
	fn fill_nonce(nonce: Option<U256>, from: &Address, miner: &M, client: &C) -> U256 {
		nonce
			.or_else(|| miner.last_nonce(from).map(|nonce| nonce + U256::one()))
			.unwrap_or_else(|| client.latest_nonce(from))
	}
}

impl<C: MiningBlockChainClient, M: MinerService> Dispatcher for FullDispatcher<C, M> {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest, Error>
	{
		let (client, miner) = (self.client.clone(), self.miner.clone());
		let request = request;
		let from = request.from.unwrap_or(default_sender);
		let nonce = match force_nonce {
			false => request.nonce,
			true => Some(Self::fill_nonce(request.nonce, &from, &miner, &client)),
		};
		future::ok(FilledTransactionRequest {
			from: from,
			used_default_from: request.from.is_none(),
			to: request.to,
			nonce: nonce,
			gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(&*client, &*miner)),
			gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
			value: request.value.unwrap_or_else(|| 0.into()),
			data: request.data.unwrap_or_else(Vec::new),
			condition: request.condition,
		}).boxed()
	}

	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>, Error>
	{
		let (client, miner) = (self.client.clone(), self.miner.clone());
		let chain_id = client.signing_chain_id();
		let address = filled.from;
		future::done({
			let t = Transaction {
				nonce: Self::fill_nonce(filled.nonce, &filled.from, &miner, &client),
				action: filled.to.map_or(Action::Create, Action::Call),
				gas: filled.gas,
				gas_price: filled.gas_price,
				value: filled.value,
				data: filled.data,
			};

			if accounts.is_hardware_address(address) {
				hardware_signature(&*accounts, address, t, chain_id).map(WithToken::No)
			} else {
				let hash = t.hash(chain_id);
				let signature = try_bf!(signature(&*accounts, address, hash, password));
				Ok(signature.map(|sig| {
					SignedTransaction::new(t.with_signature(sig, chain_id))
						.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
				}))
			}
		}).boxed()
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256, Error> {
		let hash = signed_transaction.transaction.hash();

		self.miner.import_own_transaction(&*self.client, signed_transaction)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

/// Get a recent gas price corpus.
// TODO: this could be `impl Trait`.
pub fn fetch_gas_price_corpus(
	sync: Arc<LightSync>,
	client: Arc<LightChainClient>,
	on_demand: Arc<OnDemand>,
	cache: Arc<Mutex<LightDataCache>>,
) -> BoxFuture<Corpus<U256>, Error> {
	const GAS_PRICE_SAMPLE_SIZE: usize = 100;

	if let Some(cached) = { cache.lock().gas_price_corpus() } {
		return future::ok(cached).boxed()
	}

	let cache = cache.clone();
	let eventual_corpus = sync.with_context(|ctx| {
		// get some recent headers with gas used,
		// and request each of the blocks from the network.
		let block_requests = client.ancestry_iter(BlockId::Latest)
			.filter(|hdr| hdr.gas_used() != U256::default())
			.take(GAS_PRICE_SAMPLE_SIZE)
			.map(|hdr| request::Body(hdr.into()))
			.collect::<Vec<_>>();

		// when the blocks come in, collect gas prices into a vector
		on_demand.request(ctx, block_requests)
			.expect("no back-references; therefore all back-references are valid; qed")
			.map(|bodies| {
				bodies.into_iter().fold(Vec::new(), |mut v, block| {
					for t in block.transaction_views().iter() {
						v.push(t.gas_price())
					}

					v
				})
			})
			.map(move |prices| {
				// produce a corpus from the vector, cache it, and return
				// the median as the intended gas price.
				let corpus: ::stats::Corpus<_> = prices.into();
				cache.lock().set_gas_price_corpus(corpus.clone());
				corpus
			})
	});

	match eventual_corpus {
		Some(corp) => corp.map_err(|_| errors::no_light_peers()).boxed(),
		None => future::err(errors::network_disabled()).boxed(),
	}
}

/// Returns a eth_sign-compatible hash of data to sign.
/// The data is prepended with special message to prevent
/// chosen-plaintext attacks.
pub fn eth_data_hash(mut data: Bytes) -> H256 {
	let mut message_data =
		format!("\x19Ethereum Signed Message:\n{}", data.len())
		.into_bytes();
	message_data.append(&mut data);
	keccak(message_data)
}

/// Dispatcher for light clients -- fetches default gas price, next nonce, etc. from network.
#[derive(Clone)]
pub struct LightDispatcher {
	/// Sync service.
	pub sync: Arc<LightSync>,
	/// Header chain client.
	pub client: Arc<LightChainClient>,
	/// On-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// Data cache.
	pub cache: Arc<Mutex<LightDataCache>>,
	/// Transaction queue.
	pub transaction_queue: Arc<RwLock<LightTransactionQueue>>,
}

impl LightDispatcher {
	/// Create a new `LightDispatcher` from its requisite parts.
	///
	/// For correct operation, the OnDemand service is assumed to be registered as a network handler,
	pub fn new(
		sync: Arc<LightSync>,
		client: Arc<LightChainClient>,
		on_demand: Arc<OnDemand>,
		cache: Arc<Mutex<LightDataCache>>,
		transaction_queue: Arc<RwLock<LightTransactionQueue>>,
	) -> Self {
		LightDispatcher {
			sync: sync,
			client: client,
			on_demand: on_demand,
			cache: cache,
			transaction_queue: transaction_queue,
		}
	}

	/// Get a recent gas price corpus.
	// TODO: this could be `impl Trait`.
	pub fn gas_price_corpus(&self) -> BoxFuture<Corpus<U256>, Error> {
		fetch_gas_price_corpus(
			self.sync.clone(),
			self.client.clone(),
			self.on_demand.clone(),
			self.cache.clone(),
		)
	}

	/// Get an account's next nonce.
	pub fn next_nonce(&self, addr: Address) -> BoxFuture<U256, Error> {
		// fast path where we don't go to network; nonce provided or can be gotten from queue.
		let maybe_nonce = self.transaction_queue.read().next_nonce(&addr);
		if let Some(nonce) = maybe_nonce {
			return future::ok(nonce).boxed()
		}

		let best_header = self.client.best_block_header();
		let account_start_nonce = self.client.engine().account_start_nonce(best_header.number());
		let nonce_future = self.sync.with_context(|ctx| self.on_demand.request(ctx, request::Account {
			header: best_header.into(),
			address: addr,
		}).expect("no back-references; therefore all back-references valid; qed"));

		match nonce_future {
			Some(x) =>
				x.map(move |acc| acc.map_or(account_start_nonce, |acc| acc.nonce))
					.map_err(|_| errors::no_light_peers())
					.boxed(),
			None =>  future::err(errors::network_disabled()).boxed()
		}
	}
}

impl Dispatcher for LightDispatcher {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest, Error>
	{
		const DEFAULT_GAS_PRICE: U256 = U256([0, 0, 0, 21_000_000]);

		let gas_limit = self.client.best_block_header().gas_limit();
		let request_gas_price = request.gas_price.clone();
		let request_nonce = request.nonce.clone();
		let from = request.from.unwrap_or(default_sender);

		let with_gas_price = move |gas_price| {
			let request = request;
			FilledTransactionRequest {
				from: from.clone(),
				used_default_from: request.from.is_none(),
				to: request.to,
				nonce: request.nonce,
				gas_price: gas_price,
				gas: request.gas.unwrap_or_else(|| gas_limit / 3.into()),
				value: request.value.unwrap_or_else(|| 0.into()),
				data: request.data.unwrap_or_else(Vec::new),
				condition: request.condition,
			}
		};

		// fast path for known gas price.
		let gas_price = match request_gas_price {
			Some(gas_price) => future::ok(with_gas_price(gas_price)).boxed(),
			None => fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone()
			).and_then(|corp| match corp.median() {
				Some(median) => future::ok(*median),
				None => future::ok(DEFAULT_GAS_PRICE), // fall back to default on error.
			}).map(with_gas_price).boxed()
		};

		match (request_nonce, force_nonce) {
			(_, false) | (Some(_), true) => gas_price,
			(None, true) => {
				let next_nonce = self.next_nonce(from);
				gas_price.and_then(move |mut filled| next_nonce
					.map_err(|_| errors::no_light_peers())
					.map(move |nonce| {
						filled.nonce = Some(nonce);
						filled
					})
				).boxed()
			},
		}
	}

	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>, Error>
	{
		let chain_id = self.client.signing_chain_id();
		let address = filled.from;

		let with_nonce = move |filled: FilledTransactionRequest, nonce| {
			let t = Transaction {
				nonce: nonce,
				action: filled.to.map_or(Action::Create, Action::Call),
				gas: filled.gas,
				gas_price: filled.gas_price,
				value: filled.value,
				data: filled.data,
			};

			if accounts.is_hardware_address(address) {
				return hardware_signature(&*accounts, address, t, chain_id).map(WithToken::No)
			}

			let hash = t.hash(chain_id);
			let signature = signature(&*accounts, address, hash, password)?;

			Ok(signature.map(|sig| {
				SignedTransaction::new(t.with_signature(sig, chain_id))
					.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
			}))
		};

		// fast path for pre-filled nonce.
		if let Some(nonce) = filled.nonce {
			return future::done(with_nonce(filled, nonce)).boxed()
		}

		self.next_nonce(address)
			.map_err(|_| errors::no_light_peers())
			.and_then(move |nonce| with_nonce(filled, nonce))
			.boxed()
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256, Error> {
		let hash = signed_transaction.transaction.hash();

		self.transaction_queue.write().import(signed_transaction)
			.map_err(Into::into)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

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
	accounts: Arc<AccountProvider>,
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
		ConfirmationPayload::EthSignMessage(address, data) => {
			if accounts.is_hardware_address(address) {
				return future::err(errors::unsupported("Signing via hardware wallets is not supported.", None)).boxed();
			}

			let hash = eth_data_hash(data);
			let res = signature(&accounts, address, hash, pass)
				.map(|result| result
					.map(|rsv| H520(rsv.into_electrum()))
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				);
			future::done(res).boxed()
		},
		ConfirmationPayload::Decrypt(address, data) => {
			if accounts.is_hardware_address(address) {
				return future::err(errors::unsupported("Decrypting via hardware wallets is not supported.", None)).boxed();
			}

			let res = decrypt(&accounts, address, data, pass)
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
		SignWith::Nothing => errors::signing(e),
		_ => errors::password(e),
	})
}

// obtain a hardware signature from the given account.
fn hardware_signature(accounts: &AccountProvider, address: Address, t: Transaction, chain_id: Option<u64>)
	-> Result<SignedTransaction, Error>
{
	debug_assert!(accounts.is_hardware_address(address));

	let mut stream = rlp::RlpStream::new();
	t.rlp_append_unsigned_transaction(&mut stream, chain_id);
	let signature = accounts.sign_with_hardware(address, &stream.as_raw())
		.map_err(|e| {
			debug!(target: "miner", "Error signing transaction with hardware wallet: {}", e);
			errors::account("Error signing transaction with hardware wallet", e)
		})?;

	SignedTransaction::new(t.with_signature(signature, chain_id))
		.map_err(|e| {
		  debug!(target: "miner", "Hardware wallet has produced invalid signature: {}", e);
		  errors::account("Invalid signature generated", e)
		})
}

fn decrypt(accounts: &AccountProvider, address: Address, msg: Bytes, password: SignWith) -> Result<WithToken<Bytes>, Error> {
	match password.clone() {
		SignWith::Nothing => accounts.decrypt(address, None, &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Password(pass) => accounts.decrypt(address, Some(pass), &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Token(token) => accounts.decrypt_with_token(address, token, &DEFAULT_MAC, &msg).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::signing(e),
		_ => errors::password(e),
	})
}

/// Extract the default gas price from a client and miner.
pub fn default_gas_price<C, M>(client: &C, miner: &M) -> U256 where
	C: MiningBlockChainClient,
	M: MinerService,
{
	client.gas_price_corpus(100).median().cloned().unwrap_or_else(|| miner.sensible_gas_price())
}

/// Convert RPC confirmation payload to signer confirmation payload.
/// May need to resolve in the future to fetch things like gas price.
pub fn from_rpc<D>(payload: RpcConfirmationPayload, default_account: Address, dispatcher: &D) -> BoxFuture<ConfirmationPayload, Error>
	where D: Dispatcher
{
	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SendTransaction)
				.boxed()
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SignTransaction)
				.boxed()
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			future::ok(ConfirmationPayload::Decrypt(address.into(), msg.into())).boxed()
		},
		RpcConfirmationPayload::EthSignMessage(RpcSignRequest { address, data }) => {
			future::ok(ConfirmationPayload::EthSignMessage(address.into(), data.into())).boxed()
		},
	}
}
