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

//! Standardized JSON VM output.

use std::collections::HashMap;
use std::io;

use ethereum_types::{H256, U256};
use bytes::ToPretty;
use ethcore::trace;

use display;
use info as vm;

pub trait Writer: io::Write + Send + Sized {
	fn clone(&self) -> Self;
}

impl Writer for io::Stdout {
	fn clone(&self) -> Self {
		io::stdout()
	}
}

/// JSON formatting informant.
pub struct Informant<T: Writer = io::Stdout> {
	code: Vec<u8>,
	instruction: u8,
	depth: usize,
	stack: Vec<U256>,
	storage: HashMap<H256, H256>,
	sink: T,
}

impl Default for Informant {
	fn default() -> Self {
		Self::new(io::stdout())
	}
}

impl<T: Writer> Informant<T> {
	pub fn new(sink: T) -> Self {
		Informant {
			code: Default::default(),
			instruction: Default::default(),
			depth: Default::default(),
			stack: Default::default(),
			storage: Default::default(),
			sink,
		}
	}
}

impl<T: Writer> Informant<T> {
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

impl<T: Writer> vm::Informant for Informant<T> {
	fn before_test(&mut self, name: &str, action: &str) {
		writeln!(
			&mut self.sink,
			"{{\"test\":\"{name}\",\"action\":\"{action}\"}}",
			name = name,
			action = action,
		).expect("The sink must be writeable.");
	}

	fn set_gas(&mut self, _gas: U256) {}

