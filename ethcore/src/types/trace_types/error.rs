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
use rlp::{RlpEncodable, Encodable, RlpStream, Decodable, Decoder, DecoderError, View};
use evm::Error as EvmError;

/// Trace evm errors.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "ipc", binary)]
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
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	Internal,
}

impl<'a> From<&'a EvmError> for Error {
	fn from(e: &'a EvmError) -> Self {
		match *e {
			EvmError::OutOfGas => Error::OutOfGas,
			EvmError::BadJumpDestination { .. } => Error::BadJumpDestination,
			EvmError::BadInstruction { .. } => Error::BadInstruction,
			EvmError::StackUnderflow { .. } => Error::StackUnderflow,
			EvmError::OutOfStack { .. } => Error::OutOfStack,
			EvmError::Internal(_) => Error::Internal,
		}
	}
}

impl From<EvmError> for Error {
	fn from(e: EvmError) -> Self {
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
			Internal => "Internal error",
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
		};
		RlpEncodable::rlp_append(&value, s);
	}
}

impl Decodable for Error {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		use self::Error::*;
		let value: u8 = decoder.as_rlp().as_val()?;
		match value {
			0 => Ok(OutOfGas),
			1 => Ok(BadJumpDestination),
			2 => Ok(BadInstruction),
			3 => Ok(StackUnderflow),
			4 => Ok(OutOfStack),
			5 => Ok(Internal),
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
