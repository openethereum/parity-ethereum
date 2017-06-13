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

//! Evm interface.

use std::{ops, cmp, fmt};
use util::{U128, U256, U512, trie};
use action_params::ActionParams;
use evm::Ext;
use builtin;

/// Evm errors.
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
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	Internal(String),
}

impl From<Box<trie::TrieError>> for Error {
	fn from(err: Box<trie::TrieError>) -> Self {
		Error::Internal(format!("Internal error: {}", err))
	}
}

impl From<builtin::Error> for Error {
	fn from(err: builtin::Error) -> Self {
		Error::BuiltIn(err.0)
	}
}

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
		}
	}
}

/// A specialized version of Result over EVM errors.
pub type Result<T> = ::std::result::Result<T, Error>;


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

/// Finalization result. Gas Left: either it is a known value, or it needs to be computed by processing
/// a return instruction.
#[derive(Debug)]
pub struct FinalizationResult {
	/// Final amount of gas left.
	pub gas_left: U256,
	/// Apply execution state changes or revert them.
	pub apply_state: bool,
	/// Return data buffer.
	pub return_data: ReturnData,
}

/// Types that can be "finalized" using an EVM.
///
/// In practice, this is just used to define an inherent impl on
/// `Reult<GasLeft<'a>>`.
pub trait Finalize {
	/// Consume the externalities, call return if necessary, and produce call result.
	fn finalize<E: Ext>(self, ext: E) -> Result<FinalizationResult>;
}

impl Finalize for Result<GasLeft> {
	fn finalize<E: Ext>(self, ext: E) -> Result<FinalizationResult> {
		match self {
			Ok(GasLeft::Known(gas_left)) => Ok(FinalizationResult { gas_left: gas_left, apply_state: true, return_data: ReturnData::empty() }),
			Ok(GasLeft::NeedsReturn {gas_left, data, apply_state}) => ext.ret(&gas_left, &data).map(|gas_left| FinalizationResult {
				gas_left: gas_left,
				apply_state: apply_state,
				return_data: data,
			}),
			Err(err) => Err(err),
		}
	}
}

/// Cost calculation type. For low-gas usage we calculate costs using usize instead of U256
pub trait CostType: Sized + From<usize> + Copy
	+ ops::Mul<Output=Self> + ops::Div<Output=Self> + ops::Add<Output=Self> +ops::Sub<Output=Self>
	+ ops::Shr<usize, Output=Self> + ops::Shl<usize, Output=Self>
	+ cmp::Ord + fmt::Debug {
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
	/// Single-step full multiplication and shift: `(self*other) >> shr`
	/// Should not overflow on intermediate steps
	fn overflow_mul_shr(self, other: Self, shr: usize) -> (Self, bool);
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
		self.overflowing_add(other)
	}

	fn overflow_mul(self, other: Self) -> (Self, bool) {
		self.overflowing_mul(other)
	}

	fn overflow_mul_shr(self, other: Self, shr: usize) -> (Self, bool) {
		let x = self.full_mul(other);
		let U512(parts) = x;
		let overflow = (parts[4] | parts[5] | parts[6] | parts[7]) > 0;
		let U512(parts) = x >> shr;
		(
			U256([parts[0], parts[1], parts[2], parts[3]]),
			overflow
		)
	}
}

impl CostType for usize {
	fn as_u256(&self) -> U256 {
		U256::from(*self)
	}

	fn from_u256(val: U256) -> Result<Self> {
		let res = val.low_u64() as usize;

		// validate if value fits into usize
		if U256::from(res) != val {
			return Err(Error::OutOfGas);
		}

		Ok(res)
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

	fn overflow_mul_shr(self, other: Self, shr: usize) -> (Self, bool) {
		let (c, o) = U128::from(self).overflowing_mul(U128::from(other));
		let U128(parts) = c;
		let overflow = o | (parts[1] > 0);
		let U128(parts) = c >> shr;
		let result = parts[0] as usize;
		let overflow = overflow | (parts[0] > result as u64);
		(result, overflow)
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

#[cfg(test)]
mod tests {
	use util::U256;
	use super::CostType;

	#[test]
	fn should_calculate_overflow_mul_shr_without_overflow() {
		// given
		let num = 1048576;

		// when
		let (res1, o1) = U256::from(num).overflow_mul_shr(U256::from(num), 20);
		let (res2, o2) = num.overflow_mul_shr(num, 20);

		// then
		assert_eq!(res1, U256::from(num));
		assert!(!o1);
		assert_eq!(res2, num);
		assert!(!o2);
	}

	#[test]
	fn should_calculate_overflow_mul_shr_with_overflow() {
		// given
		let max = u64::max_value();
		let num1 = U256([max, max, max, max]);
		let num2 = usize::max_value();

		// when
		let (res1, o1) = num1.overflow_mul_shr(num1, 256);
		let (res2, o2) = num2.overflow_mul_shr(num2, 64);

		// then
		assert_eq!(res2, num2 - 1);
		assert!(o2);

		assert_eq!(res1, !U256::zero() - U256::one());
		assert!(o1);
	}

	#[test]
	fn should_validate_u256_to_usize_conversion() {
		// given
		let v = U256::from(usize::max_value()) + U256::from(1);

		// when
		let res = usize::from_u256(v);

		// then
		assert!(res.is_err());
	}
}