	fn finish(result: vm::RunResult<Self::Output>) {
		match result {
			Ok(success) => {
				println!("{{\"stateRoot\":\"{:?}\"}}", success.state_root);
				println!(
					"{{\"output\":\"0x{output}\",\"gasUsed\":\"{gas:x}\",\"time\":{time}}}",
					output = success.output.to_hex(),
					gas = success.gas_used,
					time = display::as_micros(&success.time),
				);
			},
			Err(failure) => {
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

impl<T: Writer> trace::VMTracer for Informant<T> {
	type Output = ();

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8, current_gas: U256) -> bool {
		let info = ::evm::Instruction::from_u8(instruction).map(|i| i.info());
		self.instruction = instruction;
		let storage = self.storage();
		let stack = self.stack();

		writeln!(
			&mut self.sink,
			"{{\"pc\":{pc},\"op\":{op},\"opName\":\"{name}\",\"gas\":\"0x{gas:x}\",\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = pc,
			op = instruction,
			name = info.map(|i| i.name).unwrap_or(""),
			gas = current_gas,
			stack = stack,
			storage = storage,
			depth = self.depth,
		).expect("The sink must be writeable.");

		true
	}

	fn trace_prepare_execute(&mut self, _pc: usize, _instruction: u8, _gas_cost: U256) {
	}

	fn trace_executed(&mut self, _gas_used: U256, stack_push: &[U256], _mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		let info = ::evm::Instruction::from_u8(self.instruction).map(|i| i.info());

		let len = self.stack.len();
		let info_args = info.map(|i| i.args).unwrap_or(0);
		self.stack.truncate(if len > info_args { len - info_args } else { 0 });
		self.stack.extend_from_slice(stack_push);

		if let Some((pos, val)) = store_diff {
			self.storage.insert(pos.into(), val.into());
		}
	}

	fn prepare_subtrace(&self, code: &[u8]) -> Self where Self: Sized {
		let mut vm = Informant::new(self.sink.clone());
		vm.depth = self.depth + 1;
		vm.code = code.to_vec();
		vm
	}

	fn done_subtrace(&mut self, _sub: Self) {}

	fn drain(self) -> Option<Self::Output> { None }
}

#[cfg(test)]
mod tests {
	use std::sync::{Arc, Mutex};
	use super::*;
	use info::tests::run_test;

	#[derive(Debug, Clone, Default)]
	struct TestWriter(pub Arc<Mutex<Vec<u8>>>);

	impl Writer for TestWriter {
		fn clone(&self) -> Self { Clone::clone(self) }
	}

	impl io::Write for TestWriter {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			self.0.lock().unwrap().write(buf)
		}

		fn flush(&mut self) -> io::Result<()> {
			self.0.lock().unwrap().flush()
		}
	}

	fn informant() -> (Informant<TestWriter>, Arc<Mutex<Vec<u8>>>) {
		let writer = TestWriter::default();
		let res = writer.0.clone();
		(Informant::new(writer), res)
	}

	#[test]
	fn should_trace_failure() {
		let (inf, res) = informant();
		run_test(
			inf,
			move |_, expected| {
				let bytes = res.lock().unwrap();
				assert_eq!(expected, &String::from_utf8_lossy(&**bytes))
			},
			"60F8d6",
			0xffff,
			r#"{"pc":0,"op":96,"opName":"PUSH1","gas":"0xffff","stack":[],"storage":{},"depth":1}
{"pc":2,"op":214,"opName":"","gas":"0xfffc","stack":["0xf8"],"storage":{},"depth":1}
"#,
		);

		let (inf, res) = informant();
		run_test(
			inf,
			move |_, expected| {
				let bytes = res.lock().unwrap();
				assert_eq!(expected, &String::from_utf8_lossy(&**bytes))
			},
			"F8d6",
			0xffff,
			r#"{"pc":0,"op":248,"opName":"","gas":"0xffff","stack":[],"storage":{},"depth":1}
"#,
		);
	}

	#[test]
	fn should_trace_create_correctly() {
		let (informant, res) = informant();
		run_test(
			informant,
			move |_, expected| {
				let bytes = res.lock().unwrap();
				assert_eq!(expected, &String::from_utf8_lossy(&**bytes))
			},
			"32343434345830f138343438323439f0",
			0xffff,
			r#"{"pc":0,"op":50,"opName":"ORIGIN","gas":"0xffff","stack":[],"storage":{},"depth":1}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0xfffd","stack":["0x0"],"storage":{},"depth":1}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0xfffb","stack":["0x0","0x0"],"storage":{},"depth":1}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0xfff9","stack":["0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0xfff7","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":5,"op":88,"opName":"PC","gas":"0xfff5","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":1}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0xfff3","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":1}
{"pc":7,"op":241,"opName":"CALL","gas":"0xfff1","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0x0"],"storage":{},"depth":1}
{"pc":8,"op":56,"opName":"CODESIZE","gas":"0x9e21","stack":["0x1"],"storage":{},"depth":1}
{"pc":9,"op":52,"opName":"CALLVALUE","gas":"0x9e1f","stack":["0x1","0x10"],"storage":{},"depth":1}
{"pc":10,"op":52,"opName":"CALLVALUE","gas":"0x9e1d","stack":["0x1","0x10","0x0"],"storage":{},"depth":1}
{"pc":11,"op":56,"opName":"CODESIZE","gas":"0x9e1b","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":12,"op":50,"opName":"ORIGIN","gas":"0x9e19","stack":["0x1","0x10","0x0","0x0","0x10"],"storage":{},"depth":1}
{"pc":13,"op":52,"opName":"CALLVALUE","gas":"0x9e17","stack":["0x1","0x10","0x0","0x0","0x10","0x0"],"storage":{},"depth":1}
{"pc":14,"op":57,"opName":"CODECOPY","gas":"0x9e15","stack":["0x1","0x10","0x0","0x0","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":15,"op":240,"opName":"CREATE","gas":"0x9e0c","stack":["0x1","0x10","0x0","0x0"],"storage":{},"depth":1}
{"pc":0,"op":50,"opName":"ORIGIN","gas":"0x210c","stack":[],"storage":{},"depth":2}
{"pc":1,"op":52,"opName":"CALLVALUE","gas":"0x210a","stack":["0x0"],"storage":{},"depth":2}
{"pc":2,"op":52,"opName":"CALLVALUE","gas":"0x2108","stack":["0x0","0x0"],"storage":{},"depth":2}
{"pc":3,"op":52,"opName":"CALLVALUE","gas":"0x2106","stack":["0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":4,"op":52,"opName":"CALLVALUE","gas":"0x2104","stack":["0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":5,"op":88,"opName":"PC","gas":"0x2102","stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{},"depth":2}
{"pc":6,"op":48,"opName":"ADDRESS","gas":"0x2100","stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{},"depth":2}
{"pc":7,"op":241,"opName":"CALL","gas":"0x20fe","stack":["0x0","0x0","0x0","0x0","0x0","0x5","0xbd770416a3345f91e4b34576cb804a576fa48eb1"],"storage":{},"depth":2}
"#,
		)
	}
}
