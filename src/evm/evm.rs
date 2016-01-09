//! Evm interface.

use evm::{EvmParams, Ext};

#[derive(Debug, Eq, PartialEq)]
pub enum ReturnCode {
	Stop,
	Return,
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, params: &EvmParams, ext: &mut Ext) -> ReturnCode;
}
