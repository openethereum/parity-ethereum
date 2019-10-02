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

//! Eth rpc implementation.

use std::thread;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use rlp::Rlp;
use ethereum_types::{Address, H64, H160, H256, U64, U256, BigEndianHash};
use parking_lot::Mutex;

use account_state::state::StateInfo;
use client_traits::{BlockChainClient, StateClient, ProvingBlockChainClient, StateOrBlock, StateResult};
use ethash::{self, SeedHashCompute};
use ethcore::client::{Call, EngineInfo};
use ethcore::miner::{self, MinerService};
use snapshot::SnapshotService;
use hash::keccak;
use miner::external::ExternalMinerService;
use sync::SyncProvider;
use types::{
	BlockNumber as EthBlockNumber,
	encoded,
	ids::{BlockId, TransactionId, UncleId},
	filter::Filter as EthcoreFilter,
	transaction::{SignedTransaction, LocalizedTransaction},
	snapshot::RestorationStatus,
};

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::future;

use v1::helpers::{self, errors, limit_logs, fake_sign};
use v1::helpers::deprecated::{self, DeprecationNotice};
use v1::helpers::dispatch::{FullDispatcher, default_gas_price};
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo,
	Transaction, CallRequest, Index, Filter, Log, Receipt, Work, EthAccount, StorageProof,
	block_number_to_id
};
use v1::metadata::Metadata;

const EXTRA_INFO_PROOF: &str = "Object exists in blockchain (fetched earlier), extra_info is always available if object exists; qed";

/// Eth RPC options
#[derive(Copy, Clone)]
pub struct EthClientOptions {
	/// Return nonce from transaction queue when pending block not available.
	pub pending_nonce_from_queue: bool,
	/// Returns receipt from pending blocks
	pub allow_pending_receipt_query: bool,
	/// Send additional block number when asking for work
	pub send_block_number_in_get_work: bool,
	/// Gas Price Percentile used as default gas price.
	pub gas_price_percentile: usize,
	/// Return 'null' instead of an error if ancient block sync is still in
	/// progress and the block information requested could not be found.
	pub allow_missing_blocks: bool,
	/// Enable Experimental RPC-Calls
	pub allow_experimental_rpcs: bool,
	/// flag for ancient block sync
	pub no_ancient_blocks: bool,
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
			gas_price_percentile: 50,
			allow_missing_blocks: false,
			allow_experimental_rpcs: false,
			no_ancient_blocks: false,
		}
	}
}

/// Eth rpc implementation.
pub struct EthClient<C, SN: ?Sized, S: ?Sized, M, EM> where
	C: miner::BlockChainClient + BlockChainClient,
	SN: SnapshotService,
	S: SyncProvider,
	M: MinerService,
	EM: ExternalMinerService {

	client: Arc<C>,
	snapshot: Arc<SN>,
	sync: Arc<S>,
	accounts: Arc<dyn Fn() -> Vec<Address> + Send + Sync>,
	miner: Arc<M>,
	external_miner: Arc<EM>,
	seed_compute: Mutex<SeedHashCompute>,
	options: EthClientOptions,
	deprecation_notice: DeprecationNotice,
}

#[derive(Debug)]
enum BlockNumberOrId {
	Number(BlockNumber),
	Id(BlockId),
}

impl From<BlockId> for BlockNumberOrId {
	fn from(value: BlockId) -> BlockNumberOrId {
		BlockNumberOrId::Id(value)
	}
}

impl From<BlockNumber> for BlockNumberOrId {
	fn from(value: BlockNumber) -> BlockNumberOrId {
		BlockNumberOrId::Number(value)
	}
}

enum PendingOrBlock {
	Block(BlockId),
	Pending,
}

struct PendingUncleId {
	id: PendingOrBlock,
	position: usize,
}

enum PendingTransactionId {
	Hash(H256),
	Location(PendingOrBlock, usize)
}

