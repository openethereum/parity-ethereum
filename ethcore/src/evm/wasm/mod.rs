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
mod env;

use std::sync::Arc;

const DEFAULT_STACK_SPACE: u32 = 5 * 1024 * 1024;

use parity_wasm::{interpreter, elements};
use parity_wasm::interpreter::ModuleInstanceInterface;
use wasm_utils;

use evm::{self, GasLeft, ReturnData};
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

		let code = params.code.expect("exec is only called on contract with code; qed");

		trace!(target: "wasm", "Started wasm interpreter with code.len={:?}", code.len());

		let env_instance = self.program.module("env")
			// prefer explicit panic here
			.expect("Wasm program to contain env module");

		let env_memory = env_instance.memory(interpreter::ItemIndex::Internal(0))
			// prefer explicit panic here
			.expect("Linear memory to exist in wasm runtime");

		if params.gas > ::std::u64::MAX.into() {
			return Err(evm::Error::Wasm("Wasm interpreter cannot run contracts with gas >= 2^64".to_owned()));
		}
		
		let mut runtime = Runtime::with_params(
			ext,
			env_memory,
			DEFAULT_STACK_SPACE,
			params.gas.low_u64(),
		);

		let mut cursor = ::std::io::Cursor::new(&*code);

		let contract_module = wasm_utils::inject_gas_counter(
			elements::Module::deserialize(
				&mut cursor
			).map_err(|err| {
				evm::Error::Wasm(format!("Error deserializing contract code ({:?})", err))
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
							// todo: prefer explicit panic here also?
							evm::Error::Wasm(format!("Error instantiating native bindings: {:?}", err))
						})?
				)
			).add_argument(interpreter::RuntimeValue::I32(d_ptr.as_raw() as i32));
		
			let module_instance = self.program.add_module("contract", contract_module, Some(&execution_params.externals))
				.map_err(|err| {
					trace!(target: "wasm", "Error adding contract module: {:?}", err);
					evm::Error::from(RuntimeError::Interpreter(err))
				})?;

			module_instance.execute_export("_call", execution_params)
				.map_err(|err| {
					trace!(target: "wasm", "Error executing contract: {:?}", err);
					evm::Error::from(RuntimeError::Interpreter(err))
				})?;
		}

		let result = result::WasmResult::new(d_ptr);
		if result.peek_empty(&*runtime.memory())? {
			trace!(target: "wasm", "Contract execution result is empty.");
			Ok(GasLeft::Known(runtime.gas_left()?.into()))
		} else {
			self.result.clear();
			// todo: use memory views to avoid copy
			self.result.extend(result.pop(&*runtime.memory())?);
			let len = self.result.len();
			Ok(GasLeft::NeedsReturn { 
				gas_left: runtime.gas_left()?.into(), 
				data: ReturnData::new(
					::std::mem::replace(&mut self.result, Vec::with_capacity(DEFAULT_RESULT_BUFFER)), 
					0, 
					len,
				),
				apply_state: true,
			})
		}
	}
}

fn native_bindings<'a>(runtime: &'a mut Runtime) -> interpreter::UserFunctions<'a> {
	interpreter::UserFunctions {
		executor: runtime,
		functions: ::std::borrow::Cow::from(env::SIGNATURES),
	}
}