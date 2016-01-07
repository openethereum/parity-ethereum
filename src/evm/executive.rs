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

		match transaction.kind() {
			TransactionKind::MessageCall => self.call(transaction),
			TransactionKind::ContractCreation => self.create(transaction)
		}
	}

	fn call(&mut self, transaction: &Transaction) -> ExecutiveResult {
		ExecutiveResult::Ok
	}

	fn create(&mut self, transaction: &Transaction) -> ExecutiveResult {
		ExecutiveResult::Ok
	}
}
