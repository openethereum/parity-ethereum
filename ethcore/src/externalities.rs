// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Transaction Execution environment.
use common::*;
use state::*;
use engine::*;
use executive::*;
use evm::{self, Schedule, Ext, ContractCreateResult, MessageCallResult};
use substate::*;
use trace::Tracer;

/// Policy for handling output data on `RETURN` opcode.
pub enum OutputPolicy<'a, 'b> {
	/// Return reference to fixed sized output.
	/// Used for message calls.
	Return(BytesRef<'a>, Option<&'b mut Bytes>),
	/// Init new contract as soon as `RETURN` is called.
	InitContract(Option<&'b mut Bytes>),
}

/// Transaction properties that externalities need to know about.
pub struct OriginInfo {
	address: Address,
	origin: Address,
	gas_price: U256,
	value: U256
}

impl OriginInfo {
	/// Populates origin info from action params.
	pub fn from(params: &ActionParams) -> Self {
		OriginInfo {
			address: params.address.clone(),
			origin: params.origin.clone(),
			gas_price: params.gas_price,
			value: match params.value {
				ActionValue::Transfer(val) | ActionValue::Apparent(val) => val
			}
		}
	}
}

/// Implementation of evm Externalities.
pub struct Externalities<'a, T> where T: 'a + Tracer {
	state: &'a mut State,
	env_info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
	origin_info: OriginInfo,
	substate: &'a mut Substate,
	schedule: Schedule,
	output: OutputPolicy<'a, 'a>,
	tracer: &'a mut T,
}

impl<'a, T> Externalities<'a, T> where T: 'a + Tracer {

	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	/// Basic `Externalities` constructor.
	pub fn new(state: &'a mut State,
		env_info: &'a EnvInfo,
		engine: &'a Engine,
		depth: usize,
		origin_info: OriginInfo,
		substate: &'a mut Substate,
		output: OutputPolicy<'a, 'a>,
		tracer: &'a mut T,
	) -> Self {
		Externalities {
			state: state,
			env_info: env_info,
			engine: engine,
			depth: depth,
			origin_info: origin_info,
			substate: substate,
			schedule: engine.schedule(env_info),
			output: output,
			tracer: tracer,
		}
	}
}

