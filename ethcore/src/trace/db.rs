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

//! Trace database.
use std::collections::HashMap;
use std::sync::Arc;

use blockchain::BlockChainDB;
use db::cache_manager::CacheManager;
use db::{self, Key, Writable, Readable, CacheUpdatePolicy};
use ethereum_types::{H256, H264};
use heapsize::HeapSizeOf;
use kvdb::{DBTransaction};
use parking_lot::RwLock;
use types::BlockNumber;

use trace::{LocalizedTrace, Config, Filter, Database as TraceDatabase, ImportRequest, DatabaseExtras};
use trace::flat::{FlatTrace, FlatBlockTraces, FlatTransactionTraces};

const TRACE_DB_VER: &'static [u8] = b"1.0";

#[derive(Debug, Copy, Clone)]
enum TraceDBIndex {
	/// Block traces index.
	BlockTraces = 0,
}

impl Key<FlatBlockTraces> for H256 {
	type Target = H264;

	fn key(&self) -> H264 {
		let mut result = H264::default();
		{
			let bytes = result.as_bytes_mut();
			bytes[0] = TraceDBIndex::BlockTraces as u8;
			bytes[1..33].copy_from_slice(self.as_bytes());
		}
		result
	}
}

/// Database to store transaction execution trace.
///
/// Whenever a transaction is executed by EVM it's execution trace is stored
/// in trace database. Each trace has information, which contracts have been
/// touched, which have been created during the execution of transaction, and
/// which calls failed.
pub struct TraceDB<T> where T: DatabaseExtras {
	/// cache
	traces: RwLock<HashMap<H256, FlatBlockTraces>>,
	/// hashes of cached traces
	cache_manager: RwLock<CacheManager<H256>>,
	/// db
	db: Arc<BlockChainDB>,
	/// tracing enabled
	enabled: bool,
	/// extras
	extras: Arc<T>,
}

impl<T> TraceDB<T> where T: DatabaseExtras {
	/// Creates new instance of `TraceDB`.
	pub fn new(config: Config, db: Arc<BlockChainDB>, extras: Arc<T>) -> Self {
		let mut batch = DBTransaction::new();
		let genesis = extras.block_hash(0)
			.expect("Genesis block is always inserted upon extras db creation qed");
		batch.write(db::COL_TRACE, &genesis, &FlatBlockTraces::default());
		batch.put(db::COL_TRACE, b"version", TRACE_DB_VER);
		db.key_value().write(batch).expect("failed to update version");

		TraceDB {
			traces: RwLock::new(HashMap::new()),
			cache_manager: RwLock::new(CacheManager::new(config.pref_cache_size, config.max_cache_size, 10 * 1024)),
			db,
			enabled: config.enabled,
			extras: extras,
		}
	}

	fn cache_size(&self) -> usize {
		self.traces.read().heap_size_of_children()
	}

	/// Let the cache system know that a cacheable item has been used.
	fn note_trace_used(&self, trace_id: H256) {
		let mut cache_manager = self.cache_manager.write();
		cache_manager.note_used(trace_id);
	}

	/// Ticks our cache system and throws out any old data.
	pub fn collect_garbage(&self) {
		let current_size = self.cache_size();

		let mut traces = self.traces.write();
		let mut cache_manager = self.cache_manager.write();

		cache_manager.collect_garbage(current_size, | ids | {
			for id in &ids {
				traces.remove(id);
			}
			traces.shrink_to_fit();

			traces.heap_size_of_children()
		});
	}

	/// Returns traces for block with hash.
	fn traces(&self, block_hash: &H256) -> Option<FlatBlockTraces> {
		let result = self.db.key_value().read_with_cache(db::COL_TRACE, &self.traces, block_hash);
		self.note_trace_used(*block_hash);
		result
	}

	/// Returns vector of transaction traces for given block.
	fn transactions_traces(&self, block_hash: &H256) -> Option<Vec<FlatTransactionTraces>> {
		self.traces(block_hash).map(Into::into)
	}

