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

extern crate ethash;

use std::collections::HashSet;
use std::sync::{Arc, Weak, Mutex};
use std::ops::Deref;
use ethsync::{SyncProvider, SyncState};
use ethminer::{MinerService, AccountDetails, ExternalMinerService};
use jsonrpc_core::*;
use util::numbers::*;
use util::sha3::*;
use util::bytes::{ToPretty};
use util::rlp::{encode, decode, UntrustedRlp, View};
use ethcore::client::{BlockChainClient, BlockID, TransactionID, UncleID};
use ethcore::block::IsBlock;
use ethcore::views::*;
use ethcore::ethereum::Ethash;
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};
use ethcore::log_entry::LogEntry;
use ethcore::filter::Filter as EthcoreFilter;
use self::ethash::SeedHashCompute;
use v1::traits::{Eth, EthFilter};
use v1::types::{Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo, Transaction, TransactionRequest, CallRequest, OptionalValue, Index, Filter, Log, Receipt};
use v1::helpers::{PollFilter, PollManager};
use util::keys::store::AccountProvider;
use serde;

/// Eth rpc implementation.
pub struct EthClient<C, S, A, M, EM> where
	C: BlockChainClient,
	S: SyncProvider,
	A: AccountProvider,
	M: MinerService,
	EM: ExternalMinerService {

	client: Weak<C>,
	sync: Weak<S>,
	accounts: Weak<A>,
	miner: Weak<M>,
	external_miner: Arc<EM>,
	seed_compute: Mutex<SeedHashCompute>,
}

impl<C, S, A, M, EM> EthClient<C, S, A, M, EM> where
	C: BlockChainClient,
	S: SyncProvider,
	A: AccountProvider,
	M: MinerService,
	EM: ExternalMinerService {

	/// Creates new EthClient.
	pub fn new(client: &Arc<C>, sync: &Arc<S>, accounts: &Arc<A>, miner: &Arc<M>, em: &Arc<EM>)
		-> EthClient<C, S, A, M, EM> {
		EthClient {
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
			external_miner: em.clone(),
			seed_compute: Mutex::new(SeedHashCompute::new()),
		}
	}

	fn block(&self, id: BlockID, include_txs: bool) -> Result<Value, Error> {
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
					seal_fields: view.seal().into_iter().map(|f| decode(&f)).map(Bytes::new).collect(),
					uncles: block_view.uncle_hashes(),
					transactions: {
						if include_txs {
							BlockTransactions::Full(block_view.localized_transactions().into_iter().map(From::from).collect())
						} else {
							BlockTransactions::Hashes(block_view.transaction_hashes())
						}
					},
					extra_data: Bytes::new(view.extra_data())
				};
				to_value(&block)
			},
			_ => Ok(Value::Null)
		}
	}

	fn transaction(&self, id: TransactionID) -> Result<Value, Error> {
		match take_weak!(self.client).transaction(id) {
			Some(t) => to_value(&Transaction::from(t)),
			None => Ok(Value::Null)
		}
	}

	fn uncle(&self, id: UncleID) -> Result<Value, Error> {
		let client = take_weak!(self.client);
		match client.uncle(id).and_then(|u| client.block_total_difficulty(BlockID::Hash(u.parent_hash().clone())).map(|diff| (diff, u))) {
			Some((parent_difficulty, uncle)) => {
				let block = Block {
					hash: OptionalValue::Value(uncle.hash()),
					parent_hash: uncle.parent_hash,
					uncles_hash: uncle.uncles_hash,
					author: uncle.author,
					miner: uncle.author,
					state_root: uncle.state_root,
					transactions_root: uncle.transactions_root,
					number: OptionalValue::Value(U256::from(uncle.number)),
					gas_used: uncle.gas_used,
					gas_limit: uncle.gas_limit,
					logs_bloom: uncle.log_bloom,
					timestamp: U256::from(uncle.timestamp),
					difficulty: uncle.difficulty,
					total_difficulty: uncle.difficulty + parent_difficulty,
					receipts_root: uncle.receipts_root,
					extra_data: Bytes::new(uncle.extra_data),
					seal_fields: uncle.seal.into_iter().map(|f| decode(&f)).map(Bytes::new).collect(),
					uncles: vec![],
					transactions: BlockTransactions::Hashes(vec![]),
				};
				to_value(&block)
			},
			None => Ok(Value::Null)
		}
	}

	fn sign_and_dispatch(&self, request: TransactionRequest, secret: H256) -> Result<Value, Error> {
		let signed_transaction = {
			let client = take_weak!(self.client);
			let miner = take_weak!(self.miner);
			EthTransaction {
				nonce: request.nonce
					.or_else(|| miner
							 .last_nonce(&request.from)
							 .map(|nonce| nonce + U256::one()))
					.unwrap_or_else(|| client.nonce(&request.from)),
					action: request.to.map_or(Action::Create, Action::Call),
					gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
					gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
					value: request.value.unwrap_or_else(U256::zero),
					data: request.data.map_or_else(Vec::new, |d| d.to_vec()),
			}.sign(&secret)
		};
		trace!(target: "miner", "send_transaction: dispatching tx: {}", encode(&signed_transaction).to_vec().pretty());
		self.dispatch_transaction(signed_transaction)
	}

	fn sign_call(&self, request: CallRequest) -> Result<SignedTransaction, Error> {
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);
		let from = request.from.unwrap_or(Address::zero());
		Ok(EthTransaction {
			nonce: request.nonce.unwrap_or_else(|| client.nonce(&from)),
			action: request.to.map_or(Action::Create, Action::Call),
			gas: request.gas.unwrap_or(U256::from(50_000_000)),
			gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
			value: request.value.unwrap_or_else(U256::zero),
			data: request.data.map_or_else(Vec::new, |d| d.to_vec())
		}.fake_sign(from))
	}

	fn dispatch_transaction(&self, signed_transaction: SignedTransaction) -> Result<Value, Error> {
		let hash = signed_transaction.hash();

		let import = {
			let client = take_weak!(self.client);
			let miner = take_weak!(self.miner);

			miner.import_own_transaction(client.deref(), signed_transaction, |a: &Address| {
				AccountDetails {
					nonce: client.nonce(&a),
					balance: client.balance(&a),
				}
			})
		};

		match import {
			Ok(_) => to_value(&hash),
			Err(e) => {
				warn!("Error sending transaction: {:?}", e);
				to_value(&H256::zero())
			}
		}
	}
}

