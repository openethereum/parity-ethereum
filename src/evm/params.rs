use util::hash::*;
use util::uint::*;
use util::bytes::*;

#[derive(Eq, PartialEq, Clone)]
pub enum ParamsKind {
	Create,
	Call
}

#[derive(Clone)]
pub struct EvmParams {
	pub address: Address,
	pub sender: Address,
	pub origin: Address,
	pub gas: U256,
	pub gas_price: U256,
	pub value: U256,
	pub code: Bytes,
	pub data: Bytes,
	pub kind: ParamsKind
}

impl EvmParams {
	pub fn new(kind: ParamsKind) -> EvmParams {
		EvmParams {
			address: Address::new(),
			sender: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: U256::zero(),
			code: vec![],
			data: vec![],
			kind: kind
		}
	}

	pub fn new_call() -> EvmParams {
		EvmParams::new(ParamsKind::Call)
	}

	pub fn new_create() -> EvmParams {
		EvmParams::new(ParamsKind::Create)
	}

	pub fn kind(&self) -> ParamsKind {
		//TODO
		ParamsKind::Create
	}
}
