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
use std::sync::RwLock;
use std::path::Path;
use bloomchain::Number;
use bloomchain::group::{BloomGroupDatabase, BloomGroupChain, GroupPosition, BloomGroup};
use blockchain::ImportRoute;
use util::{FixedHash, H256, H264, Database, DBTransaction};
use header::BlockNumber;
use trace::BlockTraces;
use db::{Key, Writable, Readable, BatchWriter, DatabaseReader, CacheUpdatePolicy};
use super::Config;
use super::trace::{Filter, TraceGroupPosition, BlockTracesBloom, BlockTracesBloomGroup, BlockTracesDetails};

#[derive(Debug, Copy, Clone)]
pub enum FatdbIndex {
	/// Block traces index.
	BlockTraces = 0,
	/// Trace bloom group index.
	BlockTracesBloomGroups = 1,
}

fn with_index(hash: &H256, i: FatdbIndex) -> H264 {
	let mut slice = H264::from_slice(hash);
	slice[32] = i as u8;
	slice
}

impl Key<BlockTraces> for H256 {
	fn key(&self) -> H264 {
		with_index(self, FatdbIndex::BlockTraces)
	}
}

impl Key<BlockTracesBloomGroup> for TraceGroupPosition {
	fn key(&self) -> H264 {
		with_index(&self.hash(), FatdbIndex::BlockTracesBloomGroups)
	}
}

/// Fat database.
pub struct Fatdb {
	// cache
	traces: RwLock<HashMap<H256, BlockTraces>>,
	blooms: RwLock<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>,
	// db
	db: Database,
	// config,
	config: Config,
}

impl BloomGroupDatabase for Fatdb {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		let position = TraceGroupPosition::from(position.clone());
		DatabaseReader::new(&self.db, &self.blooms)
			.read(&position)
			.map(Into::into)
	}
}

impl Fatdb {
	/// Creates new instance of `Fatdb`.
	pub fn new(config: Config, path: &Path) -> Self {
		let mut fatdb_path = path.to_path_buf();
		fatdb_path.push("fatdb");
		let fatdb = Database::open_default(fatdb_path.to_str().unwrap()).unwrap();

		Fatdb {
			traces: RwLock::new(HashMap::new()),
			blooms: RwLock::new(HashMap::new()),
			db: fatdb,
			config: config,
		}
	}

	/// Returns true if trasing is enabled. Otherwise false.
	pub fn is_tracing_enabled(&self) -> bool {
		self.config.tracing.enabled
	}

	/// Imports new block traces and rebuilds the blooms based on the import route.
	///
	/// Traces of all import route's enacted blocks are expected to be already in database
	/// or to be the currenly inserted trace.
	pub fn import_traces(&self, details: BlockTracesDetails, route: &ImportRoute) {
		// fast return if tracing is disabled
		if !self.is_tracing_enabled() {
			return;
		}
		// in real world it's impossible to get import route with retracted blocks
		// and no enacted blocks and this function does not handle this case
		// so let's just panic.
		assert!(! (!route.retracted.is_empty() && route.enacted.is_empty() ), "Invalid import route!");

		let batch = DBTransaction::new();

		// at first, let's insert new block traces
		{
			let mut traces = self.traces.write().unwrap();
			// it's important to use overwrite here,
			// cause this value might be queried by hash later
			BatchWriter::new(&batch, &mut traces).write(details.hash, details.traces, CacheUpdatePolicy::Overwrite);
		}

		// now let's rebuild the blooms
		// do it only if some blocks have been enacted
		if !route.enacted.is_empty() {
			let replaced_range = details.number as Number - route.retracted.len()..details.number as Number;
			let enacted_blooms = route.enacted
				.iter()
				// all traces are expected to be found here. That's why `expect` has been used
				// instead of `filter_map`. If some traces haven't been found, it meens that
				// traces database is malformed or incomplete.
				.map(|block_hash| self.traces(block_hash).expect("Traces database is incomplete."))
				.map(|block_traces| block_traces.bloom())
				.map(BlockTracesBloom::from)
				.map(Into::into)
				.collect();

			let chain = BloomGroupChain::new(self.config.tracing.blooms, self);
			let trace_blooms = chain.replace(&replaced_range, enacted_blooms);
			let blooms_to_insert = trace_blooms.into_iter()
				.map(|p| (From::from(p.0), From::from(p.1)))
				.collect::<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>();

			let mut blooms = self.blooms.write().unwrap();
			BatchWriter::new(&batch, &mut blooms).extend(blooms_to_insert, CacheUpdatePolicy::Remove);
		}

		self.db.write(batch).unwrap();
	}

	/// Returns traces for block with hash.
	pub fn traces(&self, block_hash: &H256) -> Option<BlockTraces> {
		DatabaseReader::new(&self.db, &self.traces).read(block_hash)
	}

	/// Returns traces matching given filter.
	pub fn filter_traces(&self, filter: &Filter) -> Vec<BlockNumber> {
		let chain = BloomGroupChain::new(self.config.tracing.blooms, self);
		let numbers = chain.filter(filter);
		numbers.into_iter()
			.map(|n| n as BlockNumber)
			.collect()
	}
}
