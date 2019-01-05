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

//! Helpers for fetching blockchain data either from the light client or the network.

use std::cmp;
use std::sync::Arc;

use types::basic_account::BasicAccount;
use types::encoded;
use types::filter::Filter as EthcoreFilter;
use types::ids::BlockId;
use types::receipt::Receipt;
use ethcore::executed::ExecutionError;

use jsonrpc_core::{Result, Error};
use jsonrpc_core::futures::{future, Future};
use jsonrpc_core::futures::future::Either;
use jsonrpc_macros::Trailing;

use light::cache::Cache;
use light::client::LightChainClient;
use light::{cht, MAX_HEADERS_PER_REQUEST};
use light::on_demand::{
	request, OnDemand, HeaderRef, Request as OnDemandRequest,
	Response as OnDemandResponse, ExecutionResult,
};
use light::on_demand::error::Error as OnDemandError;
use light::request::Field;

use sync::LightSync;
use ethereum_types::{U256, Address};
use hash::H256;
use parking_lot::Mutex;
use fastmap::H256FastMap;
use std::collections::BTreeMap;
use types::transaction::{Action, Transaction as EthTransaction, PendingTransaction, SignedTransaction, LocalizedTransaction};

use v1::helpers::{CallRequest as CallRequestHelper, errors, dispatch};
use v1::types::{BlockNumber, CallRequest, Log, Transaction};

const NO_INVALID_BACK_REFS_PROOF: &str = "Fails only on invalid back-references; back-references here known to be valid; qed";

const WRONG_RESPONSE_AMOUNT_TYPE_PROOF: &str = "responses correspond directly with requests in amount and type; qed";

pub fn light_all_transactions(dispatch: &Arc<dispatch::LightDispatcher>) -> impl Iterator<Item=PendingTransaction> {
	let txq = dispatch.transaction_queue.read();
	let chain_info = dispatch.client.chain_info();

	let current = txq.ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp);
	let future = txq.future_transactions(chain_info.best_block_number, chain_info.best_block_timestamp);
	current.into_iter().chain(future.into_iter())
}

/// Helper for fetching blockchain data either from the light client or the network
/// as necessary.
#[derive(Clone)]
pub struct LightFetch {
	/// The light client.
	pub client: Arc<LightChainClient>,
	/// The on-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// Handle to the network.
	pub sync: Arc<LightSync>,
	/// The light data cache.
	pub cache: Arc<Mutex<Cache>>,
	/// Gas Price percentile
	pub gas_price_percentile: usize,
}

