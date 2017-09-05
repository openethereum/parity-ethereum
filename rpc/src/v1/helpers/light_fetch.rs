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
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::transaction::{Action, Transaction as EthTransaction};

use futures::{future, Future, BoxFuture};
use futures::future::Either;
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use light::cache::Cache;
use light::client::LightChainClient;
use light::cht;
use light::on_demand::{request, OnDemand, HeaderRef, Request as OnDemandRequest, Response as OnDemandResponse};
use light::request::Field;

use ethsync::LightSync;
use util::{Address, U256};
use parking_lot::Mutex;

use v1::helpers::{CallRequest as CallRequestHelper, errors, dispatch};
use v1::types::{BlockNumber, CallRequest, Log};

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

// extract the header indicated by the given `HeaderRef` from the given responses.
// fails only if they do not correspond.
fn extract_header(res: &[OnDemandResponse], header: HeaderRef) -> Option<encoded::Header> {
	match header {
		HeaderRef::Stored(hdr) => Some(hdr),
		HeaderRef::Unresolved(idx, _) => match res.get(idx) {
			Some(&OnDemandResponse::HeaderByHash(ref hdr)) => Some(hdr.clone()),
			_ => None,
		},
	}
}

impl LightFetch {
	// push the necessary requests onto the request chain to get the header by the given ID.
	// yield a header reference which other requests can use.
	fn make_header_requests(&self, id: BlockId, reqs: &mut Vec<OnDemandRequest>) -> Result<HeaderRef, Error> {
		if let Some(h) = self.client.block_header(id) {
			return Ok(h.into());
		}

		match id {
			BlockId::Number(n) => {
				let cht_root = cht::block_to_cht_number(n).and_then(|cn| self.client.cht_root(cn as usize));
				match cht_root {
					None => Err(errors::unknown_block()),
					Some(root) => {
						let req = request::HeaderProof::new(n, root)
							.expect("only fails for 0; client always stores genesis; client already queried; qed");

						let idx = reqs.len();
						let hash_ref = Field::back_ref(idx, 0);
						reqs.push(req.into());
						reqs.push(request::HeaderByHash(hash_ref.clone()).into());

						Ok(HeaderRef::Unresolved(idx + 1, hash_ref))
					}
				}
			}
			BlockId::Hash(h) => {
				reqs.push(request::HeaderByHash(h.into()).into());

				let idx = reqs.len();
				Ok(HeaderRef::Unresolved(idx, h.into()))
			}
			_ => Err(errors::unknown_block()) // latest, earliest, and pending will have all already returned.
		}
	}

	/// Get a block header from the on demand service or client, or error.
	pub fn header(&self, id: BlockId) -> BoxFuture<encoded::Header, Error> {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return future::err(e).boxed(),
		};

		let maybe_future = self.sync.with_context(move |ctx| {
			self.on_demand.request_raw(ctx, reqs)
				.expect("all back-references known to be valid; qed")
				.map(|res| extract_header(&res, header_ref)
					.expect("these responses correspond to requests that header_ref belongs to. \
							 therefore it will not fail; qed"))
				.map_err(errors::on_demand_cancel)
				.boxed()
		});

