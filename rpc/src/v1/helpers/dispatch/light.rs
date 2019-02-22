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

use std::sync::Arc;

use ethereum_types::{H256, Address, U256};
use light::TransactionQueue as LightTransactionQueue;
use light::cache::Cache as LightDataCache;
use light::client::LightChainClient;
use light::on_demand::{request, OnDemand};
use parking_lot::{Mutex, RwLock};
use stats::Corpus;
use sync::{LightSyncProvider, LightNetworkDispatcher, ManageNetwork};
use types::basic_account::BasicAccount;
use types::ids::BlockId;
use types::transaction::{SignedTransaction, PendingTransaction, Error as TransactionError};

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::{future, Future, IntoFuture};
use jsonrpc_core::futures::future::Either;
use v1::helpers::{errors, nonce, TransactionRequest, FilledTransactionRequest};
use v1::types::{RichRawTransaction as RpcRichRawTransaction,};

use super::{Dispatcher, Accounts, SignWith, PostSign};

/// Dispatcher for light clients -- fetches default gas price, next nonce, etc. from network.
pub struct LightDispatcher<S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static> {
	/// Sync service.
	pub sync: Arc<S>,
	/// Header chain client.
	pub client: Arc<LightChainClient>,
	/// On-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// Data cache.
	pub cache: Arc<Mutex<LightDataCache>>,
	/// Transaction queue.
	pub transaction_queue: Arc<RwLock<LightTransactionQueue>>,
	/// Nonce reservations
	pub nonces: Arc<Mutex<nonce::Reservations>>,
	/// Gas Price percentile value used as default gas price.
	pub gas_price_percentile: usize,
}

impl<S> LightDispatcher<S>
where
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static
{
	/// Create a new `LightDispatcher` from its requisite parts.
	///
	/// For correct operation, the OnDemand service is assumed to be registered as a network handler,
	pub fn new(
		sync: Arc<S>,
		client: Arc<LightChainClient>,
		on_demand: Arc<OnDemand>,
		cache: Arc<Mutex<LightDataCache>>,
		transaction_queue: Arc<RwLock<LightTransactionQueue>>,
		nonces: Arc<Mutex<nonce::Reservations>>,
		gas_price_percentile: usize,
	) -> Self {
		LightDispatcher {
			sync,
			client,
			on_demand,
			cache,
			transaction_queue,
			nonces,
			gas_price_percentile,
		}
	}

	/// Get a recent gas price corpus.
	// TODO: this could be `impl Trait`.
	pub fn gas_price_corpus(&self) -> BoxFuture<Corpus<U256>> {
		fetch_gas_price_corpus(
			self.sync.clone(),
			self.client.clone(),
			self.on_demand.clone(),
			self.cache.clone(),
		)
	}

	/// Get an account's state
	fn account(&self, addr: Address) -> BoxFuture<Option<BasicAccount>> {
		let best_header = self.client.best_block_header();
		let account_future = self.sync.with_context(|ctx| self.on_demand.request(ctx, request::Account {
			header: best_header.into(),
			address: addr,
		}).expect("no back-references; therefore all back-references valid; qed"));

		match account_future {
			Some(response) => Box::new(response.map_err(|_| errors::no_light_peers())),
			None => Box::new(future::err(errors::network_disabled())),
		}
	}

	/// Get an account's next nonce.
	pub fn next_nonce(&self, addr: Address) -> BoxFuture<U256> {
		let account_start_nonce = self.client.engine().account_start_nonce(self.client.best_block_header().number());
		Box::new(self.account(addr)
			.and_then(move |maybe_account| {
				future::ok(maybe_account.map_or(account_start_nonce, |account| account.nonce))
			})
		)
	}
}

impl<S> Clone for LightDispatcher<S>
where
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static
{
	fn clone(&self) -> Self {
		Self {
			sync: self.sync.clone(),
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			cache: self.cache.clone(),
			transaction_queue: self.transaction_queue.clone(),
			nonces: self.nonces.clone(),
			gas_price_percentile: self.gas_price_percentile
		}
	}
}

