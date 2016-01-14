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
		destination: usize
	},
	/// `BadInstructions` is returned when given instruction is not supported
	BadInstruction {
		instruction: &'static str,
	},
	/// `StackUnderflow` when there is not enough stack elements to execute instruction
	/// First parameter says how many elements were needed and the second how many were actually on Stack
	StackUnderflow {
		instruction: &'static str,
		wanted: usize, 
		on_stack: usize
	},
	/// When execution would exceed defined Stack Limit
	OutOfStack {
		instruction: &'static str,
		wanted: usize, 
		limit: usize
	},
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	Internal,
}

/// Evm result.
/// 
/// Returns gas_left if execution is successfull, otherwise error.
pub type Result = result::Result<U256, Error>;

/// Evm interface.
pub trait Evm {
	/// This function should be used to execute transaction.
	fn exec(&self, params: &ActionParams, ext: &mut Ext) -> Result;
}
