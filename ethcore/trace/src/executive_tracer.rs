// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Simple executive tracer.

use std::cmp::min;
use ethereum_types::{U256, Address};
use vm::{Error as VmError, ActionParams};
use log::{debug, warn};
use crate::{
	Tracer, VMTracer, FlatTrace,
	trace::{Call, Create, Action, Res, CreateResult, CallResult, VMTrace, VMOperation, VMExecutedOperation, MemoryDiff, StorageDiff, Suicide, Reward, RewardType},
};

/// Simple executive tracer. Traces all calls and creates. Ignores delegatecalls.
#[derive(Default)]
pub struct ExecutiveTracer {
	traces: Vec<FlatTrace>,
	index_stack: Vec<usize>,
	vecindex_stack: Vec<usize>,
	sublen_stack: Vec<usize>,
	skip_one: bool,
}

impl Tracer for ExecutiveTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams, depth: usize, is_builtin: bool) {
		assert!(!self.skip_one, "skip_one is used only for builtin contracts that do not have subsequent calls; in prepare_trace_call it cannot be true; qed");

		if depth != 0 && is_builtin && params.value.value() == U256::zero() {
			self.skip_one = true;
			return;
		}

		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			trace_address: self.index_stack.clone(),
			subtraces: self.sublen_stack.last().cloned().unwrap_or(0),
			action: Action::Call(Call::from(params.clone())),
			result: Res::Call(CallResult {
				gas_used: U256::zero(),
				output: Vec::new()
			}),
		};
		self.vecindex_stack.push(self.traces.len());
		self.traces.push(trace);
		self.index_stack.push(0);
		self.sublen_stack.push(0);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		assert!(!self.skip_one, "skip_one is used only for builtin contracts that do not have subsequent calls; in prepare_trace_create it cannot be true; qed");

		if let Some(parentlen) = self.sublen_stack.last_mut() {
			*parentlen += 1;
		}

		let trace = FlatTrace {
			trace_address: self.index_stack.clone(),
			subtraces: self.sublen_stack.last().cloned().unwrap_or(0),
			action: Action::Create(Create::from(params.clone())),
			result: Res::Create(CreateResult {
				gas_used: U256::zero(),
				code: Vec::new(),
				address: Address::zero(),
			}),
		};
		self.vecindex_stack.push(self.traces.len());
		self.traces.push(trace);
		self.index_stack.push(0);
		self.sublen_stack.push(0);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		if self.skip_one {
			self.skip_one = false;
			return;
		}

		let vecindex = self.vecindex_stack.pop().expect("Executive invoked prepare_trace_call before this function; vecindex_stack is never empty; qed");
		let sublen = self.sublen_stack.pop().expect("Executive invoked prepare_trace_call before this function; sublen_stack is never empty; qed");
		self.index_stack.pop();

		self.traces[vecindex].result = Res::Call(CallResult {
			gas_used,
			output: output.into(),
		});
		self.traces[vecindex].subtraces = sublen;

		if let Some(index) = self.index_stack.last_mut() {
			*index += 1;
		}
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		assert!(!self.skip_one, "skip_one is only set with prepare_trace_call for builtin contracts with no subsequent calls; skip_one cannot be true after the same level prepare_trace_create; qed");

		let vecindex = self.vecindex_stack.pop().expect("Executive invoked prepare_trace_create before this function; vecindex_stack is never empty; qed");
		let sublen = self.sublen_stack.pop().expect("Executive invoked prepare_trace_create before this function; sublen_stack is never empty; qed");
		self.index_stack.pop();

		self.traces[vecindex].result = Res::Create(CreateResult {
			gas_used, address,
			code: code.into(),
		});
		self.traces[vecindex].subtraces = sublen;

		if let Some(index) = self.index_stack.last_mut() {
			*index += 1;
		}
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		if self.skip_one {
			self.skip_one = false;
			return;
		}

		let vecindex = self.vecindex_stack.pop().expect("Executive invoked prepare_trace_create/call before this function; vecindex_stack is never empty; qed");
		let sublen = self.sublen_stack.pop().expect("Executive invoked prepare_trace_create/call before this function; vecindex_stack is never empty; qed");
		self.index_stack.pop();

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

		if let Some(index) = self.index_stack.last_mut() {
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
			trace_address: self.index_stack.clone(),
		};
		debug!(target: "trace", "Traced suicide {:?}", trace);
		self.traces.push(trace);

		if let Some(index) = self.index_stack.last_mut() {
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
			trace_address: self.index_stack.clone(),
		};
		debug!(target: "trace", "Traced reward {:?}", trace);
		self.traces.push(trace);

		if let Some(index) = self.index_stack.last_mut() {
			*index += 1;
		}
	}

	fn drain(self) -> Vec<FlatTrace> {
		self.traces
	}
}

