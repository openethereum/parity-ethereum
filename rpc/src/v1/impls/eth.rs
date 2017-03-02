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
use std::sync::{Arc, Weak};

use futures::{self, future, BoxFuture, Future};
use rlp::{self, UntrustedRlp, View};
use time::get_time;
use util::{H160, H256, Address, FixedHash, U256, H64, Uint};
use util::sha3::Hashable;
use util::Mutex;

use ethash::SeedHashCompute;
use ethcore::account_provider::{AccountProvider, DappId};
use ethcore::block::IsBlock;
use ethcore::client::{MiningBlockChainClient, BlockId, TransactionId, UncleId};
use ethcore::ethereum::Ethash;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::header::{Header as BlockHeader, BlockNumber as EthBlockNumber};
use ethcore::log_entry::LogEntry;
use ethcore::miner::{MinerService, ExternalMinerService};
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};
use ethcore::snapshot::SnapshotService;
use ethsync::{SyncProvider};

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use v1::helpers::{CallRequest as CRequest, errors, limit_logs};
use v1::helpers::dispatch::{Dispatcher, FullDispatcher, default_gas_price};
use v1::helpers::block_import::is_major_importing;
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
	/// Returns receipt from pending blocks
	pub allow_pending_receipt_query: bool,
	/// Send additional block number when asking for work
	pub send_block_number_in_get_work: bool,
}

