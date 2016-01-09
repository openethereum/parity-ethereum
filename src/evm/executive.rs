use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::sha3::*;
use util::bytes::*;
use state::*;
use env_info::*;
use engine::*;
use transaction::*;
use evm::{VmFactory, Ext, LogEntry, EvmParams, EvmResult};

/// Returns new address created from address and given nonce.
pub fn contract_address(address: &Address, nonce: &U256) -> Address {
	let mut stream = RlpStream::new_list(2);
	stream.append(address);
	stream.append(nonce);
	From::from(stream.out().sha3())
}

#[derive(PartialEq, Debug)]
pub enum ExecutiveResult {
	Ok,
	OutOfGas,
	InternalError
}

pub struct Executive<'a> {
	state: &'a mut State,
	info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
}

impl<'a> Executive<'a> {
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine) -> Self {
		Executive::new_with_depth(state, info, engine, 0)
	}

	fn from_parent(e: &'a mut Externalities) -> Self {
		Executive::new_with_depth(e.state, e.info, e.engine, e.depth + 1)
	}

	fn new_with_depth(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, depth: usize) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			depth: depth,
		}
	}

	pub fn transact(e: &mut Executive<'a>, t: &Transaction) -> ExecutiveResult {
		// TODO: validate that we have enough funds
		// TODO: validate nonce ?
	
		let sender = t.sender();

		match t.kind() {
			TransactionKind::ContractCreation => {
				let params = EvmParams {
					address: contract_address(&sender, &t.nonce),
					sender: sender.clone(),
					origin: sender.clone(),
					gas: t.gas,
					gas_price: t.gas_price,
					value: t.value,
					code: t.data.clone(),
					data: vec![],
				};
				e.state.inc_nonce(&params.address);
				unimplemented!()
				//Executive::call(e, &params)
			},
			TransactionKind::MessageCall => {
				let params = EvmParams {
					address: t.to.clone().unwrap(),
					sender: sender.clone(),
					origin: sender.clone(),
					gas: t.gas,
					gas_price: t.gas_price,
					value: t.value,
					code: e.state.code(&t.to.clone().unwrap()).unwrap_or(vec![]),
					data: t.data.clone(),
				};
				e.state.inc_nonce(&params.address);
				Executive::create(e, &params)
			}
		}
	}

	fn call(_e: &mut Executive<'a>, _p: &EvmParams) -> ExecutiveResult {
		//let _ext = Externalities::from_executive(e, &p);
		ExecutiveResult::Ok
	}
	
	fn create(e: &mut Executive<'a>, params: &EvmParams) -> ExecutiveResult {
		//self.state.require_or_from(&self.params.address, false, ||Account::new_contract(U256::from(0)));
		//TODO: ensure that account at given address is created
		e.state.new_contract(&params.address);
		e.state.transfer_balance(&params.sender, &params.address, &params.value);

		let code = {
			let mut ext = Externalities::new(e.state, e.info, e.engine, e.depth, params);
			let evm = VmFactory::create();
			evm.exec(&params, &mut ext)
		};

		match code {
			EvmResult::Stop => {
				ExecutiveResult::Ok
			},
			EvmResult::Return(output) => {
				e.state.init_code(&params.address, output);
				ExecutiveResult::Ok
			},
			EvmResult::OutOfGas => {
				ExecutiveResult::OutOfGas
			},
			_err => {
				ExecutiveResult::InternalError
			}
		}
	}
}

pub struct Externalities<'a> {
	state: &'a mut State,
	info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
	params: &'a EvmParams,
	logs: Vec<LogEntry>,
	refunds: U256
}

impl<'a> Externalities<'a> {
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, depth: usize, params: &'a EvmParams) -> Self {
		Externalities {
			state: state,
			info: info,
			engine: engine,
			depth: depth,
			params: params,
			logs: vec![],
			refunds: U256::zero()
		}
	}

	// TODO: figure out how to use this function
	// so the lifetime checker is satisfied
	//pub fn from_executive(e: &mut Executive<'a>, params: &EvmParams) -> Self {
		//Externalities::new(e.state, e.info, e.engine, e.depth, params)
	//}
	
	pub fn logs(&self) -> &[LogEntry] {
		&self.logs
	}
}

impl<'a> Ext for Externalities<'a> {
	fn sload(&self, key: &H256) -> H256 {
		self.state.storage_at(&self.params.address, key)
	}

