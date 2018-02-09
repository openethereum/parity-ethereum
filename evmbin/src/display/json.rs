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
		let items = self.stack.iter().map(|i| format!("\"0x{:x}\"", i)).collect::<Vec<_>>();
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
	fn before_test(&mut self, name: &str, action: &str) {
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

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8, _current_gas: U256) -> bool {
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
			"{{\"pc\":{pc},\"op\":{op},\"opName\":\"{name}\",\"gas\":\"0x{gas:x}\",\"gasCost\":\"0x{gas_cost:x}\",\"memory\":{memory},\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = self.pc,
			op = self.instruction,
			name = info.name,
			gas = gas_used.saturating_add(self.gas_cost),
			gas_cost = self.gas_cost,
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

		// TODO [ToDr] Align memory?
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

#[cfg(test)]
mod tests {
	use super::*;
	use info::tests::run_test;

	fn assert_traces_eq(
		a: &[String],
		b: &[String],
	) {
		let mut ita = a.iter();
		let mut itb = b.iter();

		loop {
			match (ita.next(), itb.next()) {
				(Some(a), Some(b)) => {
					assert_eq!(a, b);
					println!("{}", a);
				},
				(None, None) => return,
				e => {
					panic!("Traces mismatch: {:?}", e);
				}
			}
		}
	}

	fn compare_json(traces: Option<Vec<String>>, expected: &str) {
		let expected = expected.split("\n")
			.map(|x| x.trim())
			.map(|x| x.to_owned())
			.filter(|x| !x.is_empty())
			.collect::<Vec<_>>();
		assert_traces_eq(&traces.unwrap(), &expected);
	}

	#[test]
	fn should_trace_failure() {
		run_test(
			Informant::default(),
			&compare_json,
			"60F8d6",
			0xffff,
			r#"
{"pc":0,"op":96,"opName":"PUSH1","gas":"0xffff","gasCost":"0x3","memory":"0x","stack":[],"storage":{},"depth":1}
{"pc":2,"op":214,"opName":"","gas":"0xfffc","gasCost":"0x0","memory":"0x","stack":["0xf8"],"storage":{},"depth":1}
			"#,
		);

		run_test(
			Informant::default(),
			&compare_json,
			"F8d6",
			0xffff,
			r#"
{"pc":0,"op":248,"opName":"","gas":"0xffff","gasCost":"0x0","memory":"0x","stack":[],"storage":{},"depth":1}
			"#,
		);
	}

	#[test]
	fn should_trace_create_correctly() {
		run_test(
			Informant::default(),
			&compare_json,
			"32343434345830f138343438323439f0",
			0xffff,
			r#"
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0xffff","gasCost":"0x2","memory":"0x","stack":[],"storage":{},"depth":1}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0xfffd","gasCost":"0x2","memory":"0x","stack":["0x0"],"storage":{},"depth":1}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0xfffb","gasCost":"0x2","memory":"0x","stack":["0x0","0x0"],"storage":{},"depth":1}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0xfff9","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0xfff7","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":5,"op":88,"opName":"PC","gas":"0xfff5","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0xfff3","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":1}
{"pc":7,"op":241,"opName":"CALL","gas":"0xfff1","gasCost":"0x61d0","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0x0"],"storage":{},"depth":1}
{"pc":8,"op":56,"opName":"CODESIZE","gas":"0x9e21","gasCost":"0x2","memory":"0x","stack":["0x1"],"storage":{},"depth":1}
{"pc":9,"op":52,"opName":"CALLVALUE","gas":"0x9e1f","gasCost":"0x2","memory":"0x","stack":["0x1","0x10"],"storage":{},"depth":1}
{"pc":10,"op":52,"opName":"CALLVALUE","gas":"0x9e1d","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0"],"storage":{},"depth":1}
{"pc":11,"op":56,"opName":"CODESIZE","gas":"0x9e1b","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":12,"op":50,"opName":"ORIGIN","gas":"0x9e19","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10"],"storage":{},"depth":1}
{"pc":13,"op":52,"opName":"CALLVALUE","gas":"0x9e17","gasCost":"0x2","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10","0x0"],"storage":{},"depth":1}
{"pc":14,"op":57,"opName":"CODECOPY","gas":"0x9e15","gasCost":"0x9","memory":"0x","stack":["0x1","0x10","0x0","0x0","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":15,"op":240,"opName":"CREATE","gas":"0x9e0c","gasCost":"0x9e0c","memory":"0x32343434345830f138343438323439f0","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0x210c","gasCost":"0x2","memory":"0x","stack":[],"storage":{},"depth":2}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0x210a","gasCost":"0x2","memory":"0x","stack":["0x0"],"storage":{},"depth":2}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0x2108","gasCost":"0x2","memory":"0x","stack":["0x0","0x0"],"storage":{},"depth":2}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0x2106","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0x2104","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":5,"op":88,"opName":"PC","gas":"0x2102","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0x2100","gasCost":"0x2","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":2}
{"pc":7,"op":241,"opName":"CALL","gas":"0x20fe","gasCost":"0x0","memory":"0x","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0xbd770416a3345f91e4b34576cb804a576fa48eb1"],"storage":{},"depth":2}
			"#,
		)
	}
}
