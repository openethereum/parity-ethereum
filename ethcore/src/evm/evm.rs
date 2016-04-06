// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Evm interface.

use common::*;
use evm::Ext;

/// Evm errors.
#[derive(Debug)]
pub enum Error {
	/// `OutOfGas` is returned when transaction execution runs out of gas.
	/// The state should be reverted to the state from before the
	/// transaction execution. But it does not mean that transaction
	/// was invalid. Balance still should be transfered and nonce
	/// should be increased.
	OutOfGas,
	/// `BadJumpDestination` is returned when execution tried to move
	/// to position that wasn't marked with JUMPDEST instruction
	BadJumpDestination {
		/// Position the code tried to jump to.
		destination: usize
	},
	/// `BadInstructions` is returned when given instruction is not supported
	BadInstruction {
		/// Unrecognized opcode
		instruction: u8,
	},
	/// `StackUnderflow` when there is not enough stack elements to execute instruction
	StackUnderflow {
		/// Invoked instruction
		instruction: &'static str,
		/// How many stack elements was requested by instruction
		wanted: usize,
		/// How many elements were on stack
		on_stack: usize
	},
	/// When execution would exceed defined Stack Limit
	OutOfStack {
		/// Invoked instruction
		instruction: &'static str,
		/// How many stack elements instruction wanted to push
		wanted: usize,
		/// What was the stack limit
		limit: usize
	},
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	#[allow(dead_code)] // created only by jit
	Internal,
}

/// Evm result.
///
/// Returns `gas_left` if execution is successful, otherwise error.
pub type Result = result::Result<U256, Error>;

/// Evm interface.
pub trait Evm {
	/// This function should be used to execute transaction.
	fn exec(&self, params: ActionParams, ext: &mut Ext) -> Result;
}
