//! Evm interface.

use evm::Ext;

#[derive(Debug, Eq, PartialEq)]
pub enum ReturnCode {
	Stop,
	Return,
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, ext: &mut Ext) -> ReturnCode;
}
