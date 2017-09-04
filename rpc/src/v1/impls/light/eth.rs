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

//! Eth RPC interface for the light client.

// TODO: remove when complete.
#![allow(unused_imports, unused_variables)]

use std::sync::Arc;

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use light::cache::Cache as LightDataCache;
use light::client::LightChainClient;
use light::{cht, TransactionQueue};
use light::on_demand::{request, OnDemand};

use ethcore::account_provider::{AccountProvider, DappId};
use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::executed::{Executed, ExecutionError};
use ethcore::ids::BlockId;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::transaction::{Action, SignedTransaction, Transaction as EthTransaction};
use ethsync::LightSync;
use rlp::UntrustedRlp;
use hash::{KECCAK_NULL_RLP, KECCAK_EMPTY_LIST_RLP};
use util::U256;
use parking_lot::{RwLock, Mutex};

use futures::{future, Future, BoxFuture, IntoFuture};
use futures::sync::oneshot;

use v1::impls::eth_filter::Filterable;
use v1::helpers::{CallRequest as CRequest, errors, limit_logs, dispatch};
use v1::helpers::{PollFilter, PollManager};
use v1::helpers::block_import::is_major_importing;
use v1::helpers::light_fetch::LightFetch;
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo,
	Transaction, CallRequest, Index, Filter, Log, Receipt, Work,
	H64 as RpcH64, H256 as RpcH256, H160 as RpcH160, U256 as RpcU256,
};
use v1::metadata::Metadata;

use util::Address;

const NO_INVALID_BACK_REFS: &'static str = "Fails only on invalid back-references; back-references here known to be valid; qed";

/// Light client `ETH` (and filter) RPC.
pub struct EthClient<T> {
	sync: Arc<LightSync>,
	client: Arc<T>,
	on_demand: Arc<OnDemand>,
	transaction_queue: Arc<RwLock<TransactionQueue>>,
	accounts: Arc<AccountProvider>,
	cache: Arc<Mutex<LightDataCache>>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl<T> Clone for EthClient<T> {
	fn clone(&self) -> Self {
		// each instance should have its own poll manager.
		EthClient {
			sync: self.sync.clone(),
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			transaction_queue: self.transaction_queue.clone(),
			accounts: self.accounts.clone(),
			cache: self.cache.clone(),
			polls: Mutex::new(PollManager::new()),
		}
	}
}


impl<T: LightChainClient + 'static> EthClient<T> {
	/// Create a new `EthClient` with a handle to the light sync instance, client,
	/// and on-demand request service, which is assumed to be attached as a handler.
	pub fn new(
		sync: Arc<LightSync>,
		client: Arc<T>,
		on_demand: Arc<OnDemand>,
		transaction_queue: Arc<RwLock<TransactionQueue>>,
		accounts: Arc<AccountProvider>,
		cache: Arc<Mutex<LightDataCache>>,
	) -> Self {
		EthClient {
			sync: sync,
			client: client,
			on_demand: on_demand,
			transaction_queue: transaction_queue,
			accounts: accounts,
			cache: cache,
			polls: Mutex::new(PollManager::new()),
		}
	}

	/// Create a light data fetcher instance.
	fn fetcher(&self) -> LightFetch {
		LightFetch {
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			sync: self.sync.clone(),
			cache: self.cache.clone(),
		}
	}

	// get a "rich" block structure. Fails on unknown block.
	fn rich_block(&self, id: BlockId, include_txs: bool) -> BoxFuture<RichBlock, Error> {
		let (on_demand, sync) = (self.on_demand.clone(), self.sync.clone());
		let (client, engine) = (self.client.clone(), self.client.engine().clone());
		let eip86_transition = self.client.eip86_transition();

		// helper for filling out a rich block once we've got a block and a score.
		let fill_rich = move |block: encoded::Block, score: Option<U256>| {
			let header = block.decode_header();
			let extra_info = engine.extra_info(&header);
			RichBlock {
				inner: Block {
					hash: Some(header.hash().into()),
					size: Some(block.rlp().as_raw().len().into()),
					parent_hash: header.parent_hash().clone().into(),
					uncles_hash: header.uncles_hash().clone().into(),
					author: header.author().clone().into(),
					miner: header.author().clone().into(),
					state_root: header.state_root().clone().into(),
					transactions_root: header.transactions_root().clone().into(),
					receipts_root: header.receipts_root().clone().into(),
					number: Some(header.number().into()),
					gas_used: header.gas_used().clone().into(),
					gas_limit: header.gas_limit().clone().into(),
					logs_bloom: header.log_bloom().clone().into(),
					timestamp: header.timestamp().into(),
					difficulty: header.difficulty().clone().into(),
					total_difficulty: score.map(Into::into),
					seal_fields: header.seal().into_iter().cloned().map(Into::into).collect(),
					uncles: block.uncle_hashes().into_iter().map(Into::into).collect(),
					transactions: match include_txs {
						true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(|t| Transaction::from_localized(t, eip86_transition)).collect()),
						_ => BlockTransactions::Hashes(block.transaction_hashes().into_iter().map(Into::into).collect()),
					},
					extra_data: Bytes::new(header.extra_data().clone()),
				},
				extra_info: extra_info
			}
		};

