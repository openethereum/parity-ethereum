use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::sha3::*;
use state::*;
use env_info::*;
use engine::*;
use transaction::*;
use evm::VmFactory;

fn contract_address(address: &Address, nonce: &U256) -> Address {
	let mut stream = RlpStream::new_list(2);
	stream.append(address);
	stream.append(nonce);
	From::from(stream.out().sha3())
}

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
			TransactionKind::ContractCreation => self.create(&transaction.sender(), 
															 &transaction.value, 
															 &transaction.gas_price, 
															 &transaction.gas,
															 &transaction.data,
															 &transaction.sender())
		}
	}

	fn call(&mut self, transaction: &Transaction) -> ExecutiveResult {
		ExecutiveResult::Ok
	}

	fn create(&mut self, sender: &Address, endowment: &U256, gas_price: &U256, gas: &U256, init: &[u8], origin: &Address) -> ExecutiveResult {
		let _new_address = contract_address(&sender, &(self.state.nonce(sender) - U256::one()));
		let _evm = VmFactory::create();

		ExecutiveResult::Ok
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::uint::*;

	#[test]
	fn test_contract_address() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let contract_address = Address::from_str("3f09c73a5ed19289fb9bdc72f1742566df146f56").unwrap();
		assert_eq!(contract_address, super::contract_address(&address, &U256::from(88)));
	}
}