const MAX_QUEUE_SIZE_TO_MINE_ON: usize = 4;	// because uncles go back 6.

fn params_len(params: &Params) -> usize {
	match params {
		&Params::Array(ref vec) => vec.len(),
		_ => 0,
	}
}

fn from_params_default_second<F>(params: Params) -> Result<(F, BlockNumber, ), Error> where F: serde::de::Deserialize {
	match params_len(&params) {
		1 => from_params::<(F, )>(params).map(|(f,)| (f, BlockNumber::Latest)),
		_ => from_params::<(F, BlockNumber)>(params),
	}
}

fn from_params_default_third<F1, F2>(params: Params) -> Result<(F1, F2, BlockNumber, ), Error> where F1: serde::de::Deserialize, F2: serde::de::Deserialize {
	match params_len(&params) {
		2 => from_params::<(F1, F2, )>(params).map(|(f1, f2)| (f1, f2, BlockNumber::Latest)),
		_ => from_params::<(F1, F2, BlockNumber)>(params)
	}
}

fn pending_logs<M>(miner: &M, filter: &EthcoreFilter) -> Vec<Log> where M: MinerService {
	let receipts = miner.pending_receipts();

	let pending_logs = receipts.into_iter()
		.flat_map(|(hash, r)| r.logs.into_iter().map(|l| (hash.clone(), l)).collect::<Vec<(H256, LogEntry)>>())
		.collect::<Vec<(H256, LogEntry)>>();

	let result = pending_logs.into_iter()
		.filter(|pair| filter.matches(&pair.1))
		.map(|pair| {
			let mut log = Log::from(pair.1);
			log.transaction_hash = Some(pair.0);
			log
		})
		.collect();

	result
}

