//! Evm interface.

use util::bytes::Bytes;
use evm::{EvmParams, Ext};

#[derive(Debug, Eq, PartialEq)]
pub enum EvmResult {
	Stop,
	Return(Bytes),
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, params: &EvmParams, ext: &mut Ext) -> EvmResult;
}
