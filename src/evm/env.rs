use util::hash::*;
use util::uint::*;

pub struct Env;

impl Env {
	pub fn new() -> Env {
		Env
	}

	pub fn sload(&self, _index: &H256) -> H256 {
		println!("sload!: {:?}", _index);
		//unimplemented!();
		H256::new()
	}

	pub fn sstore(&self, _index: &H256, _value: &H256) {
		println!("sstore!: {:?} , {:?}", _index, _value);
		//unimplemented!();
		
	}

	pub fn balance(&self, _address: &Address) -> U256 {
		unimplemented!();
	}

	pub fn blockhash(&self, _number: &U256) -> H256 {
		unimplemented!();
	}

	/// Creates new contract
	/// Returns new contract address gas used
	pub fn create(&self, _gas: u64, _endowment: &U256, _code: &[u8]) -> (Address, u64) {
		unimplemented!();
	}

	/// Calls existing contract
	/// Returns call output and gas used
	pub fn call(&self, _gas: u64, _call_gas: u64, _receive_address: &H256, _value: &U256, _data: &[u8], _code_address: &Address) -> Option<(Vec<u8>, u64)>{
		unimplemented!();
	}

	/// Returns code at given address
	pub fn extcode(&self, _address: &Address) -> Vec<u8> {
		unimplemented!();
	}

	pub fn log(&self, _topics: &[H256], _data: &[u8]) {
		unimplemented!();
	}
}


