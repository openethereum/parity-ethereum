//! Immutable runtime data.

use util::hash::*;
use util::uint::*;

pub struct RuntimeData {
	pub gas: U256,
	pub gas_price: U256,
	pub call_data: Vec<u8>,
	pub address: Address,
	pub caller: Address,
	pub origin: Address,
	pub call_value: U256,
	pub coinbase: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub number: u64,
	pub timestamp: u64,
	pub code: Vec<u8>
}

impl RuntimeData {
	pub fn new() -> RuntimeData {
		RuntimeData {
			gas: U256::zero(),
			gas_price: U256::zero(),
			call_data: vec![],
			address: Address::new(),
			caller: Address::new(),
			origin: Address::new(),
			call_value: U256::zero(),
			coinbase: Address::new(),
			difficulty: U256::zero(),
			gas_limit: U256::zero(),
			number: 0,
			timestamp: 0,
			code: vec![]
		}
	}
}