	fn sstore(&mut self, key: H256, value: H256) {
		if value == H256::new() && self.state.storage_at(&self.params.address, &key) != H256::new() {
			self.refunds = self.refunds + U256::from(self.engine.evm_schedule(self.info).sstore_refund_gas);
		}
		self.state.set_storage(&self.params.address, key, value)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.state.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		match *number < self.info.number {
			false => H256::from(&U256::zero()),
			true => {
				let index = self.info.number - *number - U256::one();
				self.info.last_hashes[index.low_u32() as usize].clone()
			}
		}
	}

	fn create(&mut self, gas: u64, endowment: &U256, code: &[u8]) -> (Address, u64) {
		match self.state.balance(&self.params.address) >= *endowment && self.depth < 1024 {
			false => (Address::new(), gas),
			true => {
				let address = contract_address(&self.params.address, &self.state.nonce(&self.params.address));
				let params = EvmParams {
					address: address.clone(),
					sender: self.params.address.clone(),
					origin: self.params.origin.clone(),
					gas: U256::from(gas),
					gas_price: self.params.gas_price.clone(),
					value: endowment.clone(),
					code: code.to_vec(),
					data: vec![],
				};
				let mut ex = Executive::from_parent(self);
				ex.state.inc_nonce(&address);
				let res = Executive::create(&mut ex, &params);
				println!("res: {:?}", res);
				(address, gas)
			}
		}
	}

	fn call(&mut self, gas: u64, call_gas: u64, receive_address: &Address, value: &U256, data: &[u8], code_address: &Address) -> Option<(Vec<u8>, u64)>{
		// TODO: validation of the call
		
		let params = EvmParams {
			address: code_address.clone(),
			sender: receive_address.clone(),
			origin: self.params.origin.clone(),
			gas: U256::from(call_gas), // TODO: 
			gas_price: self.params.gas_price.clone(),
			value: value.clone(),
			code: self.state.code(code_address).unwrap_or(vec![]),
			data: data.to_vec(),
		};

		{
			let mut ex = Executive::from_parent(self);
			Executive::call(&mut ex, &params);
			unimplemented!();
			
		}
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
		self.state.code(address).unwrap_or(vec![])
	}

	fn log(&mut self, topics: Vec<H256>, data: Bytes) {
		let address = self.params.address.clone();
		self.logs.push(LogEntry::new(address, topics, data));
	}

}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use std::str::FromStr;
	use util::hash::*;
	use util::uint::*;
	use evm::*;
	use transaction::*;
	use env_info::*;
	use state::*;
	use spec::*;
	use engine::*;
	use evm_schedule::*;
	use super::contract_address;

	struct TestEngine;

	impl TestEngine {
		fn new() -> Self {
			TestEngine
		}
	}

	impl Engine for TestEngine {
		fn name(&self) -> &str { "TestEngine" }
		fn spec(&self) -> &Spec { unimplemented!() }
		fn evm_schedule(&self, _env_info: &EnvInfo) -> EvmSchedule { EvmSchedule::new_frontier() }
	}

	#[test]
	fn test_contract_address() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let expected_address = Address::from_str("3f09c73a5ed19289fb9bdc72f1742566df146f56").unwrap();
		assert_eq!(expected_address, contract_address(&address, &U256::from(88)));
	}

	#[test]
	// TODO: replace params with transactions!
	fn test_executive() {
		let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let address = contract_address(&sender, &U256::zero());
		let mut params = EvmParams::new();
		params.address = address.clone();
		params.sender = sender.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "3331600055".from_hex().unwrap();
		params.value = U256::from(0x7);
		let mut state = State::new_temp();
		state.add_balance(&sender, &U256::from(0x100u64));
		let info = EnvInfo::new();
		let engine = TestEngine::new();

		{
			let mut ex = Executive::new(&mut state, &info, &engine);
			assert_eq!(Executive::create(&mut ex, &params), ExecutiveResult::Ok);
		}

		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(&U256::from(0xf9u64)));
		assert_eq!(state.balance(&sender), U256::from(0xf9));
		assert_eq!(state.balance(&address), U256::from(0x7));
	}

	#[test]
	fn test_create_contract() {
		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		let next_address = contract_address(&address, &U256::zero());
		let mut params = EvmParams::new();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(0x174876e800u64);
		params.code = "7c601080600c6000396000f3006000355415600957005b60203560003555600052601d60036000f0600055".from_hex().unwrap();
		let mut state = State::new_temp();
		state.add_balance(&sender, &U256::from(0x100u64));
		let info = EnvInfo::new();
		let engine = TestEngine::new();

		{
			let mut ex = Executive::new(&mut state, &info, &engine);
			assert_eq!(Executive::create(&mut ex, &params), ExecutiveResult::Ok);
		}
		
		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(next_address.clone()));
		assert_eq!(state.code(&next_address).unwrap(), "6000355415600957005b602035600035".from_hex().unwrap());
	}
}
