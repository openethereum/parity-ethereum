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

mod trace;

pub use self::trace::*;
use util::bytes::Bytes;
use util::hash::Address;
use util::numbers::U256;
use action_params::ActionParams;

/// This trait is used by executive to build traces.
pub trait Tracer: Send {
	/// Prepares call trace for given params. Noop tracer should return None.
	fn prepare_trace_call(&self, params: &ActionParams) -> Option<TraceCall>;

	/// Prepares create trace for given params. Noop tracer should return None.
	fn prepare_trace_create(&self, params: &ActionParams) -> Option<TraceCreate>;

	/// Prepare trace output. Noop tracer should return None.
	fn prepare_trace_output(&self) -> Option<Bytes>;

	/// Stores trace call info.
	fn trace_call(
		&mut self,
		call: Option<TraceCall>,
		gas_used: U256,
		output: Option<Bytes>,
		depth: usize,
		subs: Vec<Trace>,
		delegate_call: bool
	);

	/// Stores trace create info.
	fn trace_create(
		&mut self,
		create: Option<TraceCreate>,
		gas_used: U256,
		code: Option<Bytes>,
		address: Address,
		depth: usize,
		subs: Vec<Trace>
	);

	/// Stores failed call trace.
	fn trace_failed_call(&mut self, call: Option<TraceCall>, depth: usize, subs: Vec<Trace>, delegate_call: bool);

	/// Stores failed create trace.
	fn trace_failed_create(&mut self, create: Option<TraceCreate>, depth: usize, subs: Vec<Trace>);

	/// Spawn subracer which will be used to trace deeper levels of execution.
	fn subtracer(&self) -> Self where Self: Sized;

	/// Consumes self and returns all traces.
	fn traces(self) -> Vec<Trace>;
}

/// Nonoperative tracer. Does not trace anything.
pub struct NoopTracer;

impl Tracer for NoopTracer {
	fn prepare_trace_call(&self, _: &ActionParams) -> Option<TraceCall> {
		None
	}

	fn prepare_trace_create(&self, _: &ActionParams) -> Option<TraceCreate> {
		None
	}

	fn prepare_trace_output(&self) -> Option<Bytes> {
		None
	}

	fn trace_call(&mut self, call: Option<TraceCall>, _: U256, output: Option<Bytes>, _: usize, _: Vec<Trace>,
				  _: bool) {
		assert!(call.is_none());
		assert!(output.is_none());
	}

	fn trace_create(&mut self, create: Option<TraceCreate>, _: U256, code: Option<Bytes>, _: Address, _: usize, _: Vec<Trace>) {
		assert!(create.is_none());
		assert!(code.is_none());
	}

	fn trace_failed_call(&mut self, call: Option<TraceCall>, _: usize, _: Vec<Trace>, _: bool) {
		assert!(call.is_none());
	}

	fn trace_failed_create(&mut self, create: Option<TraceCreate>, _: usize, _: Vec<Trace>) {
		assert!(create.is_none());
	}

	fn subtracer(&self) -> Self {
		NoopTracer
	}

	fn traces(self) -> Vec<Trace> {
		vec![]
	}
}

/// Simple executive tracer. Traces all calls and creates. Ignores delegatecalls.
#[derive(Default)]
pub struct ExecutiveTracer {
	traces: Vec<Trace>
}

impl Tracer for ExecutiveTracer {
	fn prepare_trace_call(&self, params: &ActionParams) -> Option<TraceCall> {
		Some(TraceCall::from(params.clone()))
	}

	fn prepare_trace_create(&self, params: &ActionParams) -> Option<TraceCreate> {
		Some(TraceCreate::from(params.clone()))
	}

	fn prepare_trace_output(&self) -> Option<Bytes> {
		Some(vec![])
	}

	fn trace_call(&mut self, call: Option<TraceCall>, gas_used: U256, output: Option<Bytes>, depth: usize, subs:
				  Vec<Trace>, delegate_call: bool) {
		// don't trace if it's DELEGATECALL or CALLCODE.
		if delegate_call {
			return;
		}

		let trace = Trace {
			depth: depth,
			subs: subs,
			action: TraceAction::Call(call.expect("Trace call expected to be Some.")),
			result: TraceResult::Call(TraceCallResult {
				gas_used: gas_used,
				output: output.expect("Trace call output expected to be Some.")
			})
		};
		self.traces.push(trace);
	}

	fn trace_create(&mut self, create: Option<TraceCreate>, gas_used: U256, code: Option<Bytes>, address: Address, depth: usize, subs: Vec<Trace>) {
		let trace = Trace {
			depth: depth,
			subs: subs,
			action: TraceAction::Create(create.expect("Trace create expected to be Some.")),
			result: TraceResult::Create(TraceCreateResult {
				gas_used: gas_used,
				code: code.expect("Trace create code expected to be Some."),
				address: address
			})
		};
		self.traces.push(trace);
	}

	fn trace_failed_call(&mut self, call: Option<TraceCall>, depth: usize, subs: Vec<Trace>, delegate_call: bool) {
		// don't trace if it's DELEGATECALL or CALLCODE.
		if delegate_call {
			return;
		}

		let trace = Trace {
			depth: depth,
			subs: subs,
			action: TraceAction::Call(call.expect("Trace call expected to be Some.")),
			result: TraceResult::FailedCall,
		};
		self.traces.push(trace);
	}

	fn trace_failed_create(&mut self, create: Option<TraceCreate>, depth: usize, subs: Vec<Trace>) {
		let trace = Trace {
			depth: depth,
			subs: subs,
			action: TraceAction::Create(create.expect("Trace create expected to be Some.")),
			result: TraceResult::FailedCreate,
		};
		self.traces.push(trace);
	}

	fn subtracer(&self) -> Self {
		ExecutiveTracer::default()
	}

	fn traces(self) -> Vec<Trace> {
		self.traces
	}
}
