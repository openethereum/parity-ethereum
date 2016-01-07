//! Evm interface.

use evm::{RuntimeData, ExtFace};

#[derive(Debug, Eq, PartialEq)]
pub enum ReturnCode {
	Stop,
	Return,
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, data: RuntimeData, ext: &mut ExtFace) -> ReturnCode;
}
