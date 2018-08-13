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

//! Env module glue for wasmi interpreter

use std::cell::RefCell;
use vm::WasmCosts;
use wasmi::{
	self, Signature, Error, FuncRef, FuncInstance, MemoryDescriptor,
	MemoryRef, MemoryInstance, memory_units,
};

/// Internal ids all functions runtime supports. This is just a glue for wasmi interpreter
/// that lacks high-level api and later will be factored out
pub mod ids {
	pub const STORAGE_WRITE_FUNC: usize = 0;
	pub const STORAGE_READ_FUNC: usize = 10;
	pub const RET_FUNC: usize = 20;
	pub const GAS_FUNC: usize = 30;
	pub const FETCH_INPUT_FUNC: usize = 40;
	pub const INPUT_LENGTH_FUNC: usize = 50;
	pub const CCALL_FUNC: usize = 60;
	pub const SCALL_FUNC: usize = 70;
	pub const DCALL_FUNC: usize = 80;
	pub const VALUE_FUNC: usize = 90;
	pub const CREATE_FUNC: usize = 100;
	pub const SUICIDE_FUNC: usize = 110;
	pub const BLOCKHASH_FUNC: usize = 120;
	pub const BLOCKNUMBER_FUNC: usize = 130;
	pub const COINBASE_FUNC: usize = 140;
	pub const DIFFICULTY_FUNC: usize = 150;
	pub const GASLIMIT_FUNC: usize = 160;
	pub const TIMESTAMP_FUNC: usize = 170;
	pub const ADDRESS_FUNC: usize = 180;
	pub const SENDER_FUNC: usize = 190;
	pub const ORIGIN_FUNC: usize = 200;
	pub const ELOG_FUNC: usize = 210;
	pub const CREATE2_FUNC: usize = 220;

	pub const PANIC_FUNC: usize = 1000;
	pub const DEBUG_FUNC: usize = 1010;
}

/// Signatures of all functions runtime supports. The actual dispatch happens at
/// impl runtime::Runtime methods.
pub mod signatures {
	use wasmi::{self, ValueType};
	use wasmi::ValueType::*;

	pub struct StaticSignature(pub &'static [ValueType], pub Option<ValueType>);

	pub const STORAGE_READ: StaticSignature = StaticSignature(
		&[I32, I32],
		None,
	);

	pub const STORAGE_WRITE: StaticSignature = StaticSignature(
		&[I32, I32],
		None,
	);

	pub const RET: StaticSignature = StaticSignature(
		&[I32, I32],
		None,
	);

	pub const GAS: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const FETCH_INPUT: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const INPUT_LENGTH: StaticSignature = StaticSignature(
		&[],
		Some(I32),
	);

	pub const CCALL: StaticSignature = StaticSignature(
		&[I64, I32, I32, I32, I32, I32, I32],
		Some(I32),
	);

	pub const DCALL: StaticSignature = StaticSignature(
		&[I64, I32, I32, I32, I32, I32],
		Some(I32),
	);

	pub const SCALL: StaticSignature = StaticSignature(
		&[I64, I32, I32, I32, I32, I32],
		Some(I32),
	);

	pub const PANIC: StaticSignature = StaticSignature(
		&[I32, I32],
		None,
	);

	pub const DEBUG: StaticSignature = StaticSignature(
		&[I32, I32],
		None,
	);

	pub const VALUE: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const CREATE: StaticSignature = StaticSignature(
		&[I32, I32, I32, I32],
		Some(I32),
	);

	pub const CREATE2: StaticSignature = StaticSignature(
		&[I32, I32, I32, I32, I32],
		Some(I32),
	);

	pub const SUICIDE: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const BLOCKHASH: StaticSignature = StaticSignature(
		&[I64, I32],
		None,
	);

	pub const BLOCKNUMBER: StaticSignature = StaticSignature(
		&[],
		Some(I64),
	);

	pub const COINBASE: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const DIFFICULTY: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const GASLIMIT: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const TIMESTAMP: StaticSignature = StaticSignature(
		&[],
		Some(I64),
	);

	pub const ADDRESS: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const SENDER: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const ORIGIN: StaticSignature = StaticSignature(
		&[I32],
		None,
	);

	pub const ELOG: StaticSignature = StaticSignature(
		&[I32, I32, I32, I32],
		None,
	);

	impl Into<wasmi::Signature> for StaticSignature {
		fn into(self) -> wasmi::Signature {
			wasmi::Signature::new(self.0, self.1)
		}
	}
}

fn host(signature: signatures::StaticSignature, idx: usize) -> FuncRef {
	FuncInstance::alloc_host(signature.into(), idx)
}

/// Import resolver for wasmi
/// Maps all functions that runtime support to the corresponding contract import
/// entries.
/// Also manages initial memory request from the runtime.
pub struct ImportResolver {
	max_memory: u32,
	memory: RefCell<Option<MemoryRef>>,

