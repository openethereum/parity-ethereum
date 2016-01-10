//! Evm interface.

use util::uint::U256;
use util::bytes::Bytes;
use evm::{EvmParams, Ext};

#[derive(Debug, Eq, PartialEq)]
pub enum EvmResult {
	Stop { gas_left: U256 },
	Return(Bytes),
	Suicide,
	OutOfGas,
	InternalError
}

pub trait Evm {
	fn exec(&self, params: &EvmParams, ext: &mut Ext) -> EvmResult;
}