	fn matching_block_traces(
		&self,
		filter: &Filter,
		traces: FlatBlockTraces,
		block_hash: H256,
		block_number: BlockNumber
	) -> Vec<LocalizedTrace> {
		let tx_traces: Vec<FlatTransactionTraces> = traces.into();
		tx_traces.into_iter()
			.enumerate()
			.flat_map(|(tx_number, tx_trace)| {
				self.matching_transaction_traces(filter, tx_trace, block_hash.clone(), block_number, tx_number)
			})
			.collect()
	}

	fn matching_transaction_traces(
		&self,
		filter: &Filter,
		traces: FlatTransactionTraces,
		block_hash: H256,
		block_number: BlockNumber,
		tx_number: usize
	) -> Vec<LocalizedTrace> {
		let (trace_tx_number, trace_tx_hash) = match self.extras.transaction_hash(block_number, tx_number) {
			Some(hash) => (Some(tx_number), Some(hash.clone())),
			//None means trace without transaction (reward)
			None => (None, None),
		};

		let flat_traces: Vec<FlatTrace> = traces.into();
		flat_traces.into_iter()
			.filter_map(|trace| {
				match filter.matches(&trace) {
					true => Some(LocalizedTrace {
						action: trace.action,
						result: trace.result,
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.into_iter().collect(),
						transaction_number: trace_tx_number,
						transaction_hash: trace_tx_hash,
						block_number: block_number,
						block_hash: block_hash
					}),
					false => None
				}
			})
			.collect()
	}
}

impl<T> TraceDatabase for TraceDB<T> where T: DatabaseExtras {
	fn tracing_enabled(&self) -> bool {
		self.enabled
	}

	/// Traces of import request's enacted blocks are expected to be already in database
	/// or to be the currently inserted trace.
	fn import(&self, batch: &mut DBTransaction, request: ImportRequest) {
		// valid (canon):  retracted 0, enacted 1 => false, true,
		// valid (branch): retracted 0, enacted 0 => false, false,
		// valid (bbcc):   retracted 1, enacted 1 => true, true,
		// invalid:	       retracted 1, enacted 0 => true, false,
		let ret = request.retracted != 0;
		let ena = !request.enacted.is_empty();
		assert!(!(ret && !ena));
		// fast return if tracing is disabled
		if !self.tracing_enabled() {
			return;
		}

		// now let's rebuild the blooms
		if !request.enacted.is_empty() {
			let range_start = request.block_number + 1 - request.enacted.len() as u64;
			let enacted_blooms: Vec<_> = request.enacted
				.iter()
				// all traces are expected to be found here. That's why `expect` has been used
				// instead of `filter_map`. If some traces haven't been found, it meens that
				// traces database is corrupted or incomplete.
				.map(|block_hash| if block_hash == &request.block_hash {
					request.traces.bloom()
				} else {
					self.traces(block_hash).expect("Traces database is incomplete.").bloom()
				})
				.collect();

			self.db.trace_blooms()
				.insert_blooms(range_start, enacted_blooms.iter())
				.expect("Low level database error. Some issue with disk?");
		}

		// insert new block traces into the cache and the database
		{
			let mut traces = self.traces.write();
			// it's important to use overwrite here,
			// cause this value might be queried by hash later
			batch.write_with_cache(db::COL_TRACE, &mut *traces, request.block_hash, request.traces, CacheUpdatePolicy::Overwrite);
			// note_used must be called after locking traces to avoid cache/traces deadlock on garbage collection
			self.note_trace_used(request.block_hash);
		}
	}

	fn trace(&self, block_number: BlockNumber, tx_position: usize, trace_position: Vec<usize>) -> Option<LocalizedTrace> {
		self.extras.block_hash(block_number)
			.and_then(|block_hash| self.transactions_traces(&block_hash)
				.and_then(|traces| traces.into_iter().nth(tx_position))
				.map(Into::<Vec<FlatTrace>>::into)
				// this may and should be optimized
				.and_then(|traces| traces.into_iter().find(|trace| trace.trace_address == trace_position))
				.map(|trace| {
					let tx_hash = self.extras.transaction_hash(block_number, tx_position)
						.expect("Expected to find transaction hash. Database is probably corrupted");

					LocalizedTrace {
						action: trace.action,
						result: trace.result,
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.into_iter().collect(),
						transaction_number: Some(tx_position),
						transaction_hash: Some(tx_hash),
						block_number: block_number,
						block_hash: block_hash,
					}
				})
			)
	}

