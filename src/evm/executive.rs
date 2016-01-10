use std::collections::HashSet;
use std::cmp;
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

/// State changes which should be applied in finalize,
/// after transaction is fully executed.
pub struct Substate {
	/// Any accounts that have suicided.
	suicides: HashSet<Address>,
	/// Any logs.
	logs: Vec<LogEntry>,
	/// Refund counter of SSTORE nonzero->zero.
	refunds_count: U256,
}

impl Substate {
	/// Creates new substate.
	pub fn new() -> Self {
		Substate {
			suicides: HashSet::new(),
			logs: vec![],
			refunds_count: U256::zero(),
		}
	}

	pub fn logs(&self) -> &[LogEntry] {
		&self.logs
	}

	/// Appends another substate to this substate.
	fn accrue(&mut self, s: Substate) {
		self.suicides.extend(s.suicides.into_iter());
		self.logs.extend(s.logs.into_iter());
		self.refunds_count = self.refunds_count + s.refunds_count;
	}
}

#[derive(PartialEq, Debug)]
pub enum ExecutiveResult {
	Ok,
	BlockGasLimitReached { gas_limit: U256, gas_used: U256, gas: U256 },
	InvalidNonce { expected: U256, is: U256 },
	NotEnoughCash { required: U256, is: U256 },
	OutOfGas,
	InternalError
}

/// Message-call/contract-creation executor; useful for executing transactions.
pub struct Executive<'a> {
	state: &'a mut State,
	info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
}

impl<'a> Executive<'a> {
	/// Creates new executive with depth equal 0.
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine) -> Self {
		Executive::new_with_depth(state, info, engine, 0)
	}

	/// Populates executive from parent externalities. Increments executive depth.
	fn from_parent(e: &'a mut Externalities) -> Self {
		Executive::new_with_depth(e.state, e.info, e.engine, e.depth + 1)
	}

	/// Helper constructor. Should be used to create `Executive` with desired depth.
	/// Private.
	fn new_with_depth(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, depth: usize) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			depth: depth,
		}
	}

	/// This funtion should be used to execute transaction.
	pub fn transact(e: &mut Executive<'a>, t: &Transaction) -> ExecutiveResult {
		// validate if transaction fits into given block
		if e.info.gas_used + t.gas > e.info.gas_limit {
			return ExecutiveResult::BlockGasLimitReached { 
				gas_limit: e.info.gas_limit, 
				gas_used: e.info.gas_used, 
				gas: t.gas 
			};
		}

		let sender = t.sender();
		let nonce = e.state.nonce(&sender);

		// validate transaction nonce
		if t.nonce != nonce {
			return ExecutiveResult::InvalidNonce { expected: nonce, is: t.nonce };
		}
		
		// TODO: we might need bigints here, or at least check overflows.
		let balance = e.state.balance(&sender);
		let gas_cost = t.gas * t.gas_price;
		let total_cost = t.value + gas_cost;

		// avoid unaffordable transactions
		if balance < total_cost {
			return ExecutiveResult::NotEnoughCash { required: total_cost, is: balance };
		}

		e.state.inc_nonce(&sender);
		let mut substate = Substate::new();

		let res = match t.kind() {
			TransactionKind::ContractCreation => {
				let params = EvmParams {
					address: contract_address(&sender, &nonce),
					sender: sender.clone(),
					origin: sender.clone(),
					gas: t.gas,
					gas_price: t.gas_price,
					value: t.value,
					code: t.data.clone(),
					data: vec![],
				};
				Executive::call(e, &params, &mut substate)
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
				Executive::create(e, &params, &mut substate)
			}
		};

		// finalize here!
		e.finalize(substate, &sender, U256::zero(), U256::zero(), t.gas_price);
		res
	}

	/// Calls contract function with given contract params.
	/// *Note. It does not finalize the transaction (doesn't do refunds, nor suicides).
	fn call(e: &mut Executive<'a>, params: &EvmParams, substate: &mut Substate) -> ExecutiveResult {
		// at first, transfer value to destination
		e.state.transfer_balance(&params.sender, &params.address, &params.value);

		// if destination is builtin, try to execute it, or quickly return
		if e.engine.is_builtin(&params.address) {
			return match e.engine.cost_of_builtin(&params.address, &params.data) > params.gas {
				true => ExecutiveResult::OutOfGas,
				false => {
					// TODO: substract gas for execution
					let mut out = vec![];
					e.engine.execute_builtin(&params.address, &params.data, &mut out);
					ExecutiveResult::Ok
				}
			}
		}

		// otherwise do `normal` execution if destination is a contract
		// TODO: is executing contract with no code different from not executing contract at all?
		// if yes, there is a logic issue here. mk
		if params.code.len() > 0 {
			return match {
				let mut ext = Externalities::new(e.state, e.info, e.engine, e.depth, params, substate);
				let evm = VmFactory::create();
				evm.exec(&params, &mut ext)
			} {
				EvmResult::Stop => ExecutiveResult::Ok,
				EvmResult::Return(_) => ExecutiveResult::Ok,
				EvmResult::Suicide => {
					substate.suicides.insert(params.address.clone());
					ExecutiveResult::Ok
				},
				EvmResult::OutOfGas => ExecutiveResult::OutOfGas,
				_err => ExecutiveResult::InternalError
			}
		}
		
		ExecutiveResult::Ok
	}
	
	/// Creates contract with given contract params.
	/// *Note. It does not finalize the transaction (doesn't do refunds, nor suicides).
	fn create(e: &mut Executive<'a>, params: &EvmParams, substate: &mut Substate) -> ExecutiveResult {
		// at first create new contract
		e.state.new_contract(&params.address);
		// then transfer value to it
		e.state.transfer_balance(&params.sender, &params.address, &params.value);

		match {
			let mut ext = Externalities::new(e.state, e.info, e.engine, e.depth, params, substate);
			let evm = VmFactory::create();
			evm.exec(&params, &mut ext)
		} {
			EvmResult::Stop => {
				ExecutiveResult::Ok
			},
			EvmResult::Return(output) => {
				e.state.init_code(&params.address, output);
				ExecutiveResult::Ok
			},
			EvmResult::Suicide => {
				substate.suicides.insert(params.address.clone());
				ExecutiveResult::Ok
			},
			EvmResult::OutOfGas => ExecutiveResult::OutOfGas,
			_err => ExecutiveResult::InternalError
		}
	}

	/// Finalizes the transaction (does refunds and suicides).
	fn finalize(&mut self, substate: Substate, sender: &Address, gas: U256, gas_left: U256, gas_price: U256) {
		let schedule = self.engine.evm_schedule(self.info);

		// refunds from SSTORE nonzero -> zero
		let sstore_refunds = U256::from(schedule.sstore_refund_gas) * substate.refunds_count;
		// refunds from contract suicides
		let suicide_refunds = U256::from(schedule.suicide_refund_gas) * U256::from(substate.suicides.len());

		// real ammount to refund
		let refund = cmp::min(sstore_refunds + suicide_refunds, (gas - gas_left) / U256::from(2)) + gas_left;
		let refund_value = refund * gas_price;
		self.state.add_balance(sender, &refund_value);
		
		// fees earned by author
		let fees = (gas - refund) * gas_price;
		let author = &self.info.author;
		self.state.add_balance(author, &fees);

		// perform suicides
		for address in substate.suicides.iter() {
			self.state.kill_account(address);
		}
	}
}