/// Extract a transaction at given index.
pub fn extract_transaction_at_index(block: encoded::Block, index: usize) -> Option<Transaction> {
	block.transactions().into_iter().nth(index)
		// Verify if transaction signature is correct.
		.and_then(|tx| SignedTransaction::new(tx).ok())
		.map(|signed_tx| {
			let (signed, sender, _) = signed_tx.deconstruct();
			let block_hash = block.hash();
			let block_number = block.number();
			let transaction_index = index;
			let cached_sender = Some(sender);

			LocalizedTransaction {
				signed,
				block_number,
				block_hash,
				transaction_index,
				cached_sender,
			}
		})
		.map(|tx| Transaction::from_localized(tx))
}

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
	fn make_header_requests(&self, id: BlockId, reqs: &mut Vec<OnDemandRequest>) -> Result<HeaderRef> {
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
				let idx = reqs.len();
				reqs.push(request::HeaderByHash(h.into()).into());
				Ok(HeaderRef::Unresolved(idx, h.into()))
			}
			_ => Err(errors::unknown_block()) // latest, earliest, and pending will have all already returned.
		}
	}

	/// Get a block header from the on demand service or client, or error.
	pub fn header(&self, id: BlockId) -> impl Future<Item = encoded::Header, Error = Error> + Send {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return Either::A(future::err(e)),
		};

		Either::B(self.send_requests(reqs, |res|
			extract_header(&res, header_ref)
				.expect("these responses correspond to requests that header_ref belongs to \
					therefore it will not fail; qed")
		))
	}

	/// Helper for getting contract code at a given block.
	pub fn code(&self, address: Address, id: BlockId) -> impl Future<Item = Vec<u8>, Error = Error> + Send {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return Either::A(future::err(e)),
		};

		reqs.push(request::Account { header: header_ref.clone(), address: address }.into());
		let account_idx = reqs.len() - 1;
		reqs.push(request::Code { header: header_ref, code_hash: Field::back_ref(account_idx, 0) }.into());

		Either::B(self.send_requests(reqs, |mut res| match res.pop() {
			Some(OnDemandResponse::Code(code)) => code,
			_ => panic!(WRONG_RESPONSE_AMOUNT_TYPE_PROOF),
		}))
	}

	/// Helper for getting account info at a given block.
	/// `None` indicates the account doesn't exist at the given block.
	pub fn account(&self, address: Address, id: BlockId) -> impl Future<Item = Option<BasicAccount>, Error = Error> + Send {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return Either::A(future::err(e)),
		};

		reqs.push(request::Account { header: header_ref, address: address }.into());

		Either::B(self.send_requests(reqs, |mut res|match res.pop() {
			Some(OnDemandResponse::Account(acc)) => acc,
			_ => panic!(WRONG_RESPONSE_AMOUNT_TYPE_PROOF),
		}))
	}

	/// Helper for getting proved execution.
	pub fn proved_read_only_execution(&self, req: CallRequest, num: Trailing<BlockNumber>) -> impl Future<Item = ExecutionResult, Error = Error> + Send {
		const DEFAULT_GAS_PRICE: u64 = 21_000;
		// (21000 G_transaction + 32000 G_create + some marginal to allow a few operations)
		const START_GAS: u64 = 60_000;

		let (sync, on_demand, client) = (self.sync.clone(), self.on_demand.clone(), self.client.clone());
		let req: CallRequestHelper = req.into();

		// Note: Here we treat `Pending` as `Latest`.
		//       Since light clients don't produce pending blocks
		//       (they don't have state) we can safely fallback to `Latest`.
		let id = match num.unwrap_or_default() {
			BlockNumber::Num(n) => BlockId::Number(n),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,
			BlockNumber::Pending => {
				warn!("`Pending` is deprecated and may be removed in future versions. Falling back to `Latest`");
				BlockId::Latest
			}
		};

		let from = req.from.unwrap_or_else(|| Address::zero());
		let nonce_fut = match req.nonce {
			Some(nonce) => Either::A(future::ok(Some(nonce))),
			None => Either::B(self.account(from, id).map(|acc| acc.map(|a| a.nonce))),
		};

		let gas_price_percentile = self.gas_price_percentile;
		let gas_price_fut = match req.gas_price {
			Some(price) => Either::A(future::ok(price)),
			None => Either::B(dispatch::fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone(),
			).map(move |corp| match corp.percentile(gas_price_percentile) {
				Some(percentile) => *percentile,
				None => DEFAULT_GAS_PRICE.into(),
			}))
		};

		// if nonce resolves, this should too since it'll be in the LRU-cache.
		let header_fut = self.header(id);

		// fetch missing transaction fields from the network.
		Box::new(nonce_fut.join(gas_price_fut).and_then(move |(nonce, gas_price)| {
			future::done(
				Ok((req.gas.is_some(), EthTransaction {
					nonce: nonce.unwrap_or_default(),
					action: req.to.map_or(Action::Create, Action::Call),
					gas: req.gas.unwrap_or_else(|| START_GAS.into()),
					gas_price,
					value: req.value.unwrap_or_else(U256::zero),
					data: req.data.unwrap_or_default(),
				}))
			)
		}).join(header_fut).and_then(move |((gas_known, tx), hdr)| {
			// then request proved execution.
			// TODO: get last-hashes from network.
			let hash = hdr.hash();
			let env_info = match client.env_info(BlockId::Hash(hash)) {
				Some(env_info) => env_info,
				_ => return Either::A(future::err(errors::unknown_block())),
			};

			Either::B(execute_read_only_tx(gas_known, ExecuteParams {
				from,
				tx,
				hdr,
				env_info,
				engine: client.engine().clone(),
				on_demand,
				sync,
			}))
		}))
	}

	/// Get a block itself. Fails on unknown block ID.
	pub fn block(&self, id: BlockId) -> impl Future<Item = encoded::Block, Error = Error> + Send {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return Either::A(future::err(e)),
		};

		reqs.push(request::Body(header_ref).into());

		Either::B(self.send_requests(reqs, |mut res| match res.pop() {
			Some(OnDemandResponse::Body(b)) => b,
			_ => panic!(WRONG_RESPONSE_AMOUNT_TYPE_PROOF),
		}))
	}

	/// Get the block receipts. Fails on unknown block ID.
	pub fn receipts(&self, id: BlockId) -> impl Future<Item = Vec<Receipt>, Error = Error> + Send {
		let mut reqs = Vec::new();
		let header_ref = match self.make_header_requests(id, &mut reqs) {
			Ok(r) => r,
			Err(e) => return Either::A(future::err(e)),
		};

		reqs.push(request::BlockReceipts(header_ref).into());

		Either::B(self.send_requests(reqs, |mut res| match res.pop() {
			Some(OnDemandResponse::Receipts(b)) => b,
			_ => panic!(WRONG_RESPONSE_AMOUNT_TYPE_PROOF),
		}))
	}

	pub fn logs_no_tx_hash(&self, filter: EthcoreFilter) -> impl Future<Item = Vec<Log>, Error = Error> + Send {
		use jsonrpc_core::futures::stream::{self, Stream};

		const MAX_BLOCK_RANGE: u64 = 1000;

		let fetcher = self.clone();
		self.headers_range_by_block_id(filter.from_block, filter.to_block, MAX_BLOCK_RANGE)
			.and_then(move |mut headers| {
				if headers.is_empty() {
					return Either::A(future::ok(Vec::new()));
				}

				let on_demand = &fetcher.on_demand;

				let maybe_future = fetcher.sync.with_context(move |ctx| {
					// find all headers which match the filter, and fetch the receipts for each one.
					// match them with their numbers for easy sorting later.
					let bit_combos = filter.bloom_possibilities();
					let receipts_futures: Vec<_> = headers.drain(..)
						.filter(|ref hdr| {
							let hdr_bloom = hdr.log_bloom();
							bit_combos.iter().any(|bloom| hdr_bloom.contains_bloom(bloom))
						})
						.map(|hdr| (hdr.number(), hdr.hash(), request::BlockReceipts(hdr.into())))
						.map(|(num, hash, req)| on_demand.request(ctx, req).expect(NO_INVALID_BACK_REFS_PROOF).map(move |x| (num, hash, x)))
						.collect();

					// as the receipts come in, find logs within them which match the filter.
					// insert them into a BTreeMap to maintain order by number and block index.
					stream::futures_unordered(receipts_futures)
						.fold(BTreeMap::new(), move |mut matches, (num, hash, receipts)| {
							let mut block_index: usize = 0;
							for (transaction_index, receipt) in receipts.into_iter().enumerate() {
								for (transaction_log_index, log) in receipt.logs.into_iter().enumerate() {
									if filter.matches(&log) {
										matches.insert((num, block_index), Log {
											address: log.address.into(),
											topics: log.topics.into_iter().map(Into::into).collect(),
											data: log.data.into(),
											block_hash: Some(hash.into()),
											block_number: Some(num.into()),
											// No way to easily retrieve transaction hash, so let's just skip it.
											transaction_hash: None,
											transaction_index: Some(transaction_index.into()),
											log_index: Some(block_index.into()),
											transaction_log_index: Some(transaction_log_index.into()),
											log_type: "mined".into(),
											removed: false,
										});
									}
									block_index += 1;
								}
							}
							future::ok::<_,OnDemandError>(matches)
						})
						.map_err(errors::on_demand_error)
						.map(|matches| matches.into_iter().map(|(_, v)| v).collect())
				});

				match maybe_future {
					Some(fut) => Either::B(Either::A(fut)),
					None => Either::B(Either::B(future::err(errors::network_disabled()))),
				}
			})
	}


	/// Get transaction logs
	pub fn logs(&self, filter: EthcoreFilter) -> impl Future<Item = Vec<Log>, Error = Error> + Send {
		use jsonrpc_core::futures::stream::{self, Stream};
		let fetcher_block = self.clone();
		self.logs_no_tx_hash(filter)
			// retrieve transaction hash.
			.and_then(move |mut result| {
				let mut blocks = BTreeMap::new();
				for log in result.iter() {
						let block_hash = log.block_hash.as_ref().expect("Previously initialized with value; qed");
						blocks.entry(block_hash.clone()).or_insert_with(|| {
							fetcher_block.block(BlockId::Hash(block_hash.clone().into()))
						});
				}
				// future get blocks (unordered it)
				stream::futures_unordered(blocks.into_iter().map(|(_, v)| v)).collect().map(move |blocks| {
					let transactions_per_block: BTreeMap<_, _> = blocks.iter()
						.map(|block| (block.hash(), block.transactions())).collect();
					for log in result.iter_mut() {
						let log_index: U256 = log.transaction_index.expect("Previously initialized with value; qed").into();
						let block_hash = log.block_hash.clone().expect("Previously initialized with value; qed").into();
						let tx_hash = transactions_per_block.get(&block_hash)
							// transaction index is from an enumerate call in log common so not need to check value
							.and_then(|txs| txs.get(log_index.as_usize()))
							.map(|tr| tr.hash().into());
						log.transaction_hash = tx_hash;
					}
					result
				})
			})
	}

	// Get a transaction by hash. also returns the index in the block.
	// Only returns transactions in the canonical chain.
	pub fn transaction_by_hash(&self, tx_hash: H256)
		-> impl Future<Item = Option<(Transaction, usize)>, Error = Error> + Send
	{
		let params = (self.sync.clone(), self.on_demand.clone());
		let fetcher: Self = self.clone();

		Box::new(future::loop_fn(params, move |(sync, on_demand)| {
			let maybe_future = sync.with_context(|ctx| {
				let req = request::TransactionIndex(tx_hash.clone().into());
				on_demand.request(ctx, req)
			});

			let eventual_index = match maybe_future {
				Some(e) => e.expect(NO_INVALID_BACK_REFS_PROOF).map_err(errors::on_demand_error),
				None => return Either::A(future::err(errors::network_disabled())),
			};

			let fetcher = fetcher.clone();
			let extract_transaction = eventual_index.and_then(move |index| {
				// check that the block is known by number.
				// that ensures that it is within the chain that we are aware of.
				fetcher.block(BlockId::Number(index.num)).then(move |blk| match blk {
					Ok(blk) => {
						// if the block is known by number, make sure the
						// index from earlier isn't garbage.

						if blk.hash() != index.hash {
							// index is on a different chain from us.
							return Ok(future::Loop::Continue((sync, on_demand)))
						}

						let index = index.index as usize;
						let transaction = extract_transaction_at_index(blk, index);

						if transaction.as_ref().map_or(true, |tx| tx.hash != tx_hash.into()) {
							// index is actively wrong: indicated block has
							// fewer transactions than necessary or the transaction
							// at that index had a different hash.
							// TODO: punish peer/move into OnDemand somehow?
							Ok(future::Loop::Continue((sync, on_demand)))
						} else {
							let transaction = transaction.map(move |tx| (tx, index));
							Ok(future::Loop::Break(transaction))
						}
					}
					Err(ref e) if e == &errors::unknown_block() => {
						// block by number not in the canonical chain.
						Ok(future::Loop::Break(None))
					}
					Err(e) => Err(e),
				})
			});

			Either::B(extract_transaction)
		}))
	}

	fn send_requests<T, F>(&self, reqs: Vec<OnDemandRequest>, parse_response: F) -> impl Future<Item = T, Error = Error> + Send where
		F: FnOnce(Vec<OnDemandResponse>) -> T + Send + 'static,
		T: Send + 'static,
	{
		let maybe_future = self.sync.with_context(move |ctx| {
			Box::new(self.on_demand.request_raw(ctx, reqs)
					 .expect(NO_INVALID_BACK_REFS_PROOF)
					 .map_err(errors::on_demand_cancel)
					 .and_then(|responses| {
						 match responses {
							 Ok(responses) => Ok(parse_response(responses)),
							 Err(e) => Err(errors::on_demand_error(e)),
						 }
					 })
			)
		});

		match maybe_future {
			Some(recv) => recv,
			None => Box::new(future::err(errors::network_disabled())) as Box<Future<Item = _, Error = _> + Send>
		}
	}

	fn headers_range_by_block_id(
		&self,
		from_block: BlockId,
		to_block: BlockId,
		max: u64
	) -> impl Future<Item = Vec<encoded::Header>, Error = Error> {
		let fetch_hashes = [from_block, to_block].iter()
			.filter_map(|block_id| match block_id {
				BlockId::Hash(hash) => Some(hash.clone()),
				_ => None,
			})
			.collect::<Vec<_>>();

		let best_number = self.client.chain_info().best_block_number;

		let fetcher = self.clone();
		self.headers_by_hash(&fetch_hashes[..]).and_then(move |mut header_map| {
			let (from_block_num, to_block_num) = {
				let block_number = |id| match id {
					&BlockId::Earliest => 0,
					&BlockId::Latest => best_number,
					&BlockId::Hash(ref h) =>
						header_map.get(h).map(|hdr| hdr.number())
						.expect("from_block and to_block headers are fetched by hash; this closure is only called on from_block and to_block; qed"),
					&BlockId::Number(x) => x,
				};
				(block_number(&from_block), block_number(&to_block))
			};

			if to_block_num < from_block_num {
				// early exit for "to" block before "from" block.
				return Either::A(future::err(errors::filter_block_not_found(to_block)));
			} else if to_block_num - from_block_num >= max {
				return Either::A(future::err(errors::request_rejected_param_limit(max, "blocks")));
			}

			let to_header_hint = match to_block {
				BlockId::Hash(ref h) => header_map.remove(h),
				_ => None,
			};
			let headers_fut = fetcher.headers_range(from_block_num, to_block_num, to_header_hint);
			Either::B(headers_fut.map(move |headers| {
				// Validate from_block if it's a hash
				let last_hash = headers.last().map(|hdr| hdr.hash());
				match (last_hash, from_block) {
					(Some(h1), BlockId::Hash(h2)) if h1 != h2 => Vec::new(),
					_ => headers,
				}
			}))
		})
	}

	fn headers_by_hash(&self, hashes: &[H256]) -> impl Future<Item = H256FastMap<encoded::Header>, Error = Error> {
		let mut refs = H256FastMap::with_capacity_and_hasher(hashes.len(), Default::default());
		let mut reqs = Vec::with_capacity(hashes.len());

		for hash in hashes {
			refs.entry(*hash).or_insert_with(|| {
				self.make_header_requests(BlockId::Hash(*hash), &mut reqs)
					.expect("make_header_requests never fails for BlockId::Hash; qed")
			});
		}

		self.send_requests(reqs, move |res| {
			let headers = refs.drain()
				.map(|(hash, header_ref)| {
					let hdr = extract_header(&res, header_ref)
						.expect("these responses correspond to requests that header_ref belongs to; \
								qed");
					(hash, hdr)
				})
				.collect();
			headers
		})
	}

	fn headers_range(
		&self,
		from_number: u64,
		to_number: u64,
		to_header_hint: Option<encoded::Header>
	) -> impl Future<Item = Vec<encoded::Header>, Error = Error> {
		let range_length = (to_number - from_number + 1) as usize;
		let mut headers: Vec<encoded::Header> = Vec::with_capacity(range_length);

		let iter_start = match to_header_hint {
			Some(hdr) => {
				let block_id = BlockId::Hash(hdr.parent_hash());
				headers.push(hdr);
				block_id
			}
			None => BlockId::Number(to_number),
		};
		headers.extend(self.client.ancestry_iter(iter_start)
				.take_while(|hdr| hdr.number() >= from_number));

		let fetcher = self.clone();
		future::loop_fn(headers, move |mut headers| {
			let remaining = range_length - headers.len();
			if remaining == 0 {
				return Either::A(future::ok(future::Loop::Break(headers)));
			}

			let mut reqs: Vec<request::Request> = Vec::with_capacity(2);

			let start_hash = if let Some(hdr) = headers.last() {
				hdr.parent_hash().into()
			} else {
				let cht_root = cht::block_to_cht_number(to_number)
					.and_then(|cht_num| fetcher.client.cht_root(cht_num as usize));

				let cht_root = match cht_root {
					Some(cht_root) => cht_root,
					None => return Either::A(future::err(errors::unknown_block())),
				};

				let header_proof = request::HeaderProof::new(to_number, cht_root)
					.expect("HeaderProof::new is Some(_) if cht::block_to_cht_number() is Some(_); \
							this would return above if block_to_cht_number returned None; qed");

				let idx = reqs.len();
				let hash_ref = Field::back_ref(idx, 0);
				reqs.push(header_proof.into());

				hash_ref
			};

			let max = cmp::min(remaining as u64, MAX_HEADERS_PER_REQUEST);
			reqs.push(request::HeaderWithAncestors {
				block_hash: start_hash,
				ancestor_count: max - 1,
			}.into());

			Either::B(fetcher.send_requests(reqs, |mut res| {
				match res.last_mut() {
					Some(&mut OnDemandResponse::HeaderWithAncestors(ref mut res_headers)) =>
						headers.extend(res_headers.drain(..)),
					_ => panic!("reqs has at least one entry; each request maps to a response; qed"),
				};
				future::Loop::Continue(headers)
			}))
		})
	}
}