pub fn base_logs<C, M, T: StateInfo + 'static> (client: &C, miner: &M, filter: Filter) -> BoxFuture<Vec<Log>> where
	C: miner::BlockChainClient + BlockChainClient + StateClient<State=T> + Call<State=T>,
	M: MinerService<State=T> {
	let include_pending = filter.to_block == Some(BlockNumber::Pending);
	let filter: EthcoreFilter = match filter.try_into() {
		Ok(value) => value,
		Err(err) => return Box::new(future::err(err)),
	};
	let mut logs = match client.logs(filter.clone()) {
		Ok(logs) => logs
			.into_iter()
			.map(From::from)
			.collect::<Vec<Log>>(),
		Err(id) => return Box::new(future::err(errors::filter_block_not_found(id))),
	};

	if include_pending {
		let best_block = client.chain_info().best_block_number;
		let pending = pending_logs(&*miner, best_block, &filter);
		logs.extend(pending);
	}

	let logs = limit_logs(logs, filter.limit);

	Box::new(future::ok(logs))
}

impl<C, SN: ?Sized, S: ?Sized, M, EM, T: StateInfo + 'static> EthClient<C, SN, S, M, EM> where
	C: miner::BlockChainClient + BlockChainClient + StateClient<State=T> + Call<State=T> + EngineInfo,
	SN: SnapshotService,
	S: SyncProvider,
	M: MinerService<State=T>,
	EM: ExternalMinerService {

	/// Creates new EthClient.
	pub fn new(
		client: &Arc<C>,
		snapshot: &Arc<SN>,
		sync: &Arc<S>,
		accounts: &Arc<dyn Fn() -> Vec<Address> + Send + Sync>,
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
			seed_compute: Mutex::new(SeedHashCompute::default()),
			options,
			deprecation_notice: Default::default(),
		}
	}

	fn rich_block(&self, id: BlockNumberOrId, include_txs: bool) -> Result<Option<RichBlock>> {
		let client = &self.client;

		let client_query = |id| (client.block(id), client.block_total_difficulty(id), client.block_extra_info(id), false);

		let (block, difficulty, extra, is_pending) = match id {
			BlockNumberOrId::Number(BlockNumber::Pending) => {
				let info = self.client.chain_info();
				match self.miner.pending_block(info.best_block_number) {
					Some(pending_block) => {
						warn!("`Pending` is deprecated and may be removed in future versions.");

						let difficulty = {
							let latest_difficulty = self.client.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed");
							let pending_difficulty = self.miner.pending_block_header(info.best_block_number).map(|header| *header.difficulty());

							if let Some(difficulty) = pending_difficulty {
								difficulty + latest_difficulty
							} else {
								latest_difficulty
							}
						};

						let extra = self.client.engine().extra_info(&pending_block.header);

						(Some(encoded::Block::new(pending_block.rlp_bytes())), Some(difficulty), Some(extra), true)
					},
					None => {
						warn!("`Pending` is deprecated and may be removed in future versions. Falling back to `Latest`");
						client_query(BlockId::Latest)
					}
				}
			},

			BlockNumberOrId::Number(num) => {
				let id = match num {
					BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
					BlockNumber::Latest => BlockId::Latest,
					BlockNumber::Earliest => BlockId::Earliest,
					BlockNumber::Num(n) => BlockId::Number(n),
					BlockNumber::Pending => unreachable!() // Already covered
				};

				client_query(id)
			},

			BlockNumberOrId::Id(id) => client_query(id),
		};

		match (block, difficulty) {
			(Some(block), Some(total_difficulty)) => {
				let view = block.header_view();
				Ok(Some(RichBlock {
					inner: Block {
						hash: match is_pending {
							true => None,
							false => Some(view.hash()),
						},
						size: Some(block.rlp().as_raw().len().into()),
						parent_hash: view.parent_hash(),
						uncles_hash: view.uncles_hash(),
						author: view.author(),
						miner: view.author(),
						state_root: view.state_root(),
						transactions_root: view.transactions_root(),
						receipts_root: view.receipts_root(),
						number: match is_pending {
							true => None,
							false => Some(view.number().into()),
						},
						gas_used: view.gas_used(),
						gas_limit: view.gas_limit(),
						logs_bloom: match is_pending {
							true => None,
							false => Some(view.log_bloom()),
						},
						timestamp: view.timestamp().into(),
						difficulty: view.difficulty(),
						total_difficulty: Some(total_difficulty),
						seal_fields: view.seal().into_iter().map(Into::into).collect(),
						uncles: block.uncle_hashes(),
						transactions: match include_txs {
							true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(Transaction::from_localized).collect()),
							false => BlockTransactions::Hashes(block.transaction_hashes()),
						},
						extra_data: Bytes::new(view.extra_data()),
					},
					extra_info: extra.expect(EXTRA_INFO_PROOF),
				}))
			},
			_ => Ok(None)
		}
	}

	fn transaction(&self, id: PendingTransactionId) -> Result<Option<Transaction>> {
		let client_transaction = |id| match self.client.transaction(id) {
			Some(t) => Ok(Some(Transaction::from_localized(t))),
			None => Ok(None),
		};

		match id {
			PendingTransactionId::Hash(hash) => client_transaction(TransactionId::Hash(hash)),

			PendingTransactionId::Location(PendingOrBlock::Block(block), index) => {
				client_transaction(TransactionId::Location(block, index))
			},

			PendingTransactionId::Location(PendingOrBlock::Pending, index) => {
				let info = self.client.chain_info();
				let pending_block = match self.miner.pending_block(info.best_block_number) {
					Some(block) => block,
					None => return Ok(None),
				};

				// Implementation stolen from `extract_transaction_at_index`
				let transaction = pending_block.transactions.get(index)
					// Verify if transaction signature is correct.
					.and_then(|tx| SignedTransaction::new(tx.clone()).ok())
					.map(|signed_tx| {
						let (signed, sender, _) = signed_tx.deconstruct();
						let block_hash = pending_block.header.hash();
						let block_number = pending_block.header.number();
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
					.map(Transaction::from_localized);

				Ok(transaction)
			}
		}
	}

	fn uncle(&self, id: PendingUncleId) -> Result<Option<RichBlock>> {
		let client = &self.client;

		let (uncle, parent_difficulty, extra) = match id {
			PendingUncleId { id: PendingOrBlock::Pending, position } => {
				let info = self.client.chain_info();

				let pending_block = match self.miner.pending_block(info.best_block_number) {
					Some(block) => block,
					None => return Ok(None),
				};

				let uncle = match pending_block.uncles.get(position) {
					Some(uncle) => uncle.clone(),
					None => return Ok(None),
				};

				let difficulty = {
					let latest_difficulty = self.client.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed");
					let pending_difficulty = self.miner.pending_block_header(info.best_block_number).map(|header| *header.difficulty());

					if let Some(difficulty) = pending_difficulty {
						difficulty + latest_difficulty
					} else {
						latest_difficulty
					}
				};

				let extra = self.client.engine().extra_info(&pending_block.header);

				(uncle, difficulty, extra)
			},

			PendingUncleId { id: PendingOrBlock::Block(block_id), position } => {
				let uncle_id = UncleId { block: block_id, position };

				let uncle = match client.uncle(uncle_id) {
					Some(hdr) => match hdr.decode() {
						Ok(h) => h,
						Err(e) => return Err(errors::decode(e))
					},
					None => { return Ok(None); }
				};

				let parent_difficulty = match client.block_total_difficulty(BlockId::Hash(*uncle.parent_hash())) {
					Some(difficulty) => difficulty,
					None => { return Ok(None); }
				};

				let extra = client.uncle_extra_info(uncle_id).expect(EXTRA_INFO_PROOF);

				(uncle, parent_difficulty, extra)
			}
		};

		let size = client.block(BlockId::Hash(uncle.hash()))
			.map(|block| block.into_inner().len())
			.map(U256::from);

		let block = RichBlock {
			inner: Block {
				hash: Some(uncle.hash()),
				size,
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
				total_difficulty: Some(uncle.difficulty() + parent_difficulty),
				receipts_root: *uncle.receipts_root(),
				extra_data: uncle.extra_data().clone().into(),
				seal_fields: uncle.seal().iter().cloned().map(Into::into).collect(),
				uncles: vec![],
				transactions: BlockTransactions::Hashes(vec![]),
			},
			extra_info: extra,
		};
		Ok(Some(block))
	}

	fn get_state(&self, number: BlockNumber) -> StateOrBlock {
		match number {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash).into(),
			BlockNumber::Num(num) => BlockId::Number(num).into(),
			BlockNumber::Earliest => BlockId::Earliest.into(),
			BlockNumber::Latest => BlockId::Latest.into(),
			BlockNumber::Pending => {
				let info = self.client.chain_info();

				self.miner
					.pending_state(info.best_block_number)
					.map(|s| Box::new(s) as Box<dyn StateInfo>)
					.unwrap_or(Box::new(self.client.latest_state()) as Box<dyn StateInfo>)
					.into()
			}
		}
	}
}

