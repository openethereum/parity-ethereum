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
use super::trace::{Filter, TraceGroupPosition, TraceBloom, TraceBloomGroup};

/// Fat database.
struct Fatdb {
	// cache
	traces: RwLock<HashMap<H256, Trace>>,
	cache: RwLock<HashMap<TraceGroupPosition, TraceBloomGroup>>,
	// db
	traces_db: Database,
	// config,
	bloom_config: Config,
}

impl BloomGroupDatabase for Fatdb {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		unimplemented!();
	}
}

impl Fatdb {
	/// Creates new instance of `Fatdb`.
	pub fn new(path: &Path) -> Fatdb {
		unimplemented!();
	}

	/// Inserts new trace to database.
	pub fn insert_trace(&self, number: BlockNumber, trace: Trace) {
		let modified_blooms = {
			let chain = BloomGroupChain::new(self.bloom_config, self);
			let trace_bloom = TraceBloom::from(trace.bloom());
			chain.insert(number as Number, trace_bloom.into())
		};

		let trace_blooms = modified_blooms
			.into_iter()
			.map(|p| (From::from(p.0), From::from(p.1)))
			.collect::<HashMap<TraceGroupPosition, TraceBloomGroup>>();
		unimplemented!();
	}

	/// Returns traces at block with given number.
	pub fn traces(&self, block_number: BlockNumber) -> Vec<Trace> {
		unimplemented!();
	}

	/// Returns traces matching given filter.
	pub fn filter_traces(&self, filter: &Filter) -> Vec<Trace> {
		let numbers = {
			let chain = BloomGroupChain::new(self.bloom_config, self);
			chain.filter(filter)
		};

		numbers.into_iter()
			.flat_map(|block_number| self.traces(block_number as BlockNumber))
			.filter(|trace| filter.matches(trace))
			.collect()
	}
}
