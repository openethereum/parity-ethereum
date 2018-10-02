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
	fn default() -> Self;
}

impl Writer for io::Stdout {
	fn clone(&self) -> Self {
		io::stdout()
	}

	fn default() -> Self {
		io::stdout()
	}
}

impl Writer for io::Stderr {
	fn clone(&self) -> Self {
		io::stderr()
	}

	fn default() -> Self {
		io::stderr()
	}
}

/// JSON formatting informant.
pub struct Informant<Trace = io::Stderr, Out = io::Stdout> {
	code: Vec<u8>,
	instruction: u8,
	depth: usize,
	stack: Vec<U256>,
	storage: HashMap<H256, H256>,
	trace_sink: Trace,
	out_sink: Out,
}

impl Default for Informant {
	fn default() -> Self {
		Self::new(io::stderr(), io::stdout())
	}
}

impl<Trace: Writer, Out: Writer> Informant<Trace, Out> {
	pub fn new(trace_sink: Trace, out_sink: Out) -> Self {
		Informant {
			code: Default::default(),
			instruction: Default::default(),
			depth: Default::default(),
			stack: Default::default(),
			storage: Default::default(),
			trace_sink, out_sink
		}
	}
}

impl<Trace: Writer, Out: Writer> vm::Informant for Informant<Trace, Out> {
	fn before_test(&mut self, name: &str, action: &str) {
		let out_data = json!({
			"action": action,
			"test": name,
		});

		writeln!(&mut self.out_sink, "{}", out_data).expect("The sink must be writeable.");
	}

	fn set_gas(&mut self, _gas: U256) {}

	fn finish(result: vm::RunResult<<Self as trace::VMTracer>::Output>) {
		let mut trace_sink = Trace::default();
		let mut out_sink = Out::default();

		match result {
			Ok(success) => {
				let trace_data = json!({"stateRoot": success.state_root});
				writeln!(&mut trace_sink, "{}", trace_data)
					.expect("The sink must be writeable.");

				let out_data = json!({
					"output": format!("0x{}", success.output.to_hex()),
					"gasUsed": format!("{:#x}", success.gas_used),
					"time": display::as_micros(&success.time),
				});

				writeln!(&mut out_sink, "{}", out_data).expect("The sink must be writeable.");
			},
			Err(failure) => {
				let out_data = json!({
					"error": &failure.error.to_string(),
					"gasUsed": format!("{:#x}", failure.gas_used),
					"time": display::as_micros(&failure.time),
				});

				writeln!(&mut out_sink, "{}", out_data).expect("The sink must be writeable.");
			},
		}
	}
}

impl<Trace: Writer, Out: Writer> trace::VMTracer for Informant<Trace, Out> {
	type Output = ();

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8, current_gas: U256) -> bool {
		let info = ::evm::Instruction::from_u8(instruction).map(|i| i.info());
		self.instruction = instruction;
		let trace_data = json!({
			"pc": pc,
			"op": instruction,
			"opName": info.map(|i| i.name).unwrap_or(""),
			"gas": format!("{:#x}", current_gas),
			"stack": self.stack,
			"storage": self.storage,
			"depth": self.depth,
		});

		writeln!(&mut self.trace_sink, "{}", trace_data).expect("The sink must be writeable.");

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
		let mut vm = Informant::new(self.trace_sink.clone(), self.out_sink.clone());
		vm.depth = self.depth + 1;
		vm.code = code.to_vec();
		vm
	}

	fn done_subtrace(&mut self, _sub: Self) {}

	fn drain(self) -> Option<Self::Output> { None }
}

#[cfg(test)]
pub mod tests {
	use std::sync::{Arc, Mutex};
	use super::*;
	use info::tests::run_test;

	#[derive(Debug, Clone, Default)]
	pub struct TestWriter(pub Arc<Mutex<Vec<u8>>>);

	impl Writer for TestWriter {
		fn clone(&self) -> Self { Clone::clone(self) }
		fn default() -> Self { Default::default() }
	}

	impl io::Write for TestWriter {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			self.0.lock().unwrap().write(buf)
		}

