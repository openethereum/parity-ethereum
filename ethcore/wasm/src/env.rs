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

//! Wasm env module bindings

use parity_wasm::elements::ValueType::*;
use parity_wasm::interpreter::{self, UserFunctionDescriptor};
use parity_wasm::interpreter::UserFunctionDescriptor::*;
use super::runtime::{Runtime, UserTrap};

pub const SIGNATURES: &'static [UserFunctionDescriptor] = &[
	Static(
		"_storage_read",
		&[I32; 2],
		None,
	),
	Static(
		"_storage_write",
		&[I32; 2],
		None,
	),
	Static(
		"_balance",
		&[I32; 2],
		None,
	),
	Static(
		"_ext_malloc",
		&[I32],
		Some(I32),
	),
	Static(
		"_ext_free",
		&[I32],
		None,
	),
	Static(
		"gas",
		&[I32],
		None,
	),
	Static(
		"_debug",
		&[I32; 2],
		None,
	),
	Static(
		"_suicide",
		&[I32],
		None,
	),
	Static(
		"_create",
		&[I32; 4],
		Some(I32),
	),
	Static(
		"_ccall",
		&[I32; 6],
		Some(I32),
	),
	Static(
		"_dcall",
		&[I32; 5],
		Some(I32),
	),
	Static(
		"_scall",
		&[I32; 5],
		Some(I32),
	),
	Static(
		"abort",
		&[I32],
		None,
	),
	Static(
		"_emscripten_memcpy_big",
		&[I32; 3],
		Some(I32),
	),
	Static(
		"_ext_memcpy",
		&[I32; 3],
		Some(I32),
	),
	Static(
		"_ext_memset",
		&[I32; 3],
		Some(I32),
	),
	Static(
		"_ext_memmove",
		&[I32; 3],
		Some(I32),
	),
	Static(
		"_panic",
		&[I32; 2],
		None,
	),
	Static(
		"_blockhash",
		&[I64, I32],
		None,
	),
	Static(
		"_coinbase",
		&[I32],
		None,
	),
	Static(
		"_sender",
		&[I32],
		None,
	),
	Static(
		"_origin",
		&[I32],
		None,
	),
	Static(
		"_address",
		&[I32],
		None,
	),
	Static(
		"_value",
		&[I32],
		None,
	),
	Static(
		"_timestamp",
		&[],
		Some(I64),
	),
	Static(
		"_blocknumber",
		&[],
		Some(I64),
	),
	Static(
		"_difficulty",
		&[I32],
		None,
	),
	Static(
		"_gaslimit",
		&[I32],
		None,
	),
	Static(
		"_elog",
		&[I32; 4],
		None,
	),

	// TODO: Get rid of it also somehow?
	Static(
		"_llvm_trap",
		&[I32; 0],
		None
	),

	Static(
		"_llvm_bswap_i64",
		&[I64],
		Some(I64)
	),
];

pub fn native_bindings<'a>(runtime: &'a mut Runtime) -> interpreter::UserDefinedElements<'a, UserTrap> {
	interpreter::UserDefinedElements {
		executor: Some(runtime),
		globals: ::std::collections::HashMap::new(),
		functions: ::std::borrow::Cow::from(SIGNATURES),
	}
}