	fn transaction_traces(&self, block_number: BlockNumber, tx_position: usize) -> Option<Vec<LocalizedTrace>> {
		self.extras.block_hash(block_number)
			.and_then(|block_hash| self.transactions_traces(&block_hash)
				.and_then(|traces| traces.into_iter().nth(tx_position))
				.map(Into::<Vec<FlatTrace>>::into)
				.map(|traces| {
					let tx_hash = self.extras.transaction_hash(block_number, tx_position)
						.expect("Expected to find transaction hash. Database is probably corrupted");

					traces.into_iter()
					.map(|trace| LocalizedTrace {
						action: trace.action,
						result: trace.result,
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.into_iter().collect(),
						transaction_number: Some(tx_position),
						transaction_hash: Some(tx_hash.clone()),
						block_number: block_number,
						block_hash: block_hash
					})
					.collect()
				})
			)
	}

	fn block_traces(&self, block_number: BlockNumber) -> Option<Vec<LocalizedTrace>> {
		self.extras.block_hash(block_number)
			.and_then(|block_hash| self.transactions_traces(&block_hash)
				.map(|traces| {
					traces.into_iter()
						.map(Into::<Vec<FlatTrace>>::into)
						.enumerate()
						.flat_map(|(tx_position, traces)| {
							let (trace_tx_number, trace_tx_hash) = match self.extras.transaction_hash(block_number, tx_position) {
								Some(hash) => (Some(tx_position), Some(hash.clone())),
								//None means trace without transaction (reward)
								None => (None, None),
							};

							traces.into_iter()
								.map(|trace| LocalizedTrace {
									action: trace.action,
									result: trace.result,
									subtraces: trace.subtraces,
									trace_address: trace.trace_address.into_iter().collect(),
									transaction_number: trace_tx_number,
									transaction_hash: trace_tx_hash,
									block_number: block_number,
									block_hash: block_hash,
								})
								.collect::<Vec<LocalizedTrace>>()
						})
						.collect::<Vec<LocalizedTrace>>()
				})
			)
	}

	fn filter(&self, filter: &Filter) -> Vec<LocalizedTrace> {
		let possibilities = filter.bloom_possibilities();
		let numbers = self.db.trace_blooms()
			.filter(filter.range.start as u64, filter.range.end as u64, &possibilities)
			.expect("Low level database error. Some issue with disk?");

		numbers.into_iter()
			.flat_map(|n| {
				let number = n as BlockNumber;
				let hash = self.extras.block_hash(number)
					.expect("Expected to find block hash. Extras db is probably corrupted");
				let traces = self.traces(&hash)
					.expect("Expected to find a trace. Db is probably corrupted.");
				self.matching_block_traces(filter, traces, hash, number)
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::sync::Arc;
	use ethereum_types::{H256, U256, Address};
	use kvdb::{DBTransaction};
	use types::BlockNumber;
	use trace::{Config, TraceDB, Database as TraceDatabase, DatabaseExtras, ImportRequest};
	use trace::{Filter, LocalizedTrace, AddressesFilter, TraceError};
	use trace::trace::{Call, Action, Res};
	use trace::flat::{FlatTrace, FlatBlockTraces, FlatTransactionTraces};
	use evm::CallType;
	use test_helpers::new_db;

	struct NoopExtras;

	impl DatabaseExtras for NoopExtras {
		fn block_hash(&self, block_number: BlockNumber) -> Option<H256> {
			if block_number == 0 {
				Some(H256::zero())
			} else {
				unimplemented!()
			}
		}

		fn transaction_hash(&self, _block_number: BlockNumber, _tx_position: usize) -> Option<H256> {
			unimplemented!();
		}
	}

	#[derive(Clone)]
	struct Extras {
		block_hashes: HashMap<BlockNumber, H256>,
		transaction_hashes: HashMap<BlockNumber, Vec<H256>>,
	}

	impl Default for Extras {
		fn default() -> Self {
			Extras {
				block_hashes: HashMap::new(),
				transaction_hashes: HashMap::new(),
			}
		}
	}

	impl DatabaseExtras for Extras {
		fn block_hash(&self, block_number: BlockNumber) -> Option<H256> {
			self.block_hashes.get(&block_number).cloned()
		}

		fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256> {
			self.transaction_hashes.get(&block_number)
				.and_then(|hashes| hashes.iter().cloned().nth(tx_position))
		}
	}

	#[test]
	fn test_reopening_db_with_tracing_off() {
		let db = new_db();
		let mut config = Config::default();

		// set autotracing
		config.enabled = false;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), false);
		}
	}