		match maybe_future {
			Some(recv) => recv,
			None => future::err(errors::network_disabled()).boxed()
		}
	}

	/// helper for getting account info at a given block.
	/// `None` indicates the account doesn't exist at the given block.
	pub fn account(&self, address: Address, id: BlockId) -> BoxFuture<Option<BasicAccount>, Error> {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return future::err(e).boxed(),
		};

		reqs.push(request::Account { header: header_ref, address: address }.into());

		let maybe_future = self.sync.with_context(move |ctx| {
			self.on_demand.request_raw(ctx, reqs)
				.expect("all back-references known to be valid; qed")
				.map(|mut res| match res.pop() {
					Some(OnDemandResponse::Account(acc)) => acc,
					_ => panic!("responses correspond directly with requests in amount and type; qed"),
				})
				.map_err(errors::on_demand_cancel)
				.boxed()
		});

		match maybe_future {
			Some(recv) => recv,
			None => future::err(errors::network_disabled()).boxed()
		}
	}

	/// helper for getting proved execution.
	pub fn proved_execution(&self, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<ExecutionResult, Error> {
		const DEFAULT_GAS_PRICE: u64 = 21_000;
		// starting gas when gas not provided.
		const START_GAS: u64 = 50_000;

		let (sync, on_demand, client) = (self.sync.clone(), self.on_demand.clone(), self.client.clone());
		let req: CallRequestHelper = req.into();
		let id = num.unwrap_or_default().into();

		let from = req.from.unwrap_or(Address::zero());
		let nonce_fut = match req.nonce {
			Some(nonce) => Either::A(future::ok(Some(nonce))),
			None => Either::B(self.account(from, id).map(|acc| acc.map(|a| a.nonce))),
		};

		let gas_price_fut = match req.gas_price {
			Some(price) => Either::A(future::ok(price)),
			None => Either::B(dispatch::fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone(),
			).map(|corp| match corp.median() {
				Some(median) => *median,
				None => DEFAULT_GAS_PRICE.into(),
			}))
		};

		// if nonce resolves, this should too since it'll be in the LRU-cache.
		let header_fut = self.header(id);

		// fetch missing transaction fields from the network.
		nonce_fut.join(gas_price_fut).and_then(move |(nonce, gas_price)| {
			let action = req.to.map_or(Action::Create, Action::Call);
			let value = req.value.unwrap_or_else(U256::zero);
			let data = req.data.unwrap_or_default();

			future::done(match (nonce, req.gas) {
				(Some(n), Some(gas)) => Ok((true, EthTransaction {
					nonce: n,
					action: action,
					gas: gas,
					gas_price: gas_price,
					value: value,
					data: data,
				})),
				(Some(n), None) => Ok((false, EthTransaction {
					nonce: n,
					action: action,
					gas: START_GAS.into(),
					gas_price: gas_price,
					value: value,
					data: data,
				})),
				(None, _) => Err(errors::unknown_block()),
			})
		}).join(header_fut).and_then(move |((gas_known, tx), hdr)| {
			// then request proved execution.
			// TODO: get last-hashes from network.
			let env_info = match client.env_info(id) {
				Some(env_info) => env_info,
				_ => return future::err(errors::unknown_block()).boxed(),
			};

			execute_tx(gas_known, ExecuteParams {
				from: from,
				tx: tx,
				hdr: hdr,
				env_info: env_info,
				engine: client.engine().clone(),
				on_demand: on_demand,
				sync: sync,
			})
		}).boxed()
	}

	/// get a block itself. fails on unknown block ID.
	pub fn block(&self, id: BlockId) -> BoxFuture<encoded::Block, Error> {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return future::err(e).boxed(),
		};

		reqs.push(request::Body(header_ref).into());

		let maybe_future = self.sync.with_context(move |ctx| {
			self.on_demand.request_raw(ctx, reqs)
				.expect("all back-references known to be valid; qed")
				.map(|mut res| match res.pop() {
					Some(OnDemandResponse::Body(b)) => b,
					_ => panic!("responses correspond directly with requests in amount and type; qed"),
				})
				.map_err(errors::on_demand_cancel)
				.boxed()
		});

		match maybe_future {
			Some(recv) => recv,
			None => future::err(errors::network_disabled()).boxed()
		}
	}

	/// get transaction logs
	pub fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>, Error> {
		use std::collections::BTreeMap;

		use futures::stream::{self, Stream};

		const NO_INVALID_BACK_REFS: &'static str = "Fails only on invalid back-references; back-references here known to be valid; qed";

		// early exit for "to" block before "from" block.
		let best_number = self.client.chain_info().best_block_number;
		let block_number = |id| match id {
			BlockId::Earliest => Some(0),
			BlockId::Latest | BlockId::Pending => Some(best_number),
			BlockId::Hash(h) => self.client.block_header(BlockId::Hash(h)).map(|hdr| hdr.number()),
			BlockId::Number(x) => Some(x),
		};

		match (block_number(filter.to_block), block_number(filter.from_block)) {
			(Some(to), Some(from)) if to < from => return future::ok(Vec::new()).boxed(),
			(Some(_), Some(_)) => {},
			_ => return future::err(errors::unknown_block()).boxed(),
		}

		let maybe_future = self.sync.with_context(move |ctx| {
			// find all headers which match the filter, and fetch the receipts for each one.
			// match them with their numbers for easy sorting later.
			let bit_combos = filter.bloom_possibilities();
			let receipts_futures: Vec<_> = self.client.ancestry_iter(filter.to_block)
				.take_while(|ref hdr| BlockId::Number(hdr.number()) != filter.from_block)
				.take_while(|ref hdr| BlockId::Hash(hdr.hash()) != filter.from_block)
				.filter(|ref hdr| {
					let hdr_bloom = hdr.log_bloom();
					bit_combos.iter().find(|&bloom| hdr_bloom & *bloom == *bloom).is_some()
				})
				.map(|hdr| (hdr.number(), request::BlockReceipts(hdr.into())))
				.map(|(num, req)| self.on_demand.request(ctx, req).expect(NO_INVALID_BACK_REFS).map(move |x| (num, x)))
				.collect();

			// as the receipts come in, find logs within them which match the filter.
			// insert them into a BTreeMap to maintain order by number and block index.
			stream::futures_unordered(receipts_futures)
				.fold(BTreeMap::new(), move |mut matches, (num, receipts)| {
					for (block_index, log) in receipts.into_iter().flat_map(|r| r.logs).enumerate() {
						if filter.matches(&log) {
							matches.insert((num, block_index), log.into());
						}
					}
					future::ok(matches)
				}) // and then collect them into a vector.
				.map(|matches| matches.into_iter().map(|(_, v)| v).collect())
				.map_err(errors::on_demand_cancel)
		});

		match maybe_future {
			Some(fut) => fut.boxed(),
			None => future::err(errors::network_disabled()).boxed(),
		}
	}
}

