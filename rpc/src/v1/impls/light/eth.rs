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

//! Eth RPC interface for the light client.

use std::collections::BTreeSet;
use std::sync::Arc;

use jsonrpc_core::{Result, BoxFuture};
use jsonrpc_core::futures::{future, Future};
use jsonrpc_core::futures::future::Either;

use light::cache::Cache as LightDataCache;
use light::client::LightChainClient;
use light::{cht, TransactionQueue};
use light::on_demand::{request, OnDemandRequester};

use ethereum_types::{Address, H64, H160, H256, U64, U256};
use hash::{KECCAK_NULL_RLP, KECCAK_EMPTY_LIST_RLP};
use parking_lot::{RwLock, Mutex};
use rlp::Rlp;
use types::transaction::SignedTransaction;
use types::encoded;
use types::filter::Filter as EthcoreFilter;
use types::ids::BlockId;

use v1::impls::eth_filter::Filterable;
use v1::helpers::{errors, limit_logs, SyncPollFilter, PollManager};
use v1::helpers::deprecated::{self, DeprecationNotice};
use v1::helpers::light_fetch::{self, LightFetch};
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, LightBlockNumber, Bytes, SyncStatus as RpcSyncStatus,
	SyncInfo as RpcSyncInfo, Transaction, CallRequest, Index, Filter, Log, Receipt, Work, EthAccount
};
use v1::metadata::Metadata;

use sync::{LightSyncInfo, LightSyncProvider, LightNetworkDispatcher, ManageNetwork};

const NO_INVALID_BACK_REFS: &str = "Fails only on invalid back-references; back-references here known to be valid; qed";

/// Light client `ETH` (and filter) RPC.
pub struct EthClient<C, S: LightSyncProvider + LightNetworkDispatcher + 'static, OD: OnDemandRequester + 'static> {
	sync: Arc<S>,
	client: Arc<C>,
	on_demand: Arc<OD>,
	transaction_queue: Arc<RwLock<TransactionQueue>>,
	accounts: Arc<dyn Fn() -> Vec<Address> + Send + Sync>,
	cache: Arc<Mutex<LightDataCache>>,
	polls: Mutex<PollManager<SyncPollFilter>>,
	poll_lifetime: u32,
	gas_price_percentile: usize,
	deprecation_notice: DeprecationNotice,
}

impl<C, S, OD> Clone for EthClient<C, S, OD>
where
	S: LightSyncProvider + LightNetworkDispatcher + 'static,
	OD: OnDemandRequester + 'static
{
	fn clone(&self) -> Self {
		// each instance should have its own poll manager.
		EthClient {
			sync: self.sync.clone(),
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			transaction_queue: self.transaction_queue.clone(),
			accounts: self.accounts.clone(),
			cache: self.cache.clone(),
			polls: Mutex::new(PollManager::new(self.poll_lifetime)),
			poll_lifetime: self.poll_lifetime,
			gas_price_percentile: self.gas_price_percentile,
			deprecation_notice: Default::default(),
		}
	}
}

