//! Evm interface.

use evm::{RuntimeData, Env};

#[derive(Debug, Eq, PartialEq)]
pub enum ReturnCode {
	Stop,
	Return,
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, data: RuntimeData, env: &mut Env) -> ReturnCode;
}
