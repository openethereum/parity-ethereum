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

//! Wasm evm results helper

use byteorder::{LittleEndian, ByteOrder};

use parity_wasm::interpreter;

use super::ptr::WasmPtr;
use super::runtime::Error as RuntimeError;

/// Wrapper for wasm contract call result
pub struct WasmResult {
	ptr: WasmPtr,
}

impl WasmResult {
	/// New call result from given ptr
	pub fn new(descriptor_ptr: WasmPtr) -> WasmResult {
		WasmResult { ptr: descriptor_ptr }
	}

	/// Check if the result contains any data
	pub fn peek_empty(&self, mem: &interpreter::MemoryInstance) -> Result<bool, RuntimeError> {
		let result_len = LittleEndian::read_u32(&self.ptr.slice(16, mem)?[12..16]);
		Ok(result_len == 0)
	}

	/// Consume the result ptr and return the actual data from wasm linear memory
	pub fn pop(self, mem: &interpreter::MemoryInstance) -> Result<Vec<u8>, RuntimeError> {
		let result_ptr = LittleEndian::read_u32(&self.ptr.slice(16, mem)?[8..12]);
		let result_len = LittleEndian::read_u32(&self.ptr.slice(16, mem)?[12..16]);
		trace!(target: "wasm", "contract result: {} bytes at @{}", result_len, result_ptr);

		Ok(mem.get(result_ptr, result_len as usize)?)
	}
}