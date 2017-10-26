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

//! Eth rpc implementation.

use std::thread;
use std::time::{Instant, Duration};
use std::sync::Arc;

use rlp::{self, UntrustedRlp};
use time::get_time;
use bigint::prelude::U256;
use bigint::hash::{H64, H160, H256};
use util::Address;
use parking_lot::Mutex;

use ethash::SeedHashCompute;
use ethcore::account_provider::{AccountProvider, DappId};
use ethcore::block::IsBlock;
use ethcore::client::{MiningBlockChainClient, BlockId, TransactionId, UncleId};
use ethcore::ethereum::Ethash;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::header::{Header as BlockHeader, BlockNumber as EthBlockNumber};
use ethcore::log_entry::LogEntry;
use ethcore::miner::{MinerService, ExternalMinerService};
use ethcore::transaction::SignedTransaction;
use ethcore::snapshot::SnapshotService;
use ethsync::{SyncProvider};

use jsonrpc_core::{BoxFuture, Error};
use jsonrpc_core::futures::future;
use jsonrpc_macros::Trailing;

use v1::helpers::{errors, limit_logs, fake_sign};
use v1::helpers::dispatch::{Dispatcher, FullDispatcher, default_gas_price};
use v1::helpers::block_import::is_major_importing;
use v1::helpers::accounts::unwrap_provider;
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo,
	Transaction, CallRequest, Index, Filter, Log, Receipt, Work,
	H64 as RpcH64, H256 as RpcH256, H160 as RpcH160, U256 as RpcU256,
};
use v1::metadata::Metadata;

const EXTRA_INFO_PROOF: &'static str = "Object exists in in blockchain (fetched earlier), extra_info is always available if object exists; qed";

/// Eth RPC options
pub struct EthClientOptions {
	/// Return nonce from transaction queue when pending block not available.
	pub pending_nonce_from_queue: bool,
	/// Returns receipt from pending blocks
	pub allow_pending_receipt_query: bool,
	/// Send additional block number when asking for work
	pub send_block_number_in_get_work: bool,
}

impl EthClientOptions {
	/// Creates new default `EthClientOptions` and allows alterations
	/// by provided function.
	pub fn with<F: Fn(&mut Self)>(fun: F) -> Self {
		let mut options = Self::default();
		fun(&mut options);
		options
	}
}

impl Default for EthClientOptions {
	fn default() -> Self {
		EthClientOptions {
			pending_nonce_from_queue: false,
			allow_pending_receipt_query: true,
			send_block_number_in_get_work: true,
		}
	}
}

/// Eth rpc implementation.
pub struct EthClient<C, SN: ?Sized, S: ?Sized, M, EM> where
	C: MiningBlockChainClient,
	SN: SnapshotService,
	S: SyncProvider,
	M: MinerService,
	EM: ExternalMinerService {

	client: Arc<C>,
	snapshot: Arc<SN>,
	sync: Arc<S>,
	accounts: Option<Arc<AccountProvider>>,
	miner: Arc<M>,
	external_miner: Arc<EM>,
	seed_compute: Mutex<SeedHashCompute>,
	options: EthClientOptions,
	eip86_transition: u64,
}

