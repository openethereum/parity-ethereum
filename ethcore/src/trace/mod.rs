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

pub mod trace;
mod filter;
mod noop_tracer;
mod executive_tracer;

pub use self::trace::{Trace, Call, Create, TraceAction};
pub use self::filter::Filter;
pub use self::noop_tracer::NoopTracer;
pub use self::executive_tracer::ExecutiveTracer;
use util::{Bytes, Address, U256};
use action_params::ActionParams;

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