#[derive(Clone)]
struct ExecuteParams {
	from: Address,
	tx: EthTransaction,
	hdr: encoded::Header,
	env_info: ::vm::EnvInfo,
	engine: Arc<::ethcore::engines::Engine>,
	on_demand: Arc<OnDemand>,
	sync: Arc<LightSync>,
}

// has a peer execute the transaction with given params. If `gas_known` is false,
// this will double the gas on each `OutOfGas` error.
fn execute_tx(gas_known: bool, params: ExecuteParams) -> BoxFuture<ExecutionResult, Error> {
	if !gas_known {
		future::loop_fn(params, |mut params| {
			execute_tx(true, params.clone()).and_then(move |res| {
				match res {
					Ok(executed) => {
						// TODO: how to distinguish between actual OOG and
						// exception?
						if executed.exception.is_some() {
							let old_gas = params.tx.gas;
							params.tx.gas = params.tx.gas * 2.into();
							if params.tx.gas > params.hdr.gas_limit() {
								params.tx.gas = old_gas;
							} else {
								return Ok(future::Loop::Continue(params))
							}
						}

						Ok(future::Loop::Break(Ok(executed)))
					}
					failed => Ok(future::Loop::Break(failed)),
				}
			})
		}).boxed()
	} else {
		trace!(target: "light_fetch", "Placing execution request for {} gas in on_demand",
			params.tx.gas);

		let request = request::TransactionProof {
			tx: params.tx.fake_sign(params.from),
			header: params.hdr.into(),
			env_info: params.env_info,
			engine: params.engine,
		};

		let on_demand = params.on_demand;
		let proved_future = params.sync.with_context(move |ctx| {
			on_demand
				.request(ctx, request)
				.expect("no back-references; therefore all back-refs valid; qed")
				.map_err(errors::on_demand_cancel)
		});

		match proved_future {
			Some(fut) => fut.boxed(),
			None => future::err(errors::network_disabled()).boxed(),
		}
	}
}