pub fn pending_logs<M>(miner: &M, best_block: EthBlockNumber, filter: &EthcoreFilter) -> Vec<Log> where M: MinerService {
	let receipts = miner.pending_receipts(best_block).unwrap_or_default();

	receipts.into_iter()
		.flat_map(|r| {
			let hash = r.transaction_hash;
			r.logs.into_iter().map(move |l| (hash, l))
		})
		.filter(|pair| filter.matches(&pair.1))
		.map(|pair| {
			let mut log = Log::from(pair.1);
			log.transaction_hash = Some(pair.0);
			log
		})
		.collect()
}

fn check_known<C>(client: &C, number: BlockNumber) -> Result<()> where C: BlockChainClient {
	use types::block_status::BlockStatus;

	let id = match number {
		BlockNumber::Pending => return Ok(()),
		BlockNumber::Num(n) => BlockId::Number(n),
		BlockNumber::Latest => BlockId::Latest,
		BlockNumber::Earliest => BlockId::Earliest,
		BlockNumber::Hash { hash, require_canonical } => {
			// block check takes precedence over canon check.
			match client.block_status(BlockId::Hash(hash.clone())) {
				BlockStatus::InChain => {},
				_ => return Err(errors::unknown_block()),
			};

			if require_canonical && !client.chain().is_canon(&hash) {
				return Err(errors::invalid_input())
			}

			return Ok(())
		}
	};

	match client.block_status(id) {
		BlockStatus::InChain => Ok(()),
		_ => Err(errors::unknown_block()),
	}
}