impl Default for EthClientOptions {
	fn default() -> Self {
		EthClientOptions {
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

	client: Weak<C>,
	snapshot: Weak<SN>,
	sync: Weak<S>,
	accounts: Weak<AccountProvider>,
	miner: Weak<M>,
	external_miner: Arc<EM>,
	seed_compute: Mutex<SeedHashCompute>,
	options: EthClientOptions,
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
		accounts: &Arc<AccountProvider>,
		miner: &Arc<M>,
		em: &Arc<EM>,
		options: EthClientOptions
	) -> Self {
		EthClient {
			client: Arc::downgrade(client),
			snapshot: Arc::downgrade(snapshot),
			sync: Arc::downgrade(sync),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
			external_miner: em.clone(),
			seed_compute: Mutex::new(SeedHashCompute::new()),
			options: options,
		}
	}

	fn block(&self, id: BlockId, include_txs: bool) -> Result<Option<RichBlock>, Error> {
		let client = take_weak!(self.client);
		match (client.block(id.clone()), client.block_total_difficulty(id)) {
			(Some(block), Some(total_difficulty)) => {
				let view = block.header_view();
				Ok(Some(RichBlock {
					block: Block {
						hash: Some(view.sha3().into()),
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
							true => BlockTransactions::Full(block.view().localized_transactions().into_iter().map(Into::into).collect()),
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
		match take_weak!(self.client).transaction(id) {
			Some(t) => Ok(Some(Transaction::from(t))),
			None => Ok(None),
		}
	}

	fn uncle(&self, id: UncleId) -> Result<Option<RichBlock>, Error> {
		let client = take_weak!(self.client);
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
			block: Block {
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

	fn sign_call(&self, request: CRequest) -> Result<SignedTransaction, Error> {
		let (client, miner) = (take_weak!(self.client), take_weak!(self.miner));
		let from = request.from.unwrap_or(Address::zero());
		Ok(EthTransaction {
			nonce: request.nonce.unwrap_or_else(|| client.latest_nonce(&from)),
			action: request.to.map_or(Action::Create, Action::Call),
			gas: request.gas.unwrap_or(U256::from(50_000_000)),
			gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(&*client, &*miner)),
			value: request.value.unwrap_or_else(U256::zero),
			data: request.data.map_or_else(Vec::new, |d| d.to_vec())
		}.fake_sign(from))
	}

	fn dapp_accounts(&self, dapp: DappId) -> Result<Vec<H160>, Error> {
		let store = take_weak!(self.accounts);
		store
			.note_dapp_used(dapp.clone())
			.and_then(|_| store.dapp_addresses(dapp))
			.map_err(|e| errors::internal("Could not fetch accounts.", e))
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
		let version = take_weak!(self.sync).status().protocol_version.to_owned();
		Ok(format!("{}", version))
	}

	fn syncing(&self) -> Result<SyncStatus, Error> {
		use ethcore::snapshot::RestorationStatus;

		let status = take_weak!(self.sync).status();
		let client = take_weak!(self.client);
		let snapshot_status = take_weak!(self.snapshot).status();

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

	fn author(&self, meta: Metadata) -> BoxFuture<RpcH160, Error> {
		let dapp = meta.dapp_id();

		let author = move || {
			let mut miner = take_weak!(self.miner).author();
			if miner == 0.into() {
				miner = self.dapp_accounts(dapp.into())?.get(0).cloned().unwrap_or_default();
			}

			Ok(RpcH160::from(miner))
		};

		futures::done(author()).boxed()
	}

	fn is_mining(&self) -> Result<bool, Error> {
		Ok(take_weak!(self.miner).is_sealing())
	}

	fn hashrate(&self) -> Result<RpcU256, Error> {
		Ok(RpcU256::from(self.external_miner.hashrate()))
	}

	fn gas_price(&self) -> Result<RpcU256, Error> {
		let (client, miner) = (take_weak!(self.client), take_weak!(self.miner));
		Ok(RpcU256::from(default_gas_price(&*client, &*miner)))
	}

	fn accounts(&self, meta: Metadata) -> BoxFuture<Vec<RpcH160>, Error> {
		let dapp = meta.dapp_id();

		let accounts = move || {
			let accounts = self.dapp_accounts(dapp.into())?;
			Ok(accounts.into_iter().map(Into::into).collect())
		};

		futures::done(accounts()).boxed()
	}

	fn block_number(&self) -> Result<RpcU256, Error> {
		Ok(RpcU256::from(take_weak!(self.client).chain_info().best_block_number))
	}

	fn balance(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		let address = address.into();

		let res = match num.0.clone() {
			BlockNumber::Pending => {
				let client = take_weakf!(self.client);
				match take_weakf!(self.miner).balance(&*client, &address) {
					Some(balance) => Ok(balance.into()),
					None => Err(errors::internal("Unable to load balance from database", ""))
				}
			}
			id => {
				let client = take_weakf!(self.client);

				try_bf!(check_known(&*client, id.clone()));
				match client.balance(&address, id.into()) {
					Some(balance) => Ok(balance.into()),
					None => Err(errors::state_pruned()),
				}
			}
		};

		future::done(res).boxed()
	}

	fn storage_at(&self, address: RpcH160, pos: RpcU256, num: Trailing<BlockNumber>) -> BoxFuture<RpcH256, Error> {
		let address: Address = RpcH160::into(address);
		let position: U256 = RpcU256::into(pos);

		let res = match num.0.clone() {
			BlockNumber::Pending => {
				let client = take_weakf!(self.client);
				match take_weakf!(self.miner).storage_at(&*client, &address, &H256::from(position)) {
					Some(s) => Ok(s.into()),
					None => Err(errors::internal("Unable to load storage from database", ""))
				}
			}
			id => {
				let client = take_weakf!(self.client);

				try_bf!(check_known(&*client, id.clone()));
				match client.storage_at(&address, &H256::from(position), id.into()) {
					Some(s) => Ok(s.into()),
					None => Err(errors::state_pruned()),
				}
			}
		};

		future::done(res).boxed()
	}

	fn transaction_count(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		let address: Address = RpcH160::into(address);
		let res = match num.0.clone() {
			BlockNumber::Pending => {
				let client = take_weakf!(self.client);
				match take_weakf!(self.miner).nonce(&*client, &address) {
					Some(nonce) => Ok(nonce.into()),
					None => Err(errors::internal("Unable to load nonce from database", ""))
				}
			}
			id => {
				let client = take_weakf!(self.client);

				try_bf!(check_known(&*client, id.clone()));
				match client.nonce(&address, id.into()) {
					Some(nonce) => Ok(nonce.into()),
					None => Err(errors::state_pruned()),
				}
			}
		};

		future::done(res).boxed()
	}

	fn block_transaction_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		future::ok(take_weakf!(self.client).block(BlockId::Hash(hash.into()))
			.map(|block| block.transactions_count().into())).boxed()
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		future::ok(match num {
			BlockNumber::Pending => Some(
				take_weakf!(self.miner).status().transactions_in_pending_block.into()
			),
			_ =>
				take_weakf!(self.client).block(num.into())
					.map(|block| block.transactions_count().into())
		}).boxed()
	}

	fn block_uncles_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
			future::ok(take_weakf!(self.client).block(BlockId::Hash(hash.into()))
				.map(|block| block.uncles_count().into()))
				.boxed()
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		future::ok(match num {
			BlockNumber::Pending => Some(0.into()),
			_ => take_weakf!(self.client).block(num.into())
					.map(|block| block.uncles_count().into()
			),
		}).boxed()
	}

	fn code_at(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		let address: Address = RpcH160::into(address);

		let res = match num.0.clone() {
			BlockNumber::Pending => {
				let client = take_weakf!(self.client);
				match take_weakf!(self.miner).code(&*client, &address) {
					Some(code) => Ok(code.map_or_else(Bytes::default, Bytes::new)),
					None => Err(errors::internal("Unable to load code from database", ""))
				}
			}
			id => {
				let client = take_weakf!(self.client);

				try_bf!(check_known(&*client, id.clone()));
				match client.code(&address, id.into()) {
					Some(code) => Ok(code.map_or_else(Bytes::default, Bytes::new)),
					None => Err(errors::state_pruned()),
				}
			}
		};

		future::done(res).boxed()
	}

	fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::done(self.block(BlockId::Hash(hash.into()), include_txs)).boxed()
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::done(self.block(num.into(), include_txs)).boxed()
	}

	fn transaction_by_hash(&self, hash: RpcH256) -> Result<Option<Transaction>, Error> {
		let hash: H256 = hash.into();
		let miner = take_weak!(self.miner);
		let client = take_weak!(self.client);
		Ok(self.transaction(TransactionId::Hash(hash))?.or_else(|| miner.transaction(client.chain_info().best_block_number, &hash).map(Into::into)))
	}

	fn transaction_by_block_hash_and_index(&self, hash: RpcH256, index: Index) -> Result<Option<Transaction>, Error> {
		self.transaction(TransactionId::Location(BlockId::Hash(hash.into()), index.value()))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> Result<Option<Transaction>, Error> {
		self.transaction(TransactionId::Location(num.into(), index.value()))
	}

	fn transaction_receipt(&self, hash: RpcH256) -> Result<Option<Receipt>, Error> {
		let miner = take_weak!(self.miner);
		let best_block = take_weak!(self.client).chain_info().best_block_number;
		let hash: H256 = hash.into();
		match (miner.pending_receipt(best_block, &hash), self.options.allow_pending_receipt_query) {
			(Some(receipt), true) => Ok(Some(receipt.into())),
			_ => {
				let client = take_weak!(self.client);
				let receipt = client.transaction_receipt(TransactionId::Hash(hash));
				Ok(receipt.map(Into::into))
			}
		}
	}

	fn uncle_by_block_hash_and_index(&self, hash: RpcH256, index: Index) -> Result<Option<RichBlock>, Error> {
		self.uncle(UncleId { block: BlockId::Hash(hash.into()), position: index.value() })
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, index: Index) -> Result<Option<RichBlock>, Error> {
		self.uncle(UncleId { block: num.into(), position: index.value() })
	}

	fn compilers(&self) -> Result<Vec<String>, Error> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))
	}

	fn logs(&self, filter: Filter) -> Result<Vec<Log>, Error> {
		let include_pending = filter.to_block == Some(BlockNumber::Pending);
		let filter: EthcoreFilter = filter.into();
		let mut logs = take_weak!(self.client).logs(filter.clone())
			.into_iter()
			.map(From::from)
			.collect::<Vec<Log>>();

		if include_pending {
			let best_block = take_weak!(self.client).chain_info().best_block_number;
			let pending = pending_logs(&*take_weak!(self.miner), best_block, &filter);
			logs.extend(pending);
		}

		let logs = limit_logs(logs, filter.limit);

		Ok(logs)
	}

	fn work(&self, no_new_work_timeout: Trailing<u64>) -> Result<Work, Error> {
		let no_new_work_timeout = no_new_work_timeout.0;

		let client = take_weak!(self.client);
		// check if we're still syncing and return empty strings in that case
		{
			//TODO: check if initial sync is complete here
			//let sync = take_weak!(self.sync);
			if /*sync.status().state != SyncState::Idle ||*/ client.queue_info().total_queue_size() > MAX_QUEUE_SIZE_TO_MINE_ON {
				trace!(target: "miner", "Syncing. Cannot give any work.");
				return Err(errors::no_work());
			}

			// Otherwise spin until our submitted block has been included.
			let timeout = Instant::now() + Duration::from_millis(1000);
			while Instant::now() < timeout && client.queue_info().total_queue_size() > 0 {
				thread::sleep(Duration::from_millis(1));
			}
		}

		let miner = take_weak!(self.miner);
		if miner.author().is_zero() {
			warn!(target: "miner", "Cannot give work package - no author is configured. Use --author to configure!");
			return Err(errors::no_author())
		}
		miner.map_sealing_work(&*client, |b| {
			let pow_hash = b.hash();
			let target = Ethash::difficulty_to_boundary(b.block().header().difficulty());
			let seed_hash = self.seed_compute.lock().get_seedhash(b.block().header().number());

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
		}).unwrap_or(Err(Error::internal_error()))	// no work found.
	}

	fn submit_work(&self, nonce: RpcH64, pow_hash: RpcH256, mix_hash: RpcH256) -> Result<bool, Error> {
		let nonce: H64 = nonce.into();
		let pow_hash: H256 = pow_hash.into();
		let mix_hash: H256 = mix_hash.into();
		trace!(target: "miner", "submit_work: Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);

		let miner = take_weak!(self.miner);
		let client = take_weak!(self.client);
		let seal = vec![rlp::encode(&mix_hash).to_vec(), rlp::encode(&nonce).to_vec()];
		Ok(miner.submit_seal(&*client, pow_hash, seal).is_ok())
	}

	fn submit_hashrate(&self, rate: RpcU256, id: RpcH256) -> Result<bool, Error> {
		self.external_miner.submit_hashrate(rate.into(), id.into());
		Ok(true)
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		UntrustedRlp::new(&raw.into_vec()).as_val()
			.map_err(errors::from_rlp_error)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::from_transaction_error))
			.and_then(|signed_transaction| {
				FullDispatcher::new(self.client.clone(), self.miner.clone())
					.dispatch_transaction(signed_transaction.into())
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, request: CallRequest, num: Trailing<BlockNumber>) -> Result<Bytes, Error> {
		let request = CallRequest::into(request);
		let signed = self.sign_call(request)?;

		let result = match num.0 {
			BlockNumber::Pending => take_weak!(self.miner).call(&*take_weak!(self.client), &signed, Default::default()),
			num => take_weak!(self.client).call(&signed, num.into(), Default::default()),
		};

		result
			.map(|b| b.output.into())
			.map_err(errors::from_call_error)
	}

	fn estimate_gas(&self, request: CallRequest, num: Trailing<BlockNumber>) -> Result<RpcU256, Error> {
		let request = CallRequest::into(request);
		let signed = self.sign_call(request)?;
		take_weak!(self.client).estimate_gas(&signed, num.0.into())
			.map(Into::into)
			.map_err(errors::from_call_error)
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