impl<C, SN: ?Sized, S: ?Sized, M, EM> EthClient<C, SN, S, M, EM> where
	C: MiningBlockChainClient,
	SN: SnapshotService,
	S: SyncProvider,
	M: MinerService,
	EM: ExternalMinerService {

	/// Creates new EthClient.
	pub fn new(
		client: &Arc<C>,
		snapshot: &Arc<SN>,
		sync: &Arc<S>,
		accounts: &Option<Arc<AccountProvider>>,
		miner: &Arc<M>,
		em: &Arc<EM>,
		options: EthClientOptions
	) -> Self {
		EthClient {
			client: client.clone(),
			snapshot: snapshot.clone(),
			sync: sync.clone(),
			miner: miner.clone(),
			accounts: accounts.clone(),
			external_miner: em.clone(),
			seed_compute: Mutex::new(SeedHashCompute::new()),
			options: options,
			eip86_transition: client.eip86_transition(),
		}
	}

	/// Attempt to get the `Arc<AccountProvider>`, errors if provider was not
	/// set.
	fn account_provider(&self) -> Result<Arc<AccountProvider>, Error> {
		unwrap_provider(&self.accounts)
	}

	fn block(&self, id: BlockId, include_txs: bool) -> Result<Option<RichBlock>, Error> {
		let client = &self.client;
		match (client.block(id.clone()), client.block_total_difficulty(id)) {
			(Some(block), Some(total_difficulty)) => {
				let view = block.header_view();
				Ok(Some(RichBlock {
					inner: Block {
						hash: Some(view.hash().into()),
						size: Some(block.rlp().as_raw().len().into()),
						parent_hash: view.parent_hash().into(),
						uncles_hash: view.uncles_hash().into(),
						author: view.author().into(),
						miner: view.author().into(),
						state_root: view.state_root().into(),
						transactions_root: view.transactions_root().into(),
						receipts_root: view.receipts_root().into(),
						number: Some(view.number().into()),
						gas_used: view.gas_used().into(),
						gas_limit: view.gas_limit().into(),
						logs_bloom: view.log_bloom().into(),
						timestamp: view.timestamp().into(),
						difficulty: view.difficulty().into(),
						total_difficulty: Some(total_difficulty.into()),
						seal_fields: view.seal().into_iter().map(Into::into).collect(),
						uncles: block.uncle_hashes().into_iter().map(Into::into).collect(),
						transactions: match include_txs {
							true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(|t| Transaction::from_localized(t, self.eip86_transition)).collect()),
							false => BlockTransactions::Hashes(block.transaction_hashes().into_iter().map(Into::into).collect()),
						},
						extra_data: Bytes::new(view.extra_data()),
					},
					extra_info: client.block_extra_info(id.clone()).expect(EXTRA_INFO_PROOF),
				}))
			},
			_ => Ok(None)
		}
	}

	fn transaction(&self, id: TransactionId) -> Result<Option<Transaction>, Error> {
		match self.client.transaction(id) {
			Some(t) => Ok(Some(Transaction::from_localized(t, self.eip86_transition))),
			None => Ok(None),
		}
	}

	fn uncle(&self, id: UncleId) -> Result<Option<RichBlock>, Error> {
		let client = &self.client;
		let uncle: BlockHeader = match client.uncle(id) {
			Some(hdr) => hdr.decode(),
			None => { return Ok(None); }
		};
		let parent_difficulty = match client.block_total_difficulty(BlockId::Hash(uncle.parent_hash().clone())) {
			Some(difficulty) => difficulty,
			None => { return Ok(None); }
		};

		let size = client.block(BlockId::Hash(uncle.hash()))
			.map(|block| block.into_inner().len())
			.map(U256::from)
			.map(Into::into);

		let block = RichBlock {
			inner: Block {
				hash: Some(uncle.hash().into()),
				size: size,
				parent_hash: uncle.parent_hash().clone().into(),
				uncles_hash: uncle.uncles_hash().clone().into(),
				author: uncle.author().clone().into(),
				miner: uncle.author().clone().into(),
				state_root: uncle.state_root().clone().into(),
				transactions_root: uncle.transactions_root().clone().into(),
				number: Some(uncle.number().into()),
				gas_used: uncle.gas_used().clone().into(),
				gas_limit: uncle.gas_limit().clone().into(),
				logs_bloom: uncle.log_bloom().clone().into(),
				timestamp: uncle.timestamp().into(),
				difficulty: uncle.difficulty().clone().into(),
				total_difficulty: Some((uncle.difficulty().clone() + parent_difficulty).into()),
				receipts_root: uncle.receipts_root().clone().into(),
				extra_data: uncle.extra_data().clone().into(),
				seal_fields: uncle.seal().into_iter().cloned().map(Into::into).collect(),
				uncles: vec![],
				transactions: BlockTransactions::Hashes(vec![]),
			},
			extra_info: client.uncle_extra_info(id).expect(EXTRA_INFO_PROOF),
		};
		Ok(Some(block))
	}

	fn dapp_accounts(&self, dapp: DappId) -> Result<Vec<H160>, Error> {
		let store = self.account_provider()?;
		store
			.note_dapp_used(dapp.clone())
			.and_then(|_| store.dapp_addresses(dapp))
			.map_err(|e| errors::account("Could not fetch accounts.", e))
	}
}

