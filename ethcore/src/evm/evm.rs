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

/// A specialized version of Result over EVM errors.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Gas Left: either it is a known value, or it needs to be computed by processing
/// a return instruction.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GasLeft<'a> {
	/// Known gas left
	Known(U256),
	/// Return instruction must be processed.
	NeedsReturn(U256, &'a [u8]),
}

/// Types that can be "finalized" using an EVM.
///
/// In practice, this is just used to define an inherent impl on
/// `Reult<GasLeft<'a>>`.
pub trait Finalize {
	/// Consume the externalities, call return if necessary, and produce a final amount of gas left.
	fn finalize<E: Ext>(self, ext: E) -> Result<U256>;
}

impl<'a> Finalize for Result<GasLeft<'a>> {
	fn finalize<E: Ext>(self, ext: E) -> Result<U256> {
		match self {
			Ok(GasLeft::Known(gas)) => Ok(gas),
			Ok(GasLeft::NeedsReturn(gas, ret_code)) => ext.ret(&gas, ret_code),
			Err(err) => Err(err),
		}
	}
}

/// Cost calculation type. For low-gas usage we calculate costs using usize instead of U256
pub trait CostType: ops::Mul<Output=Self> + ops::Div<Output=Self> + ops::Add<Output=Self> + ops::Sub<Output=Self> + ops::Shr<usize, Output=Self> + ops::Shl<usize, Output=Self> + cmp::Ord + Sized + From<usize> + Copy {
	/// Converts this cost into `U256`
	fn as_u256(&self) -> U256;
	/// Tries to fit `U256` into this `Cost` type
	fn from_u256(val: U256) -> Result<Self>;
	/// Convert to usize (may panic)
	fn as_usize(&self) -> usize;
	/// Add with overflow
	fn overflow_add(self, other: Self) -> (Self, bool);
	/// Multiple with overflow
	fn overflow_mul(self, other: Self) -> (Self, bool);
}

impl CostType for U256 {
	fn as_u256(&self) -> U256 {
		*self
	}

	fn from_u256(val: U256) -> Result<Self> {
		Ok(val)
	}

	fn as_usize(&self) -> usize {
		self.as_u64() as usize
	}

	fn overflow_add(self, other: Self) -> (Self, bool) {
		Uint::overflowing_add(self, other)
	}

	fn overflow_mul(self, other: Self) -> (Self, bool) {
		Uint::overflowing_mul(self, other)
	}
}

impl CostType for usize {
	fn as_u256(&self) -> U256 {
		U256::from(*self)
	}

	fn from_u256(val: U256) -> Result<Self> {
		if U256::from(val.low_u64()) != val {
			return Err(Error::OutOfGas);
		}
		Ok(val.low_u64() as usize)
	}

	fn as_usize(&self) -> usize {
		*self
	}

	fn overflow_add(self, other: Self) -> (Self, bool) {
		self.overflowing_add(other)
	}

	fn overflow_mul(self, other: Self) -> (Self, bool) {
		self.overflowing_mul(other)
	}
}

/// Evm interface
pub trait Evm {
	/// This function should be used to execute transaction.
	///
	/// It returns either an error, a known amount of gas left, or parameters to be used
	/// to compute the final gas left.
	fn exec(&mut self, params: ActionParams, ext: &mut Ext) -> Result<GasLeft>;
}
