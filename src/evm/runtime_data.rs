use util::hash::*;
use util::uint::*;

pub struct RuntimeData {
	pub gas: u64,
	pub gas_price: u64,
	pub call_data: Vec<u8>,
	pub address: Address,
	pub caller: Address,
	pub origin: Address,
	pub coinbase: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub number: u64,
	pub timestamp: u64,
	pub code: Vec<u8>,
	pub code_hash: H256
}

impl RuntimeData {
	pub fn new() -> RuntimeData {
		RuntimeData {
			gas: 0,
			gas_price: 0,
			call_data: vec![],
			address: Address::new(),
			caller: Address::new(),
			origin: Address::new(),
			coinbase: Address::new(),
			difficulty: U256::from(0),
			gas_limit: U256::from(0),
			number: 0,
			timestamp: 0,
			code: vec![],
			code_hash: H256::new()
		}
	}
}
