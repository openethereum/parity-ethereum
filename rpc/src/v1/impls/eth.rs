// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use std::collections::HashSet;
use std::sync::{Arc, Weak, Mutex};
use std::ops::Deref;
use ethsync::{SyncProvider, SyncState};
use ethminer::{MinerService, AccountDetails};
use jsonrpc_core::*;
use util::numbers::*;
use util::sha3::*;
use util::rlp::encode;
use ethcore::client::*;
use ethcore::block::IsBlock;
use ethcore::views::*;
use ethcore::ethereum::Ethash;
use ethcore::ethereum::denominations::shannon;
use ethcore::transaction::Transaction as EthTransaction;
use v1::traits::{Eth, EthFilter};
use v1::types::{Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo, Transaction, TransactionRequest, OptionalValue, Index, Filter, Log};
use v1::helpers::{PollFilter, PollManager, ExternalMinerService, ExternalMiner};
use util::keys::store::AccountProvider;

/// Eth rpc implementation.
pub struct EthClient<C, S, A, M, EM = ExternalMiner>
	where C: BlockChainClient,
		  S: SyncProvider,
		  A: AccountProvider,
		  M: MinerService,
		  EM: ExternalMinerService {
	client: Weak<C>,
	sync: Weak<S>,
	accounts: Weak<A>,
	miner: Weak<M>,
	external_miner: EM,
}

impl<C, S, A, M> EthClient<C, S, A, M, ExternalMiner>
	where C: BlockChainClient,
		  S: SyncProvider,
		  A: AccountProvider,
		  M: MinerService {

	/// Creates new EthClient.
	pub fn new(client: &Arc<C>, sync: &Arc<S>, accounts: &Arc<A>, miner: &Arc<M>) -> Self {
		EthClient::new_with_external_miner(client, sync, accounts, miner, ExternalMiner::default())
	}
}


impl<C, S, A, M, EM> EthClient<C, S, A, M, EM>
	where C: BlockChainClient,
		  S: SyncProvider,
		  A: AccountProvider,
		  M: MinerService,
		  EM: ExternalMinerService {

	/// Creates new EthClient with custom external miner.
	pub fn new_with_external_miner(client: &Arc<C>, sync: &Arc<S>, accounts: &Arc<A>, miner: &Arc<M>, em: EM)
		-> EthClient<C, S, A, M, EM> {
		EthClient {
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
			external_miner: em,
		}
	}

	fn block(&self, id: BlockId, include_txs: bool) -> Result<Value, Error> {
		let client = take_weak!(self.client);
		match (client.block(id.clone()), client.block_total_difficulty(id)) {
			(Some(bytes), Some(total_difficulty)) => {
				let block_view = BlockView::new(&bytes);
				let view = block_view.header_view();
				let block = Block {
					hash: OptionalValue::Value(view.sha3()),
					parent_hash: view.parent_hash(),
					uncles_hash: view.uncles_hash(),
					author: view.author(),
					miner: view.author(),
					state_root: view.state_root(),
					transactions_root: view.transactions_root(),
					receipts_root: view.receipts_root(),
					number: OptionalValue::Value(U256::from(view.number())),
					gas_used: view.gas_used(),
					gas_limit: view.gas_limit(),
					logs_bloom: view.log_bloom(),
					timestamp: U256::from(view.timestamp()),
					difficulty: view.difficulty(),
					total_difficulty: total_difficulty,
					uncles: vec![],
					transactions: {
						if include_txs {
							BlockTransactions::Full(block_view.localized_transactions().into_iter().map(From::from).collect())
						} else {
							BlockTransactions::Hashes(block_view.transaction_hashes())
						}
					},
					extra_data: Bytes::default()
				};
				to_value(&block)
			},
			_ => Ok(Value::Null)
		}
	}

	fn transaction(&self, id: TransactionId) -> Result<Value, Error> {
		match take_weak!(self.client).transaction(id) {
			Some(t) => to_value(&Transaction::from(t)),
			None => Ok(Value::Null)
		}
	}

	fn uncle(&self, _block: BlockId, _index: usize) -> Result<Value, Error> {
		// TODO: implement!
		Ok(Value::Null)
	}
}

