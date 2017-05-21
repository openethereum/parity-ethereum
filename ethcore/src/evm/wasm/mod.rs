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

use parity_wasm::interpreter;

use evm::{self, GasLeft};
use action_params::{ActionParams, ActionValue};

pub struct WasmInterpreter;

impl evm::Evm for WasmInterpreter {

    fn exec(&mut self, params: ActionParams, ext: &mut evm::Ext) -> evm::Result<GasLeft> {      

		Ok(GasLeft::Known(0.into()))
    }

}