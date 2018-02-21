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

extern crate byteorder;
extern crate ethcore_logger;
extern crate ethereum_types;
#[macro_use] extern crate log;
extern crate libc;
extern crate parity_wasm;
extern crate vm;
extern crate wasm_utils;
extern crate wasmi;

mod runtime;
#[cfg(test)]
mod tests;
mod env;
mod panic_payload;
mod parser;

use vm::{GasLeft, ReturnData, ActionParams};
use wasmi::Error as InterpreterError;

use runtime::{Runtime, RuntimeContext};

use ethereum_types::U256;

/// Wrapped interpreter error
#[derive(Debug)]
pub struct Error(InterpreterError);

impl From<InterpreterError> for Error {
	fn from(e: InterpreterError) -> Self {
		Error(e)
	}
}

impl From<Error> for vm::Error {
	fn from(e: Error) -> Self {
		vm::Error::Wasm(format!("Wasm runtime error: {:?}", e.0))
	}
}

/// Wasm interpreter instance
pub struct WasmInterpreter;

impl From<runtime::Error> for vm::Error {
	fn from(e: runtime::Error) -> Self {
		vm::Error::Wasm(format!("Wasm runtime error: {:?}", e))
	}
}

impl vm::Vm for WasmInterpreter {

	fn exec(&mut self, params: ActionParams, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
		let (module, data) = parser::payload(&params, ext.schedule().wasm())?;

		let loaded_module = wasmi::Module::from_parity_wasm_module(module).map_err(Error)?;

		let instantiation_resolover = env::ImportResolver::with_limit(16);

		let module_instance = wasmi::ModuleInstance::new(
			&loaded_module,
			&wasmi::ImportsBuilder::new().with_resolver("env", &instantiation_resolover)
		).map_err(Error)?;

		let adjusted_gas = params.gas * U256::from(ext.schedule().wasm().opcodes_div) /
			U256::from(ext.schedule().wasm().opcodes_mul);

		if adjusted_gas > ::std::u64::MAX.into()
		{
			return Err(vm::Error::Wasm("Wasm interpreter cannot run contracts with gas (wasm adjusted) >= 2^64".to_owned()));
		}

		let initial_memory = instantiation_resolover.memory_size().map_err(Error)?;
		trace!(target: "wasm", "Contract requested {:?} pages of initial memory", initial_memory);

		let (gas_left, result) = {
			let mut runtime = Runtime::with_params(
				ext,
				instantiation_resolover.memory_ref(),
				// cannot overflow, checked above
				adjusted_gas.low_u64(),
				data.to_vec(),
				RuntimeContext {
					address: params.address,
					sender: params.sender,
					origin: params.origin,
					code_address: params.code_address,
					value: params.value.value(),
				},
			);

			// cannot overflow if static_region < 2^16,
			// initial_memory ∈ [0..2^32)
			// total_charge <- static_region * 2^32 * 2^16
			// total_charge ∈ [0..2^64) if static_region ∈ [0..2^16)
			// qed
			assert!(runtime.schedule().wasm().initial_mem < 1 << 16);
			runtime.charge(|s| initial_memory as u64 * s.wasm().initial_mem as u64)?;

			let module_instance = module_instance.run_start(&mut runtime).map_err(Error)?;

			match module_instance.invoke_export("call", &[], &mut runtime) {
				Ok(_) => { },
				Err(InterpreterError::Host(boxed)) => {
					match boxed.downcast_ref::<runtime::Error>() {
						None => {
							return Err(vm::Error::Wasm("Invalid user error used in interpreter".to_owned()));
						}
						Some(runtime_err) => {
							match *runtime_err {
								runtime::Error::Suicide => {
									// Suicide uses trap to break execution
								}
								ref any_err => {
									trace!(target: "wasm", "Error executing contract: {:?}", boxed);
									return Err(vm::Error::from(Error::from(InterpreterError::Host(Box::new(any_err.clone())))));
								}
							}
						}
					}
				},
				Err(err) => {
					trace!(target: "wasm", "Error executing contract: {:?}", err);
					return Err(vm::Error::from(Error::from(err)))
				}
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