impl<C, S, A, M, EM> Eth for EthClient<C, S, A, M, EM>
	where C: BlockChainClient + 'static,
		  S: SyncProvider + 'static,
		  A: AccountProvider + 'static,
		  M: MinerService + 'static,
		  EM: ExternalMinerService + 'static {

	fn protocol_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::String(format!("{}", take_weak!(self.sync).status().protocol_version).to_owned())),
			_ => Err(Error::invalid_params())
		}
	}

	fn syncing(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let status = take_weak!(self.sync).status();
				let res = match status.state {
					SyncState::NotSynced | SyncState::Idle => SyncStatus::None,
					SyncState::Waiting | SyncState::Blocks | SyncState::NewBlocks => SyncStatus::Info(SyncInfo {
						starting_block: U256::from(status.start_block_number),
						current_block: U256::from(take_weak!(self.client).chain_info().best_block_number),
						highest_block: U256::from(status.highest_block_number.unwrap_or(status.start_block_number))
					})
				};
				to_value(&res)
			}
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: do not hardcode author.
	fn author(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&Address::new()),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: return real value of mining once it's implemented.
	fn is_mining(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&self.external_miner.is_mining()),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: return real hashrate once we have mining
	fn hashrate(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&self.external_miner.hashrate()),
			_ => Err(Error::invalid_params())
		}
	}

	fn gas_price(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&(shannon() * U256::from(50))),
			_ => Err(Error::invalid_params())
		}
	}

	fn accounts(&self, _: Params) -> Result<Value, Error> {
		let store = take_weak!(self.accounts);
		match store.accounts() {
			Ok(account_list) => to_value(&account_list),
			Err(_) => Err(Error::internal_error())
		}
	}

	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&U256::from(take_weak!(self.client).chain_info().best_block_number)),
			_ => Err(Error::invalid_params())
		}
	}

	fn balance(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, BlockNumber)>(params)
			.and_then(|(address, _block_number)| to_value(&take_weak!(self.client).balance(&address)))
	}

	fn storage_at(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, U256, BlockNumber)>(params)
			.and_then(|(address, position, _block_number)|
				to_value(&U256::from(take_weak!(self.client).storage_at(&address, &H256::from(position)))))
	}

	fn transaction_count(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, BlockNumber)>(params)
			.and_then(|(address, _block_number)| to_value(&take_weak!(self.client).nonce(&address)))
	}

	fn block_transaction_count_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| // match
				to_value(&take_weak!(self.client).block(BlockId::Hash(hash))
					.map_or_else(U256::zero, |bytes| U256::from(BlockView::new(&bytes).transactions_count()))))
	}

	fn block_transaction_count_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| match block_number {
				BlockNumber::Pending => to_value(
					&U256::from(take_weak!(self.miner).status().transactions_in_pending_block)
				),
				_ => to_value(&take_weak!(self.client).block(block_number.into())
						.map_or_else(U256::zero, |bytes| U256::from(BlockView::new(&bytes).transactions_count())))
			})
	}

	fn block_uncles_count_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)|
				to_value(&take_weak!(self.client).block(BlockId::Hash(hash))
					.map_or_else(U256::zero, |bytes| U256::from(BlockView::new(&bytes).uncles_count()))))
	}

	fn block_uncles_count_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| match block_number {
				BlockNumber::Pending => to_value(&U256::from(0)),
				_ => to_value(&take_weak!(self.client).block(block_number.into())
						.map_or_else(U256::zero, |bytes| U256::from(BlockView::new(&bytes).uncles_count())))
			})
	}

	// TODO: do not ignore block number param
	fn code_at(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, BlockNumber)>(params)
			.and_then(|(address, _block_number)|
				to_value(&take_weak!(self.client).code(&address).map_or_else(Bytes::default, Bytes::new)))
	}

	fn block_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, bool)>(params)
			.and_then(|(hash, include_txs)| self.block(BlockId::Hash(hash), include_txs))
	}

	fn block_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, bool)>(params)
			.and_then(|(number, include_txs)| self.block(number.into(), include_txs))
	}

	fn transaction_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| self.transaction(TransactionId::Hash(hash)))
	}

	fn transaction_by_block_hash_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, Index)>(params)
			.and_then(|(hash, index)| self.transaction(TransactionId::Location(BlockId::Hash(hash), index.value())))
	}

	fn transaction_by_block_number_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, Index)>(params)
			.and_then(|(number, index)| self.transaction(TransactionId::Location(number.into(), index.value())))
	}

	fn uncle_by_block_hash_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, Index)>(params)
			.and_then(|(hash, index)| self.uncle(BlockId::Hash(hash), index.value()))
	}

	fn uncle_by_block_number_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, Index)>(params)
			.and_then(|(number, index)| self.uncle(number.into(), index.value()))
	}

	fn compilers(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&vec![] as &Vec<String>),
			_ => Err(Error::invalid_params())
		}
	}

	fn logs(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let logs = take_weak!(self.client).logs(filter.into())
					.into_iter()
					.map(From::from)
					.collect::<Vec<Log>>();
				to_value(&logs)
			})
	}

	fn work(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let client = take_weak!(self.client);
				// check if we're still syncing and return empty strings int that case
				{
					let sync = take_weak!(self.sync);
					if sync.status().state != SyncState::Idle && client.queue_info().is_empty() {
						return to_value(&(String::new(), String::new(), String::new()));
					}
				}

				let miner = take_weak!(self.miner);
				let client = take_weak!(self.client);
				let u = miner.sealing_block(client.deref()).lock().unwrap();
				match *u {
					Some(ref b) => {
						let pow_hash = b.hash();
						let target = Ethash::difficulty_to_boundary(b.block().header().difficulty());
						let seed_hash = Ethash::get_seedhash(b.block().header().number());
						to_value(&(pow_hash, seed_hash, target))
					}
					_ => Err(Error::internal_error())
				}
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn submit_work(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H64, H256, H256)>(params).and_then(|(nonce, pow_hash, mix_hash)| {
//			trace!("Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);
			let miner = take_weak!(self.miner);
			let client = take_weak!(self.client);
			let seal = vec![encode(&mix_hash).to_vec(), encode(&nonce).to_vec()];
			let r = miner.submit_seal(client.deref(), pow_hash, seal);
			to_value(&r.is_ok())
		})
	}

	fn submit_hashrate(&self, params: Params) -> Result<Value, Error> {
		// TODO: Index should be U256.
		from_params::<(U256, H256)>(params).and_then(|(rate, id)| {
			self.external_miner.submit_hashrate(rate, id);
			to_value(&true)
		})
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(transaction_request, )| {
				let accounts = take_weak!(self.accounts);
				match accounts.account_secret(&transaction_request.from) {
					Ok(secret) => {
						let miner = take_weak!(self.miner);
						let client = take_weak!(self.client);

						let transaction: EthTransaction = transaction_request.into();
						let signed_transaction = transaction.sign(&secret);
						let hash = signed_transaction.hash();

						let import = miner.import_transactions(vec![signed_transaction], |a: &Address| AccountDetails {
							nonce: client.nonce(a),
							balance: client.balance(a),
						});
						match import.into_iter().collect::<Result<Vec<_>, _>>() {
							Ok(_) => to_value(&hash),
							Err(e) => {
								warn!("Error sending transaction: {:?}", e);
								to_value(&U256::zero())
							}
						}
					},
					Err(_) => { to_value(&U256::zero()) }
				}
		})
	}
}