		// get the block itself.
		self.fetcher().block(id).and_then(move |block| {
			// then fetch the total difficulty (this is much easier after getting the block).
			match client.score(id) {
				Some(score) => future::ok(fill_rich(block, Some(score))).boxed(),
				None => {
					// make a CHT request to fetch the chain score.
					let req = cht::block_to_cht_number(block.number())
						.and_then(|num| client.cht_root(num as usize))
						.and_then(|root| request::HeaderProof::new(block.number(), root));

					let req = match req {
						Some(req) => req,
						None => {
							// somehow the genesis block slipped past other checks.
							// return it now.
							let score = client.block_header(BlockId::Number(0))
								.expect("genesis always stored; qed")
								.difficulty();

							return future::ok(fill_rich(block, Some(score))).boxed()
						}
					};

					// three possible outcomes:
					//   - network is down.
					//   - we get a score, but our hash is non-canonical.
					//   - we get a score, and our hash is canonical.
					let maybe_fut = sync.with_context(move |ctx| on_demand.request(ctx, req).expect(NO_INVALID_BACK_REFS));
					match maybe_fut {
						Some(fut) => fut
							.map(move |(hash, score)| {
								let score = if hash == block.hash() {
									Some(score)
								} else {
									None
								};

							fill_rich(block, score)
							}).map_err(errors::on_demand_cancel).boxed(),
						None => return future::err(errors::network_disabled()).boxed(),
					}
				}
			}
		}).boxed()
	}
}

