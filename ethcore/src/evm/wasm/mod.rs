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

mod runtime;
mod ptr;
mod descriptor;

use std::sync::Arc;

const DEFAULT_STACK_SPACE: u32 = 5 * 1024 * 1024;

use parity_wasm::{interpreter, elements};
use parity_wasm::elements::Deserialize;

use evm::{self, GasLeft};
use action_params::{ActionParams, ActionValue};
use self::runtime::{Runtime, Error as RuntimeError};

pub struct WasmInterpreter {
	program: interpreter::ProgramInstance,
}

impl WasmInterpreter {

	fn new() -> Result<WasmInterpreter, RuntimeError> {
		Ok(WasmInterpreter {
			program: interpreter::ProgramInstance::new()?,
		})
	}

}

impl evm::Evm for WasmInterpreter {

	fn exec(&mut self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result<GasLeft> {

		let env_instance = self.program.module("env")
			.ok_or(evm::Error::Wasm("Env module somehow does not exist in wasm runner runtime"))?;

		let env_memory = env_instance.memory(interpreter::ItemIndex::Internal(0))
			.map_err(|_| evm::Error::Wasm("Linear memory somehow does not exist in wasm runner runtime"))?;
		
		let runtime = Runtime::with_params(
			ext,
			env_memory,
			DEFAULT_STACK_SPACE,
			65546,
		);

		let code = params.code.expect("exec is only called on contract with code; qed");
		let mut cursor = ::std::io::Cursor::new(&*code);
		let contract_module = elements::Module::deserialize(
			&mut cursor
		).map_err(|e| {
			warn!("Error deserializing contract code as wasm module: {:?}", e);
			evm::Error::Wasm("Error deserializing contract code")
		})?;

		Ok(GasLeft::Known(0.into()))
	}
}