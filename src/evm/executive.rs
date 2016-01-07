use util::hash::*;
use util::uint::*;
use state::*;
use env_info::*;
use engine::*;
use transaction::*;

pub enum ExecutiveResult {
	Ok
}

pub struct Executive<'a> {
	state: &'a mut State,
	info: &'a EnvInfo,
	engine: &'a Engine,
	level: usize
}

impl<'a> Executive<'a> {
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, level: usize) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			level: level
		}
	}

	pub fn exec(&mut self, transaction: &Transaction) -> ExecutiveResult {
		// TODO: validate that we have enough funds

		self.state.inc_nonce(&transaction.sender());

		match transaction.kind() {
			TransactionKind::MessageCall => self.call(transaction),
			TransactionKind::ContractCreation => { unimplemented!(); }// self.create(&self.sender(), )
		}
	}

	fn call(&mut self, transaction: &Transaction) -> ExecutiveResult {
		ExecutiveResult::Ok
	}

	fn create(&mut self, address: &Address, endowment: &U256, gas_price: &U256, gas: &U256, init: &[u8], origin: &Address) -> ExecutiveResult {
		ExecutiveResult::Ok
	}
}