impl<'a, T> Ext for Externalities<'a, T> where T: 'a + Tracer {
	fn storage_at(&self, key: &H256) -> H256 {
		self.state.storage_at(&self.origin_info.address, key)
	}

	fn set_storage(&mut self, key: H256, value: H256) {
		self.state.set_storage(&self.origin_info.address, key, value)
	}

	fn exists(&self, address: &Address) -> bool {
		self.state.exists(address)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.state.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		// TODO: comment out what this function expects from env_info, since it will produce panics if the latter is inconsistent
		match *number < U256::from(self.env_info.number) && number.low_u64() >= cmp::max(256, self.env_info.number) - 256 {
			true => {
				let index = self.env_info.number - number.low_u64() - 1;
				assert!(index < self.env_info.last_hashes.len() as u64, format!("Inconsistent env_info, should contain at least {:?} last hashes", index+1));
				let r = self.env_info.last_hashes[index as usize].clone();
				trace!("ext: blockhash({}) -> {} self.env_info.number={}\n", number, r, self.env_info.number);
				r
			},
			false => {
				trace!("ext: blockhash({}) -> null self.env_info.number={}\n", number, self.env_info.number);
				H256::zero()
			},
		}
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> ContractCreateResult {
		// create new contract address
		let address = contract_address(&self.origin_info.address, &self.state.nonce(&self.origin_info.address));

		// prepare the params
		let params = ActionParams {
			code_address: address.clone(),
			address: address.clone(),
			sender: self.origin_info.address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price,
			value: ActionValue::Transfer(*value),
			code: Some(code.to_vec()),
			data: None,
		};

		self.state.inc_nonce(&self.origin_info.address);
		let mut ex = Executive::from_parent(self.state, self.env_info, self.engine, self.depth);

		// TODO: handle internal error separately
		match ex.create(params, self.substate, self.tracer) {
			Ok(gas_left) => {
				self.substate.contracts_created.push(address.clone());
				ContractCreateResult::Created(address, gas_left)
			},
			_ => ContractCreateResult::Failed
		}
	}

	fn call(&mut self,
		gas: &U256,
		sender_address: &Address,
		receive_address: &Address,
		value: Option<U256>,
		data: &[u8],
		code_address: &Address,
		output: &mut [u8]
	) -> MessageCallResult {
		trace!(target: "externalities", "call");

		let mut params = ActionParams {
			sender: sender_address.clone(),
			address: receive_address.clone(),
			value: ActionValue::Apparent(self.origin_info.value),
			code_address: code_address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price,
			code: self.state.code(code_address),
			data: Some(data.to_vec()),
		};

		if let Some(value) = value {
			params.value = ActionValue::Transfer(value);
		}

		let mut ex = Executive::from_parent(self.state, self.env_info, self.engine, self.depth);

		match ex.call(params, self.substate, BytesRef::Fixed(output), self.tracer) {
			Ok(gas_left) => MessageCallResult::Success(gas_left),
			_ => MessageCallResult::Failed
		}
	}

	fn extcode(&self, address: &Address) -> Bytes {
		self.state.code(address).unwrap_or_else(|| vec![])
	}

	#[cfg_attr(feature="dev", allow(match_ref_pats))]
	fn ret(&mut self, gas: &U256, data: &[u8]) -> Result<U256, evm::Error> {
		let handle_copy = |to: &mut Option<&mut Bytes>| {
			to.as_mut().map(|b| **b = data.to_owned());
		};
		match self.output {
			OutputPolicy::Return(BytesRef::Fixed(ref mut slice), ref mut copy) => {
				handle_copy(copy);

				let len = cmp::min(slice.len(), data.len());
				unsafe {
					ptr::copy(data.as_ptr(), slice.as_mut_ptr(), len);
				}
				Ok(*gas)
			},
			OutputPolicy::Return(BytesRef::Flexible(ref mut vec), ref mut copy) => {
				handle_copy(copy);

				vec.clear();
				vec.reserve(data.len());
				unsafe {
					ptr::copy(data.as_ptr(), vec.as_mut_ptr(), data.len());
					vec.set_len(data.len());
				}
				Ok(*gas)
			},
			OutputPolicy::InitContract(ref mut copy) => {
				let return_cost = U256::from(data.len()) * U256::from(self.schedule.create_data_gas);
				if return_cost > *gas {
					return match self.schedule.exceptional_failed_code_deposit {
						true => Err(evm::Error::OutOfGas),
						false => Ok(*gas)
					}
				}

				handle_copy(copy);

				let mut code = vec![];
				code.reserve(data.len());
				unsafe {
					ptr::copy(data.as_ptr(), code.as_mut_ptr(), data.len());
					code.set_len(data.len());
				}
				self.state.init_code(&self.origin_info.address, code);
				Ok(*gas - return_cost)
			}
		}
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) {
		let address = self.origin_info.address.clone();
		self.substate.logs.push(LogEntry {
			address: address,
			topics: topics,
			data: data.to_vec()
		});
	}

	fn suicide(&mut self, refund_address: &Address) {
		let address = self.origin_info.address.clone();
		let balance = self.balance(&address);
		if &address == refund_address {
			// TODO [todr] To be consisted with CPP client we set balance to 0 in that case.
			self.state.sub_balance(&address, &balance);
		} else {
			trace!("Suiciding {} -> {} (xfer: {})", address, refund_address, balance);
			self.state.transfer_balance(&address, refund_address, &balance);
		}
		self.substate.suicides.insert(address);
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.env_info
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn inc_sstore_clears(&mut self) {
		self.substate.sstore_clears_count = self.substate.sstore_clears_count + U256::one();
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use state::*;
	use engine::*;
	use evm::{Ext};
	use substate::*;
	use tests::helpers::*;
	use super::*;
	use trace::{NoopTracer};

	fn get_test_origin() -> OriginInfo {
		OriginInfo {
			address: Address::zero(),
			origin: Address::zero(),
			gas_price: U256::zero(),
			value: U256::zero()
		}
	}

	fn get_test_env_info() -> EnvInfo {
		EnvInfo {
			number: 100,
			author: x!(0),
			timestamp: 0,
			difficulty: x!(0),
			last_hashes: vec![],
			gas_used: x!(0),
			gas_limit: x!(0)
		}
	}

	struct TestSetup {
		state: GuardedTempResult<State>,
		engine: Box<Engine>,
		sub_state: Substate,
		env_info: EnvInfo
	}

	impl Default for TestSetup {
		fn default() -> Self {
			TestSetup::new()
		}
	}

	impl TestSetup {
		fn new() -> Self {
			TestSetup {
				state: get_temp_state(),
				engine: get_test_spec().engine,
				sub_state: Substate::new(),
				env_info: get_test_env_info()
			}
		}
	}

	#[test]
	fn can_be_created() {
		let mut setup = TestSetup::new();
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		let ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);

		assert_eq!(ext.env_info().number, 100);
	}

	#[test]
	fn can_return_block_hash_no_env() {
		let mut setup = TestSetup::new();
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		let ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);

		let hash = ext.blockhash(&U256::from_str("0000000000000000000000000000000000000000000000000000000000120000").unwrap());

		assert_eq!(hash, H256::zero());
	}

	#[test]
	fn can_return_block_hash() {
		let test_hash = H256::from("afafafafafafafafafafafbcbcbcbcbcbcbcbcbcbeeeeeeeeeeeeedddddddddd");
		let test_env_number = 0x120001;

		let mut setup = TestSetup::new();
		{
			let env_info = &mut setup.env_info;
			env_info.number = test_env_number;
			env_info.last_hashes.push(test_hash.clone());
		}
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		let ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);

		let hash = ext.blockhash(&U256::from_str("0000000000000000000000000000000000000000000000000000000000120000").unwrap());

		assert_eq!(test_hash, hash);
	}

	#[test]
	#[should_panic]
	fn can_call_fail_empty() {
		let mut setup = TestSetup::new();
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		let mut ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);

		let mut output = vec![];

		// this should panic because we have no balance on any account
		ext.call(
			&U256::from_str("0000000000000000000000000000000000000000000000000000000000120000").unwrap(),
			&Address::new(),
			&Address::new(),
			Some(U256::from_str("0000000000000000000000000000000000000000000000000000000000150000").unwrap()),
			&[],
			&Address::new(),
			&mut output);
	}

	#[test]
	fn can_log() {
		let log_data = vec![120u8, 110u8];
		let log_topics = vec![H256::from("af0fa234a6af46afa23faf23bcbc1c1cb4bcb7bcbe7e7e7ee3ee2edddddddddd")];

		let mut setup = TestSetup::new();
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		{
			let mut ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);
			ext.log(log_topics, &log_data);
		}

		assert_eq!(setup.sub_state.logs.len(), 1);
	}

	#[test]
	fn can_suicide() {
		let refund_account = &Address::new();

		let mut setup = TestSetup::new();
		let state = setup.state.reference_mut();
		let mut tracer = NoopTracer;

		{
			let mut ext = Externalities::new(state, &setup.env_info, &*setup.engine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer);
			ext.suicide(&refund_account);
		}

		assert_eq!(setup.sub_state.suicides.len(), 1);
	}
}