impl<C, S, OD> EthClient<C, S, OD>
where
	C: LightChainClient + 'static,
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	/// Create a new `EthClient` with a handle to the light sync instance, client,
	/// and on-demand request service, which is assumed to be attached as a handler.
	pub fn new(
		sync: Arc<S>,
		client: Arc<C>,
		on_demand: Arc<OD>,
		transaction_queue: Arc<RwLock<TransactionQueue>>,
		accounts: Arc<dyn Fn() -> Vec<Address> + Send + Sync>,
		cache: Arc<Mutex<LightDataCache>>,
		gas_price_percentile: usize,
		poll_lifetime: u32
	) -> Self {
		EthClient {
			sync,
			client,
			on_demand,
			transaction_queue,
			accounts,
			cache,
			polls: Mutex::new(PollManager::new(poll_lifetime)),
			poll_lifetime,
			gas_price_percentile,
			deprecation_notice: Default::default(),
		}
	}

	/// Create a light data fetcher instance.
	fn fetcher(&self) -> LightFetch<S, OD>
	{
		LightFetch {
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			sync: self.sync.clone(),
			cache: self.cache.clone(),
			gas_price_percentile: self.gas_price_percentile,
		}
	}

	// get a "rich" block structure. Fails on unknown block.
	fn rich_block(&self, id: BlockId, include_txs: bool) -> BoxFuture<RichBlock> {
		let (on_demand, sync) = (self.on_demand.clone(), self.sync.clone());
		let (client, engine) = (self.client.clone(), self.client.engine().clone());

		// helper for filling out a rich block once we've got a block and a score.
		let fill_rich = move |block: encoded::Block, score: Option<U256>| {
			let header = block.decode_header();
			let extra_info = engine.extra_info(&header);
			RichBlock {
				inner: Block {
					hash: Some(header.hash()),
					size: Some(block.rlp().as_raw().len().into()),
					parent_hash: *header.parent_hash(),
					uncles_hash: *header.uncles_hash(),
					author: *header.author(),
					miner: *header.author(),
					state_root: *header.state_root(),
					transactions_root: *header.transactions_root(),
					receipts_root: *header.receipts_root(),
					number: Some(header.number().into()),
					gas_used: *header.gas_used(),
					gas_limit: *header.gas_limit(),
					logs_bloom: Some(*header.log_bloom()),
					timestamp: header.timestamp().into(),
					difficulty: *header.difficulty(),
					total_difficulty: score.map(Into::into),
					seal_fields: header.seal().iter().cloned().map(Into::into).collect(),
					uncles: block.uncle_hashes().into_iter().map(Into::into).collect(),
					transactions: match include_txs {
						true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(Transaction::from_localized).collect()),
						_ => BlockTransactions::Hashes(block.transaction_hashes().into_iter().map(Into::into).collect()),
					},
					extra_data: Bytes::new(header.extra_data().clone()),
				},
				extra_info,
			}
		};

		// get the block itself.
		Box::new(self.fetcher().block(id).and_then(move |block| {
			// then fetch the total difficulty (this is much easier after getting the block).
			match client.score(id) {
				Some(score) => Either::A(future::ok(fill_rich(block, Some(score)))),
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

							return Either::A(future::ok(fill_rich(block, Some(score))))
						}
					};

					// three possible outcomes:
					//   - network is down.
					//   - we get a score, but our hash is non-canonical.
					//   - we get a score, and our hash is canonical.
					let maybe_fut = sync.with_context(move |ctx| on_demand.request(ctx, req).expect(NO_INVALID_BACK_REFS));
					match maybe_fut {
						Some(fut) => Either::B(fut
							.map(move |(hash, score)| {
								let score = if hash == block.hash() {
									Some(score)
								} else {
									None
								};

								fill_rich(block, score)
							}).map_err(errors::on_demand_error)),
						None => Either::A(future::err(errors::network_disabled())),
					}
				}
			}
		}))
	}
}