impl<S> Dispatcher for LightDispatcher<S>
where
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static
{
	// Ignore the `force_nonce` flag in order to always query the network when fetching the nonce and
	// the account state. If the nonce is specified in the transaction use that nonce instead but do the
	// network request anyway to the account state (balance)
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, _force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>
	{
		const DEFAULT_GAS_PRICE: U256 = U256([0, 0, 0, 21_000_000]);

		let gas_limit = self.client.best_block_header().gas_limit();
		let request_gas_price = request.gas_price.clone();
		let from = request.from.unwrap_or(default_sender);

		let with_gas_price = move |gas_price| {
			let request = request;
			FilledTransactionRequest {
				from: from.clone(),
				used_default_from: request.from.is_none(),
				to: request.to,
				nonce: request.nonce,
				gas_price: gas_price,
				gas: request.gas.unwrap_or_else(|| gas_limit / 3),
				value: request.value.unwrap_or_else(|| 0.into()),
				data: request.data.unwrap_or_else(Vec::new),
				condition: request.condition,
			}
		};

		// fast path for known gas price.
		let gas_price_percentile = self.gas_price_percentile;
		let gas_price = match request_gas_price {
			Some(gas_price) => Either::A(future::ok(with_gas_price(gas_price))),
			None => Either::B(fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone()
			).and_then(move |corp| match corp.percentile(gas_price_percentile) {
				Some(percentile) => Ok(*percentile),
				None => Ok(DEFAULT_GAS_PRICE), // fall back to default on error.
			}).map(with_gas_price))
		};

		let future_account = self.account(from);

		Box::new(gas_price.and_then(move |mut filled| {
			future_account
				.and_then(move |maybe_account| {
					let cost = filled.value.saturating_add(filled.gas.saturating_mul(filled.gas_price));
					match maybe_account {
						Some(ref account) if cost > account.balance => {
							Err(errors::transaction(TransactionError::InsufficientBalance {
								balance: account.balance,
								cost,
							}))
						}
						Some(account) => {
							if filled.nonce.is_none() {
								filled.nonce = Some(account.nonce);
							}
							Ok(filled)
						}
						None => Err(errors::account("Account not found", "")),
					}
				})
		}))
	}

	fn sign<P>(
		&self,
		filled: FilledTransactionRequest,
		signer: &Arc<Accounts>,
		password: SignWith,
		post_sign: P
	) -> BoxFuture<P::Item>
		where
			P: PostSign + 'static,
		    <P::Out as futures::future::IntoFuture>::Future: Send,
	{
		let chain_id = self.client.signing_chain_id();
		let nonce = filled.nonce.expect("nonce is always provided; qed");
		let future = signer.sign_transaction(filled, chain_id, nonce, password)
			.into_future()
			.and_then(move |signed| post_sign.execute(signed));
		Box::new(future)
	}

	fn enrich(&self, signed_transaction: SignedTransaction) -> RpcRichRawTransaction {
		RpcRichRawTransaction::from_signed(signed_transaction)
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256> {
		let hash = signed_transaction.transaction.hash();

		self.transaction_queue.write().import(signed_transaction)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

/// Get a recent gas price corpus.
// TODO: this could be `impl Trait`.
pub fn fetch_gas_price_corpus<S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static>(
	sync: Arc<S>,
	client: Arc<LightChainClient>,
	on_demand: Arc<OnDemand>,
	cache: Arc<Mutex<LightDataCache>>,
) -> BoxFuture<Corpus<U256>> {
	const GAS_PRICE_SAMPLE_SIZE: usize = 100;

	if let Some(cached) = { cache.lock().gas_price_corpus() } {
		return Box::new(future::ok(cached))
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
				// produce a corpus from the vector and cache it.
				// It's later used to get a percentile for default gas price.
				let corpus: ::stats::Corpus<_> = prices.into();
				cache.lock().set_gas_price_corpus(corpus.clone());
				corpus
			})
	});

	match eventual_corpus {
		Some(corp) => Box::new(corp.map_err(|_| errors::no_light_peers())),
		None => Box::new(future::err(errors::network_disabled())),
	}
}
