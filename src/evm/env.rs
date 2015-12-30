//! Contract execution environment.

use util::hash::*;
use util::uint::*;
use state::*;

/// This structure represents contract execution environment.
/// It should be initalized with `State` and contract address.
/// 
/// ```markdown
/// extern crate ethcore_util as util;
/// extern crate ethcore;
/// use util::hash::*;
/// use ethcore::state::*;
/// use ethcore::evm::*;
///
/// fn main() {
/// 	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
/// 	let mut data = RuntimeData::new();
/// 	let mut env = Env::new(State::new_temp(), address);
/// }	
/// ```
pub struct Env {
	state: State,
	address: Address
}

impl Env {
	/// Creates new evm environment object with backing state.
	pub fn new(state: State, address: Address) -> Env {
		Env {
			state: state,
			address: address
		}
	}

	/// Returns a value for given key.
	pub fn sload(&self, key: &H256) -> H256 {
		self.state.storage_at(&self.address, key)
	}

	/// Stores a value for given key.
	pub fn sstore(&mut self, key: H256, value: H256) {
		self.state.set_storage(&self.address, key, value)
	}

	/// Returns address balance.
	pub fn balance(&self, address: &Address) -> U256 {
		self.state.balance(address)
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


