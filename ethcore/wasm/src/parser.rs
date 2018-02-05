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

//! ActionParams parser for wasm

use vm;
use wasm_utils::{self, rules};
use parity_wasm::elements::{self, Deserialize};
use parity_wasm::peek_size;

fn gas_rules(schedule: &vm::Schedule) -> rules::Set {
	rules::Set::new({
		let mut vals = ::std::collections::HashMap::with_capacity(4);
		vals.insert(rules::InstructionType::Load, schedule.wasm.mem as u32);
		vals.insert(rules::InstructionType::Store, schedule.wasm.mem as u32);
		vals.insert(rules::InstructionType::Div, schedule.wasm.div as u32);
		vals.insert(rules::InstructionType::Mul, schedule.wasm.mul as u32);
		vals
	}).with_grow_cost(schedule.wasm.grow_mem)
}

/// Splits payload to code and data according to params.params_type, also
/// loads the module instance from payload and injects gas counter according
/// to schedule.
pub fn payload<'a>(params: &'a vm::ActionParams, schedule: &vm::Schedule)
	-> Result<(elements::Module, &'a [u8]), vm::Error>
{
	let code = match params.code {
		Some(ref code) => &code[..],
		None => { return Err(vm::Error::Wasm("Invalid wasm call".to_owned())); }
	};

	let (mut cursor, data_position) = match params.params_type {
		vm::ParamsType::Embedded => {
			let module_size = peek_size(&*code);
			(
				::std::io::Cursor::new(&code[..module_size]),
				module_size
			)
		},
		vm::ParamsType::Separate => {
			(::std::io::Cursor::new(&code[..]), 0)
		},
	};

	let contract_module = wasm_utils::inject_gas_counter(
		elements::Module::deserialize(
			&mut cursor
		).map_err(|err| {
			vm::Error::Wasm(format!("Error deserializing contract code ({:?})", err))
		})?,
		&gas_rules(schedule),
	);

	let data = match params.params_type {
		vm::ParamsType::Embedded => {
			if data_position < code.len() { &code[data_position..] } else { &[] }
		},
		vm::ParamsType::Separate => {
			match params.data {
				Some(ref s) => &s[..],
				None => &[]
			}
		}
	};

	Ok((contract_module, data))
}