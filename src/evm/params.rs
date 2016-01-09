use util::hash::*;
use util::uint::*;
use util::bytes::*;

#[derive(Clone)]
pub struct EvmParams {
	pub address: Address,
	pub sender: Address,
	pub origin: Address,
	pub gas: U256,
	pub gas_price: U256,
	pub value: U256,
	pub code: Bytes,
	pub data: Bytes
}

impl EvmParams {
	pub fn new() -> EvmParams {
		EvmParams {
			address: Address::new(),
			sender: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: U256::zero(),
			code: vec![],
			data: vec![],
		}
	}
}
