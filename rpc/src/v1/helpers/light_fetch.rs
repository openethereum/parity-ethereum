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

//! Helpers for fetching blockchain data either from the light client or the network.

use std::sync::Arc;

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::executed::{Executed, ExecutionError};
use ethcore::ids::BlockId;
use ethcore::transaction::{Action, Transaction as EthTransaction};

use futures::{future, Future, BoxFuture};
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use light::cache::Cache;
use light::client::LightChainClient;
use light::cht;
use light::on_demand::{OnDemand, request};

use ethsync::LightSync;
use util::{Address, Mutex, Uint, U256};

use v1::helpers::{CallRequest as CallRequestHelper, errors, dispatch};
use v1::types::{BlockNumber, CallRequest};

/// Helper for fetching blockchain data either from the light client or the network
/// as necessary.
pub struct LightFetch {
	/// The light client.
	pub client: Arc<LightChainClient>,
	/// The on-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// Handle to the network.
	pub sync: Arc<LightSync>,
	/// The light data cache.
	pub cache: Arc<Mutex<Cache>>,
}

/// Type alias for convenience.
pub type ExecutionResult = Result<Executed, ExecutionError>;

impl LightFetch {
	/// Get a block header from the on demand service or client, or error.
	pub fn header(&self, id: BlockId) -> BoxFuture<encoded::Header, Error> {
		if let Some(h) = self.client.block_header(id) {
			return future::ok(h).boxed()
		}

		let maybe_future = match id {
			BlockId::Number(n) => {
				let cht_root = cht::block_to_cht_number(n).and_then(|cn| self.client.cht_root(cn as usize));
				match cht_root {
					None => return future::err(errors::unknown_block()).boxed(),
					Some(root) => {
						let req = request::HeaderProof::new(n, root)
							.expect("only fails for 0; client always stores genesis; client already queried; qed");

						let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());
						self.sync.with_context(|ctx| {
							let fut = self.on_demand.hash_by_number(ctx, req)
								.map(request::HeaderByHash)
								.map_err(errors::on_demand_cancel);

							fut.and_then(move |req| {
								match sync.with_context(|ctx| on_demand.header_by_hash(ctx, req)) {
									Some(fut) => fut.map_err(errors::on_demand_cancel).boxed(),
									None => future::err(errors::network_disabled()).boxed(),
								}
							}).boxed()
						})
					}
				}
			}
			BlockId::Hash(h) => {
				self.sync.with_context(|ctx|
					self.on_demand.header_by_hash(ctx, request::HeaderByHash(h))
						.then(|res| future::done(match res {
							Ok(h) => Ok(h),
							Err(e) => Err(errors::on_demand_cancel(e)),
						}))
						.boxed()
				)
			}
			_ => None, // latest, earliest, and pending will have all already returned.
		};

		match maybe_future {
			Some(recv) => recv,
			None => future::err(errors::network_disabled()).boxed()
		}
	}

	/// helper for getting account info at a given block.
	/// `None` indicates the account doesn't exist at the given block.
	pub fn account(&self, address: Address, id: BlockId) -> BoxFuture<Option<BasicAccount>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.header(id).and_then(move |header| {
			let maybe_fut = sync.with_context(|ctx| on_demand.account(ctx, request::Account {
				header: header,
				address: address,
			}));

			match maybe_fut {
				Some(fut) => fut.map_err(errors::on_demand_cancel).boxed(),
				None => future::err(errors::network_disabled()).boxed(),
			}
		}).boxed()
	}

	/// helper for getting proved execution.
	pub fn proved_execution(&self, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<ExecutionResult, Error> {
		const DEFAULT_GAS_PRICE: U256 = U256([0, 0, 0, 21_000_000]);

		let (sync, on_demand, client) = (self.sync.clone(), self.on_demand.clone(), self.client.clone());
		let req: CallRequestHelper = req.into();
		let id = num.0.into();

		let from = req.from.unwrap_or(Address::zero());
		let nonce_fut = match req.nonce {
			Some(nonce) => future::ok(Some(nonce)).boxed(),
			None => self.account(from, id).map(|acc| acc.map(|a| a.nonce)).boxed(),
		};

		let gas_price_fut = match req.gas_price {
			Some(price) => future::ok(price).boxed(),
			None => dispatch::fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone(),
			).map(|corp| match corp.median() {
				Some(median) => *median,
				None => DEFAULT_GAS_PRICE,
			}).boxed()
		};

		// if nonce resolves, this should too since it'll be in the LRU-cache.
		let header_fut = self.header(id);

		// fetch missing transaction fields from the network.
		nonce_fut.join(gas_price_fut).and_then(move |(nonce, gas_price)| {
			let action = req.to.map_or(Action::Create, Action::Call);
			let gas = req.gas.unwrap_or(U256::from(10_000_000)); // better gas amount?
			let value = req.value.unwrap_or_else(U256::zero);
			let data = req.data.map_or_else(Vec::new, |d| d.to_vec());

			future::done(match nonce {
				Some(n) => Ok(EthTransaction {
					nonce: n,
					action: action,
					gas: gas,
					gas_price: gas_price,
					value: value,
					data: data,
				}.fake_sign(from)),
				None => Err(errors::unknown_block()),
			})
		}).join(header_fut).and_then(move |(tx, hdr)| {
			// then request proved execution.
			// TODO: get last-hashes from network.
			let env_info = match client.env_info(id) {
				Some(env_info) => env_info,
				_ => return future::err(errors::unknown_block()).boxed(),
			};

			let request = request::TransactionProof {
				tx: tx,
				header: hdr,
				env_info: env_info,
				engine: client.engine().clone(),
			};

			let proved_future = sync.with_context(move |ctx| {
				on_demand.transaction_proof(ctx, request).map_err(errors::on_demand_cancel).boxed()
			});

			match proved_future {
				Some(fut) => fut.boxed(),
				None => future::err(errors::network_disabled()).boxed(),
			}
		}).boxed()
	}

	/// get a block itself. fails on unknown block ID.
	pub fn block(&self, id: BlockId) -> BoxFuture<encoded::Block, Error> {
		let (on_demand, sync) = (self.on_demand.clone(), self.sync.clone());

		self.header(id).map(request::Body::new).and_then(move |req| {
			match sync.with_context(move |ctx| on_demand.block(ctx, req)) {
				Some(fut) => fut.map_err(errors::on_demand_cancel).boxed(),
				None => future::err(errors::network_disabled()).boxed(),
			}
		}).boxed()
	}
}
