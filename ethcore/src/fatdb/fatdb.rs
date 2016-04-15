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
use bloomchain::{Config, Number};
use bloomchain::group::{BloomGroupDatabase, BloomGroupChain, GroupPosition, BloomGroup};
use util::{H256, Database};
use header::BlockNumber;
use trace::Trace;
use blockchain::BlockProvider;
use basic_types::LogBloom;
use super::trace::{Filter, TraceGroupPosition, TraceBloom, TraceBloomGroup};

/// Fat database.
struct Fatdb {
	// cache
	traces: RwLock<HashMap<BlockNumber, Vec<Trace>>>,
	blooms: RwLock<HashMap<TraceGroupPosition, TraceBloomGroup>>,
	// db
	db: Database,
	// config,
	bloom_config: Config,
}

impl BloomGroupDatabase for Fatdb {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		// TODO: look for blooms in db
		self.blooms.read()
			.unwrap()
			.get(&TraceGroupPosition::from(position.clone()))
			.cloned()
			.map(Into::into)
	}
}

impl Fatdb {
	/// Creates new instance of `Fatdb`.
	pub fn new(path: &Path) -> Self {
		let mut fatdb_path = path.to_path_buf();
		fatdb_path.push("fatdb");
		let fatdb = Database::open_default(fatdb_path.to_str().unwrap()).unwrap();

		Fatdb {
			traces: RwLock::new(HashMap::new()),
			blooms: RwLock::new(HashMap::new()),
			db: fatdb,
			bloom_config: Config {
				levels: 3,
				elements_per_index: 16
			},
		}
	}

	/// Inserts new trace to database.
	pub fn insert_traces(&self, number: BlockNumber, traces: Vec<Trace>) {
		let modified_blooms = {
			let chain = BloomGroupChain::new(self.bloom_config, self);
			let bloom = traces.iter()
				.fold(LogBloom::default(), |acc, trace| acc | trace.bloom());
			let trace_bloom = TraceBloom::from(bloom);
			chain.insert(number as Number, trace_bloom.into())
		};

		let trace_blooms = modified_blooms
			.into_iter()
			.map(|p| (From::from(p.0), From::from(p.1)))
			.collect::<HashMap<TraceGroupPosition, TraceBloomGroup>>();

		self.traces.write().unwrap().insert(number, traces);
		self.blooms.write().unwrap().extend(trace_blooms);
		// TODO: insert them to db.
	}

	/// Returns traces at block with given number.
	pub fn traces(&self, block_number: BlockNumber) -> Option<Vec<Trace>> {
		// TODO: look for traces in db
		self.traces.read().unwrap().get(&block_number).cloned()
	}

	/// Returns traces matching given filter.
	pub fn filter_traces(&self, filter: &Filter) -> Vec<Trace> {
		let numbers = {
			let chain = BloomGroupChain::new(self.bloom_config, self);
			chain.filter(filter)
		};

		numbers.into_iter()
			.filter_map(|block_number| self.traces(block_number as BlockNumber))
			.flat_map(|trace| trace)
			.filter(|trace| filter.matches(trace))
			.collect()
	}
}