	#[test]
	fn test_reopening_db_with_tracing_on() {
		let db = new_db();
		let mut config = Config::default();

		// set tracing on
		config.enabled = true;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), true);
		}
	}

	fn create_simple_import_request(block_number: BlockNumber, block_hash: H256) -> ImportRequest {
		ImportRequest {
			traces: FlatBlockTraces::from(vec![FlatTransactionTraces::from(vec![FlatTrace {
				trace_address: Default::default(),
				subtraces: 0,
				action: Action::Call(Call {
					from: Address::from_low_u64_be(1),
					to: Address::from_low_u64_be(2),
					value: 3.into(),
					gas: 4.into(),
					input: vec![],
					call_type: CallType::Call,
				}),
				result: Res::FailedCall(TraceError::OutOfGas),
			}])]),
			block_hash: block_hash.clone(),
			block_number: block_number,
			enacted: vec![block_hash],
			retracted: 0,
		}
	}

	fn create_noncanon_import_request(block_number: BlockNumber, block_hash: H256) -> ImportRequest {
		ImportRequest {
			traces: FlatBlockTraces::from(vec![FlatTransactionTraces::from(vec![FlatTrace {
				trace_address: Default::default(),
				subtraces: 0,
				action: Action::Call(Call {
					from: Address::from_low_u64_be(1),
					to: Address::from_low_u64_be(2),
					value: 3.into(),
					gas: 4.into(),
					input: vec![],
					call_type: CallType::Call,
				}),
				result: Res::FailedCall(TraceError::OutOfGas),
			}])]),
			block_hash: block_hash.clone(),
			block_number: block_number,
			enacted: vec![],
			retracted: 0,
		}
	}

	fn create_simple_localized_trace(block_number: BlockNumber, block_hash: H256, tx_hash: H256) -> LocalizedTrace {
		LocalizedTrace {
			action: Action::Call(Call {
				from: Address::from_low_u64_be(1),
				to: Address::from_low_u64_be(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![],
				call_type: CallType::Call,
			}),
			result: Res::FailedCall(TraceError::OutOfGas),
			trace_address: vec![],
			subtraces: 0,
			transaction_number: Some(0),
			transaction_hash: Some(tx_hash),
			block_number: block_number,
			block_hash: block_hash,
		}
	}

	#[test]
	fn test_import_non_canon_traces() {
		let db = new_db();
		let mut config = Config::default();
		config.enabled = true;
		let block_0 = H256::from_low_u64_be(0xa1);
		let block_1 = H256::from_low_u64_be(0xa2);
		let tx_0 = H256::from_low_u64_be(0xff);
		let tx_1 = H256::from_low_u64_be(0xaf);

		let mut extras = Extras::default();
		extras.block_hashes.insert(0, block_0.clone());
		extras.block_hashes.insert(1, block_1.clone());
		extras.transaction_hashes.insert(0, vec![tx_0.clone()]);
		extras.transaction_hashes.insert(1, vec![tx_1.clone()]);

		let tracedb = TraceDB::new(config, db.clone(), Arc::new(extras));

		// import block 0
		let request = create_noncanon_import_request(0, block_0.clone());
		let mut batch = DBTransaction::new();
		tracedb.import(&mut batch, request);
		db.key_value().write(batch).unwrap();

		assert!(tracedb.traces(&block_0).is_some(), "Traces should be available even if block is non-canon.");
	}

	#[test]
	fn test_import() {
		let db = new_db();
		let mut config = Config::default();
		config.enabled = true;
		let block_1 = H256::from_low_u64_be(0xa1);
		let block_2 = H256::from_low_u64_be(0xa2);
		let tx_1 = H256::from_low_u64_be(0xff);
		let tx_2 = H256::from_low_u64_be(0xaf);

		let mut extras = Extras::default();
		extras.block_hashes.insert(0, H256::zero());

		extras.block_hashes.insert(1, block_1.clone());
		extras.block_hashes.insert(2, block_2.clone());
		extras.transaction_hashes.insert(1, vec![tx_1.clone()]);
		extras.transaction_hashes.insert(2, vec![tx_2.clone()]);

		let tracedb = TraceDB::new(config, db.clone(), Arc::new(extras));

		// import block 1
		let request = create_simple_import_request(1, block_1.clone());
		let mut batch = DBTransaction::new();
		tracedb.import(&mut batch, request);
		db.key_value().write(batch).unwrap();

		let filter = Filter {
			range: (1..1),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let traces = tracedb.filter(&filter);
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		// import block 2
		let request = create_simple_import_request(2, block_2.clone());
		let mut batch = DBTransaction::new();
		tracedb.import(&mut batch, request);
		db.key_value().write(batch).unwrap();

		let filter = Filter {
			range: (1..2),
			from_address: AddressesFilter::from(vec![Address::from_low_u64_be(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let traces = tracedb.filter(&filter);
		assert_eq!(traces.len(), 2);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));
		assert_eq!(traces[1], create_simple_localized_trace(2, block_2.clone(), tx_2.clone()));

		assert!(tracedb.block_traces(0).is_some(), "Genesis trace should be always present.");

		let traces = tracedb.block_traces(1).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		let traces = tracedb.block_traces(2).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(2, block_2.clone(), tx_2.clone()));

		assert_eq!(None, tracedb.block_traces(3));

		let traces = tracedb.transaction_traces(1, 0).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		let traces = tracedb.transaction_traces(2, 0).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(2, block_2.clone(), tx_2.clone()));

		assert_eq!(None, tracedb.transaction_traces(2, 1));

		assert_eq!(tracedb.trace(1, 0, vec![]).unwrap(), create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));
		assert_eq!(tracedb.trace(2, 0, vec![]).unwrap(), create_simple_localized_trace(2, block_2.clone(), tx_2.clone()));
	}

	#[test]
	fn query_trace_after_reopen() {
		let db = new_db();
		let mut config = Config::default();
		let mut extras = Extras::default();
		let block_0 = H256::from_low_u64_be(0xa1);
		let tx_0 = H256::from_low_u64_be(0xff);

		extras.block_hashes.insert(0, H256::zero());
		extras.transaction_hashes.insert(0, vec![]);
		extras.block_hashes.insert(1, block_0.clone());
		extras.transaction_hashes.insert(1, vec![tx_0.clone()]);

		// set tracing on
		config.enabled = true;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(extras.clone()));

			// import block 1
			let request = create_simple_import_request(1, block_0.clone());
			let mut batch = DBTransaction::new();
			tracedb.import(&mut batch, request);
			db.key_value().write(batch).unwrap();
		}

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(extras));
			let traces = tracedb.transaction_traces(1, 0);
			assert_eq!(traces.unwrap(), vec![create_simple_localized_trace(1, block_0, tx_0)]);
		}
	}

	#[test]
	fn query_genesis() {
		let db = new_db();
		let mut config = Config::default();
		let mut extras = Extras::default();
		let block_0 = H256::from_low_u64_be(0xa1);

		extras.block_hashes.insert(0, block_0.clone());
		extras.transaction_hashes.insert(0, vec![]);

		// set tracing on
		config.enabled = true;

		let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(extras.clone()));
		let traces = tracedb.block_traces(0).unwrap();

		assert_eq!(traces.len(), 0);
	}
}