impl<C, S, A, M, EM> Eth for EthClient<C, S, A, M, EM> where
	C: BlockChainClient + 'static,
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
					SyncState::Idle => SyncStatus::None,
					SyncState::Waiting | SyncState::Blocks | SyncState::NewBlocks | SyncState::ChainHead => {
						let info = SyncInfo {
							starting_block: U256::from(status.start_block_number),
							current_block: U256::from(take_weak!(self.client).chain_info().best_block_number),
							highest_block: U256::from(status.highest_block_number.unwrap_or(status.start_block_number))
						};
						match info.highest_block > info.starting_block + U256::from(6) {
							true => SyncStatus::Info(info),
							false => SyncStatus::None,
						}
					}
				};
				to_value(&res)
			}
			_ => Err(Error::invalid_params())
		}
	}

	fn author(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&take_weak!(self.miner).author()),
			_ => Err(Error::invalid_params()),
		}
	}

	fn is_mining(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&self.external_miner.is_mining()),
			_ => Err(Error::invalid_params())
		}
	}

	fn hashrate(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&self.external_miner.hashrate()),
			_ => Err(Error::invalid_params())
		}
	}

	fn gas_price(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&take_weak!(self.miner).sensible_gas_price()),
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
		from_params_default_second(params)
			.and_then(|(address, block_number,)| match block_number {
				BlockNumber::Latest => to_value(&take_weak!(self.client).balance(&address)),
				BlockNumber::Pending => to_value(&take_weak!(self.miner).balance(take_weak!(self.client).deref(), &address)),
				_ => Err(Error::invalid_params()),
			})
	}

	fn storage_at(&self, params: Params) -> Result<Value, Error> {
		from_params_default_third::<Address, U256>(params)
			.and_then(|(address, position, block_number,)| match block_number {
				BlockNumber::Pending => to_value(&U256::from(take_weak!(self.miner).storage_at(take_weak!(self.client).deref(), &address, &H256::from(position)))),
				BlockNumber::Latest => to_value(&U256::from(take_weak!(self.client).storage_at(&address, &H256::from(position)))),
				_ => Err(Error::invalid_params()),
			})
	}

	fn transaction_count(&self, params: Params) -> Result<Value, Error> {
		from_params_default_second(params)
			.and_then(|(address, block_number,)| match block_number {
				BlockNumber::Pending => to_value(&take_weak!(self.miner).nonce(take_weak!(self.client).deref(), &address)),
				BlockNumber::Latest => to_value(&take_weak!(self.client).nonce(&address)),
				_ => Err(Error::invalid_params()),
			})
	}

	fn block_transaction_count_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| // match
				take_weak!(self.client).block(BlockID::Hash(hash))
					.map_or(Ok(Value::Null), |bytes| to_value(&U256::from(BlockView::new(&bytes).transactions_count()))))
	}

	fn block_transaction_count_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| match block_number {
				BlockNumber::Pending => to_value(
					&U256::from(take_weak!(self.miner).status().transactions_in_pending_block)
				),
				_ => take_weak!(self.client).block(block_number.into())
						.map_or(Ok(Value::Null), |bytes| to_value(&U256::from(BlockView::new(&bytes).transactions_count())))
			})
	}

	fn block_uncles_count_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)|
				take_weak!(self.client).block(BlockID::Hash(hash))
					.map_or(Ok(Value::Null), |bytes| to_value(&U256::from(BlockView::new(&bytes).uncles_count()))))
	}

	fn block_uncles_count_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| match block_number {
				BlockNumber::Pending => to_value(&U256::from(0)),
				_ => take_weak!(self.client).block(block_number.into())
						.map_or(Ok(Value::Null), |bytes| to_value(&U256::from(BlockView::new(&bytes).uncles_count())))
			})
	}

	fn code_at(&self, params: Params) -> Result<Value, Error> {
		from_params_default_second(params)
			.and_then(|(address, block_number,)| match block_number {
				BlockNumber::Pending => to_value(&take_weak!(self.miner).code(take_weak!(self.client).deref(), &address).map_or_else(Bytes::default, Bytes::new)),
				BlockNumber::Latest => to_value(&take_weak!(self.client).code(&address).map_or_else(Bytes::default, Bytes::new)),
				_ => Err(Error::invalid_params()),
			})
	}

	fn block_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, bool)>(params)
			.and_then(|(hash, include_txs)| self.block(BlockID::Hash(hash), include_txs))
	}

	fn block_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, bool)>(params)
			.and_then(|(number, include_txs)| self.block(number.into(), include_txs))
	}

	fn transaction_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| {
				let miner = take_weak!(self.miner);
				match miner.transaction(&hash) {
					Some(pending_tx) => to_value(&Transaction::from(pending_tx)),
					None => self.transaction(TransactionID::Hash(hash))
				}
			})
	}

	fn transaction_by_block_hash_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, Index)>(params)
			.and_then(|(hash, index)| self.transaction(TransactionID::Location(BlockID::Hash(hash), index.value())))
	}

	fn transaction_by_block_number_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, Index)>(params)
			.and_then(|(number, index)| self.transaction(TransactionID::Location(number.into(), index.value())))
	}

	fn transaction_receipt(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| {
				let client = take_weak!(self.client);
				let receipt = client.transaction_receipt(TransactionID::Hash(hash));
				to_value(&receipt.map(Receipt::from))
			})
	}

	fn uncle_by_block_hash_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, Index)>(params)
			.and_then(|(hash, index)| self.uncle(UncleID(BlockID::Hash(hash), index.value())))
	}

	fn uncle_by_block_number_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, Index)>(params)
			.and_then(|(number, index)| self.uncle(UncleID(number.into(), index.value())))
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
				let include_pending = filter.to_block == Some(BlockNumber::Pending);
				let filter: EthcoreFilter = filter.into();
				let mut logs = take_weak!(self.client).logs(filter.clone())
					.into_iter()
					.map(From::from)
					.collect::<Vec<Log>>();

				if include_pending {
					let pending = pending_logs(take_weak!(self.miner).deref(), &filter);
					logs.extend(pending);
				}

				to_value(&logs)
			})
	}

	fn work(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let client = take_weak!(self.client);
				// check if we're still syncing and return empty strings in that case
				{
					//TODO: check if initial sync is complete here
					//let sync = take_weak!(self.sync);
					if /*sync.status().state != SyncState::Idle ||*/ client.queue_info().total_queue_size() > MAX_QUEUE_SIZE_TO_MINE_ON {
						trace!(target: "miner", "Syncing. Cannot give any work.");
						return to_value(&(String::new(), String::new(), String::new()));
					}
				}

				let miner = take_weak!(self.miner);
				miner.map_sealing_work(client.deref(), |b| {
					let pow_hash = b.hash();
					let target = Ethash::difficulty_to_boundary(b.block().header().difficulty());
					let seed_hash = &self.seed_compute.lock().unwrap().get_seedhash(b.block().header().number());
					to_value(&(pow_hash, H256::from_slice(&seed_hash[..]), target))
				}).unwrap_or(Err(Error::internal_error()))	// no work found.
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn submit_work(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H64, H256, H256)>(params).and_then(|(nonce, pow_hash, mix_hash)| {
			trace!(target: "miner", "submit_work: Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);
			let miner = take_weak!(self.miner);
			let client = take_weak!(self.client);
			let seal = vec![encode(&mix_hash).to_vec(), encode(&nonce).to_vec()];
			let r = miner.submit_seal(client.deref(), pow_hash, seal);
			to_value(&r.is_ok())
		})
	}

	fn submit_hashrate(&self, params: Params) -> Result<Value, Error> {
		from_params::<(U256, H256)>(params).and_then(|(rate, id)| {
			self.external_miner.submit_hashrate(rate, id);
			to_value(&true)
		})
	}

	fn send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, )>(params)
			.and_then(|(request, )| {
				let accounts = take_weak!(self.accounts);
				match accounts.account_secret(&request.from) {
					Ok(secret) => self.sign_and_dispatch(request, secret),
					Err(_) => to_value(&H256::zero())
				}
		})
	}

	fn sign_and_send_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TransactionRequest, String)>(params)
			.and_then(|(request, password)| {
				let accounts = take_weak!(self.accounts);
				match accounts.locked_account_secret(&request.from, &password) {
					Ok(secret) => self.sign_and_dispatch(request, secret),
					Err(_) => to_value(&H256::zero()),
				}
		})
	}

	fn send_raw_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Bytes, )>(params)
			.and_then(|(raw_transaction, )| {
				let raw_transaction = raw_transaction.to_vec();
				match UntrustedRlp::new(&raw_transaction).as_val() {
					Ok(signed_transaction) => self.dispatch_transaction(signed_transaction),
					Err(_) => to_value(&H256::zero()),
				}
		})
	}

	fn call(&self, params: Params) -> Result<Value, Error> {
		trace!(target: "jsonrpc", "call: {:?}", params);
		from_params_default_second(params)
			.and_then(|(request, block_number,)| {
				let signed = try!(self.sign_call(request));
				let r = match block_number {
					BlockNumber::Pending => take_weak!(self.miner).call(take_weak!(self.client).deref(), &signed),
					BlockNumber::Latest => take_weak!(self.client).call(&signed),
					_ => panic!("{:?}", block_number),
				};
				to_value(&r.map(|e| Bytes(e.output)).unwrap_or(Bytes::new(vec![])))
			})
	}

	fn estimate_gas(&self, params: Params) -> Result<Value, Error> {
		from_params_default_second(params)
			.and_then(|(request, block_number,)| {
				let signed = try!(self.sign_call(request));
				let r = match block_number {
					BlockNumber::Pending => take_weak!(self.miner).call(take_weak!(self.client).deref(), &signed),
					BlockNumber::Latest => take_weak!(self.client).call(&signed),
					_ => return Err(Error::invalid_params()),
				};
				to_value(&r.map(|res| res.gas_used + res.refunded).unwrap_or(From::from(0)))
			})
	}
}

