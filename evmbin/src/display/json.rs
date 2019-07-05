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

//! JSON VM output.

use std::collections::HashMap;
use std::mem;

use ethereum_types::{U256, H256, BigEndianHash};
use bytes::ToPretty;
use trace;

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
	mem_written: Option<(usize, usize)>,
	store_written: Option<(U256, U256)>,
	stack: Vec<U256>,
	memory: Vec<u8>,
	storage: HashMap<H256, H256>,
	traces: Vec<String>,
	subtraces: Vec<String>,
	subinfos: Vec<Informant>,
	subdepth: usize,
	unmatched: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraceData<'a> {
	pc: usize,
	op: u8,
	op_name: &'a str,
	gas: &'a str,
	gas_cost: &'a str,
	memory: &'a str,
	stack: &'a [U256],
	storage: &'a HashMap<H256, H256>,
	depth: usize,
}

#[derive(Serialize, Debug)]
pub struct MessageInitial<'a> {
	action: &'a str,
	test: &'a str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageSuccess<'a> {
	output: &'a str,
	gas_used: &'a str,
	time: &'a u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageFailure<'a> {
	error: &'a str,
	gas_used: &'a str,
	time: &'a u64,
}

impl Informant {
	fn with_informant_in_depth<F: Fn(&mut Informant)>(informant: &mut Informant, depth: usize, f: F) {
		if depth == 0 {
			f(informant);
		} else {
			Self::with_informant_in_depth(informant.subinfos.last_mut().expect("prepare/done_trace are not balanced"), depth - 1, f);
		}
	}

	fn informant_trace(informant: &Informant, gas_used: U256) -> String {
		let info = ::evm::Instruction::from_u8(informant.instruction).map(|i| i.info());

		let trace_data =
			TraceData {
				pc: informant.pc,
				op: informant.instruction,
				op_name: info.map(|i| i.name).unwrap_or(""),
				gas: &format!("{:#x}", gas_used.saturating_add(informant.gas_cost)),
				gas_cost: &format!("{:#x}", informant.gas_cost),
				memory: &format!("0x{}", informant.memory.to_hex()),
				stack: &informant.stack,
				storage: &informant.storage,
				depth: informant.depth,
			}
		;

		serde_json::to_string(&trace_data).expect("Serialization cannot fail; qed")
	}
}

impl vm::Informant for Informant {
	type Sink = ();

	fn before_test(&mut self, name: &str, action: &str) {
		let message_init =
			MessageInitial {
				action,
				test: &name,
			}
		;

		let s = serde_json::to_string(&message_init).expect("Serialization cannot fail; qed");
		println!("{}", s);
	}

	fn set_gas(&mut self, gas: U256) {
		self.gas_used = gas;
	}

	fn clone_sink(&self) -> Self::Sink { () }

	fn finish(result: vm::RunResult<Self::Output>, _sink: &mut Self::Sink) {
		match result {
			Ok(success) => {
				for trace in success.traces.unwrap_or_else(Vec::new) {
					println!("{}", trace);
				}

				let message_success =
					MessageSuccess {
						output: &format!("0x{}", success.output.to_hex()),
						gas_used: &format!("{:#x}", success.gas_used),
						time: &display::as_micros(&success.time),
					}
				;

				let s = serde_json::to_string(&message_success).expect("Serialization cannot fail; qed");
				println!("{}", s);
			},
			Err(failure) => {
				for trace in failure.traces.unwrap_or_else(Vec::new) {
					println!("{}", trace);
				}

				let message_failure =
					MessageFailure {
						error: &failure.error.to_string(),
						gas_used: &format!("{:#x}", failure.gas_used),
						time: &display::as_micros(&failure.time),
					}
				;

				let s = serde_json::to_string(&message_failure).expect("Serialization cannot fail; qed");
				println!("{}", s);
			},
		}
	}
}

impl trace::VMTracer for Informant {
	type Output = Vec<String>;

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8, _current_gas: U256) -> bool {
		let subdepth = self.subdepth;
		Self::with_informant_in_depth(self, subdepth, |informant: &mut Informant| {
			informant.pc = pc;
			informant.instruction = instruction;
			informant.unmatched = true;
		});
		true
	}

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256, mem_written: Option<(usize, usize)>, store_written: Option<(U256, U256)>) {
		let subdepth = self.subdepth;
		Self::with_informant_in_depth(self, subdepth, |informant: &mut Informant| {
			informant.pc = pc;
			informant.instruction = instruction;
			informant.gas_cost = gas_cost;
			informant.mem_written = mem_written;
			informant.store_written = store_written;
		});
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem: &[u8]) {
		let subdepth = self.subdepth;
		Self::with_informant_in_depth(self, subdepth, |informant: &mut Informant| {
			let store_diff = informant.store_written.clone();
			let info = ::evm::Instruction::from_u8(informant.instruction).map(|i| i.info());

			let trace = Self::informant_trace(informant, gas_used);
			informant.traces.push(trace);

			informant.unmatched = false;
			informant.gas_used = gas_used;

			let len = informant.stack.len();
			let info_args = info.map(|i| i.args).unwrap_or(0);
			informant.stack.truncate(if len > info_args { len - info_args } else { 0 });
			informant.stack.extend_from_slice(stack_push);

			// TODO [ToDr] Align memory?
			if let Some((pos, size)) = informant.mem_written.clone() {
				if informant.memory.len() < (pos + size) {
					informant.memory.resize(pos + size, 0);
				}
				informant.memory[pos..(pos + size)].copy_from_slice(&mem[pos..(pos + size)]);
			}

			if let Some((pos, val)) = store_diff {
				informant.storage.insert(BigEndianHash::from_uint(&pos), BigEndianHash::from_uint(&val));
			}

			if !informant.subtraces.is_empty() {
				informant.traces.extend(mem::replace(&mut informant.subtraces, vec![]));
			}
		});
	}

	fn prepare_subtrace(&mut self, code: &[u8]) {
		let subdepth = self.subdepth;
		Self::with_informant_in_depth(self, subdepth, |informant: &mut Informant| {
			let mut vm = Informant::default();
			vm.depth = informant.depth + 1;
			vm.code = code.to_vec();
			vm.gas_used = informant.gas_used;
			informant.subinfos.push(vm);
		});
		self.subdepth += 1;
	}

	fn done_subtrace(&mut self) {
		self.subdepth -= 1;
		let subdepth = self.subdepth;
		Self::with_informant_in_depth(self, subdepth, |informant: &mut Informant| {
			if let Some(subtraces) = informant.subinfos.pop().expect("prepare/done_subtrace are not balanced").drain() {
				informant.subtraces.extend(subtraces);
			}
		});
	}

	fn drain(mut self) -> Option<Self::Output> {
		if self.unmatched {
			// print last line with final state:
			self.gas_cost = 0.into();
			let gas_used = self.gas_used;
			let subdepth = self.subdepth;

			Self::with_informant_in_depth(&mut self, subdepth, |informant: &mut Informant| {
				let trace = Self::informant_trace(informant, gas_used);
				informant.traces.push(trace);
			});
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
	use serde_json;

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	#[serde(rename_all = "camelCase")]
	struct TestTrace {
		pc: usize,
		#[serde(rename = "op")]
		instruction: u8,
		op_name: String,
		#[serde(rename = "gas")]
		gas_used: U256,
		gas_cost: U256,
		memory: String,
		stack: Vec<U256>,
		storage: HashMap<H256, H256>,
		depth: usize,
	}

	fn assert_traces_eq(
		a: &[String],
		b: &[String],
	) {
		let mut ita = a.iter();
		let mut itb = b.iter();

		loop {
			match (ita.next(), itb.next()) {
				(Some(a), Some(b)) => {
					// Compare both without worrying about the order of the fields
					let actual: TestTrace = serde_json::from_str(a).unwrap();
					let expected: TestTrace = serde_json::from_str(b).unwrap();
					assert_eq!(actual, expected);
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

		run_test(
			Informant::default(),
			&compare_json,
			"5A51",
			0xfffff,
			r#"
{"depth":1,"gas":"0xfffff","gasCost":"0x2","memory":"0x","op":90,"opName":"GAS","pc":0,"stack":[],"storage":{}}
{"depth":1,"gas":"0xffffd","gasCost":"0x0","memory":"0x","op":81,"opName":"MLOAD","pc":1,"stack":["0xffffd"],"storage":{}}
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
		);

		run_test(
			Informant::default(),
			&compare_json,
			"3260D85554",
			0xffff,
			r#"
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0xffff","gasCost":"0x2","memory":"0x","stack":[],"storage":{},"depth":1}
{"pc":1,"op":96,"opName":"PUSH1","gas":"0xfffd","gasCost":"0x3","memory":"0x","stack":["0x0"],"storage":{},"depth":1}
{"pc":3,"op":85,"opName":"SSTORE","gas":"0xfffa","gasCost":"0x1388","memory":"0x","stack":["0x0","0xd8"],"storage":{},"depth":1}
{"pc":4,"op":84,"opName":"SLOAD","gas":"0xec72","gasCost":"0x0","memory":"0x","stack":[],"storage":{"0x00000000000000000000000000000000000000000000000000000000000000d8":"0x0000000000000000000000000000000000000000000000000000000000000000"},"depth":1}
"#,
		)
	}
}