impl<T: LightChainClient + 'static> Eth for EthClient<T> {
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String, Error> {
		Ok(format!("{}", ::light::net::MAX_PROTOCOL_VERSION))
	}

	fn syncing(&self) -> Result<SyncStatus, Error> {
		if self.sync.is_major_importing() {
			let chain_info = self.client.chain_info();
			let current_block = U256::from(chain_info.best_block_number);
			let highest_block = self.sync.highest_block().map(U256::from)
				.unwrap_or_else(|| current_block.clone());

			Ok(SyncStatus::Info(SyncInfo {
				starting_block: U256::from(self.sync.start_block()).into(),
				current_block: current_block.into(),
				highest_block: highest_block.into(),
				warp_chunks_amount: None,
				warp_chunks_processed: None,
			}))
		} else {
			Ok(SyncStatus::None)
		}
	}

	fn author(&self, _meta: Self::Metadata) -> BoxFuture<RpcH160, Error> {
		future::ok(Default::default()).boxed()
	}

	fn is_mining(&self) -> Result<bool, Error> {
		Ok(false)
	}

	fn hashrate(&self) -> Result<RpcU256, Error> {
		Ok(Default::default())
	}

	fn gas_price(&self) -> Result<RpcU256, Error> {
		Ok(self.cache.lock().gas_price_corpus()
			.and_then(|c| c.median().cloned())
			.map(RpcU256::from)
			.unwrap_or_else(Default::default))
	}

	fn accounts(&self, meta: Metadata) -> BoxFuture<Vec<RpcH160>, Error> {
		let dapp: DappId = meta.dapp_id().into();

		let accounts = self.accounts
			.note_dapp_used(dapp.clone())
			.and_then(|_| self.accounts.dapp_addresses(dapp))
			.map_err(|e| errors::account("Could not fetch accounts.", e))
			.map(|accs| accs.into_iter().map(Into::<RpcH160>::into).collect());

		future::done(accounts).boxed()
	}

	fn block_number(&self) -> Result<RpcU256, Error> {
		Ok(self.client.chain_info().best_block_number.into())
	}

	fn balance(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.fetcher().account(address.into(), num.unwrap_or_default().into())
			.map(|acc| acc.map_or(0.into(), |a| a.balance).into()).boxed()
	}

	fn storage_at(&self, _address: RpcH160, _key: RpcU256, _num: Trailing<BlockNumber>) -> BoxFuture<RpcH256, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		self.rich_block(BlockId::Hash(hash.into()), include_txs).map(Some).boxed()
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		self.rich_block(num.into(), include_txs).map(Some).boxed()
	}

	fn transaction_count(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.fetcher().account(address.into(), num.unwrap_or_default().into())
			.map(|acc| acc.map_or(0.into(), |a| a.nonce).into()).boxed()
	}

	fn block_transaction_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(BlockId::Hash(hash.into())).and_then(move |hdr| {
			if hdr.transactions_root() == KECCAK_NULL_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(num.into()).and_then(move |hdr| {
			if hdr.transactions_root() == KECCAK_NULL_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_uncles_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(BlockId::Hash(hash.into())).and_then(move |hdr| {
			if hdr.uncles_hash() == KECCAK_EMPTY_LIST_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(num.into()).and_then(move |hdr| {
			if hdr.uncles_hash() == KECCAK_EMPTY_LIST_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn code_at(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		let best_header = self.client.best_block_header().decode();

		UntrustedRlp::new(&raw.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| {
				self.client.engine().verify_transaction_basic(&tx, &best_header)
					.map_err(errors::transaction)?;

				let signed = SignedTransaction::new(tx).map_err(errors::transaction)?;
				let hash = signed.hash();

				self.transaction_queue.write().import(signed.into())
					.map(|_| hash)
					.map_err(Into::into)
					.map_err(errors::transaction)
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, _meta: Self::Metadata, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		self.fetcher().proved_execution(req, num).and_then(|res| {
			match res {
				Ok(exec) => Ok(exec.output.into()),
				Err(e) => Err(errors::execution(e)),
			}
		}).boxed()
	}

	fn estimate_gas(&self, _meta: Self::Metadata, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		// TODO: binary chop for more accurate estimates.
		self.fetcher().proved_execution(req, num).and_then(|res| {
			match res {
				Ok(exec) => Ok((exec.refunded + exec.gas_used).into()),
				Err(e) => Err(errors::execution(e)),
			}
		}).boxed()
	}

	fn transaction_by_hash(&self, hash: RpcH256) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_receipt(&self, hash: RpcH256) -> Result<Option<Receipt>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn compilers(&self) -> Result<Vec<String>, Error> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))

	}

	fn compile_lll(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of LLL via RPC is deprecated".to_string()))
	}

	fn compile_serpent(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of Serpent via RPC is deprecated".to_string()))
	}

	fn compile_solidity(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of Solidity via RPC is deprecated".to_string()))
	}

	fn logs(&self, filter: Filter) -> BoxFuture<Vec<Log>, Error> {
		let limit = filter.limit;

		Filterable::logs(self, filter.into())
			.map(move|logs| limit_logs(logs, limit))
			.boxed()
	}

	fn work(&self, _timeout: Trailing<u64>) -> Result<Work, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn submit_work(&self, _nonce: RpcH64, _pow_hash: RpcH256, _mix_hash: RpcH256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn submit_hashrate(&self, _rate: RpcU256, _id: RpcH256) -> Result<bool, Error> {
		Err(errors::light_unimplemented(None))
	}
}

// This trait implementation triggers a blanked impl of `EthFilter`.
impl<T: LightChainClient + 'static> Filterable for EthClient<T> {
	fn best_block_number(&self) -> u64 { self.client.chain_info().best_block_number }

	fn block_hash(&self, id: BlockId) -> Option<RpcH256> {
		self.client.block_hash(id).map(Into::into)
	}

	fn pending_transactions_hashes(&self, _block_number: u64) -> Vec<::util::H256> {
		Vec::new()
	}

	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>, Error> {
		self.fetcher().logs(filter)
	}

	fn pending_logs(&self, _block_number: u64, _filter: &EthcoreFilter) -> Vec<Log> {
		Vec::new() // light clients don't mine.
	}

	fn polls(&self) -> &Mutex<PollManager<PollFilter>> {
		&self.polls
	}
}
