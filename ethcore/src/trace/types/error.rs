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

//! Trace errors.

use std::fmt;
use rlp::{Encodable, RlpStream, Decodable, DecoderError, UntrustedRlp};
use vm::Error as VmError;

/// Trace evm errors.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
	/// `OutOfGas` is returned when transaction execution runs out of gas.
	OutOfGas,
	/// `BadJumpDestination` is returned when execution tried to move
	/// to position that wasn't marked with JUMPDEST instruction
	BadJumpDestination,
	/// `BadInstructions` is returned when given instruction is not supported
	BadInstruction,
	/// `StackUnderflow` when there is not enough stack elements to execute instruction
	StackUnderflow,
	/// When execution would exceed defined Stack Limit
	OutOfStack,
	/// When builtin contract failed on input data
	BuiltIn,
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	Internal,
	/// When execution tries to modify the state in static context
	MutableCallInStaticContext,
	/// Wasm error
	Wasm,
}

impl<'a> From<&'a VmError> for Error {
	fn from(e: &'a VmError) -> Self {
		match *e {
			VmError::OutOfGas => Error::OutOfGas,
			VmError::BadJumpDestination { .. } => Error::BadJumpDestination,
			VmError::BadInstruction { .. } => Error::BadInstruction,
			VmError::StackUnderflow { .. } => Error::StackUnderflow,
			VmError::OutOfStack { .. } => Error::OutOfStack,
			VmError::BuiltIn { .. } => Error::BuiltIn,
			VmError::Wasm { .. } => Error::Wasm,
			VmError::Internal(_) => Error::Internal,
			VmError::MutableCallInStaticContext => Error::MutableCallInStaticContext,
		}
	}
}

impl From<VmError> for Error {
	fn from(e: VmError) -> Self {
		Error::from(&e)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Error::*;
		let message = match *self {
			OutOfGas => "Out of gas",
			BadJumpDestination => "Bad jump destination",
			BadInstruction => "Bad instruction",
			StackUnderflow => "Stack underflow",
			OutOfStack => "Out of stack",
			BuiltIn => "Built-in failed",
			Wasm => "Wasm runtime error",
			Internal => "Internal error",
			MutableCallInStaticContext => "Mutable Call In Static Context",
		};
		message.fmt(f)
	}
}

impl Encodable for Error {
	fn rlp_append(&self, s: &mut RlpStream) {
		use self::Error::*;
		let value = match *self {
			OutOfGas => 0u8,
			BadJumpDestination => 1,
			BadInstruction => 2,
			StackUnderflow => 3,
			OutOfStack => 4,
			Internal => 5,
			BuiltIn => 6,
			MutableCallInStaticContext => 7,
			Wasm => 8,
		};

		s.append_internal(&value);
	}
}

impl Decodable for Error {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		use self::Error::*;
		let value: u8 = rlp.as_val()?;
		match value {
			0 => Ok(OutOfGas),
			1 => Ok(BadJumpDestination),
			2 => Ok(BadInstruction),
			3 => Ok(StackUnderflow),
			4 => Ok(OutOfStack),
			5 => Ok(Internal),
			6 => Ok(BuiltIn),
			7 => Ok(MutableCallInStaticContext),
			8 => Ok(Wasm),
			_ => Err(DecoderError::Custom("Invalid error type")),
		}
	}
}

#[cfg(test)]
mod tests {
	use rlp::*;
	use super::Error;

	#[test]
	fn encode_error() {
		let err = Error::BadJumpDestination;

		let mut s = RlpStream::new_list(2);
		s.append(&err);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&err);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}
}