		fn flush(&mut self) -> io::Result<()> {
			self.0.lock().unwrap().flush()
		}
	}

	pub fn informant() -> (Informant<TestWriter, TestWriter>, Arc<Mutex<Vec<u8>>>) {
		let trace_writer: TestWriter = Default::default();
		let out_writer: TestWriter = Default::default();
		let res = trace_writer.0.clone();
		(Informant::new(trace_writer, out_writer), res)
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
			r#"{"depth":1,"gas":"0xffff","op":96,"opName":"PUSH1","pc":0,"stack":[],"storage":{}}
{"depth":1,"gas":"0xfffc","op":214,"opName":"","pc":2,"stack":["0xf8"],"storage":{}}
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
			r#"{"depth":1,"gas":"0xffff","op":248,"opName":"","pc":0,"stack":[],"storage":{}}
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
			r#"{"depth":1,"gas":"0xffff","op":50,"opName":"ORIGIN","pc":0,"stack":[],"storage":{}}
{"depth":1,"gas":"0xfffd","op":52,"opName":"CALLVALUE","pc":1,"stack":["0x0"],"storage":{}}
{"depth":1,"gas":"0xfffb","op":52,"opName":"CALLVALUE","pc":2,"stack":["0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0xfff9","op":52,"opName":"CALLVALUE","pc":3,"stack":["0x0","0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0xfff7","op":52,"opName":"CALLVALUE","pc":4,"stack":["0x0","0x0","0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0xfff5","op":88,"opName":"PC","pc":5,"stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0xfff3","op":48,"opName":"ADDRESS","pc":6,"stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{}}
{"depth":1,"gas":"0xfff1","op":241,"opName":"CALL","pc":7,"stack":["0x0","0x0","0x0","0x0","0x0","0x5","0x0"],"storage":{}}
{"depth":1,"gas":"0x9e21","op":56,"opName":"CODESIZE","pc":8,"stack":["0x1"],"storage":{}}
{"depth":1,"gas":"0x9e1f","op":52,"opName":"CALLVALUE","pc":9,"stack":["0x1","0x10"],"storage":{}}
{"depth":1,"gas":"0x9e1d","op":52,"opName":"CALLVALUE","pc":10,"stack":["0x1","0x10","0x0"],"storage":{}}
{"depth":1,"gas":"0x9e1b","op":56,"opName":"CODESIZE","pc":11,"stack":["0x1","0x10","0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0x9e19","op":50,"opName":"ORIGIN","pc":12,"stack":["0x1","0x10","0x0","0x0","0x10"],"storage":{}}
{"depth":1,"gas":"0x9e17","op":52,"opName":"CALLVALUE","pc":13,"stack":["0x1","0x10","0x0","0x0","0x10","0x0"],"storage":{}}
{"depth":1,"gas":"0x9e15","op":57,"opName":"CODECOPY","pc":14,"stack":["0x1","0x10","0x0","0x0","0x10","0x0","0x0"],"storage":{}}
{"depth":1,"gas":"0x9e0c","op":240,"opName":"CREATE","pc":15,"stack":["0x1","0x10","0x0","0x0"],"storage":{}}
{"depth":2,"gas":"0x210c","op":50,"opName":"ORIGIN","pc":0,"stack":[],"storage":{}}
{"depth":2,"gas":"0x210a","op":52,"opName":"CALLVALUE","pc":1,"stack":["0x0"],"storage":{}}
{"depth":2,"gas":"0x2108","op":52,"opName":"CALLVALUE","pc":2,"stack":["0x0","0x0"],"storage":{}}
{"depth":2,"gas":"0x2106","op":52,"opName":"CALLVALUE","pc":3,"stack":["0x0","0x0","0x0"],"storage":{}}
{"depth":2,"gas":"0x2104","op":52,"opName":"CALLVALUE","pc":4,"stack":["0x0","0x0","0x0","0x0"],"storage":{}}
{"depth":2,"gas":"0x2102","op":88,"opName":"PC","pc":5,"stack":["0x0","0x0","0x0","0x0","0x0"],"storage":{}}
{"depth":2,"gas":"0x2100","op":48,"opName":"ADDRESS","pc":6,"stack":["0x0","0x0","0x0","0x0","0x0","0x5"],"storage":{}}
{"depth":2,"gas":"0x20fe","op":241,"opName":"CALL","pc":7,"stack":["0x0","0x0","0x0","0x0","0x0","0x5","0xbd770416a3345f91e4b34576cb804a576fa48eb1"],"storage":{}}
"#,
		)
	}
}