	have_create2: bool,
}

impl ImportResolver {
	/// New import resolver with specifed maximum amount of inital memory (in wasm pages = 64kb)
	pub fn with_limit(max_memory: u32, schedule: &WasmCosts) -> ImportResolver {
		ImportResolver {
			max_memory: max_memory,
			memory: RefCell::new(None),

			have_create2: schedule.have_create2,
		}
	}

	/// Returns memory that was instantiated during the contract module
	/// start. If contract does not use memory at all, the dummy memory of length (0, 0)
	/// will be created instead. So this method always returns memory instance
	/// unless errored.
	pub fn memory_ref(&self) -> MemoryRef {
		{
			let mut mem_ref = self.memory.borrow_mut();
			if mem_ref.is_none() {
				*mem_ref = Some(
					MemoryInstance::alloc(
						memory_units::Pages(0),
						Some(memory_units::Pages(0)),
					).expect("Memory allocation (0, 0) should not fail; qed")
				);
			}
		}

		self.memory.borrow().clone().expect("it is either existed or was created as (0, 0) above; qed")
	}

	/// Returns memory size module initially requested
	pub fn memory_size(&self) -> Result<u32, Error> {
		Ok(self.memory_ref().current_size().0 as u32)
	}
}

impl wasmi::ModuleImportResolver for ImportResolver {
	fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, Error> {
		let func_ref = match field_name {
			"storage_read" => host(signatures::STORAGE_READ, ids::STORAGE_READ_FUNC),
			"storage_write" => host(signatures::STORAGE_WRITE, ids::STORAGE_WRITE_FUNC),
			"ret" => host(signatures::RET, ids::RET_FUNC),
			"gas" => host(signatures::GAS, ids::GAS_FUNC),
			"input_length" => host(signatures::INPUT_LENGTH, ids::INPUT_LENGTH_FUNC),
			"fetch_input" => host(signatures::FETCH_INPUT, ids::FETCH_INPUT_FUNC),
			"panic" => host(signatures::PANIC, ids::PANIC_FUNC),
			"debug" => host(signatures::DEBUG, ids::DEBUG_FUNC),
			"ccall" => host(signatures::CCALL, ids::CCALL_FUNC),
			"dcall" => host(signatures::DCALL, ids::DCALL_FUNC),
			"scall" => host(signatures::SCALL, ids::SCALL_FUNC),
			"value" => host(signatures::VALUE, ids::VALUE_FUNC),
			"create" => host(signatures::CREATE, ids::CREATE_FUNC),
			"suicide" => host(signatures::SUICIDE, ids::SUICIDE_FUNC),
			"blockhash" => host(signatures::BLOCKHASH, ids::BLOCKHASH_FUNC),
			"blocknumber" => host(signatures::BLOCKNUMBER, ids::BLOCKNUMBER_FUNC),
			"coinbase" => host(signatures::COINBASE, ids::COINBASE_FUNC),
			"difficulty" => host(signatures::DIFFICULTY, ids::DIFFICULTY_FUNC),
			"gaslimit" => host(signatures::GASLIMIT, ids::GASLIMIT_FUNC),
			"timestamp" => host(signatures::TIMESTAMP, ids::TIMESTAMP_FUNC),
			"address" => host(signatures::ADDRESS, ids::ADDRESS_FUNC),
			"sender" => host(signatures::SENDER, ids::SENDER_FUNC),
			"origin" => host(signatures::ORIGIN, ids::ORIGIN_FUNC),
			"elog" => host(signatures::ELOG, ids::ELOG_FUNC),
			"create2" if self.have_create2 => host(signatures::CREATE2, ids::CREATE2_FUNC),
			_ => {
				return Err(wasmi::Error::Instantiation(
					format!("Export {} not found", field_name),
				))
			}
		};

		Ok(func_ref)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		descriptor: &MemoryDescriptor,
	) -> Result<MemoryRef, Error> {
		if field_name == "memory" {
			let effective_max = descriptor.maximum().unwrap_or(self.max_memory + 1);
			if descriptor.initial() > self.max_memory || effective_max > self.max_memory
			{
				Err(Error::Instantiation("Module requested too much memory".to_owned()))
			} else {
				let mem = MemoryInstance::alloc(
					memory_units::Pages(descriptor.initial() as usize),
					descriptor.maximum().map(|x| memory_units::Pages(x as usize)),
				)?;
				*self.memory.borrow_mut() = Some(mem.clone());
				Ok(mem)
			}
		} else {
			Err(Error::Instantiation("Memory imported under unknown name".to_owned()))
		}
	}
}
