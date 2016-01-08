use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::sha3::*;
use util::bytes::*;
use state::*;
use env_info::*;
use engine::*;
use transaction::*;
use evm::{VmFactory, Ext, LogEntry, EvmParams, ParamsKind};

/// Returns new address created from address and given nonce.
pub fn contract_address(address: &Address, nonce: &U256) -> Address {
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
	depth: usize,
	params: EvmParams,

	logs: Vec<LogEntry>,
	refunds: U256,
}

impl<'a> Executive<'a> {
	pub fn new(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, t: &Transaction) -> Self {
		// TODO: validate nonce ?
		
		let sender = t.sender();

		let params = match t.kind() {
			TransactionKind::ContractCreation => EvmParams {
				address: contract_address(&sender, &t.nonce),
				sender: sender.clone(),
				origin: sender.clone(),
				gas: t.gas,
				gas_price: t.gas_price,
				value: t.value,
				code: t.data.clone(),
				data: vec![],
				kind: ParamsKind::Create
			},
			TransactionKind::MessageCall => EvmParams {
				address: t.to.clone().unwrap(),
				sender: sender.clone(),
				origin: sender.clone(),
				gas: t.gas,
				gas_price: t.gas_price,
				value: t.value,
				code: state.code(&t.to.clone().unwrap()).unwrap_or(vec![]),
				data: t.data.clone(),
				kind: ParamsKind::Call
			}
		};

		Executive::new_from_params(state, info, engine, params)
	}

	pub fn new_from_params(state: &'a mut State, info: &'a EnvInfo, engine: &'a Engine, params: EvmParams) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			depth: 0,
			params: params,
			logs: vec![],
			refunds: U256::zero(),
		}
	}

	fn new_populated_from(e: &'a mut Executive, params: EvmParams) -> Self {
		Executive {
			state: e.state,
			info: e.info,
			engine: e.engine,
			depth: e.depth + 1,
			params: params,
			logs: vec![],
			refunds: U256::zero(),
		}
	}

	pub fn exec(&mut self) -> ExecutiveResult {
		// TODO: validate that we have enough funds

		match &self.params.kind() {
			&ParamsKind::Call => { 
				self.state.inc_nonce(&self.params.address);
				self.call()
			},
			&ParamsKind::Create => self.create()
		}
	}

	fn call(&mut self) -> ExecutiveResult {
		ExecutiveResult::Ok
	}

	fn create(&mut self) -> ExecutiveResult {
		let address = self.params.address.clone();

		//let new_address = contract_address(&address, &self.state.nonce(&address));
		let new_address = self.params.address.clone();
		self.state.inc_nonce(&address);

		{
			let evm = VmFactory::create();
			// TODO: valdidate that exec returns proper code
			evm.exec(self);
		}

		self.state.transfer_balance(&address, &new_address, &self.params.value);
		ExecutiveResult::Ok
	}

	pub fn logs(&self) -> &[LogEntry] {
		&self.logs
	}
}

impl<'a> Ext for Executive<'a> {
	fn params(&self) -> &EvmParams {
		&self.params
	}

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
		match self.state.balance(&self.params.address) > *endowment && self.depth < 1024 {
			false => (Address::new(), gas),
			true => {
				let params = EvmParams {
					address: contract_address(&self.params.address, &self.state.nonce(&self.params.address)),
					sender: self.params.address.clone(),
					origin: self.params.origin.clone(),
					gas: U256::from(gas),
					gas_price: self.params.gas_price.clone(),
					value: endowment.clone(),
					code: code.to_vec(),
					data: vec![],
					kind: ParamsKind::Create
				};
				let mut ex = Executive::new_populated_from(self, params);
				ex.create();
				unimplemented!()
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
			kind: ParamsKind::Call
		};

		{
			let mut ex = Executive::new_populated_from(self, params);
			ex.call();
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
