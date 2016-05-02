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

//! Tracing

mod block;
mod bloom;
mod config;
mod db;
mod executive_tracer;
mod filter;
mod flat;
mod import;
mod localized;
mod noop_tracer;
pub mod trace;

pub use self::block::BlockTraces;
pub use self::config::{Config, Switch};
pub use self::db::TraceDB;
pub use self::trace::Trace;
pub use self::noop_tracer::NoopTracer;
pub use self::executive_tracer::ExecutiveTracer;
pub use self::filter::{Filter, AddressesFilter};
pub use self::import::ImportRequest;
pub use self::localized::LocalizedTrace;
use util::{Bytes, Address, U256, H256};
use self::trace::{Call, Create};
use action_params::ActionParams;
use header::BlockNumber;

/// This trait is used by executive to build traces.
pub trait Tracer: Send {
	/// Prepares call trace for given params. Noop tracer should return None.
	fn prepare_trace_call(&self, params: &ActionParams) -> Option<Call>;

	/// Prepares create trace for given params. Noop tracer should return None.
	fn prepare_trace_create(&self, params: &ActionParams) -> Option<Create>;

	/// Prepare trace output. Noop tracer should return None.
	fn prepare_trace_output(&self) -> Option<Bytes>;

	/// Stores trace call info.
	fn trace_call(
		&mut self,
		call: Option<Call>,
		gas_used: U256,
		output: Option<Bytes>,
		depth: usize,
		subs: Vec<Trace>,
		delegate_call: bool
	);

	/// Stores trace create info.
	fn trace_create(
		&mut self,
		create: Option<Create>,
		gas_used: U256,
		code: Option<Bytes>,
		address: Address,
		depth: usize,
		subs: Vec<Trace>
	);

	/// Stores failed call trace.
	fn trace_failed_call(&mut self, call: Option<Call>, depth: usize, subs: Vec<Trace>, delegate_call: bool);

	/// Stores failed create trace.
	fn trace_failed_create(&mut self, create: Option<Create>, depth: usize, subs: Vec<Trace>);

	/// Spawn subracer which will be used to trace deeper levels of execution.
	fn subtracer(&self) -> Self where Self: Sized;

	/// Consumes self and returns all traces.
	fn traces(self) -> Vec<Trace>;
}

/// `DbExtras` provides an interface to query extra data which is not stored in tracesdb,
/// but necessary to work correctly.
pub trait DatabaseExtras {
	/// Returns hash of given block number.
	fn block_hash(&self, block_number: BlockNumber) -> Option<H256>;

	/// Returns hash of transaction at given position.
	fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256>;
}

/// Db provides an interface to query tracesdb.
pub trait Database {
	/// Returns true if tracing is enabled. Otherwise false.
	fn tracing_enabled(&self) -> bool;

	/// Imports new block traces.
	fn import(&self, request: ImportRequest);

	/// Returns localized trace at given position.
	fn trace(&self, block_number: BlockNumber, tx_position: usize, trace_position: Vec<usize>) -> Option<LocalizedTrace>;

	/// Returns localized traces created by a single transaction.
	fn transaction_traces(&self, block_number: BlockNumber, tx_position: usize) -> Option<Vec<LocalizedTrace>>;

	/// Returns localized traces created in given block.
	fn block_traces(&self, block_number: BlockNumber) -> Option<Vec<LocalizedTrace>>;

	/// Filter traces matching given filter.
	fn filter(&self, filter: &Filter) -> Vec<LocalizedTrace>;
}
