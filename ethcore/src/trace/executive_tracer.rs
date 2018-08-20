// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::collections::VecDeque;
use ethereum_types::{U256, Address};
use vm::{Error as VmError, ActionParams};
use trace::trace::{Call, Create, Action, Res, CreateResult, CallResult, VMTrace, VMOperation, VMExecutedOperation, MemoryDiff, StorageDiff, Suicide, Reward, RewardType};
use trace::{Tracer, VMTracer, FlatTrace, TraceError};

fn top_level_subtraces(traces: &[FlatTrace]) -> usize {
	traces.iter().filter(|t| t.trace_address.is_empty()).count()
}

fn prefix_subtrace_addresses(mut traces: &mut [FlatTrace]) {
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
	for trace in traces {
		match (first, trace.trace_address.is_empty()) {
			(true, _) => first = false,
			(_, true) => current_subtrace_index += 1,
			_ => {}
		}
		trace.trace_address.push_front(current_subtrace_index);
	}
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

/// Simple executive tracer. Traces all calls and creates. Ignores delegatecalls.
#[derive(Default)]
pub struct ExecutiveTracer {
	traces: Vec<FlatTrace>,
	index_deque: VecDeque<usize>,
	vecindex_stack: Vec<usize>,
	sublen_stack: Vec<usize>,
}

impl Tracer for ExecutiveTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams) {
		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			trace_address: self.index_deque.clone(),
			subtraces: self.sublen_stack.last().cloned().unwrap_or(0),
			action: Action::Call(Call::from(params.clone())),
			result: Res::Call(CallResult {
				gas_used: U256::zero(),
				output: Vec::new()
			}),
		};
		self.vecindex_stack.push(self.traces.len());
		self.traces.push(trace);
		self.index_deque.push_front(0);
		self.sublen_stack.push(0);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			trace_address: self.index_deque.clone(),
			subtraces: self.sublen_stack.last().cloned().unwrap_or(0),
			action: Action::Create(Create::from(params.clone())),
			result: Res::Create(CreateResult {
				gas_used: U256::zero(),
				code: Vec::new(),
				address: Address::default(),
			}),
		};
		self.vecindex_stack.push(self.traces.len());
		self.traces.push(trace);
		self.index_deque.push_front(0);
		self.sublen_stack.push(0);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		let vecindex = self.vecindex_stack.pop().expect("prepare/done_trace are not balanced");
		let sublen = self.sublen_stack.pop().expect("prepare/done_trace are not balanced");
		self.index_deque.pop_front();

		self.traces[vecindex].result = Res::Call(CallResult {
			gas_used,
			output: output.into(),
		});
		self.traces[vecindex].subtraces = sublen;

		if let Some(index) = self.index_deque.front_mut() {
			*index += 1;
		}
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		let vecindex = self.vecindex_stack.pop().expect("prepare/done_trace are not balanced");
		let sublen = self.sublen_stack.pop().expect("prepare/done_trace are not balanced");
		self.index_deque.pop_front();

		self.traces[vecindex].result = Res::Create(CreateResult {
			gas_used, address,
			code: code.into(),
		});
		self.traces[vecindex].subtraces = sublen;

		if let Some(index) = self.index_deque.front_mut() {
			*index += 1;
		}
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		let vecindex = self.vecindex_stack.pop().expect("prepare/done_trace are not balanced");
		let sublen = self.sublen_stack.pop().expect("prepare/done_trace are not balanced");
		self.index_deque.pop_front();

		let is_create = match self.traces[vecindex].action {
			Action::Create(_) => true,
			_ => false,
		};

		if is_create {
			self.traces[vecindex].result = Res::FailedCreate(error.into());
		} else {
			self.traces[vecindex].result = Res::FailedCall(error.into());
		}
		self.traces[vecindex].subtraces = sublen;

		if let Some(index) = self.index_deque.front_mut() {
			*index += 1;
		}
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			subtraces: 0,
			action: Action::Suicide(Suicide { address, refund_address, balance } ),
			result: Res::None,
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced suicide {:?}", trace);
		self.traces.push(trace);

		if let Some(index) = self.index_deque.front_mut() {
			*index += 1;
		}
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			subtraces: 0,
			action: Action::Reward(Reward { author, value, reward_type } ),
			result: Res::None,
			trace_address: Default::default(),
		};
		debug!(target: "trace", "Traced reward {:?}", trace);
		self.traces.push(trace);

		if let Some(index) = self.index_deque.front_mut() {
			*index += 1;
		}
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
	type Output = VMTrace;

	fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8, _current_gas: U256) -> bool { true }

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
