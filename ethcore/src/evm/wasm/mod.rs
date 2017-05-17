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

extern crate parity_wasm;

use self::parity_wasm::interpreter;

use evm::{self, GasLeft};
use action_params::{ActionParams, ActionValue};

pub struct WasmInterpreter;

pub struct WasmRuntime<'a> {
    ext: &'a mut evm::Ext,
}

impl<'a> interpreter::UserFunctionInterface for WasmRuntime<'a> {
    fn call(&mut self, module: &interpreter::ModuleInstance, context: interpreter::CallerContext) 
        -> Result<Option<interpreter::RuntimeValue>, interpreter::Error>
    {
        Ok(None)
    }
}

impl evm::Evm for WasmInterpreter {

    fn exec(&mut self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result<GasLeft> {

        let wasm_runtime = WasmRuntime { ext: ext };

        let mut user_functions = interpreter::UserFunctions::new();
        user_functions.insert("__invoke".to_owned(), 
            interpreter::UserFunction {
                params: vec![],
                result: None,
                closure: Box::new(wasm_runtime),
            }
        );

        let program = interpreter::ProgramInstance::with_functions(user_functions)
            .expect("Program instance to be created");        

		Ok(GasLeft::Known(0.into()))
    }

}