impl<C, S, OD> Eth for EthClient<C, S, OD>
where
	C: LightChainClient + 'static,
	S: LightSyncInfo + LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String> {
		Ok(format!("{}", ::light::net::MAX_PROTOCOL_VERSION))
	}

	fn syncing(&self) -> Result<RpcSyncStatus> {
		if self.sync.is_major_importing() {
			let chain_info = self.client.chain_info();
			let current_block = U256::from(chain_info.best_block_number);
			let highest_block = self.sync.highest_block().map(U256::from)
				.unwrap_or_else(|| current_block);

			Ok(RpcSyncStatus::Info(RpcSyncInfo {
				starting_block: U256::from(self.sync.start_block()),
				current_block,
				highest_block,
				warp_chunks_amount: None,
				warp_chunks_processed: None,
			}))
		} else {
			Ok(RpcSyncStatus::None)
		}
	}

	fn author(&self) -> Result<H160> {
		(self.accounts)()
			.first()
			.cloned()
			.map(From::from)
			.ok_or_else(|| errors::account("No accounts were found", ""))
	}

	fn is_mining(&self) -> Result<bool> {
		Ok(false)
	}

	fn chain_id(&self) -> Result<Option<U64>> {
		Ok(self.client.signing_chain_id().map(U64::from))
	}

	fn hashrate(&self) -> Result<U256> {
		Ok(Default::default())
	}

	fn gas_price(&self) -> BoxFuture<U256> {
		Box::new(self.fetcher().gas_price())
	}

	fn accounts(&self) -> Result<Vec<H160>> {
		self.deprecation_notice.print("eth_accounts", deprecated::msgs::ACCOUNTS);

		Ok((self.accounts)()
			.into_iter()
			.map(Into::into)
			.collect())
	}

	fn block_number(&self) -> Result<U256> {
		Ok(self.client.chain_info().best_block_number.into())
	}

	fn balance(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<U256> {
		Box::new(self.fetcher().account(address, num.unwrap_or_default().to_block_id(), self.transaction_queue.clone())
			.map(|acc| acc.map_or(0.into(), |a| a.balance)))
	}

	fn storage_at(&self, _address: H160, _key: U256, _num: Option<BlockNumber>) -> BoxFuture<H256> {
		Box::new(future::err(errors::unimplemented(None)))
	}

	fn block_by_hash(&self, hash: H256, include_txs: bool) -> BoxFuture<Option<RichBlock>> {
		Box::new(self.rich_block(BlockId::Hash(hash), include_txs).map(Some))
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>> {
		Box::new(self.rich_block(num.to_block_id(), include_txs).map(Some))
	}

	fn transaction_count(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<U256> {
		Box::new(self.fetcher().account(address, num.unwrap_or_default().to_block_id(), self.transaction_queue.clone())
			.map(|acc| acc.map_or(0.into(), |a| a.nonce)))
	}

	fn block_transaction_count_by_hash(&self, hash: H256) -> BoxFuture<Option<U256>> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		Box::new(self.fetcher().header(BlockId::Hash(hash)).and_then(move |hdr| {
			if hdr.transactions_root() == KECCAK_NULL_RLP {
				Either::A(future::ok(Some(U256::from(0))))
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()))))
					.map(|x| Either::B(x.map_err(errors::on_demand_error)))
					.unwrap_or_else(|| Either::A(future::err(errors::network_disabled())))
			}
		}))
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<U256>> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		Box::new(self.fetcher().header(num.to_block_id()).and_then(move |hdr| {
			if hdr.transactions_root() == KECCAK_NULL_RLP {
				Either::A(future::ok(Some(U256::from(0))))
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()))))
					.map(|x| Either::B(x.map_err(errors::on_demand_error)))
					.unwrap_or_else(|| Either::A(future::err(errors::network_disabled())))
			}
		}))
	}

	fn block_uncles_count_by_hash(&self, hash: H256) -> BoxFuture<Option<U256>> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		Box::new(self.fetcher().header(BlockId::Hash(hash)).and_then(move |hdr| {
			if hdr.uncles_hash() == KECCAK_EMPTY_LIST_RLP {
				Either::A(future::ok(Some(U256::from(0))))
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()))))
					.map(|x| Either::B(x.map_err(errors::on_demand_error)))
					.unwrap_or_else(|| Either::A(future::err(errors::network_disabled())))
			}
		}))
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<U256>> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		Box::new(self.fetcher().header(num.to_block_id()).and_then(move |hdr| {
			if hdr.uncles_hash() == KECCAK_EMPTY_LIST_RLP {
				Either::B(future::ok(Some(U256::from(0))))
			} else {
				sync.with_context(|ctx| on_demand.request(ctx, request::Body(hdr.into())))
					.map(|x| x.expect(NO_INVALID_BACK_REFS))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()))))
					.map(|x| Either::A(x.map_err(errors::on_demand_error)))
					.unwrap_or_else(|| Either::B(future::err(errors::network_disabled())))
			}
		}))
	}

	fn code_at(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<Bytes> {
		Box::new(self.fetcher().code(address, num.unwrap_or_default().to_block_id()).map(Into::into))
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<H256> {
		let best_header = self.client.best_block_header().decode().map_err(errors::decode)?;

		Rlp::new(&raw.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| {
				self.client.engine().verify_transaction_basic(&tx, &best_header)
					.map_err(errors::transaction)?;

				let signed = SignedTransaction::new(tx).map_err(errors::transaction)?;
				let hash = signed.hash();

				self.transaction_queue.write().import(signed.into())
					.map(|_| hash)
					.map_err(errors::transaction)
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<H256> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, req: CallRequest, num: Option<BlockNumber>) -> BoxFuture<Bytes> {
		Box::new(self.fetcher().proved_read_only_execution(req, num, self.transaction_queue.clone()).and_then(|res| {
			match res {
				Ok(exec) => Ok(exec.output.into()),
				Err(e) => Err(errors::execution(e)),
			}
		}))
	}

	fn estimate_gas(&self, req: CallRequest, num: Option<BlockNumber>) -> BoxFuture<U256> {
		// TODO: binary chop for more accurate estimates.
		Box::new(self.fetcher().proved_read_only_execution(req, num, self.transaction_queue.clone()).and_then(|res| {
			match res {
				Ok(exec) => Ok(exec.refunded + exec.gas_used),
				Err(e) => Err(errors::execution(e)),
			}
		}))
	}

	fn transaction_by_hash(&self, hash: H256) -> BoxFuture<Option<Transaction>> {
		let in_txqueue = self.transaction_queue.read().get(&hash).is_some();

		// The transaction is in the `local txqueue` then fetch the latest state from the network and attempt
		// to cull the transaction queue.
		if in_txqueue {
			// Note, this will block (relies on HTTP timeout) to make sure `cull` will finish to avoid having to call
			// `eth_getTransactionByHash` more than once to ensure the `txqueue` is up to `date` when it is called
			if let Err(e) = self.fetcher().light_cull(self.transaction_queue.clone()).wait() {
				debug!(target: "cull", "failed because of: {:?}", e);
			}
			if let Some(tx) = self.transaction_queue.read().get(&hash) {
				return Box::new(future::ok(Some(Transaction::from_pending(
					tx.clone(),
				))));
			}
		}
		Box::new(self.fetcher().transaction_by_hash(hash).map(|x| x.map(|(tx, _)| tx)))
	}

	fn transaction_by_block_hash_and_index(&self, hash: H256, idx: Index) -> BoxFuture<Option<Transaction>> {
		Box::new(self.fetcher().block(BlockId::Hash(hash)).map(move |block| {
			light_fetch::extract_transaction_at_index(block, idx.value())
		}))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> BoxFuture<Option<Transaction>> {
		Box::new(self.fetcher().block(num.to_block_id()).map(move |block| {
			light_fetch::extract_transaction_at_index(block, idx.value())
		}))
	}

	fn transaction_receipt(&self, hash: H256) -> BoxFuture<Option<Receipt>> {
		let fetcher = self.fetcher();
		Box::new(fetcher.transaction_by_hash(hash).and_then(move |tx| {
			// the block hash included in the transaction object here has
			// already been checked for canonicality and whether it contains
			// the transaction.
			match tx {
				Some((tx, index)) => match tx.block_hash {
					Some(block_hash) => {
						let extract_receipt = fetcher.receipts(BlockId::Hash(block_hash))
							.and_then(move |mut receipts| future::ok(receipts.swap_remove(index)))
							.map(Receipt::from)
							.map(move |mut receipt| {
								receipt.transaction_hash = Some(hash);
								receipt.transaction_index = Some(index.into());
								receipt.block_hash = Some(block_hash);
								receipt.block_number = tx.block_number;
								receipt
							})
							.map(Some);

						Either::B(extract_receipt)
					}
					None => Either::A(future::err(errors::unknown_block())),
				},
				None => Either::A(future::ok(None)),
			}
		}))
	}

	fn uncle_by_block_hash_and_index(&self, hash: H256, idx: Index) -> BoxFuture<Option<RichBlock>> {
		let client = self.client.clone();
		Box::new(self.fetcher().block(BlockId::Hash(hash)).map(move |block| {
			extract_uncle_at_index(block, idx, client)
		}))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> BoxFuture<Option<RichBlock>> {
		let client = self.client.clone();
		Box::new(self.fetcher().block(num.to_block_id()).map(move |block| {
			extract_uncle_at_index(block, idx, client)
		}))
	}

	fn proof(&self, _address: H160, _values:Vec<H256>, _num: Option<BlockNumber>) -> BoxFuture<EthAccount> {
		Box::new(future::err(errors::unimplemented(None)))
	}

	fn compilers(&self) -> Result<Vec<String>> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))
	}

	fn compile_lll(&self, _: String) -> Result<Bytes> {
		Err(errors::deprecated("Compilation of LLL via RPC is deprecated".to_string()))
	}

	fn compile_serpent(&self, _: String) -> Result<Bytes> {
		Err(errors::deprecated("Compilation of Serpent via RPC is deprecated".to_string()))
	}

	fn compile_solidity(&self, _: String) -> Result<Bytes> {
		Err(errors::deprecated("Compilation of Solidity via RPC is deprecated".to_string()))
	}

	fn logs(&self, filter: Filter) -> BoxFuture<Vec<Log>> {
		let limit = filter.limit;

		Box::new(
			Filterable::logs(self, match filter.try_into() {
				Ok(value) => value,
				Err(err) => return Box::new(future::err(err)),
			}).map(move |logs| limit_logs(logs, limit)))
	}

	fn work(&self, _timeout: Option<u64>) -> Result<Work> {
		Err(errors::light_unimplemented(None))
	}

	fn submit_work(&self, _nonce: H64, _pow_hash: H256, _mix_hash: H256) -> Result<bool> {
		Err(errors::light_unimplemented(None))
	}

	fn submit_hashrate(&self, _rate: U256, _id: H256) -> Result<bool> {
		Err(errors::light_unimplemented(None))
	}
}

// This trait implementation triggers a blanked impl of `EthFilter`.
impl<C, S, OD> Filterable for EthClient<C, S, OD>
where
	C: LightChainClient + 'static,
	S: LightSyncProvider + LightNetworkDispatcher + ManageNetwork + 'static,
	OD: OnDemandRequester + 'static
{
	fn best_block_number(&self) -> u64 { self.client.chain_info().best_block_number }

	fn block_hash(&self, id: BlockId) -> Option<H256> {
		self.client.block_hash(id)
	}

	fn pending_transaction_hashes(&self) -> BTreeSet<H256> {
		BTreeSet::new()
	}

	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>> {
		Box::new(self.fetcher().logs(filter)) as BoxFuture<_>
	}

	fn pending_logs(&self, _block_number: u64, _filter: &EthcoreFilter) -> Vec<Log> {
		Vec::new() // light clients don't mine.
	}

	fn polls(&self) -> &Mutex<PollManager<SyncPollFilter>> {
		&self.polls
	}

	fn removed_logs(&self, _block_hash: ::ethereum_types::H256, _filter: &EthcoreFilter) -> (Vec<Log>, u64) {
		(Default::default(), 0)
	}
}

fn extract_uncle_at_index<T: LightChainClient>(block: encoded::Block, index: Index, client: Arc<T>) -> Option<RichBlock> {
		let uncle = match block.uncles().into_iter().nth(index.value()) {
			Some(u) => u,
			None => return None,
		};

		let extra_info = client.engine().extra_info(&uncle);
		Some(RichBlock {
			inner: Block {
				hash: Some(uncle.hash()),
				size: None,
				parent_hash: *uncle.parent_hash(),
				uncles_hash: *uncle.uncles_hash(),
				author: *uncle.author(),
				miner: *uncle.author(),
				state_root: *uncle.state_root(),
				transactions_root: *uncle.transactions_root(),
				number: Some(uncle.number().into()),
				gas_used: *uncle.gas_used(),
				gas_limit: *uncle.gas_limit(),
				logs_bloom: Some(*uncle.log_bloom()),
				timestamp: uncle.timestamp().into(),
				difficulty: *uncle.difficulty(),
				total_difficulty: None,
				receipts_root: *uncle.receipts_root(),
				extra_data: uncle.extra_data().clone().into(),
				seal_fields: uncle.seal().iter().cloned().map(Into::into).collect(),
				uncles: vec![],
				transactions: BlockTransactions::Hashes(vec![]),
			},
			extra_info,
		})
}
