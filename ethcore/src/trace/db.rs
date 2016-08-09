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

//! Trace database.
use std::ops::Deref;
use std::collections::HashMap;
use std::sync::Arc;
use bloomchain::{Number, Config as BloomConfig};
use bloomchain::group::{BloomGroupDatabase, BloomGroupChain, GroupPosition, BloomGroup};
use util::{H256, H264, Database, DBTransaction, RwLock, HeapSizeOf};
use header::BlockNumber;
use trace::{LocalizedTrace, Config, Switch, Filter, Database as TraceDatabase, ImportRequest, DatabaseExtras, Error};
use db::{Key, Writable, Readable, CacheUpdatePolicy};
use blooms;
use super::flat::{FlatTrace, FlatBlockTraces, FlatTransactionTraces};
use client::DB_COL_TRACE;
use cache_manager::CacheManager;

const TRACE_DB_VER: &'static [u8] = b"1.0";

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="dev", allow(enum_variant_names))]
enum TraceDBIndex {
	/// Block traces index.
	BlockTraces = 0,
	/// Trace bloom group index.
	BloomGroups = 1,
}

impl Key<FlatBlockTraces> for H256 {
	type Target = H264;

	fn key(&self) -> H264 {
		let mut result = H264::default();
		result[0] = TraceDBIndex::BlockTraces as u8;
		result[1..33].copy_from_slice(self);
		result
	}
}

/// Wrapper around `blooms::GroupPosition` so it could be
/// uniquely identified in the database.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct TraceGroupPosition(blooms::GroupPosition);

impl From<GroupPosition> for TraceGroupPosition {
	fn from(position: GroupPosition) -> Self {
		TraceGroupPosition(From::from(position))
	}
}

impl HeapSizeOf for TraceGroupPosition {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}

/// Helper data structure created cause [u8; 6] does not implement Deref to &[u8].
pub struct TraceGroupKey([u8; 6]);

impl Deref for TraceGroupKey {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Key<blooms::BloomGroup> for TraceGroupPosition {
	type Target = TraceGroupKey;

	fn key(&self) -> Self::Target {
		let mut result = [0u8; 6];
		result[0] = TraceDBIndex::BloomGroups as u8;
		result[1] = self.0.level;
		result[2] = self.0.index as u8;
		result[3] = (self.0.index >> 8) as u8;
		result[4] = (self.0.index >> 16) as u8;
		result[5] = (self.0.index >> 24) as u8;
		TraceGroupKey(result)
	}
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum CacheID {
	Trace(H256),
	Bloom(TraceGroupPosition),
}

/// Trace database.
pub struct TraceDB<T> where T: DatabaseExtras {
	// cache
	traces: RwLock<HashMap<H256, FlatBlockTraces>>,
	blooms: RwLock<HashMap<TraceGroupPosition, blooms::BloomGroup>>,
	cache_manager: RwLock<CacheManager<CacheID>>,
	// db
	tracesdb: Arc<Database>,
	// config,
	bloom_config: BloomConfig,
	// tracing enabled
	enabled: bool,
	// extras
	extras: Arc<T>,
}

impl<T> BloomGroupDatabase for TraceDB<T> where T: DatabaseExtras {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		let position = TraceGroupPosition::from(position.clone());
		self.note_used(CacheID::Bloom(position.clone()));
		self.tracesdb.read_with_cache(DB_COL_TRACE, &self.blooms, &position).map(Into::into)
	}
}

impl<T> TraceDB<T> where T: DatabaseExtras {
	/// Creates new instance of `TraceDB`.
	pub fn new(config: Config, tracesdb: Arc<Database>, extras: Arc<T>) -> Result<Self, Error> {
		// check if in previously tracing was enabled
		let old_tracing = match tracesdb.get(DB_COL_TRACE, b"enabled").unwrap() {
			Some(ref value) if value as &[u8] == &[0x1] => Switch::On,
			Some(ref value) if value as &[u8] == &[0x0] => Switch::Off,
			Some(_) => { panic!("tracesdb is corrupted") },
			None => Switch::Auto,
		};

		let enabled = try!(old_tracing.turn_to(config.enabled));

		let encoded_tracing = match enabled {
			true => [0x1],
			false => [0x0]
		};

		let batch = DBTransaction::new(&tracesdb);
		batch.put(DB_COL_TRACE, b"enabled", &encoded_tracing).unwrap();
		batch.put(DB_COL_TRACE, b"version", TRACE_DB_VER).unwrap();
		tracesdb.write(batch).unwrap();

		let db = TraceDB {
			traces: RwLock::new(HashMap::new()),
			blooms: RwLock::new(HashMap::new()),
			cache_manager: RwLock::new(CacheManager::new(config.pref_cache_size, config.max_cache_size, 10 * 1024)),
			tracesdb: tracesdb,
			bloom_config: config.blooms,
			enabled: enabled,
			extras: extras,
		};

		Ok(db)
	}

