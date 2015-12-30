use util::hash::*;
use util::uint::*;
use state::*;

pub struct Env {
	state: State,
	address: Address
}

impl Env {
	pub fn new(state: State, address: Address) -> Env {
		Env {
			state: state,
			address: address
		}
	}

	pub fn sload(&self, index: &H256) -> H256 {
		self.state.storage_at(&self.address, index)
	}

	pub fn sstore(&mut self, index: H256, value: H256) {
		println!("index: {:?}, value: {:?}", index, value);
		self.state.set_storage(&self.address, index, value)
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

	/// Drain state
	// not sure if this is the best solution, but seems to be the easiest one, mk
	pub fn state(self) -> State {
		self.state
	}
}


