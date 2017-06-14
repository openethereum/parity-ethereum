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
use parity_wasm::interpreter::UserFunctionDescriptor::*;
use parity_wasm::interpreter::UserFunction;

pub const SIGNATURES: &'static [UserFunction] = &[
	UserFunction {
		desc: Static(
			"_storage_read",
			&[I32; 2],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"_storage_write",
			&[I32; 2],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"_malloc",
			&[I32],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"_free",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"gas",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"_debug",
			&[I32; 2],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"_suicide",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"_create",
			&[I32; 4],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"abort",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"_abort",
			&[],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"invoke_vii",
			&[I32; 3],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"invoke_vi",
			&[I32; 2],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"invoke_v",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"invoke_iii",
			&[I32; 3],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"___resumeException",
			&[I32],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"_rust_begin_unwind",
			&[I32; 4],
		),
		result: None,
	},
	UserFunction {
		desc: Static(
			"___cxa_find_matching_catch_2",
			&[],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"___gxx_personality_v0",
			&[I32; 6],
		),
		result: Some(I32),
	},
	UserFunction {
		desc: Static(
			"_emscripten_memcpy_big",
			&[I32; 3],
		),
		result: Some(I32),
	},
];