/// Implementation of evm Externalities.
pub struct Externalities<'a> {
	state: &'a mut State,
	info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
	params: &'a EvmParams,
	substate: &'a mut Substate
}

impl<'a> Externalities<'a> {
	/// Basic `Externalities` constructor.
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, depth: usize, params: &'a EvmParams, substate: &'a mut Substate) -> Self {
		Externalities {
			state: state,
			info: info,
			engine: engine,
			depth: depth,
			params: params,
			substate: substate
		}
	}
}

impl<'a> Ext for Externalities<'a> {
	fn sload(&self, key: &H256) -> H256 {
		self.state.storage_at(&self.params.address, key)
	}

	fn sstore(&mut self, key: H256, value: H256) {
		// if SSTORE nonzero -> zero, increment refund count
		if value == H256::new() && self.state.storage_at(&self.params.address, &key) != H256::new() {
			self.substate.refunds_count = self.substate.refunds_count + U256::one();
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

	fn create(&mut self, gas: u64, endowment: &U256, code: &[u8]) -> Option<(Address, u64)> {
		match self.state.balance(&self.params.address) >= *endowment && self.depth < 1024 {
			false => None,
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
				let mut substate = Substate::new();
				{
					let mut ex = Executive::from_parent(self);
					ex.state.inc_nonce(&address);
					let res = Executive::create(&mut ex, &params, &mut substate);
					println!("res: {:?}", res);
				}
				self.substate.accrue(substate);
				Some((address, gas))
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

		let mut substate = Substate::new();
		{
			let mut ex = Executive::from_parent(self);
			Executive::call(&mut ex, &params, &mut substate);
			unimplemented!();
		}
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
		self.state.code(address).unwrap_or(vec![])
	}

	fn log(&mut self, topics: Vec<H256>, data: Bytes) {
		let address = self.params.address.clone();
		self.substate.logs.push(LogEntry::new(address, topics, data));
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
		let mut substate = Substate::new();

		{
			let mut ex = Executive::new(&mut state, &info, &engine);
			assert_eq!(Executive::create(&mut ex, &params, &mut substate), ExecutiveResult::Ok);
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
		let mut substate = Substate::new();

		{
			let mut ex = Executive::new(&mut state, &info, &engine);
			assert_eq!(Executive::create(&mut ex, &params, &mut substate), ExecutiveResult::Ok);
		}
		
		assert_eq!(state.storage_at(&address, &H256::new()), H256::from(next_address.clone()));
		assert_eq!(state.code(&next_address).unwrap(), "6000355415600957005b602035600035".from_hex().unwrap());
	}
}