/// Eth filter rpc implementation.
pub struct EthFilterClient<C, M> where
	C: BlockChainClient,
	M: MinerService {

	client: Weak<C>,
	miner: Weak<M>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl<C, M> EthFilterClient<C, M> where
	C: BlockChainClient,
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

impl<C, M> EthFilter for EthFilterClient<C, M> where
	C: BlockChainClient + 'static,
	M: MinerService + 'static {

	fn new_filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let mut polls = self.polls.lock().unwrap();
				let block_number = take_weak!(self.client).chain_info().best_block_number;
				let id = polls.create_poll(PollFilter::Logs(block_number, Default::default(), filter));
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
								.map(BlockID::Number)
								.filter_map(|id| client.block_hash(id))
								.collect::<Vec<H256>>();

							*block_number = current_number;

							to_value(&hashes)
						},
						PollFilter::PendingTransaction(ref mut previous_hashes) => {
							// get hashes of pending transactions
							let current_hashes = take_weak!(self.miner).pending_transactions_hashes();

							let new_hashes =
							{
								let previous_hashes_set = previous_hashes.iter().collect::<HashSet<_>>();

								//	find all new hashes
								current_hashes
									.iter()
									.filter(|hash| !previous_hashes_set.contains(hash))
									.cloned()
									.collect::<Vec<H256>>()
							};

							// save all hashes of pending transactions
							*previous_hashes = current_hashes;

							// return new hashes
							to_value(&new_hashes)
						},
						PollFilter::Logs(ref mut block_number, ref mut previous_logs, ref filter) => {
							// retrive the current block number
							let current_number = client.chain_info().best_block_number;

							// check if we need to check pending hashes
							let include_pending = filter.to_block == Some(BlockNumber::Pending);

							// build appropriate filter
							let mut filter: EthcoreFilter = filter.clone().into();
							filter.from_block = BlockID::Number(*block_number);
							filter.to_block = BlockID::Latest;

							// retrieve logs in range from_block..min(BlockID::Latest..to_block)
							let mut logs = client.logs(filter.clone())
								.into_iter()
								.map(From::from)
								.collect::<Vec<Log>>();

							// additionally retrieve pending logs
							if include_pending {
								let pending_logs = pending_logs(take_weak!(self.miner).deref(), &filter);

								// remove logs about which client was already notified about
								let new_pending_logs: Vec<_> = pending_logs.iter()
									.filter(|p| !previous_logs.contains(p))
									.cloned()
									.collect();

								// save all logs retrieved by client
								*previous_logs = pending_logs.into_iter().collect();

								// append logs array with new pending logs
								logs.extend(new_pending_logs);
							}

							// save current block number as next from block number
							*block_number = current_number;

							to_value(&logs)
						}
					}
				}
			})
	}

	fn filter_logs(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				let mut polls = self.polls.lock().unwrap();
				match polls.poll(&index.value()) {
					Some(&PollFilter::Logs(ref _block_number, ref _previous_log, ref filter)) => {
						let include_pending = filter.to_block == Some(BlockNumber::Pending);
						let filter: EthcoreFilter = filter.clone().into();
						let mut logs = take_weak!(self.client).logs(filter.clone())
							.into_iter()
							.map(From::from)
							.collect::<Vec<Log>>();

						if include_pending {
							logs.extend(pending_logs(take_weak!(self.miner).deref(), &filter));
						}

						to_value(&logs)
					},
					// just empty array
					_ => Ok(Value::Array(vec![] as Vec<Value>)),
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
