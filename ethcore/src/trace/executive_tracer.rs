// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Simple executive tracer.

use bigint::prelude::U256;
use util::Address;
use bytes::Bytes;
use vm::ActionParams;
use trace::trace::{Call, Create, Action, Res, CreateResult, CallResult, VMTrace, VMOperation, VMExecutedOperation, MemoryDiff, StorageDiff, Suicide, Reward, RewardType};
use trace::{Tracer, VMTracer, FlatTrace, TraceError};

/// Simple executive tracer. Traces all calls and creates. Ignores delegatecalls.
#[derive(Default)]
pub struct ExecutiveTracer {
	traces: Vec<FlatTrace>,
}

fn top_level_subtraces(traces: &[FlatTrace]) -> usize {
	traces.iter().filter(|t| t.trace_address.is_empty()).count()
}

fn prefix_subtrace_addresses(mut traces: Vec<FlatTrace>) -> Vec<FlatTrace> {
	// input traces are expected to be ordered like
	// []
	// [0]
	// [0, 0]
	// [0, 1]
	// []
	// [0]
	//
	// so they can be transformed to
	//
	// [0]
	// [0, 0]
	// [0, 0, 0]
	// [0, 0, 1]
	// [1]
	// [1, 0]
	let mut current_subtrace_index = 0;
	let mut first = true;
	for trace in &mut traces {
		match (first, trace.trace_address.is_empty()) {
			(true, _) => first = false,
			(_, true) => current_subtrace_index += 1,
			_ => {}
		}
		trace.trace_address.push_front(current_subtrace_index);
	}
	traces
}

#[test]
fn should_prefix_address_properly() {
	use super::trace::{Action, Res, Suicide};

	let f = |v: Vec<usize>| FlatTrace {
		action: Action::Suicide(Suicide {
			address: Default::default(),
			balance: Default::default(),
			refund_address: Default::default(),
		}),
		result: Res::None,
		subtraces: 0,
		trace_address: v.into_iter().collect(),
	};
	let t = vec![vec![], vec![0], vec![0, 0], vec![0], vec![], vec![], vec![0], vec![]].into_iter().map(&f).collect();
	let t = prefix_subtrace_addresses(t);
	assert_eq!(t, vec![vec![0], vec![0, 0], vec![0, 0, 0], vec![0, 0], vec![1], vec![2], vec![2, 0], vec![3]].into_iter().map(&f).collect::<Vec<_>>());
}

impl Tracer for ExecutiveTracer {
	fn prepare_trace_call(&self, params: &ActionParams) -> Option<Call> {
		Some(Call::from(params.clone()))
	}

	fn prepare_trace_create(&self, params: &ActionParams) -> Option<Create> {
		Some(Create::from(params.clone()))
	}

	fn prepare_trace_output(&self) -> Option<Bytes> {
		Some(vec![])
	}

	fn trace_call(&mut self, call: Option<Call>, gas_used: U256, output: Option<Bytes>, subs: Vec<FlatTrace>) {
		let trace = FlatTrace {
			trace_address: Default::default(),
			subtraces: top_level_subtraces(&subs),
			action: Action::Call(call.expect("self.prepare_trace_call().is_some(): so we must be tracing: qed")),
			result: Res::Call(CallResult {
				gas_used: gas_used,
				output: output.expect("self.prepare_trace_output().is_some(): so we must be tracing: qed")
			}),
		};
		debug!(target: "trace", "Traced call {:?}", trace);
		self.traces.push(trace);
		self.traces.extend(prefix_subtrace_addresses(subs));
	}

	fn trace_create(&mut self, create: Option<Create>, gas_used: U256, code: Option<Bytes>, address: Address, subs: Vec<FlatTrace>) {
		let trace = FlatTrace {
			subtraces: top_level_subtraces(&subs),
			action: Action::Create(create.expect("self.prepare_trace_create().is_some(): so we must be tracing: qed")),
			result: Res::Create(CreateResult {
				gas_used: gas_used,
				code: code.expect("self.prepare_trace_output.is_some(): so we must be tracing: qed"),
				address: address
			}),
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced create {:?}", trace);
		self.traces.push(trace);
		self.traces.extend(prefix_subtrace_addresses(subs));
	}

	fn trace_failed_call(&mut self, call: Option<Call>, subs: Vec<FlatTrace>, error: TraceError) {
		let trace = FlatTrace {
			trace_address: Default::default(),
			subtraces: top_level_subtraces(&subs),
			action: Action::Call(call.expect("self.prepare_trace_call().is_some(): so we must be tracing: qed")),
			result: Res::FailedCall(error),
		};
		debug!(target: "trace", "Traced failed call {:?}", trace);
		self.traces.push(trace);
		self.traces.extend(prefix_subtrace_addresses(subs));
	}

	fn trace_failed_create(&mut self, create: Option<Create>, subs: Vec<FlatTrace>, error: TraceError) {
		let trace = FlatTrace {
			subtraces: top_level_subtraces(&subs),
			action: Action::Create(create.expect("self.prepare_trace_create().is_some(): so we must be tracing: qed")),
			result: Res::FailedCreate(error),
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced failed create {:?}", trace);
		self.traces.push(trace);
		self.traces.extend(prefix_subtrace_addresses(subs));
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		let trace = FlatTrace {
			subtraces: 0,
			action: Action::Suicide(Suicide { address, refund_address, balance } ),
			result: Res::None,
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced suicide {:?}", trace);
		self.traces.push(trace);
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		let trace = FlatTrace {
			subtraces: 0,
			action: Action::Reward(Reward { author, value, reward_type } ),
			result: Res::None,
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced reward {:?}", trace);
		self.traces.push(trace);
	}

	fn subtracer(&self) -> Self {
		ExecutiveTracer::default()
	}

	fn drain(self) -> Vec<FlatTrace> {
		self.traces
	}
}

/// Simple VM tracer. Traces all operations.
pub struct ExecutiveVMTracer {
	data: VMTrace,
}

impl ExecutiveVMTracer {
	/// Create a new top-level instance.
	pub fn toplevel() -> Self {
		ExecutiveVMTracer {
			data: VMTrace {
				parent_step: 0,
				code: vec![],
				operations: vec![Default::default()],	// prefill with a single entry so that prepare_subtrace can get the parent_step
				subs: vec![],
			}
		}
	}
}

impl VMTracer for ExecutiveVMTracer {
	fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8) -> bool { true }

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256) {
		self.data.operations.push(VMOperation {
			pc: pc,
			instruction: instruction,
			gas_cost: gas_cost,
			executed: None,
		});
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		let ex = VMExecutedOperation {
			gas_used: gas_used,
			stack_push: stack_push.iter().cloned().collect(),
			mem_diff: mem_diff.map(|(s, r)| MemoryDiff{ offset: s, data: r.iter().cloned().collect() }),
			store_diff: store_diff.map(|(l, v)| StorageDiff{ location: l, value: v }),
		};
		self.data.operations.last_mut().expect("trace_executed is always called after a trace_prepare_execute").executed = Some(ex);
	}

	fn prepare_subtrace(&self, code: &[u8]) -> Self {
		ExecutiveVMTracer { data: VMTrace {
			parent_step: self.data.operations.len() - 1,	// won't overflow since we must already have pushed an operation in trace_prepare_execute.
			code: code.to_vec(),
			operations: vec![],
			subs: vec![],
		}}
	}

	fn done_subtrace(&mut self, sub: Self) {
		self.data.subs.push(sub.data);
	}

	fn drain(mut self) -> Option<VMTrace> { self.data.subs.pop() }
}
