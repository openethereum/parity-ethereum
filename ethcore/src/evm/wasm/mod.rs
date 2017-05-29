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

//! Wasm Interpreter

mod runtime;
mod ptr;
mod call_args;
mod result;
#[cfg(test)]
mod tests;

use std::sync::Arc;

const DEFAULT_STACK_SPACE: u32 = 5 * 1024 * 1024;

use parity_wasm::{interpreter, elements};
use parity_wasm::interpreter::ModuleInstanceInterface;
use wasm_utils;

use evm::{self, GasLeft};
use action_params::ActionParams;
use self::runtime::Runtime;

pub use self::runtime::Error as RuntimeError;

const DEFAULT_RESULT_BUFFER: usize = 1024;

/// Wasm interpreter instance
pub struct WasmInterpreter {
	program: interpreter::ProgramInstance,
	result: Vec<u8>,
}

impl WasmInterpreter {
	/// New wasm interpreter instance
	pub fn new() -> Result<WasmInterpreter, RuntimeError> {
		Ok(WasmInterpreter {
			program: interpreter::ProgramInstance::new()?,
			result: Vec::with_capacity(DEFAULT_RESULT_BUFFER),
		})
	}
}

impl evm::Evm for WasmInterpreter {

	fn exec(&mut self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result<GasLeft> {
		use parity_wasm::elements::Deserialize;

		// todo: prefer panic?
		let env_instance = self.program.module("env")
			.ok_or(evm::Error::Wasm("Env module somehow does not exist in wasm runner runtime"))?;

		// todo: prefer panic?
		let env_memory = env_instance.memory(interpreter::ItemIndex::Internal(0))
			.map_err(|_| evm::Error::Wasm("Linear memory somehow does not exist in wasm runner runtime"))?;

		if params.gas > ::std::u64::MAX.into() {
			// todo: prefer panic?
			return Err(evm::Error::Wasm("Wasm interpreter cannot run contracts with gas >= 2^64"));
		}
		
		let mut runtime = Runtime::with_params(
			ext,
			env_memory,
			DEFAULT_STACK_SPACE,
			params.gas.low_u64(),
		);

		let code = params.code.expect("exec is only called on contract with code; qed");
		let mut cursor = ::std::io::Cursor::new(&*code);

		let contract_module = wasm_utils::inject_gas_counter(
			elements::Module::deserialize(
				&mut cursor
			).map_err(|e| {
				// todo: prefer panic?
				warn!("Error deserializing contract code as wasm module: {:?}", e);
				evm::Error::Wasm("Error deserializing contract code")
			})?
		);

		let d_ptr = runtime.write_descriptor(
			call_args::CallArgs::new(
				params.address,
				params.sender,
				params.origin,
				params.value.value(),
				params.data.unwrap_or(Vec::with_capacity(0)),
			)
		)?;

		{
			let execution_params = interpreter::ExecutionParams::with_external(
				"env".into(), 
				Arc::new(
					interpreter::env_native_module(env_instance, native_bindings(&mut runtime))
						.map_err(|err| {
							warn!("Error instantiating native bindings: {:?}", err);
							evm::Error::Wasm("Error instantiating native bindings")
						})?
				)
			).add_argument(interpreter::RuntimeValue::I32(d_ptr.as_raw() as i32));

		
			let module_instance = self.program.add_module("contract", contract_module)
				.map_err(|err| {
					warn!("Runtime error instantiating contract module: {:?}", err);
					evm::Error::Wasm("Runtime-error")				
				})?;

			module_instance.execute_export("_call", execution_params)
				.map_err(|err| {
					warn!("Runtime error running contract: {:?}", err);
					evm::Error::Wasm("Runtime-error")				
				})?;
		}

		let result = result::WasmResult::new(d_ptr);
		if result.peek_empty(&*runtime.memory())? {
			Ok(GasLeft::Known(runtime.gas_left()?.into()))
		} else {
			self.result.clear();
			// todo: use memory views to avoid copy
			self.result.extend(result.pop(&*runtime.memory())?);
			Ok(GasLeft::NeedsReturn { 
				gas_left: runtime.gas_left()?.into(), 
				data: &self.result[..],
				apply_state: true,
			})
		}
	}
}

fn native_bindings<'a>(runtime: &'a mut Runtime) -> interpreter::UserFunctions<'a> {
	interpreter::UserFunctions {
		executor: runtime,
		functions: vec![
			interpreter::UserFunction {
				name: "_storage_read".to_owned(),
				params: vec![elements::ValueType::I32, elements::ValueType::I32],
				result: Some(elements::ValueType::I32),
			},
			interpreter::UserFunction {
				name: "_storage_write".to_owned(),
				params: vec![elements::ValueType::I32, elements::ValueType::I32],
				result: Some(elements::ValueType::I32),
			},
			interpreter::UserFunction {
				name: "_malloc".to_owned(),
				params: vec![elements::ValueType::I32],
				result: Some(elements::ValueType::I32),
			},
			interpreter::UserFunction {
				name: "gas".to_owned(),
				params: vec![elements::ValueType::I32],
				result: None,
			},
			interpreter::UserFunction {
				name: "_debug".to_owned(),
				params: vec![elements::ValueType::I32, elements::ValueType::I32],
				result: None,
			},
			interpreter::UserFunction {
				name: "_free".to_owned(),
				params: vec![elements::ValueType::I32],
				result: None,
			},
			interpreter::UserFunction {
				name: "_suicide".to_owned(),
				params: vec![elements::ValueType::I32],
				result: None,
			},
		],
	}
}