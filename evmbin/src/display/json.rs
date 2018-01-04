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

//! JSON VM output.

use std::collections::HashMap;
use std::mem;

use ethereum_types::{U256, H256};
use bytes::ToPretty;
use ethcore::trace;

use display;
use info as vm;

/// JSON formatting informant.
#[derive(Default)]
pub struct Informant {
	code: Vec<u8>,
	depth: usize,
	pc: usize,
	instruction: u8,
	gas_cost: U256,
	gas_used: U256,
	stack: Vec<U256>,
	memory: Vec<u8>,
	storage: HashMap<H256, H256>,
	traces: Vec<String>,
	subtraces: Vec<String>,
	unmatched: bool,
}

impl Informant {
	fn memory(&self) -> String {
		format!("\"0x{}\"", self.memory.to_hex())
	}

	fn stack(&self) -> String {
		let items = self.stack.iter().map(display::u256_as_str).collect::<Vec<_>>();
		format!("[{}]", items.join(","))
	}

	fn storage(&self) -> String {
		let vals = self.storage.iter()
			.map(|(k, v)| format!("\"0x{:?}\": \"0x{:?}\"", k, v))
			.collect::<Vec<_>>();
		format!("{{{}}}", vals.join(","))
	}
}

impl vm::Informant for Informant {
	fn before_test(&self, name: &str, action: &str) {
		println!(
			"{{\"test\":\"{name}\",\"action\":\"{action}\"}}",
			name = name,
			action = action,
		);
	}

	fn set_gas(&mut self, gas: U256) {
		self.gas_used = gas;
	}

	fn finish(result: vm::RunResult<Self::Output>) {
		match result {
			Ok(success) => {
				for trace in success.traces.unwrap_or_else(Vec::new) {
					println!("{}", trace);
				}

				println!(
					"{{\"output\":\"0x{output}\",\"gasUsed\":\"{gas:x}\",\"time\":{time}}}",
					output = success.output.to_hex(),
					gas = success.gas_used,
					time = display::as_micros(&success.time),
				)
			},
			Err(failure) => {
				for trace in failure.traces.unwrap_or_else(Vec::new) {
					println!("{}", trace);
				}

				println!(
					"{{\"error\":\"{error}\",\"gasUsed\":\"{gas:x}\",\"time\":{time}}}",
					error = failure.error,
					gas = failure.gas_used,
					time = display::as_micros(&failure.time),
				)
			},
		}
	}
}

impl trace::VMTracer for Informant {
	type Output = Vec<String>;

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8) -> bool {
		self.pc = pc;
		self.instruction = instruction;
		self.unmatched = true;
		true
	}

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256) {
		self.pc = pc;
		self.instruction = instruction;
		self.gas_cost = gas_cost;
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		let info = ::evm::INSTRUCTIONS[self.instruction as usize];

		let trace = format!(
			"{{\"pc\":{pc},\"op\":{op},\"opName\":\"{name}\",\"gas\":{gas},\"gasCost\":{gas_cost},\"memory\":{memory},\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = self.pc,
			op = self.instruction,
			name = info.name,
			gas = display::u256_as_str(&(gas_used.saturating_add(self.gas_cost))),
			gas_cost = display::u256_as_str(&self.gas_cost),
			memory = self.memory(),
			stack = self.stack(),
			storage = self.storage(),
			depth = self.depth,
		);
		self.traces.push(trace);

		self.unmatched = false;
		self.gas_used = gas_used;

		let len = self.stack.len();
		self.stack.truncate(if len > info.args { len - info.args } else { 0 });
		self.stack.extend_from_slice(stack_push);

		if let Some((pos, data)) = mem_diff {
			if self.memory.len() < (pos + data.len()) {
				self.memory.resize(pos + data.len(), 0);
			}
			self.memory[pos..pos + data.len()].copy_from_slice(data);
		}

		if let Some((pos, val)) = store_diff {
			self.storage.insert(pos.into(), val.into());
		}


		if !self.subtraces.is_empty() {
			self.traces.extend(mem::replace(&mut self.subtraces, vec![]));
		}
	}

	fn prepare_subtrace(&self, code: &[u8]) -> Self where Self: Sized {
		let mut vm = Informant::default();
		vm.depth = self.depth + 1;
		vm.code = code.to_vec();
		vm.gas_used = self.gas_used;
		vm
	}

	fn done_subtrace(&mut self, sub: Self) {
		if let Some(subtraces) = sub.drain() {
			self.subtraces.extend(subtraces);
		}
	}

	fn drain(mut self) -> Option<Self::Output> {
		if self.unmatched {
			// print last line with final state:
			self.gas_cost = 0.into();
			let gas_used = self.gas_used;
			self.trace_executed(gas_used, &[], None, None);
		} else if !self.subtraces.is_empty() {
			self.traces.extend(mem::replace(&mut self.subtraces, vec![]));
		}
		Some(self.traces)
	}
}
