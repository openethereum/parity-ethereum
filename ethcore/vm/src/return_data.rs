// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Return data structures

use ethereum_types::U256;

/// Return data buffer. Holds memory from a previous call and a slice into that memory.
#[derive(Debug)]
pub struct ReturnData {
	mem: Vec<u8>,
	offset: usize,
	size: usize,
}

impl ::std::ops::Deref for ReturnData {
	type Target = [u8];
	fn deref(&self) -> &[u8] {
		&self.mem[self.offset..self.offset + self.size]
	}
}

impl ReturnData {
	/// Create empty `ReturnData`.
	pub fn empty() -> Self {
		ReturnData {
			mem: Vec::new(),
			offset: 0,
			size: 0,
		}
	}
	/// Create `ReturnData` from give buffer and slice.
	pub fn new(mem: Vec<u8>, offset: usize, size: usize) -> Self {
		ReturnData {
			mem: mem,
			offset: offset,
			size: size,
		}
	}
}

/// Gas Left: either it is a known value, or it needs to be computed by processing
/// a return instruction.
#[derive(Debug)]
pub enum GasLeft {
	/// Known gas left
	Known(U256),
	/// Return or Revert instruction must be processed.
	NeedsReturn {
		/// Amount of gas left.
		gas_left: U256,
		/// Return data buffer.
		data: ReturnData,
		/// Apply or revert state changes on revert.
		apply_state: bool
	},
}