struct TraceData {
	mem_written: Option<(usize, usize)>,
	store_written: Option<(U256, U256)>,
}

/// Simple VM tracer. Traces all operations.
pub struct ExecutiveVMTracer {
	data: VMTrace,
	depth: usize,
	trace_stack: Vec<TraceData>,
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
			},
			depth: 0,
			trace_stack: vec![],
		}
	}

	fn with_trace_in_depth<F: Fn(&mut VMTrace)>(trace: &mut VMTrace, depth: usize, f: F) {
		if depth == 0 {
			f(trace);
		} else {
			Self::with_trace_in_depth(trace.subs.last_mut().expect("self.depth is incremented with prepare_subtrace; a subtrace is always pushed; self.depth cannot be greater than subtrace stack; qed"), depth - 1, f);
		}
	}
}

impl VMTracer for ExecutiveVMTracer {
	type Output = VMTrace;

	fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8, _current_gas: U256) -> bool { true }

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256, mem_written: Option<(usize, usize)>, store_written: Option<(U256, U256)>) {
		Self::with_trace_in_depth(&mut self.data, self.depth, move |trace| {
			trace.operations.push(VMOperation {
				pc: pc,
				instruction: instruction,
				gas_cost: gas_cost,
				executed: None,
			});
		});
		self.trace_stack.push(TraceData { mem_written, store_written });
	}

	fn trace_failed(&mut self) {
		let _ = self.trace_stack.pop().expect("pushed in trace_prepare_execute; qed");
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem: &[u8]) {
		let TraceData { mem_written, store_written } = self.trace_stack.pop().expect("pushed in trace_prepare_execute; qed");
		let mem_diff = mem_written.map(|(o, s)| {
			if o + s > mem.len() {
				warn!(target: "trace", "mem_written is out of bounds");
			}
			(o, &mem[min(mem.len(), o)..min(o + s, mem.len())])
		});
		let store_diff = store_written;
		Self::with_trace_in_depth(&mut self.data, self.depth, move |trace| {
			let ex = VMExecutedOperation {
				gas_used: gas_used,
				stack_push: stack_push.to_vec(),
				mem_diff: mem_diff.map(|(s, r)| MemoryDiff { offset: s, data: r.to_vec() }),
				store_diff: store_diff.map(|(l, v)| StorageDiff { location: l, value: v }),
			};
			trace.operations.last_mut().expect("trace_executed is always called after a trace_prepare_execute; trace.operations cannot be empty; qed").executed = Some(ex);
		});
	}

	fn prepare_subtrace(&mut self, code: &[u8]) {
		Self::with_trace_in_depth(&mut self.data, self.depth, move |trace| {
			let parent_step = trace.operations.len() - 1; // won't overflow since we must already have pushed an operation in trace_prepare_execute.
			trace.subs.push(VMTrace {
				parent_step,
				code: code.to_vec(),
				operations: vec![],
				subs: vec![],
			});
		});
		self.depth += 1;
	}

	fn done_subtrace(&mut self) {
		self.depth -= 1;
	}

	fn drain(mut self) -> Option<VMTrace> { self.data.subs.pop() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_prefix_address_properly() {
		let mut tracer = ExecutiveTracer::default();

		tracer.prepare_trace_call(&ActionParams::default(), 0, false);
		tracer.prepare_trace_call(&ActionParams::default(), 1, false);
		tracer.prepare_trace_call(&ActionParams::default(), 2, false);
		tracer.done_trace_call(U256::zero(), &[]);
		tracer.prepare_trace_call(&ActionParams::default(), 2, false);
		tracer.done_trace_call(U256::zero(), &[]);
		tracer.prepare_trace_call(&ActionParams::default(), 2, false);
		tracer.done_trace_call(U256::zero(), &[]);
		tracer.done_trace_call(U256::zero(), &[]);
		tracer.done_trace_call(U256::zero(), &[]);

		let drained = tracer.drain();
		assert!(drained[0].trace_address.len() == 0);
		assert_eq!(&drained[1].trace_address, &[0]);
		assert_eq!(&drained[2].trace_address, &[0, 0]);
		assert_eq!(&drained[3].trace_address, &[0, 1]);
		assert_eq!(&drained[4].trace_address, &[0, 2]);
	}
}
