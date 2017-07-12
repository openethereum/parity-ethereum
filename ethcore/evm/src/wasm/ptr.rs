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

//! Wasm bound-checked ptr

use parity_wasm::interpreter;

/// Bound-checked wrapper for webassembly memory 
pub struct WasmPtr(u32);

/// Error in bound check
#[derive(Debug)]
pub enum Error {
	AccessViolation,
}

impl From<u32> for WasmPtr {
	fn from(raw: u32) -> Self { 
		WasmPtr(raw)
	}
}

impl WasmPtr {
	// todo: use memory view when they are on
	/// Check memory range and return data with given length starting from the current pointer value
	pub fn slice(&self, len: u32, mem: &interpreter::MemoryInstance) -> Result<Vec<u8>, Error> {
		mem.get(self.0, len as usize).map_err(|_| Error::AccessViolation)
	}

	// todo: maybe 2gb limit can be enhanced
	/// Convert i32 from wasm stack to the wrapped pointer
	pub fn from_i32(raw_ptr: i32) -> Result<Self, Error> {
		if raw_ptr < 0 { return Err(Error::AccessViolation); }
		Ok(WasmPtr(raw_ptr as u32))
	}

	/// Return pointer raw value
	pub fn as_raw(&self) -> u32 { self.0 }
}