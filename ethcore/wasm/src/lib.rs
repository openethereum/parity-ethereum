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

//! Wasm Interpreter

extern crate byteorder;
extern crate ethereum_types;
#[macro_use] extern crate log;
extern crate libc;
extern crate parity_wasm;
extern crate vm;
extern crate pwasm_utils as wasm_utils;
extern crate wasmi;

#[cfg(test)]
extern crate env_logger;

mod env;
mod panic_payload;
mod parser;
mod runtime;

#[cfg(test)]
mod tests;


use vm::{GasLeft, ReturnData, ActionParams};
use wasmi::{Error as InterpreterError, Trap};

use runtime::{Runtime, RuntimeContext};

use ethereum_types::U256;

/// Wrapped interpreter error
#[derive(Debug)]
pub enum Error {
	Interpreter(InterpreterError),
	Trap(Trap),
}

impl From<InterpreterError> for Error {
	fn from(e: InterpreterError) -> Self {
		Error::Interpreter(e)
	}
}

impl From<Trap> for Error {
	fn from(e: Trap) -> Self {
		Error::Trap(e)
	}
}

impl From<Error> for vm::Error {
	fn from(e: Error) -> Self {
		match e {
			Error::Interpreter(e) => vm::Error::Wasm(format!("Wasm runtime error: {:?}", e)),
			Error::Trap(e) => vm::Error::Wasm(format!("Wasm contract trap: {:?}", e)),
		}
	}
}

/// Wasm interpreter instance
pub struct WasmInterpreter {
	params: ActionParams,
}

impl WasmInterpreter {
	pub fn new(params: ActionParams) -> Self {
		WasmInterpreter { params }
	}
}

impl From<runtime::Error> for vm::Error {
	fn from(e: runtime::Error) -> Self {
		vm::Error::Wasm(format!("Wasm runtime error: {:?}", e))
	}
}

enum ExecutionOutcome {
	Suicide,
	Return,
	NotSpecial,
}

impl WasmInterpreter {
	pub fn run(self: Box<WasmInterpreter>, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
		let (module, data) = parser::payload(&self.params, ext.schedule().wasm())?;

		let loaded_module = wasmi::Module::from_parity_wasm_module(module).map_err(Error::Interpreter)?;

		let instantiation_resolver = env::ImportResolver::with_limit(16, ext.schedule().wasm());

		let module_instance = wasmi::ModuleInstance::new(
			&loaded_module,
			&wasmi::ImportsBuilder::new().with_resolver("env", &instantiation_resolver)
		).map_err(Error::Interpreter)?;

		let adjusted_gas = self.params.gas * U256::from(ext.schedule().wasm().opcodes_div) /
			U256::from(ext.schedule().wasm().opcodes_mul);

		if adjusted_gas > ::std::u64::MAX.into()
		{
			return Err(vm::Error::Wasm("Wasm interpreter cannot run contracts with gas (wasm adjusted) >= 2^64".to_owned()));
		}

		let initial_memory = instantiation_resolver.memory_size().map_err(Error::Interpreter)?;
		trace!(target: "wasm", "Contract requested {:?} pages of initial memory", initial_memory);

		let (gas_left, result) = {
			let mut runtime = Runtime::with_params(
				ext,
				instantiation_resolver.memory_ref(),
				// cannot overflow, checked above
				adjusted_gas.low_u64(),
				data.to_vec(),
				RuntimeContext {
					address: self.params.address,
					sender: self.params.sender,
					origin: self.params.origin,
					code_address: self.params.code_address,
					code_version: self.params.code_version,
					value: self.params.value.value(),
				},
			);

			// cannot overflow if static_region < 2^16,
			// initial_memory ∈ [0..2^32)
			// total_charge <- static_region * 2^32 * 2^16
			// total_charge ∈ [0..2^64) if static_region ∈ [0..2^16)
			// qed
			assert!(runtime.schedule().wasm().initial_mem < 1 << 16);
			runtime.charge(|s| initial_memory as u64 * s.wasm().initial_mem as u64)?;

			let module_instance = module_instance.run_start(&mut runtime).map_err(Error::Trap)?;

			let invoke_result = module_instance.invoke_export("call", &[], &mut runtime);

			let mut execution_outcome = ExecutionOutcome::NotSpecial;
			if let Err(InterpreterError::Trap(ref trap)) = invoke_result {
				if let wasmi::TrapKind::Host(ref boxed) = *trap.kind() {
					let ref runtime_err = boxed.downcast_ref::<runtime::Error>()
						.expect("Host errors other than runtime::Error never produced; qed");

					match **runtime_err {
						runtime::Error::Suicide => { execution_outcome = ExecutionOutcome::Suicide; },
						runtime::Error::Return => { execution_outcome = ExecutionOutcome::Return; },
						_ => {}
					}
				}
			}

			if let (ExecutionOutcome::NotSpecial, Err(e)) = (execution_outcome, invoke_result) {
				trace!(target: "wasm", "Error executing contract: {:?}", e);
				return Err(vm::Error::from(Error::from(e)));
			}

			(
				runtime.gas_left().expect("Cannot fail since it was not updated since last charge"),
				runtime.into_result(),
			)
		};

		let gas_left =
			U256::from(gas_left) * U256::from(ext.schedule().wasm().opcodes_mul)
				/ U256::from(ext.schedule().wasm().opcodes_div);

		if result.is_empty() {
			trace!(target: "wasm", "Contract execution result is empty.");
			Ok(GasLeft::Known(gas_left))
		} else {
			let len = result.len();
			Ok(GasLeft::NeedsReturn {
				gas_left: gas_left,
				data: ReturnData::new(
					result,
					0,
					len,
				),
				apply_state: true,
			})
		}
	}
}

impl vm::Exec for WasmInterpreter {
	fn exec(self: Box<WasmInterpreter>, ext: &mut vm::Ext) -> vm::ExecTrapResult<GasLeft> {
		Ok(self.run(ext))
	}
}
