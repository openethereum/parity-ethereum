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

//! Fat database.
use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::path::Path;
use bloomchain::Number;
use bloomchain::group::{BloomGroupDatabase, BloomGroupChain, GroupPosition, BloomGroup};
use util::{FixedHash, H256, H264, Database, DBTransaction};
use header::BlockNumber;
use trace::{BlockTraces, LocalizedTrace, Config, Filter, Database as TraceDatabase, ImportRequest,
DatabaseExtras};
use db::{Key, Writable, Readable, CacheUpdatePolicy};
use super::bloom::{TraceGroupPosition, BlockTracesBloom, BlockTracesBloomGroup};
use super::flat::{FlatTrace, FlatBlockTraces, FlatTransactionTraces};

#[derive(Debug, Copy, Clone)]
pub enum TracedbIndex {
	/// Block traces index.
	BlockTraces = 0,
	/// Trace bloom group index.
	BlockTracesBloomGroups = 1,
}

fn with_index(hash: &H256, i: TracedbIndex) -> H264 {
	let mut slice = H264::from_slice(hash);
	slice[32] = i as u8;
	slice
}

impl Key<BlockTraces> for H256 {
	fn key(&self) -> H264 {
		with_index(self, TracedbIndex::BlockTraces)
	}
}

impl Key<BlockTracesBloomGroup> for TraceGroupPosition {
	fn key(&self) -> H264 {
		with_index(&self.hash(), TracedbIndex::BlockTracesBloomGroups)
	}
}

/// Fat database.
pub struct Tracedb<T> where T: DatabaseExtras {
	// cache
	traces: RwLock<HashMap<H256, BlockTraces>>,
	blooms: RwLock<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>,
	// db
	tracesdb: Database,
	// config,
	config: Config,
	// extras
	extras: Arc<T>,
}

impl<T> BloomGroupDatabase for Tracedb<T> where T: DatabaseExtras {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		let position = TraceGroupPosition::from(position.clone());
		self.tracesdb.read_with_cache(&self.blooms, &position).map(Into::into)
	}
}

impl<T> Tracedb<T> where T: DatabaseExtras {
	/// Creates new instance of `Tracedb`.
	pub fn new(mut config: Config, path: &Path, extras: Arc<T>) -> Self {
		let mut tracedb_path = path.to_path_buf();
		tracedb_path.push("tracedb");
		let tracesdb = Database::open_default(tracedb_path.to_str().unwrap()).unwrap();

		// check if in previously tracing was enabled
		let tracing_was_enabled = match tracesdb.get(b"enabled").unwrap() {
			Some(ref value) if value as &[u8] == &[0x1] => Some(true),
			Some(ref value) if value as &[u8] == &[0x0] => Some(false),
			Some(_) => { panic!("tracesdb is malformed") },
			None => None,
		};

		// compare it with the current option.
		let tracing = match (tracing_was_enabled, config.enabled) {
			(Some(true), Some(true)) => true,
			(Some(true), None) => true,
			(Some(true), Some(false)) => false,
			(Some(false), Some(true)) => { panic!("Tracing can't be enabled. Resync required."); },
			(Some(false), None) => false,
			(Some(false), Some(false)) => false,
			(None, Some(true)) => true,
			_ => false,
		};

		config.enabled = Some(tracing);

		let encoded_tracing= match tracing {
			true => [0x1],
			false => [0x0]
		};

		tracesdb.put(b"enabled", &encoded_tracing).unwrap();

		Tracedb {
			traces: RwLock::new(HashMap::new()),
			blooms: RwLock::new(HashMap::new()),
			tracesdb: tracesdb,
			config: config,
			extras: extras,
		}
	}

	/// Returns traces for block with hash.
	fn traces(&self, block_hash: &H256) -> Option<BlockTraces> {
		self.tracesdb.read_with_cache(&self.traces, block_hash)
	}