const MAX_QUEUE_SIZE_TO_MINE_ON: usize = 4;	// because uncles go back 6.

impl<C, SN: ?Sized, S: ?Sized, M, EM, T: StateInfo + 'static> Eth for EthClient<C, SN, S, M, EM> where
	C: miner::BlockChainClient + StateClient<State=T> + ProvingBlockChainClient + Call<State=T> + EngineInfo + 'static,
	SN: SnapshotService + 'static,
	S: SyncProvider + 'static,
	M: MinerService<State=T> + 'static,
	EM: ExternalMinerService + 'static,
{
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String> {
		let version = self.sync.status().protocol_version.to_owned();
		Ok(format!("{}", version))
	}

	fn syncing(&self) -> Result<SyncStatus> {
		let status = self.sync.status();
		let client = &self.client;
		let snapshot_status = self.snapshot.status();

		let (warping, warp_chunks_amount, warp_chunks_processed) = match snapshot_status {
			RestorationStatus::Ongoing { state_chunks, block_chunks, state_chunks_done, block_chunks_done } =>
				(true, Some(block_chunks + state_chunks), Some(block_chunks_done + state_chunks_done)),
			_ => (false, None, None),
		};

		if warping || self.sync.is_major_syncing() {
			let chain_info = client.chain_info();
			let current_block = U256::from(chain_info.best_block_number);
			let highest_block = U256::from(status.highest_block_number.unwrap_or(status.start_block_number));

			let info = SyncInfo {
				starting_block: status.start_block_number.into(),
				current_block,
				highest_block,
				warp_chunks_amount: warp_chunks_amount.map(|x| U256::from(x as u64)).map(Into::into),
				warp_chunks_processed: warp_chunks_processed.map(|x| U256::from(x as u64)).map(Into::into),
			};
			Ok(SyncStatus::Info(info))
		} else {
			Ok(SyncStatus::None)
		}
	}

	fn author(&self) -> Result<H160> {
		let miner = self.miner.authoring_params().author;
		if miner.is_zero() {
			(self.accounts)()
				.first()
				.cloned()
				.ok_or_else(|| errors::account("No accounts were found", ""))
		} else {
			Ok(miner)
		}
	}

	fn is_mining(&self) -> Result<bool> {
		Ok(self.miner.is_currently_sealing())
	}

	fn chain_id(&self) -> Result<Option<U64>> {
		Ok(self.client.signing_chain_id().map(U64::from))
	}

	fn hashrate(&self) -> Result<U256> {
		Ok(self.external_miner.hashrate())
	}

	fn gas_price(&self) -> BoxFuture<U256> {
		Box::new(future::ok(default_gas_price(&*self.client, &*self.miner, self.options.gas_price_percentile)))
	}

	fn accounts(&self) -> Result<Vec<H160>> {
		self.deprecation_notice.print("eth_accounts", deprecated::msgs::ACCOUNTS);

		let accounts = (self.accounts)();
		Ok(accounts)
	}

	fn block_number(&self) -> Result<U256> {
		Ok(U256::from(self.client.chain_info().best_block_number))
	}

	fn balance(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<U256> {
		let num = num.unwrap_or_default();

		try_bf!(check_known(&*self.client, num.clone()));
		let res = match self.client.balance(&address, self.get_state(num)) {
			Some(balance) => Ok(balance),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn proof(&self, address: H160, values: Vec<H256>, num: Option<BlockNumber>) -> BoxFuture<EthAccount> {
		try_bf!(errors::require_experimental(self.options.allow_experimental_rpcs, "1186"));

		let key1 = keccak(address);

		let num = num.unwrap_or_default();
		let id = match num {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(n) => BlockId::Number(n),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,
			BlockNumber::Pending => {
				self.deprecation_notice.print("`Pending`", Some("falling back to `Latest`"));
				BlockId::Latest
			}
		};

		try_bf!(check_known(&*self.client, num.clone()));
		let res = match self.client.prove_account(key1, id) {
			Some((proof, account)) => Ok(EthAccount {
				address,
				balance: account.balance,
				nonce: account.nonce,
				code_hash: account.code_hash,
				storage_hash: account.storage_root,
				account_proof: proof.into_iter().map(Bytes::new).collect(),
				storage_proof: values.into_iter().filter_map(|storage_index| {
					let key2: H256 = storage_index;
					self.client.prove_storage(key1, keccak(key2), id)
					    .map(|(storage_proof, storage_value)| StorageProof {
							key: key2.into_uint(),
							value: storage_value.into_uint(),
							proof: storage_proof.into_iter().map(Bytes::new).collect()
						})
					})
					.collect::<Vec<StorageProof>>()
			}),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn storage_at(&self, address: H160, position: U256, num: Option<BlockNumber>) -> BoxFuture<H256> {
		let num = num.unwrap_or_default();

		try_bf!(check_known(&*self.client, num.clone()));
		let res = match self.client.storage_at(&address, &BigEndianHash::from_uint(&position), self.get_state(num)) {
			Some(s) => Ok(s),
			None => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn transaction_count(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<U256> {
		let res = match num.unwrap_or_default() {
			BlockNumber::Pending if self.options.pending_nonce_from_queue => {
				Ok(self.miner.next_nonce(&*self.client, &address))
			}
			BlockNumber::Pending => {
				let info = self.client.chain_info();
				let nonce = self.miner
					.pending_state(info.best_block_number)
					.and_then(|s| s.nonce(&address).ok())
					.or_else(|| {
						warn!("Fallback to `BlockId::Latest`");
						self.client.nonce(&address, BlockId::Latest)
					});

				match nonce {
					Some(nonce) => Ok(nonce),
					None => Err(errors::database("latest nonce missing"))
				}
			},
			number => {
				try_bf!(check_known(&*self.client, number.clone()));
				match self.client.nonce(&address, block_number_to_id(number)) {
					Some(nonce) => Ok(nonce),
					None => Err(errors::state_pruned()),
				}
			}
		};

		Box::new(future::done(res))
	}

	fn block_transaction_count_by_hash(&self, hash: H256) -> BoxFuture<Option<U256>> {
		let trx_count = self.client.block(BlockId::Hash(hash))
			.map(|block| block.transactions_count().into());
		let result = Ok(trx_count)
			.and_then(errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<U256>> {
		Box::new(future::done(match num {
			BlockNumber::Pending =>
				Ok(Some(self.miner.pending_transaction_hashes(&*self.client).len().into())),
			_ => {
				let trx_count = self.client.block(block_number_to_id(num.clone()))
					.map(|block| block.transactions_count().into());
				Ok(trx_count)
					.and_then(errors::check_block_number_existence(
						&*self.client,
						num,
						self.options
					))
			}
		}))
	}

	fn block_uncles_count_by_hash(&self, hash: H256) -> BoxFuture<Option<U256>> {
		let uncle_count = self.client.block(BlockId::Hash(hash))
			.map(|block| block.uncles_count().into());
		let result = Ok(uncle_count)
			.and_then(errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<U256>> {
		Box::new(future::done(match num {
			BlockNumber::Pending => Ok(Some(0.into())),
			_ => {
				let uncles_count = self.client.block(block_number_to_id(num.clone()))
					.map(|block| block.uncles_count().into());
				Ok(uncles_count)
					.and_then(errors::check_block_number_existence(
						&*self.client,
						num,
						self.options
					))
			}
		}))
	}

	fn code_at(&self, address: H160, num: Option<BlockNumber>) -> BoxFuture<Bytes> {
		let address: Address = H160::into(address);

		let num = num.unwrap_or_default();
		try_bf!(check_known(&*self.client, num.clone()));

		let res = match self.client.code(&address, self.get_state(num)) {
			StateResult::Some(code) => Ok(code.map_or_else(Bytes::default, Bytes::new)),
			StateResult::Missing => Err(errors::state_pruned()),
		};

		Box::new(future::done(res))
	}

	fn block_by_hash(&self, hash: H256, include_txs: bool) -> BoxFuture<Option<RichBlock>> {
		let result = self.rich_block(BlockId::Hash(hash).into(), include_txs)
			.and_then(errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>> {
		let result = self.rich_block(num.clone().into(), include_txs).and_then(
			errors::check_block_number_existence(&*self.client, num, self.options));
		Box::new(future::done(result))
	}

	fn transaction_by_hash(&self, hash: H256) -> BoxFuture<Option<Transaction>> {
		let tx = try_bf!(self.transaction(PendingTransactionId::Hash(hash))).or_else(|| {
			self.miner.transaction(&hash)
				.map(|t| Transaction::from_pending(t.pending().clone()))
		});
		let result = Ok(tx).and_then(
			errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn transaction_by_block_hash_and_index(&self, hash: H256, index: Index) -> BoxFuture<Option<Transaction>> {
		let id = PendingTransactionId::Location(PendingOrBlock::Block(BlockId::Hash(hash)), index.value());
		let result = self.transaction(id).and_then(
			errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> BoxFuture<Option<Transaction>> {
		let block_id = match num {
			BlockNumber::Hash { hash, .. } => PendingOrBlock::Block(BlockId::Hash(hash)),
			BlockNumber::Latest => PendingOrBlock::Block(BlockId::Latest),
			BlockNumber::Earliest => PendingOrBlock::Block(BlockId::Earliest),
			BlockNumber::Num(num) => PendingOrBlock::Block(BlockId::Number(num)),
			BlockNumber::Pending => PendingOrBlock::Pending,
		};

		let transaction_id = PendingTransactionId::Location(block_id, index.value());
		let result = self.transaction(transaction_id).and_then(
			errors::check_block_number_existence(&*self.client, num, self.options));
		Box::new(future::done(result))
	}

	fn transaction_receipt(&self, hash: H256) -> BoxFuture<Option<Receipt>> {
		if self.options.allow_pending_receipt_query {
			let best_block = self.client.chain_info().best_block_number;
			if let Some(receipt) = self.miner.pending_receipt(best_block, &hash) {
				return Box::new(future::ok(Some(receipt.into())));
			}
		}

		let receipt = self.client.transaction_receipt(TransactionId::Hash(hash));
		let result = Ok(receipt.map(Into::into))
			.and_then(errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn uncle_by_block_hash_and_index(&self, hash: H256, index: Index) -> BoxFuture<Option<RichBlock>> {
		let result = self.uncle(PendingUncleId {
			id: PendingOrBlock::Block(BlockId::Hash(hash)),
			position: index.value()
		}).and_then(errors::check_block_gap(&*self.client, self.options));
		Box::new(future::done(result))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> BoxFuture<Option<RichBlock>> {
		let id = match num {
			BlockNumber::Hash { hash, .. } => PendingUncleId { id: PendingOrBlock::Block(BlockId::Hash(hash)), position: index.value() },
			BlockNumber::Latest => PendingUncleId { id: PendingOrBlock::Block(BlockId::Latest), position: index.value() },
			BlockNumber::Earliest => PendingUncleId { id: PendingOrBlock::Block(BlockId::Earliest), position: index.value() },
			BlockNumber::Num(num) => PendingUncleId { id: PendingOrBlock::Block(BlockId::Number(num)), position: index.value() },

			BlockNumber::Pending => PendingUncleId { id: PendingOrBlock::Pending, position: index.value() },
		};

		let result = self.uncle(id)
			.and_then(errors::check_block_number_existence(
				&*self.client,
				num,
				self.options
			));

		Box::new(future::done(result))
	}

	fn compilers(&self) -> Result<Vec<String>> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))
	}

	fn logs(&self, filter: Filter) -> BoxFuture<Vec<Log>> {
		base_logs(&*self.client, &*self.miner, filter)
	}

	fn work(&self, no_new_work_timeout: Option<u64>) -> Result<Work> {
		let no_new_work_timeout = no_new_work_timeout.unwrap_or_default();

		// check if we're still syncing and return empty strings in that case
		{
			let sync_status = self.sync.status();
			let queue_info = self.client.queue_info();
			let total_queue_size = queue_info.total_queue_size();

			if sync_status.is_snapshot_syncing() || total_queue_size > MAX_QUEUE_SIZE_TO_MINE_ON {
				trace!(target: "miner", "Syncing. Cannot give any work.");
				return Err(errors::no_work());
			}

			// Otherwise spin until our submitted block has been included.
			let timeout = Instant::now() + Duration::from_millis(1000);
			while Instant::now() < timeout && self.client.queue_info().total_queue_size() > 0 {
				thread::sleep(Duration::from_millis(1));
			}
		}

		if self.miner.authoring_params().author.is_zero() {
			warn!(target: "miner", "Cannot give work package - no author is configured. Use --author to configure!");
			return Err(errors::no_author())
		}

		let work = self.miner.work_package(&*self.client).ok_or_else(|| {
			warn!(target: "miner", "Cannot give work package - engine seals internally.");
			errors::no_work_required()
		})?;

		let (pow_hash, number, timestamp, difficulty) = work;
		let target = ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = self.seed_compute.lock().hash_block_number(number);

		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
		if no_new_work_timeout > 0 && timestamp + no_new_work_timeout < now {
			Err(errors::no_new_work())
		} else if self.options.send_block_number_in_get_work {
			Ok(Work {
				pow_hash,
				seed_hash: seed_hash.into(),
				target,
				number: Some(number),
			})
		} else {
			Ok(Work {
				pow_hash,
				seed_hash: seed_hash.into(),
				target,
				number: None
			})
		}
	}

	fn submit_work(&self, nonce: H64, pow_hash: H256, mix_hash: H256) -> Result<bool> {
		match helpers::submit_work_detail(&self.client, &self.miner, nonce, pow_hash, mix_hash) {
			Ok(_)  => Ok(true),
			Err(_) => Ok(false),
		}
	}

	fn submit_hashrate(&self, rate: U256, id: H256) -> Result<bool> {
		self.external_miner.submit_hashrate(rate, id);
		Ok(true)
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<H256> {
		Rlp::new(&raw.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))
			.and_then(|signed_transaction| {
				FullDispatcher::dispatch_transaction(
					&*self.client,
					&*self.miner,
					signed_transaction.into(),
					false
				)
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<H256> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, request: CallRequest, num: Option<BlockNumber>) -> BoxFuture<Bytes> {
		let request = CallRequest::into(request);
		let signed = try_bf!(fake_sign::sign_call(request));

		let num = num.unwrap_or_default();
		try_bf!(check_known(&*self.client, num.clone()));

		let (mut state, header) = if num == BlockNumber::Pending {
			let info = self.client.chain_info();
			let state = try_bf!(self.miner.pending_state(info.best_block_number).ok_or_else(errors::state_pruned));
			let header = try_bf!(self.miner.pending_block_header(info.best_block_number).ok_or_else(errors::state_pruned));

			(state, header)
		} else {
			let id = match num {
				BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
				BlockNumber::Num(num) => BlockId::Number(num),
				BlockNumber::Earliest => BlockId::Earliest,
				BlockNumber::Latest => BlockId::Latest,
				BlockNumber::Pending => unreachable!(), // Already covered
			};

			let state = try_bf!(self.client.state_at(id).ok_or_else(errors::state_pruned));
			let header = try_bf!(self.client.block_header(id).ok_or_else(errors::state_pruned).and_then(|h| h.decode().map_err(errors::decode)));

			(state, header)
		};

		let result = self.client.call(&signed, Default::default(), &mut state, &header);

		Box::new(future::done(result
			.map_err(errors::call)
			.and_then(|executed| {
				match executed.exception {
					Some(ref exception) => Err(errors::vm(exception, &executed.output)),
					None => Ok(executed)
				}
			})
			.map(|b| b.output.into())
		))
	}

	fn estimate_gas(&self, request: CallRequest, num: Option<BlockNumber>) -> BoxFuture<U256> {
		let request = CallRequest::into(request);
		let signed = try_bf!(fake_sign::sign_call(request));
		let num = num.unwrap_or_default();

		let (state, header) = if num == BlockNumber::Pending {
			let info = self.client.chain_info();
			let state = try_bf!(self.miner.pending_state(info.best_block_number)
								.ok_or_else(errors::state_pruned));
			let header = try_bf!(self.miner.pending_block_header(info.best_block_number)
								 .ok_or_else(errors::state_pruned));

			(state, header)
		} else {
			let id = match num {
				BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
				BlockNumber::Num(num) => BlockId::Number(num),
				BlockNumber::Earliest => BlockId::Earliest,
				BlockNumber::Latest => BlockId::Latest,
				BlockNumber::Pending => unreachable!(), // Already covered
			};

			let state = try_bf!(self.client.state_at(id)
								.ok_or_else(errors::state_pruned));
			let header = try_bf!(self.client.block_header(id)
								 .ok_or_else(errors::state_pruned)
								 .and_then(|h| h.decode().map_err(errors::decode)));
			(state, header)
		};

		Box::new(future::done(self.client.estimate_gas(&signed, &state, &header)
			.map_err(errors::call)
		))
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
}
