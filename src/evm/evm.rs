//! Evm interface.

use util::uint::U256;
use evm::{EvmParams, Ext};

/// Evm errors.
pub enum EvmError {
	/// `OutOfGas` is returned when transaction execution runs out of gas.
	/// The state should be reverted to the state from before the
	/// transaction execution. But it does not mean that transaction
	/// was invalid. Balance still should be transfered and nonce
	/// should be increased.
	OutOfGas,
	/// Returned on evm internal error. Should never be ignored during development.
	/// Likely to cause consensus issues.
	Internal,
}

/// Evm result.
/// 
/// Returns gas_left if execution is successfull, otherwise error.
pub type EvmResult = Result<U256, EvmError>;

/// Evm interface.
pub trait Evm {
	/// This function should be used to execute transaction.
	fn exec(&self, params: &EvmParams, ext: &mut Ext) -> EvmResult;
}