pub fn pending_logs<M>(miner: &M, best_block: EthBlockNumber, filter: &EthcoreFilter) -> Vec<Log> where M: MinerService {
	let receipts = miner.pending_receipts(best_block);

	let pending_logs = receipts.into_iter()
		.flat_map(|(hash, r)| r.logs.into_iter().map(|l| (hash.clone(), l)).collect::<Vec<(H256, LogEntry)>>())
		.collect::<Vec<(H256, LogEntry)>>();

	let result = pending_logs.into_iter()
		.filter(|pair| filter.matches(&pair.1))
		.map(|pair| {
			let mut log = Log::from(pair.1);
			log.transaction_hash = Some(pair.0.into());
			log
		})
		.collect();

	result
}

fn check_known<C>(client: &C, number: BlockNumber) -> Result<(), Error> where C: MiningBlockChainClient {
	use ethcore::block_status::BlockStatus;

	match client.block_status(number.into()) {
		BlockStatus::InChain => Ok(()),
		BlockStatus::Pending => Ok(()),
		_ => Err(errors::unknown_block()),
	}
}

const MAX_QUEUE_SIZE_TO_MINE_ON: usize = 4;	// because uncles go back 6.

impl<C, SN: ?Sized, S: ?Sized, M, EM> Eth for EthClient<C, SN, S, M, EM> where
	C: MiningBlockChainClient + 'static,
	SN: SnapshotService + 'static,
	S: SyncProvider + 'static,
	M: MinerService + 'static,
	EM: ExternalMinerService + 'static,
{
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String, Error> {
		let version = self.sync.status().protocol_version.to_owned();
		Ok(format!("{}", version))
	}

	fn syncing(&self) -> Result<SyncStatus, Error> {
		use ethcore::snapshot::RestorationStatus;

		let status = self.sync.status();
		let client = &self.client;
		let snapshot_status = self.snapshot.status();

		let (warping, warp_chunks_amount, warp_chunks_processed) = match snapshot_status {
			RestorationStatus::Ongoing { state_chunks, block_chunks, state_chunks_done, block_chunks_done } =>
				(true, Some(block_chunks + state_chunks), Some(block_chunks_done + state_chunks_done)),
			_ => (false, None, None),
		};


		if warping || is_major_importing(Some(status.state), client.queue_info()) {
			let chain_info = client.chain_info();
			let current_block = U256::from(chain_info.best_block_number);
			let highest_block = U256::from(status.highest_block_number.unwrap_or(status.start_block_number));

			let info = SyncInfo {
				starting_block: status.start_block_number.into(),
				current_block: current_block.into(),
				highest_block: highest_block.into(),
				warp_chunks_amount: warp_chunks_amount.map(|x| U256::from(x as u64)).map(Into::into),
				warp_chunks_processed: warp_chunks_processed.map(|x| U256::from(x as u64)).map(Into::into),
			};
			Ok(SyncStatus::Info(info))
		} else {
			Ok(SyncStatus::None)
		}
	}

	fn author(&self, meta: Metadata) -> Result<RpcH160, Error> {
		let dapp = meta.dapp_id();

		let mut miner = self.miner.author();
		if miner == 0.into() {
			miner = self.dapp_accounts(dapp.into())?.get(0).cloned().unwrap_or_default();
		}

		Ok(RpcH160::from(miner))
	}

	fn is_mining(&self) -> Result<bool, Error> {
		Ok(self.miner.is_currently_sealing())
	}

	fn hashrate(&self) -> Result<RpcU256, Error> {
		Ok(RpcU256::from(self.external_miner.hashrate()))
	}

	fn gas_price(&self) -> Result<RpcU256, Error> {
		Ok(RpcU256::from(default_gas_price(&*self.client, &*self.miner)))
	}

	fn accounts(&self, meta: Metadata) -> Result<Vec<RpcH160>, Error> {
		let dapp = meta.dapp_id();

		let accounts = self.dapp_accounts(dapp.into())?;
		Ok(accounts.into_iter().map(Into::into).collect())
	}

	fn block_number(&self) -> Result<RpcU256, Error> {
		Ok(RpcU256::from(self.client.chain_info().best_block_number))
	}

	fn balance(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		let address = address.into();

		let id = num.unwrap_or_default();

		try_bf!(check_known(&*self.client, id.clone()));
		let res = match self.client.balance(&address, id.into()) {
			Some(balance) => Ok(balance.into()),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn storage_at(&self, address: RpcH160, pos: RpcU256, num: Trailing<BlockNumber>) -> BoxFuture<RpcH256, Error> {
		let address: Address = RpcH160::into(address);
		let position: U256 = RpcU256::into(pos);

		let id = num.unwrap_or_default();

		try_bf!(check_known(&*self.client, id.clone()));
		let res = match self.client.storage_at(&address, &H256::from(position), id.into()) {
			Some(s) => Ok(s.into()),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn transaction_count(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		let address: Address = RpcH160::into(address);

		let res = match num.unwrap_or_default() {
			BlockNumber::Pending if self.options.pending_nonce_from_queue => {
				let nonce = self.miner.last_nonce(&address)
					.map(|n| n + 1.into())
					.or_else(|| self.client.nonce(&address, BlockNumber::Pending.into()));
				match nonce {
					Some(nonce) => Ok(nonce.into()),
					None => Err(errors::database("latest nonce missing"))
				}
			}
			id => {
				try_bf!(check_known(&*self.client, id.clone()));
				match self.client.nonce(&address, id.into()) {
					Some(nonce) => Ok(nonce.into()),
					None => Err(errors::state_pruned()),
				}
			}
		};

		Box::new(future::done(res))
	}

	fn block_transaction_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		Box::new(future::ok(self.client.block(BlockId::Hash(hash.into()))
			.map(|block| block.transactions_count().into())))
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		Box::new(future::ok(match num {
			BlockNumber::Pending => Some(
				self.miner.status().transactions_in_pending_block.into()
			),
			_ =>
				self.client.block(num.into())
					.map(|block| block.transactions_count().into())
		}))
	}

	fn block_uncles_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		Box::new(future::ok(self.client.block(BlockId::Hash(hash.into()))
			.map(|block| block.uncles_count().into())))
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		Box::new(future::ok(match num {
			BlockNumber::Pending => Some(0.into()),
			_ => self.client.block(num.into())
					.map(|block| block.uncles_count().into()
			),
		}))
	}

	fn code_at(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		let address: Address = RpcH160::into(address);

		let id = num.unwrap_or_default();
		try_bf!(check_known(&*self.client, id.clone()));

		let res = match self.client.code(&address, id.into()) {
			Some(code) => Ok(code.map_or_else(Bytes::default, Bytes::new)),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		Box::new(future::done(self.block(BlockId::Hash(hash.into()), include_txs)))
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		Box::new(future::done(self.block(num.into(), include_txs)))
	}

	fn transaction_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<Transaction>, Error> {
		let hash: H256 = hash.into();
		let block_number = self.client.chain_info().best_block_number;
		let tx = try_bf!(self.transaction(TransactionId::Hash(hash))).or_else(|| {
			self.miner.transaction(block_number, &hash)
				.map(|t| Transaction::from_pending(t, block_number, self.eip86_transition))
		});

		Box::new(future::ok(tx))
	}

	fn transaction_by_block_hash_and_index(&self, hash: RpcH256, index: Index) -> BoxFuture<Option<Transaction>, Error> {
		Box::new(future::done(
			self.transaction(TransactionId::Location(BlockId::Hash(hash.into()), index.value()))
		))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> BoxFuture<Option<Transaction>, Error> {
		Box::new(future::done(
			self.transaction(TransactionId::Location(num.into(), index.value()))
		))
	}

	fn transaction_receipt(&self, hash: RpcH256) -> BoxFuture<Option<Receipt>, Error> {
		let best_block = self.client.chain_info().best_block_number;
		let hash: H256 = hash.into();

		match (self.miner.pending_receipt(best_block, &hash), self.options.allow_pending_receipt_query) {
			(Some(receipt), true) => Box::new(future::ok(Some(receipt.into()))),
			_ => {
				let receipt = self.client.transaction_receipt(TransactionId::Hash(hash));
				Box::new(future::ok(receipt.map(Into::into)))
			}
		}
	}

	fn uncle_by_block_hash_and_index(&self, hash: RpcH256, index: Index) -> BoxFuture<Option<RichBlock>, Error> {
		Box::new(future::done(self.uncle(UncleId {
			block: BlockId::Hash(hash.into()),
			position: index.value()
		})))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> BoxFuture<Option<RichBlock>, Error> {
		Box::new(future::done(self.uncle(UncleId {
			block: num.into(),
			position: index.value()
		})))
	}

	fn compilers(&self) -> Result<Vec<String>, Error> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))
	}

	fn logs(&self, filter: Filter) -> BoxFuture<Vec<Log>, Error> {
		let include_pending = filter.to_block == Some(BlockNumber::Pending);
		let filter: EthcoreFilter = filter.into();
		let mut logs = self.client.logs(filter.clone())
			.into_iter()
			.map(From::from)
			.collect::<Vec<Log>>();

		if include_pending {
			let best_block = self.client.chain_info().best_block_number;
			let pending = pending_logs(&*self.miner, best_block, &filter);
			logs.extend(pending);
		}

		let logs = limit_logs(logs, filter.limit);

		Box::new(future::ok(logs))
	}

	fn work(&self, no_new_work_timeout: Trailing<u64>) -> Result<Work, Error> {
		if !self.miner.can_produce_work_package() {
			warn!(target: "miner", "Cannot give work package - engine seals internally.");
			return Err(errors::no_work_required())
		}

		let no_new_work_timeout = no_new_work_timeout.unwrap_or_default();

		// check if we're still syncing and return empty strings in that case
		{
			//TODO: check if initial sync is complete here
			//let sync = self.sync;
			if /*sync.status().state != SyncState::Idle ||*/ self.client.queue_info().total_queue_size() > MAX_QUEUE_SIZE_TO_MINE_ON {
				trace!(target: "miner", "Syncing. Cannot give any work.");
				return Err(errors::no_work());
			}

			// Otherwise spin until our submitted block has been included.
			let timeout = Instant::now() + Duration::from_millis(1000);
			while Instant::now() < timeout && self.client.queue_info().total_queue_size() > 0 {
				thread::sleep(Duration::from_millis(1));
			}
		}

		if self.miner.author().is_zero() {
			warn!(target: "miner", "Cannot give work package - no author is configured. Use --author to configure!");
			return Err(errors::no_author())
		}
		self.miner.map_sealing_work(&*self.client, |b| {
			let pow_hash = b.hash();
			let target = Ethash::difficulty_to_boundary(b.block().header().difficulty());
			let seed_hash = self.seed_compute.lock().hash_block_number(b.block().header().number());

			if no_new_work_timeout > 0 && b.block().header().timestamp() + no_new_work_timeout < get_time().sec as u64 {
				Err(errors::no_new_work())
			} else if self.options.send_block_number_in_get_work {
				let block_number = b.block().header().number();
				Ok(Work {
					pow_hash: pow_hash.into(),
					seed_hash: seed_hash.into(),
					target: target.into(),
					number: Some(block_number),
				})
			} else {
				Ok(Work {
					pow_hash: pow_hash.into(),
					seed_hash: seed_hash.into(),
					target: target.into(),
					number: None
				})
			}
		}).unwrap_or(Err(errors::internal("No work found.", "")))
	}

	fn submit_work(&self, nonce: RpcH64, pow_hash: RpcH256, mix_hash: RpcH256) -> Result<bool, Error> {
		if !self.miner.can_produce_work_package() {
			warn!(target: "miner", "Cannot submit work - engine seals internally.");
			return Err(errors::no_work_required())
		}

		let nonce: H64 = nonce.into();
		let pow_hash: H256 = pow_hash.into();
		let mix_hash: H256 = mix_hash.into();
		trace!(target: "miner", "submit_work: Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);

		let seal = vec![rlp::encode(&mix_hash).into_vec(), rlp::encode(&nonce).into_vec()];
		Ok(self.miner.submit_seal(&*self.client, pow_hash, seal).is_ok())
	}

	fn submit_hashrate(&self, rate: RpcU256, id: RpcH256) -> Result<bool, Error> {
		self.external_miner.submit_hashrate(rate.into(), id.into());
		Ok(true)
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		UntrustedRlp::new(&raw.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))
			.and_then(|signed_transaction| {
				FullDispatcher::new(self.client.clone(), self.miner.clone())
					.dispatch_transaction(signed_transaction.into())
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, meta: Self::Metadata, request: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		let request = CallRequest::into(request);
		let signed = try_bf!(fake_sign::sign_call(&self.client, &self.miner, request, meta.is_dapp()));

		let num = num.unwrap_or_default();
		let result = self.client.call(&signed, Default::default(), num.into());

		Box::new(future::done(result
			.map(|b| b.output.into())
			.map_err(errors::call)
		))
	}

	fn estimate_gas(&self, meta: Self::Metadata, request: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		let request = CallRequest::into(request);
		let signed = try_bf!(fake_sign::sign_call(&self.client, &self.miner, request, meta.is_dapp()));
		Box::new(future::done(self.client.estimate_gas(&signed, num.unwrap_or_default().into())
			.map(Into::into)
			.map_err(errors::call)
		))
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
}