	/// Returns vector of transaction traces for given block.
	fn transactions_traces(&self, block_hash: &H256) -> Option<Vec<FlatTransactionTraces>> {
		self.traces(block_hash)
			.map(FlatBlockTraces::from)
			.map(Into::into)
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
			.expect("Expected to find transaction hash. Database is probably malformed");

		let flat_traces: Vec<FlatTrace> = traces.into();
		flat_traces.into_iter()
			.enumerate()
			.filter_map(|(index, trace)| {
				match filter.matches(&trace) {
					true => Some(LocalizedTrace {
						parent: trace.parent,
						children: trace.children,
						depth: trace.depth,
						action: trace.action,
						result: trace.result,
						trace_number: index,
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

impl<T> TraceDatabase for Tracedb<T> where T: DatabaseExtras {
	fn tracing_enabled(&self) -> bool {
		self.config.enabled.expect("Auto tracing hasn't been properly configured.")
	}

	/// Traces of import request's enacted blocks are expected to be already in database
	/// or to be the currently inserted trace.
	fn import(&self, request: ImportRequest) {
		// fast return if tracing is disabled
		if !self.tracing_enabled() {
			return;
		}

		let batch = DBTransaction::new();

		// at first, let's insert new block traces
		{
			let mut traces = self.traces.write().unwrap();
			// it's important to use overwrite here,
			// cause this value might be queried by hash later
			batch.write_with_cache(&mut traces, request.block_hash, request.traces, CacheUpdatePolicy::Overwrite);
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
				// traces database is malformed or incomplete.
				.map(|block_hash| self.traces(block_hash).expect("Traces database is incomplete."))
				.map(|block_traces| block_traces.bloom())
				.map(BlockTracesBloom::from)
				.map(Into::into)
				.collect();

			let chain = BloomGroupChain::new(self.config.blooms, self);
			let trace_blooms = chain.replace(&replaced_range, enacted_blooms);
			let blooms_to_insert = trace_blooms.into_iter()
				.map(|p| (From::from(p.0), From::from(p.1)))
				.collect::<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>();

			let mut blooms = self.blooms.write().unwrap();
			batch.extend_with_cache(&mut blooms, blooms_to_insert, CacheUpdatePolicy::Remove);
		}

		self.tracesdb.write(batch).unwrap();
	}

	fn trace(&self, block_number: BlockNumber, tx_position: usize, trace_position: usize) -> Option<LocalizedTrace> {
		self.extras.block_hash(block_number)
			.and_then(|block_hash| self.transactions_traces(&block_hash)
				.and_then(|traces| traces.into_iter().nth(tx_position))
				.map(Into::<Vec<FlatTrace>>::into)
				.and_then(|traces| traces.into_iter().nth(trace_position))
				.map(|trace| {
					let tx_hash = self.extras.transaction_hash(block_number, tx_position)
						.expect("Expected to find transaction hash. Database is probably malformed");

					LocalizedTrace {
						parent: trace.parent,
						children: trace.children,
						depth: trace.depth,
						action: trace.action,
						result: trace.result,
						trace_number: trace_position,
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
						.expect("Expected to find transaction hash. Database is probably malformed");

					traces.into_iter()
					.enumerate()
					.map(|(i, trace)| LocalizedTrace {
						parent: trace.parent,
						children: trace.children,
						depth: trace.depth,
						action: trace.action,
						result: trace.result,
						trace_number: i,
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
								.expect("Expected to find transaction hash. Database is probably malformed");

							traces.into_iter()
								.enumerate()
								.map(|(i, trace)| LocalizedTrace {
									parent: trace.parent,
									children: trace.children,
									depth: trace.depth,
									action: trace.action,
									result: trace.result,
									trace_number: i,
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
		let chain = BloomGroupChain::new(self.config.blooms, self);
		let numbers = chain.filter(filter);
		numbers.into_iter()
			.flat_map(|n| {
				let number = n as BlockNumber;
				let hash = self.extras.block_hash(number)
					.expect("Expected to find block hash. Extras db is probably malformed");
				let traces = self.traces(&hash)
					.expect("Expected to find a trace. Db is probably malformed.");
				let flat_block = FlatBlockTraces::from(traces);
				self.matching_block_traces(filter, flat_block, hash, number)
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::sync::Arc;
	use util::{Address, U256, H256};
	use devtools::RandomTempPath;
	use header::BlockNumber;
	use trace::{Config, Tracedb, Database, DatabaseExtras, ImportRequest};
	use trace::{BlockTraces, Trace, Filter, LocalizedTrace, AddressesFilter};
	use trace::trace::{Call, Action, Res};

	struct NoopExtras;

	impl DatabaseExtras for NoopExtras {
		fn block_hash(&self, _block_number: BlockNumber) -> Option<H256> {
			unimplemented!();
		}

		fn transaction_hash(&self, _block_number: BlockNumber, _tx_position: usize) -> Option<H256> {
			unimplemented!();
		}
	}

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
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set autotracing
		config.enabled = None;

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), false);
		}

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), false);
		}

		config.enabled = Some(false);

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), false);
		}
	}

	#[test]
	fn test_reopeining_db_with_tracing_on() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set tracing on
		config.enabled = Some(true);

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = None;

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = Some(false);

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), false);
		}
	}

	#[test]
	#[should_panic]
	fn test_invalid_reopening_db() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set tracing on
		config.enabled = Some(false);

		{
			let tracedb = Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras));
			assert_eq!(tracedb.tracing_enabled(), true);
		}

		config.enabled = Some(true);
		Tracedb::new(config.clone(), temp.as_path(), Arc::new(NoopExtras)); // should panic!
	}

	fn create_simple_import_request(block_number: BlockNumber, block_hash: H256) -> ImportRequest {
		ImportRequest {
			traces: BlockTraces::from(vec![Trace {
				depth: 0,
				action: Action::Call(Call {
					from: Address::from(1),
					to: Address::from(2),
					value: U256::from(3),
					gas: U256::from(4),
					input: vec![],
				}),
				result: Res::FailedCall,
				subs: vec![],
			}]),
			block_hash: block_hash.clone(),
			block_number: block_number,
			enacted: vec![block_hash],
			retracted: 0,
		}
	}

	fn create_simple_localized_trace(block_number: BlockNumber, block_hash: H256, tx_hash: H256) -> LocalizedTrace {
		LocalizedTrace {
			parent: None,
			children: vec![],
			depth: 0,
			action: Action::Call(Call {
				from: Address::from(1),
				to: Address::from(2),
				value: U256::from(3),
				gas: U256::from(4),
				input: vec![],
			}),
			result: Res::FailedCall,
			trace_number: 0,
			transaction_number: 0,
			transaction_hash: tx_hash,
			block_number: block_number,
			block_hash: block_hash,
		}
	}


	#[test]
	fn test_import() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();
		config.enabled = Some(true);
		let block_0 = H256::from(0xa1);
		let block_1 = H256::from(0xa2);
		let tx_0 = H256::from(0xff);
		let tx_1 = H256::from(0xaf);

		let mut extras = Extras::default();
		extras.block_hashes.insert(0, block_0.clone());
		extras.block_hashes.insert(1, block_1.clone());
		extras.transaction_hashes.insert(0, vec![tx_0.clone()]);
		extras.transaction_hashes.insert(1, vec![tx_1.clone()]);

		let tracedb = Tracedb::new(config, temp.as_path(), Arc::new(extras));

		// import block 0
		let request = create_simple_import_request(0, block_0.clone());
		tracedb.import(request);

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
		tracedb.import(request);

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
	}
}
