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

//! VM errors module

use trie;
use std::fmt;

/// VM errors.
#[derive(Debug, Clone, PartialEq)]
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
	/// Built-in contract failed on given input
	BuiltIn(&'static str),
	/// When execution tries to modify the state in static context
	MutableCallInStaticContext,
	/// Likely to cause consensus issues.
	Internal(String),
	/// Wasm runtime error
	Wasm(String),
	/// Out of bounds access in RETURNDATACOPY.
	OutOfBounds,
	/// Execution has been reverted with REVERT.
	Reverted,
}

impl From<Box<trie::TrieError>> for Error {
	fn from(err: Box<trie::TrieError>) -> Self {
		Error::Internal(format!("Internal error: {}", err))
	}
}

// impl From<wasm::RuntimeError> for Error {
// 	fn from(err: wasm::RuntimeError) -> Self {
// 		Error::Wasm(format!("Runtime error: {:?}", err))
// 	}
// }

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Error::*;
		match *self {
			OutOfGas => write!(f, "Out of gas"),
			BadJumpDestination { destination } => write!(f, "Bad jump destination {:x}", destination),
			BadInstruction { instruction } => write!(f, "Bad instruction {:x}",  instruction),
			StackUnderflow { instruction, wanted, on_stack } => write!(f, "Stack underflow {} {}/{}", instruction, wanted, on_stack),
			OutOfStack { instruction, wanted, limit } => write!(f, "Out of stack {} {}/{}", instruction, wanted, limit),
			BuiltIn(name) => write!(f, "Built-in failed: {}", name),
			Internal(ref msg) => write!(f, "Internal error: {}", msg),
			MutableCallInStaticContext => write!(f, "Mutable call in static context"),
			Wasm(ref msg) => write!(f, "Internal error: {}", msg),
			OutOfBounds => write!(f, "Out of bounds"),
			Reverted => write!(f, "Reverted"),
		}
	}
}

pub type Result<T> = ::std::result::Result<T, Error>;
