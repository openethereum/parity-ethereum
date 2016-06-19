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

use std::sync::{Arc, Weak, Mutex};
use std::ops::Deref;
use ethsync::{SyncProvider, SyncState};
use ethcore::miner::{MinerService, ExternalMinerService};
use jsonrpc_core::*;
use util::numbers::*;
use util::sha3::*;
use util::rlp::{encode, decode, UntrustedRlp, View};
use util::keys::store::AccountProvider;
use ethcore::client::{MiningBlockChainClient, BlockID, TransactionID, UncleID};
use ethcore::block::IsBlock;
use ethcore::views::*;
use ethcore::ethereum::Ethash;
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};
use ethcore::log_entry::LogEntry;
use ethcore::filter::Filter as EthcoreFilter;
use self::ethash::SeedHashCompute;
use v1::traits::Eth;
use v1::types::{Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo, Transaction, CallRequest, OptionalValue, Index, Filter, Log, Receipt};
use v1::impls::{dispatch_transaction, error_codes};
use serde;

/// Eth rpc implementation.
pub struct EthClient<C, S, A, M, EM> where
	C: MiningBlockChainClient,
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
	allow_pending_receipt_query: bool,
}

impl<C, S, A, M, EM> EthClient<C, S, A, M, EM> where
	C: MiningBlockChainClient,
	S: SyncProvider,
	A: AccountProvider,
	M: MinerService,
	EM: ExternalMinerService {

	/// Creates new EthClient.
	pub fn new(client: &Arc<C>, sync: &Arc<S>, accounts: &Arc<A>, miner: &Arc<M>, em: &Arc<EM>, allow_pending_receipt_query: bool)
		-> EthClient<C, S, A, M, EM> {
		EthClient {
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
			external_miner: em.clone(),
			seed_compute: Mutex::new(SeedHashCompute::new()),
			allow_pending_receipt_query: allow_pending_receipt_query,
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

	fn default_gas_price(&self) -> Result<U256, Error> {
		let miner = take_weak!(self.miner);
		Ok(take_weak!(self.client)
			.gas_price_statistics(100, 8)
			.map(|x| x[4])
			.unwrap_or_else(|_| miner.sensible_gas_price())
		)
	}

	fn sign_call(&self, request: CallRequest) -> Result<SignedTransaction, Error> {
		let client = take_weak!(self.client);
		let from = request.from.unwrap_or(Address::zero());
		Ok(EthTransaction {
			nonce: request.nonce.unwrap_or_else(|| client.latest_nonce(&from)),
			action: request.to.map_or(Action::Create, Action::Call),
			gas: request.gas.unwrap_or(U256::from(50_000_000)),
			gas_price: request.gas_price.unwrap_or_else(|| self.default_gas_price().expect("call only fails if client or miner are unavailable; client and miner are both available to be here; qed")),
			value: request.value.unwrap_or_else(U256::zero),
			data: request.data.map_or_else(Vec::new, |d| d.to_vec())
		}.fake_sign(from))
	}
}

pub fn pending_logs<M>(miner: &M, filter: &EthcoreFilter) -> Vec<Log> where M: MinerService {
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

fn make_unsupported_err() -> Error {
	Error {
		code: ErrorCode::ServerError(error_codes::UNSUPPORTED_REQUEST_CODE),
		message: "Unsupported request.".into(),
		data: None
	}
}

fn no_work_err() -> Error {
	Error {
		code: ErrorCode::ServerError(error_codes::NO_WORK_CODE),
		message: "Still syncing.".into(),
		data: None
	}
}

impl<C, S, A, M, EM> Eth for EthClient<C, S, A, M, EM> where
	C: MiningBlockChainClient + 'static,
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
						let current_block = U256::from(take_weak!(self.client).chain_info().best_block_number);

						let info = SyncInfo {
							starting_block: U256::from(status.start_block_number),
							current_block: current_block,
							highest_block: U256::from(status.highest_block_number.unwrap_or(status.start_block_number))
						};
						match info.highest_block > info.current_block + U256::from(6) {
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
			Params::None => to_value(&try!(self.default_gas_price())),
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
				BlockNumber::Pending => to_value(&take_weak!(self.miner).balance(take_weak!(self.client).deref(), &address)),
				id => to_value(&try!(take_weak!(self.client).balance(&address, id.into()).ok_or_else(make_unsupported_err))),
			})
	}

	fn storage_at(&self, params: Params) -> Result<Value, Error> {
		from_params_default_third::<Address, U256>(params)
			.and_then(|(address, position, block_number,)| match block_number {
				BlockNumber::Pending => to_value(&U256::from(take_weak!(self.miner).storage_at(&*take_weak!(self.client), &address, &H256::from(position)))),
				id => match take_weak!(self.client).storage_at(&address, &H256::from(position), id.into()) {
					Some(s) => to_value(&U256::from(s)),
					None => Err(make_unsupported_err()), // None is only returned on unsupported requests.
				}
			})
	}

	fn transaction_count(&self, params: Params) -> Result<Value, Error> {
		from_params_default_second(params)
			.and_then(|(address, block_number,)| match block_number {
				BlockNumber::Pending => to_value(&take_weak!(self.miner).nonce(take_weak!(self.client).deref(), &address)),
				id => to_value(&take_weak!(self.client).nonce(&address, id.into())),
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
				let miner = take_weak!(self.miner);
				match miner.pending_receipts().get(&hash) {
					Some(receipt) if self.allow_pending_receipt_query => to_value(&Receipt::from(receipt.clone())),
					_ => {
						let client = take_weak!(self.client);
						let receipt = client.transaction_receipt(TransactionID::Hash(hash));
						to_value(&receipt.map(Receipt::from))
					}
				}
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
						return Err(no_work_err());
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

	fn send_raw_transaction(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Bytes, )>(params)
			.and_then(|(raw_transaction, )| {
				let raw_transaction = raw_transaction.to_vec();
				match UntrustedRlp::new(&raw_transaction).as_val() {
					Ok(signed_transaction) => dispatch_transaction(&*take_weak!(self.client), &*take_weak!(self.miner), signed_transaction),
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
					BlockNumber::Pending => take_weak!(self.miner).call(take_weak!(self.client).deref(), &signed, Default::default()),
					BlockNumber::Latest => take_weak!(self.client).call(&signed, Default::default()),
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
					BlockNumber::Pending => take_weak!(self.miner).call(take_weak!(self.client).deref(), &signed, Default::default()),
					BlockNumber::Latest => take_weak!(self.client).call(&signed, Default::default()),
					_ => return Err(Error::invalid_params()),
				};
				to_value(&r.map(|res| res.gas_used + res.refunded).unwrap_or(From::from(0)))
			})
	}

	fn compile_lll(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}

	fn compile_serpent(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}

	fn compile_solidity(&self, _: Params) -> Result<Value, Error> {
		rpc_unimplemented!()
	}
}