	fn cache_size(&self) -> usize {
		let traces = self.traces.read().heap_size_of_children();
		let blooms = self.blooms.read().heap_size_of_children();
		traces + blooms
	}

	/// Let the cache system know that a cacheable item has been used.
	fn note_used(&self, id: CacheID) {
		let mut cache_manager = self.cache_manager.write();
		cache_manager.note_used(id);
	}

	/// Ticks our cache system and throws out any old data.
	pub fn collect_garbage(&self) {
		let mut cache_manager = self.cache_manager.write();
		cache_manager.collect_garbage(|| self.cache_size(), | ids | {
			let mut traces = self.traces.write();
			let mut blooms = self.blooms.write();

			for id in &ids {
				match *id {
					CacheID::Trace(ref h) => { traces.remove(h); },
					CacheID::Bloom(ref h) => { blooms.remove(h); },
				}
			}
			traces.shrink_to_fit();
			blooms.shrink_to_fit();
		});
	}

	/// Returns traces for block with hash.
	fn traces(&self, block_hash: &H256) -> Option<FlatBlockTraces> {
		self.note_used(CacheID::Trace(block_hash.clone()));
		self.tracesdb.read_with_cache(DB_COL_TRACE, &self.traces, block_hash)
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
		let tx_hash = self.extras.transaction_hash(block_number, tx_number)
			.expect("Expected to find transaction hash. Database is probably corrupted");

		let flat_traces: Vec<FlatTrace> = traces.into();
		flat_traces.into_iter()
			.filter_map(|trace| {
				match filter.matches(&trace) {
					true => Some(LocalizedTrace {
						action: trace.action,
						result: trace.result,
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.into_iter().collect(),
						transaction_number: tx_number,
						transaction_hash: tx_hash.clone(),
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
	fn import(&self, batch: &DBTransaction, request: ImportRequest) {
		// fast return if tracing is disabled
		if !self.tracing_enabled() {
			return;
		}

		// at first, let's insert new block traces
		{
			// note_used must be called before locking traces to avoid cache/traces deadlock on garbage collection
			self.note_used(CacheID::Trace(request.block_hash.clone()));
			let mut traces = self.traces.write();
			// it's important to use overwrite here,
			// cause this value might be queried by hash later
			batch.write_with_cache(DB_COL_TRACE, &mut *traces, request.block_hash, request.traces, CacheUpdatePolicy::Overwrite);
		}

		// now let's rebuild the blooms
		{
			let range_start = request.block_number as Number + 1 - request.enacted.len();
			let range_end = range_start + request.retracted;
			let replaced_range = range_start..range_end;
			let enacted_blooms = request.enacted
				.iter()
				// all traces are expected to be found here. That's why `expect` has been used
				// instead of `filter_map`. If some traces haven't been found, it meens that
				// traces database is corrupted or incomplete.
				.map(|block_hash| self.traces(block_hash).expect("Traces database is incomplete."))
				.map(|block_traces| block_traces.bloom())
				.map(blooms::Bloom::from)
				.map(Into::into)
				.collect();

			let chain = BloomGroupChain::new(self.bloom_config, self);
			let trace_blooms = chain.replace(&replaced_range, enacted_blooms);
			let blooms_to_insert = trace_blooms.into_iter()
				.map(|p| (From::from(p.0), From::from(p.1)))
				.collect::<HashMap<TraceGroupPosition, blooms::BloomGroup>>();

			// note_used must be called before locking blooms to avoid cache/traces deadlock on garbage collection
			for key in blooms_to_insert.keys() {
				self.note_used(CacheID::Bloom(key.clone()));
			}
			let mut blooms = self.blooms.write();
			batch.extend_with_cache(DB_COL_TRACE, &mut *blooms, blooms_to_insert, CacheUpdatePolicy::Remove);
		}
	}

	fn trace(&self, block_number: BlockNumber, tx_position: usize, trace_position: Vec<usize>) -> Option<LocalizedTrace> {
		let trace_position_deq = trace_position.into_iter().collect();
		self.extras.block_hash(block_number)
			.and_then(|block_hash| self.transactions_traces(&block_hash)
				.and_then(|traces| traces.into_iter().nth(tx_position))
				.map(Into::<Vec<FlatTrace>>::into)
				// this may and should be optimized
				.and_then(|traces| traces.into_iter().find(|trace| trace.trace_address == trace_position_deq))
				.map(|trace| {
					let tx_hash = self.extras.transaction_hash(block_number, tx_position)
						.expect("Expected to find transaction hash. Database is probably corrupted");

					LocalizedTrace {
						action: trace.action,
						result: trace.result,
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.into_iter().collect(),
						transaction_number: tx_position,
						transaction_hash: tx_hash,
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
						transaction_number: tx_position,
						transaction_hash: tx_hash.clone(),
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
							let tx_hash = self.extras.transaction_hash(block_number, tx_position)
								.expect("Expected to find transaction hash. Database is probably corrupted");

							traces.into_iter()
								.map(|trace| LocalizedTrace {
									action: trace.action,
									result: trace.result,
									subtraces: trace.subtraces,
									trace_address: trace.trace_address.into_iter().collect(),
									transaction_number: tx_position,
									transaction_hash: tx_hash.clone(),
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
		let chain = BloomGroupChain::new(self.bloom_config, self);
		let numbers = chain.filter(filter);
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
	use util::{Address, U256, H256, Database, DatabaseConfig, DBTransaction};
	use devtools::RandomTempPath;
	use header::BlockNumber;
	use trace::{Config, Switch, TraceDB, Database as TraceDatabase, DatabaseExtras, ImportRequest};
	use trace::{Filter, LocalizedTrace, AddressesFilter};
	use trace::trace::{Call, Action, Res};
	use trace::flat::{FlatTrace, FlatBlockTraces, FlatTransactionTraces};
	use client::DB_NO_OF_COLUMNS;
	use types::executed::CallType;

	struct NoopExtras;

	impl DatabaseExtras for NoopExtras {
		fn block_hash(&self, _block_number: BlockNumber) -> Option<H256> {
			unimplemented!();
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

	fn new_db(path: &str) -> Arc<Database> {
		Arc::new(Database::open(&DatabaseConfig::with_columns(DB_NO_OF_COLUMNS), path).unwrap())
	}

	#[test]
	fn test_reopening_db_with_tracing_off() {
		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let mut config = Config::default();

		// set autotracing
		config.enabled = Switch::Auto;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), false);
		}

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), false);
		}

		config.enabled = Switch::Off;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), false);
		}
	}

	#[test]
	fn test_reopening_db_with_tracing_on() {
		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let mut config = Config::default();

		// set tracing on
		config.enabled = Switch::On;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = Switch::Auto;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = Switch::Off;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), false);
		}
	}

	#[test]
	#[should_panic]
	fn test_invalid_reopening_db() {
		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let mut config = Config::default();

		// set tracing on
		config.enabled = Switch::Off;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap();
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = Switch::On;
		TraceDB::new(config.clone(), db.clone(), Arc::new(NoopExtras)).unwrap(); // should panic!
	}

	fn create_simple_import_request(block_number: BlockNumber, block_hash: H256) -> ImportRequest {
		ImportRequest {
			traces: FlatBlockTraces::from(vec![FlatTransactionTraces::from(vec![FlatTrace {
				trace_address: Default::default(),
				subtraces: 0,
				action: Action::Call(Call {
					from: 1.into(),
					to: 2.into(),
					value: 3.into(),
					gas: 4.into(),
					input: vec![],
					call_type: CallType::Call,
				}),
				result: Res::FailedCall,
			}])]),
			block_hash: block_hash.clone(),
			block_number: block_number,
			enacted: vec![block_hash],
			retracted: 0,
		}
	}

	fn create_simple_localized_trace(block_number: BlockNumber, block_hash: H256, tx_hash: H256) -> LocalizedTrace {
		LocalizedTrace {
			action: Action::Call(Call {
				from: Address::from(1),
				to: Address::from(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![],
				call_type: CallType::Call,
			}),
			result: Res::FailedCall,
			trace_address: vec![],
			subtraces: 0,
			transaction_number: 0,
			transaction_hash: tx_hash,
			block_number: block_number,
			block_hash: block_hash,
		}
	}


	#[test]
	fn test_import() {
		let temp = RandomTempPath::new();
		let db = Arc::new(Database::open(&DatabaseConfig::with_columns(DB_NO_OF_COLUMNS), temp.as_str()).unwrap());
		let mut config = Config::default();
		config.enabled = Switch::On;
		let block_0 = H256::from(0xa1);
		let block_1 = H256::from(0xa2);
		let tx_0 = H256::from(0xff);
		let tx_1 = H256::from(0xaf);

		let mut extras = Extras::default();
		extras.block_hashes.insert(0, block_0.clone());
		extras.block_hashes.insert(1, block_1.clone());
		extras.transaction_hashes.insert(0, vec![tx_0.clone()]);
		extras.transaction_hashes.insert(1, vec![tx_1.clone()]);

		let tracedb = TraceDB::new(config, db.clone(), Arc::new(extras)).unwrap();

		// import block 0
		let request = create_simple_import_request(0, block_0.clone());
		let batch = DBTransaction::new(&db);
		tracedb.import(&batch, request);
		db.write(batch).unwrap();

		let filter = Filter {
			range: (0..0),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let traces = tracedb.filter(&filter);
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(0, block_0.clone(), tx_0.clone()));

		// import block 1
		let request = create_simple_import_request(1, block_1.clone());
		let batch = DBTransaction::new(&db);
		tracedb.import(&batch, request);
		db.write(batch).unwrap();

		let filter = Filter {
			range: (0..1),
			from_address: AddressesFilter::from(vec![Address::from(1)]),
			to_address: AddressesFilter::from(vec![]),
		};

		let traces = tracedb.filter(&filter);
		assert_eq!(traces.len(), 2);
		assert_eq!(traces[0], create_simple_localized_trace(0, block_0.clone(), tx_0.clone()));
		assert_eq!(traces[1], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		let traces = tracedb.block_traces(0).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(0, block_0.clone(), tx_0.clone()));

		let traces = tracedb.block_traces(1).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		assert_eq!(None, tracedb.block_traces(2));

		let traces = tracedb.transaction_traces(0, 0).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(0, block_0.clone(), tx_0.clone()));

		let traces = tracedb.transaction_traces(1, 0).unwrap();
		assert_eq!(traces.len(), 1);
		assert_eq!(traces[0], create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));

		assert_eq!(None, tracedb.transaction_traces(1, 1));

		assert_eq!(tracedb.trace(0, 0, vec![]).unwrap(), create_simple_localized_trace(0, block_0.clone(), tx_0.clone()));
		assert_eq!(tracedb.trace(1, 0, vec![]).unwrap(), create_simple_localized_trace(1, block_1.clone(), tx_1.clone()));
	}

	#[test]
	fn query_trace_after_reopen() {
		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let mut config = Config::default();
		let mut extras = Extras::default();
		let block_0 = H256::from(0xa1);
		let tx_0 = H256::from(0xff);

		extras.block_hashes.insert(0, block_0.clone());
		extras.transaction_hashes.insert(0, vec![tx_0.clone()]);

		// set tracing on
		config.enabled = Switch::On;

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(extras.clone())).unwrap();

			// import block 0
			let request = create_simple_import_request(0, block_0.clone());
			let batch = DBTransaction::new(&db);
			tracedb.import(&batch, request);
			db.write(batch).unwrap();
		}

		{
			let tracedb = TraceDB::new(config.clone(), db.clone(), Arc::new(extras)).unwrap();
			let traces = tracedb.transaction_traces(0, 0);
			assert_eq!(traces.unwrap(), vec![create_simple_localized_trace(0, block_0, tx_0)]);
		}
	}
}