/// Eth filter rpc implementation.
pub struct EthFilterClient<C, M>
	where C: BlockChainClient,
		  M: MinerService {

	client: Weak<C>,
	miner: Weak<M>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl<C, M> EthFilterClient<C, M>
	where C: BlockChainClient,
		  M: MinerService {

	/// Creates new Eth filter client.
	pub fn new(client: &Arc<C>, miner: &Arc<M>) -> Self {
		EthFilterClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			polls: Mutex::new(PollManager::new()),
		}
	}
}

impl<C, M> EthFilter for EthFilterClient<C, M>
	where C: BlockChainClient + 'static,
		  M: MinerService + 'static {

	fn new_filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let mut polls = self.polls.lock().unwrap();
				let block_number = take_weak!(self.client).chain_info().best_block_number;
				let id = polls.create_poll(PollFilter::Logs(block_number, filter.into()));
				to_value(&U256::from(id))
			})
	}

	fn new_block_filter(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let mut polls = self.polls.lock().unwrap();
				let id = polls.create_poll(PollFilter::Block(take_weak!(self.client).chain_info().best_block_number));
				to_value(&U256::from(id))
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn new_pending_transaction_filter(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let mut polls = self.polls.lock().unwrap();
				let pending_transactions = take_weak!(self.miner).pending_transactions_hashes();
				let id = polls.create_poll(PollFilter::PendingTransaction(pending_transactions));

				to_value(&U256::from(id))
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn filter_changes(&self, params: Params) -> Result<Value, Error> {
		let client = take_weak!(self.client);
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				let mut polls = self.polls.lock().unwrap();
				match polls.poll_mut(&index.value()) {
					None => Ok(Value::Array(vec![] as Vec<Value>)),
					Some(filter) => match *filter {
						PollFilter::Block(ref mut block_number) => {
							// + 1, cause we want to return hashes including current block hash.
							let current_number = client.chain_info().best_block_number + 1;
							let hashes = (*block_number..current_number).into_iter()
								.map(BlockId::Number)
								.filter_map(|id| client.block_hash(id))
								.collect::<Vec<H256>>();

							*block_number = current_number;

							to_value(&hashes)
						},
						PollFilter::PendingTransaction(ref mut previous_hashes) => {
							let current_hashes = take_weak!(self.miner).pending_transactions_hashes();
							// calculate diff
							let previous_hashes_set = previous_hashes.into_iter().map(|h| h.clone()).collect::<HashSet<H256>>();
							let diff = current_hashes
								.iter()
								.filter(|hash| previous_hashes_set.contains(&hash))
								.cloned()
								.collect::<Vec<H256>>();

							*previous_hashes = current_hashes;

							to_value(&diff)
						},
						PollFilter::Logs(ref mut block_number, ref mut filter) => {
							filter.from_block = BlockId::Number(*block_number);
							filter.to_block = BlockId::Latest;
							let logs = client.logs(filter.clone())
								.into_iter()
								.map(From::from)
								.collect::<Vec<Log>>();

							let current_number = client.chain_info().best_block_number;

							*block_number = current_number;
							to_value(&logs)
						}
					}
				}
			})
	}

	fn uninstall_filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				self.polls.lock().unwrap().remove_poll(&index.value());
				to_value(&true)
			})
	}
}