#[derive(Clone)]
struct ExecuteParams {
	from: Address,
	tx: EthTransaction,
	hdr: encoded::Header,
	env_info: ::vm::EnvInfo,
	engine: Arc<::ethcore::engines::EthEngine>,
	on_demand: Arc<OnDemand>,
	sync: Arc<LightSync>,
}

// Has a peer execute the transaction with given params. If `gas_known` is false, this will set the `gas value` to the
// `required gas value` unless it exceeds the block gas limit
fn execute_read_only_tx(gas_known: bool, params: ExecuteParams) -> impl Future<Item = ExecutionResult, Error = Error> + Send {
	if !gas_known {
		Box::new(future::loop_fn(params, |mut params| {
			execute_read_only_tx(true, params.clone()).and_then(move |res| {
				match res {
					Ok(executed) => {
						// `OutOfGas` exception, try double the gas
						if let Some(::vm::Error::OutOfGas) = executed.exception {
							// block gas limit already tried, regard as an error and don't retry
							if params.tx.gas >= params.hdr.gas_limit() {
								trace!(target: "light_fetch", "OutOutGas exception received, gas increase: failed");
							} else {
								params.tx.gas = cmp::min(params.tx.gas * 2_u32, params.hdr.gas_limit());
								trace!(target: "light_fetch", "OutOutGas exception received, gas increased to {}",
									   params.tx.gas);
								return Ok(future::Loop::Continue(params))
							}
						}
						Ok(future::Loop::Break(Ok(executed)))
					}
					Err(ExecutionError::NotEnoughBaseGas { required, got }) => {
						trace!(target: "light_fetch", "Not enough start gas provided required: {}, got: {}",
							   required, got);
						if required <= params.hdr.gas_limit() {
							params.tx.gas = required;
							return Ok(future::Loop::Continue(params))
						} else {
							warn!(target: "light_fetch",
								  "Required gas is bigger than block header's gas dropping the request");
							Ok(future::Loop::Break(Err(ExecutionError::NotEnoughBaseGas { required, got })))
						}
					}
					// Non-recoverable execution error
					failed => Ok(future::Loop::Break(failed)),
				}
			})
		})) as Box<Future<Item = _, Error = _> + Send>
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
				.map_err(errors::on_demand_error)
		});

		match proved_future {
			Some(fut) => Box::new(fut) as Box<Future<Item = _, Error = _> + Send>,
			None => Box::new(future::err(errors::network_disabled())) as Box<Future<Item = _, Error = _> + Send>,
		}
	}
}
