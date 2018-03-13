// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
use std::cmp;
use std::sync::Arc;
use ethereum_types::{H256, U256, Address};
use bytes::{Bytes, BytesRef};
use state::{Backend as StateBackend, State, Substate, CleanupMode};
use machine::EthereumMachine as Machine;
use executive::*;
use vm::{
	self, ActionParams, ActionValue, EnvInfo, CallType, Schedule,
	Ext, ContractCreateResult, MessageCallResult, CreateContractAddress,
	ReturnData
};
use evm::FinalizationResult;
use transaction::UNSIGNED_SENDER;
use trace::{Tracer, VMTracer};

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
	value: U256,
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
			},
		}
	}
}

/// Implementation of evm Externalities.
pub struct Externalities<'a, T: 'a, V: 'a, B: 'a>
	where T: Tracer, V:  VMTracer, B: StateBackend
{
	state: &'a mut State<B>,
	env_info: &'a EnvInfo,
	machine: &'a Machine,
	depth: usize,
	origin_info: OriginInfo,
	substate: &'a mut Substate,
	schedule: Schedule,
	output: OutputPolicy<'a, 'a>,
	tracer: &'a mut T,
	vm_tracer: &'a mut V,
	static_flag: bool,
}

impl<'a, T: 'a, V: 'a, B: 'a> Externalities<'a, T, V, B>
	where T: Tracer, V: VMTracer, B: StateBackend
{
	/// Basic `Externalities` constructor.
	pub fn new(state: &'a mut State<B>,
		env_info: &'a EnvInfo,
		machine: &'a Machine,
		depth: usize,
		origin_info: OriginInfo,
		substate: &'a mut Substate,
		output: OutputPolicy<'a, 'a>,
		tracer: &'a mut T,
		vm_tracer: &'a mut V,
		static_flag: bool,
	) -> Self {
		Externalities {
			state: state,
			env_info: env_info,
			machine: machine,
			depth: depth,
			origin_info: origin_info,
			substate: substate,
			schedule: machine.schedule(env_info.number),
			output: output,
			tracer: tracer,
			vm_tracer: vm_tracer,
			static_flag: static_flag,
		}
	}
}

impl<'a, T: 'a, V: 'a, B: 'a> Ext for Externalities<'a, T, V, B>
	where T: Tracer, V: VMTracer, B: StateBackend
{
	fn storage_at(&self, key: &H256) -> vm::Result<H256> {
		self.state.storage_at(&self.origin_info.address, key).map_err(Into::into)
	}

	fn set_storage(&mut self, key: H256, value: H256) -> vm::Result<()> {
		if self.static_flag {
			Err(vm::Error::MutableCallInStaticContext)
		} else {
			self.state.set_storage(&self.origin_info.address, key, value).map_err(Into::into)
		}
	}

	fn is_static(&self) -> bool {
		return self.static_flag
	}

	fn exists(&self, address: &Address) -> vm::Result<bool> {
		self.state.exists(address).map_err(Into::into)
	}

	fn exists_and_not_null(&self, address: &Address) -> vm::Result<bool> {
		self.state.exists_and_not_null(address).map_err(Into::into)
	}

	fn origin_balance(&self) -> vm::Result<U256> {
		self.balance(&self.origin_info.address).map_err(Into::into)
	}

	fn balance(&self, address: &Address) -> vm::Result<U256> {
		self.state.balance(address).map_err(Into::into)
	}

	fn blockhash(&mut self, number: &U256) -> H256 {
		if self.env_info.number + 256 >= self.machine.params().eip210_transition {
			let blockhash_contract_address = self.machine.params().eip210_contract_address;
			let code_res = self.state.code(&blockhash_contract_address)
				.and_then(|code| self.state.code_hash(&blockhash_contract_address).map(|hash| (code, hash)));

			let (code, code_hash) = match code_res {
				Ok((code, hash)) => (code, hash),
				Err(_) => return H256::zero(),
			};

			let params = ActionParams {
				sender: self.origin_info.address.clone(),
				address: blockhash_contract_address.clone(),
				value: ActionValue::Apparent(self.origin_info.value),
				code_address: blockhash_contract_address.clone(),
				origin: self.origin_info.origin.clone(),
				gas: self.machine.params().eip210_contract_gas,
				gas_price: 0.into(),
				code: code,
				code_hash: Some(code_hash),
				data: Some(H256::from(number).to_vec()),
				call_type: CallType::Call,
				params_type: vm::ParamsType::Separate,
			};

			let mut output = H256::new();
			let mut ex = Executive::new(self.state, self.env_info, self.machine);
			let r = ex.call(params, self.substate, BytesRef::Fixed(&mut output), self.tracer, self.vm_tracer);
			trace!("ext: blockhash contract({}) -> {:?}({}) self.env_info.number={}\n", number, r, output, self.env_info.number);
			output
		} else {
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
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8], address_scheme: CreateContractAddress) -> ContractCreateResult {
		// create new contract address
		let (address, code_hash) = match self.state.nonce(&self.origin_info.address) {
			Ok(nonce) => contract_address(address_scheme, &self.origin_info.address, &nonce, &code),
			Err(e) => {
				debug!(target: "ext", "Database corruption encountered: {:?}", e);
				return ContractCreateResult::Failed
			}
		};

		// prepare the params
		let params = ActionParams {
			code_address: address.clone(),
			address: address.clone(),
			sender: self.origin_info.address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price,
			value: ActionValue::Transfer(*value),
			code: Some(Arc::new(code.to_vec())),
			code_hash: code_hash,
			data: None,
			call_type: CallType::None,
			params_type: vm::ParamsType::Embedded,
		};

		if !self.static_flag {
			if !self.schedule.eip86 || params.sender != UNSIGNED_SENDER {
				if let Err(e) = self.state.inc_nonce(&self.origin_info.address) {
					debug!(target: "ext", "Database corruption encountered: {:?}", e);
					return ContractCreateResult::Failed
				}
			}
		}
		let mut ex = Executive::from_parent(self.state, self.env_info, self.machine, self.depth, self.static_flag);

		// TODO: handle internal error separately
		match ex.create(params, self.substate, &mut None, self.tracer, self.vm_tracer) {
			Ok(FinalizationResult{ gas_left, apply_state: true, .. }) => {
				self.substate.contracts_created.push(address.clone());
				ContractCreateResult::Created(address, gas_left)
			},
			Ok(FinalizationResult{ gas_left, apply_state: false, return_data }) => {
				ContractCreateResult::Reverted(gas_left, return_data)
			},
			_ => ContractCreateResult::Failed,
		}
	}

	fn call(&mut self,
		gas: &U256,
		sender_address: &Address,
		receive_address: &Address,
		value: Option<U256>,
		data: &[u8],
		code_address: &Address,
		output: &mut [u8],
		call_type: CallType
	) -> MessageCallResult {
		trace!(target: "externalities", "call");

		let code_res = self.state.code(code_address)
			.and_then(|code| self.state.code_hash(code_address).map(|hash| (code, hash)));

		let (code, code_hash) = match code_res {
			Ok((code, hash)) => (code, hash),
			Err(_) => return MessageCallResult::Failed,
		};

		let mut params = ActionParams {
			sender: sender_address.clone(),
			address: receive_address.clone(),
			value: ActionValue::Apparent(self.origin_info.value),
			code_address: code_address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price,
			code: code,
			code_hash: Some(code_hash),
			data: Some(data.to_vec()),
			call_type: call_type,
			params_type: vm::ParamsType::Separate,
		};

		if let Some(value) = value {
			params.value = ActionValue::Transfer(value);
		}

		let mut ex = Executive::from_parent(self.state, self.env_info, self.machine, self.depth, self.static_flag);

		match ex.call(params, self.substate, BytesRef::Fixed(output), self.tracer, self.vm_tracer) {
			Ok(FinalizationResult{ gas_left, return_data, apply_state: true }) => MessageCallResult::Success(gas_left, return_data),
			Ok(FinalizationResult{ gas_left, return_data, apply_state: false }) => MessageCallResult::Reverted(gas_left, return_data),
			_ => MessageCallResult::Failed
		}
	}

	fn extcode(&self, address: &Address) -> vm::Result<Arc<Bytes>> {
		Ok(self.state.code(address)?.unwrap_or_else(|| Arc::new(vec![])))
	}

	fn extcodesize(&self, address: &Address) -> vm::Result<usize> {
		Ok(self.state.code_size(address)?.unwrap_or(0))
	}

	fn ret(mut self, gas: &U256, data: &ReturnData, apply_state: bool) -> vm::Result<U256>
		where Self: Sized {
		let handle_copy = |to: &mut Option<&mut Bytes>| {
			to.as_mut().map(|b| **b = data.to_vec());
		};
		match self.output {
			OutputPolicy::Return(BytesRef::Fixed(ref mut slice), ref mut copy) => {
				handle_copy(copy);

				let len = cmp::min(slice.len(), data.len());
				(&mut slice[..len]).copy_from_slice(&data[..len]);
				Ok(*gas)
			},
			OutputPolicy::Return(BytesRef::Flexible(ref mut vec), ref mut copy) => {
				handle_copy(copy);

				vec.clear();
				vec.extend_from_slice(&*data);
				Ok(*gas)
			},
			OutputPolicy::InitContract(ref mut copy) if apply_state => {
				let return_cost = U256::from(data.len()) * U256::from(self.schedule.create_data_gas);
				if return_cost > *gas || data.len() > self.schedule.create_data_limit {
					return match self.schedule.exceptional_failed_code_deposit {
						true => Err(vm::Error::OutOfGas),
						false => Ok(*gas)
					}
				}
				handle_copy(copy);
				self.state.init_code(&self.origin_info.address, data.to_vec())?;
				Ok(*gas - return_cost)
			},
			OutputPolicy::InitContract(_) => {
				Ok(*gas)
			},
		}
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> vm::Result<()> {
		use log_entry::LogEntry;

		if self.static_flag {
			return Err(vm::Error::MutableCallInStaticContext);
		}

		let address = self.origin_info.address.clone();
		self.substate.logs.push(LogEntry {
			address: address,
			topics: topics,
			data: data.to_vec()
		});

		Ok(())
	}

	fn suicide(&mut self, refund_address: &Address) -> vm::Result<()> {
		if self.static_flag {
			return Err(vm::Error::MutableCallInStaticContext);
		}

		let address = self.origin_info.address.clone();
		let balance = self.balance(&address)?;
		if &address == refund_address {
			// TODO [todr] To be consistent with CPP client we set balance to 0 in that case.
			self.state.sub_balance(&address, &balance, &mut CleanupMode::NoEmpty)?;
		} else {
			trace!(target: "ext", "Suiciding {} -> {} (xfer: {})", address, refund_address, balance);
			self.state.transfer_balance(
				&address,
				refund_address,
				&balance,
				self.substate.to_cleanup_mode(&self.schedule)
			)?;
		}

		self.tracer.trace_suicide(address, balance, refund_address.clone());
		self.substate.suicides.insert(address);

		Ok(())
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		self.env_info
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn inc_sstore_clears(&mut self) {
		self.substate.sstore_clears_count = self.substate.sstore_clears_count + U256::one();
	}

	fn trace_next_instruction(&mut self, pc: usize, instruction: u8, current_gas: U256) -> bool {
		self.vm_tracer.trace_next_instruction(pc, instruction, current_gas)
	}

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256) {
		self.vm_tracer.trace_prepare_execute(pc, instruction, gas_cost)
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		self.vm_tracer.trace_executed(gas_used, stack_push, mem_diff, store_diff)
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::{U256, Address};
	use evm::{EnvInfo, Ext, CallType};
	use state::{State, Substate};
	use test_helpers::get_temp_state;
	use super::*;
	use trace::{NoopTracer, NoopVMTracer};

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
			author: 0.into(),
			timestamp: 0,
			difficulty: 0.into(),
			last_hashes: Arc::new(vec![]),
			gas_used: 0.into(),
			gas_limit: 0.into(),
		}
	}

	struct TestSetup {
		state: State<::state_db::StateDB>,
		machine: ::machine::EthereumMachine,
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
				machine: ::spec::Spec::new_test_machine(),
				sub_state: Substate::new(),
				env_info: get_test_env_info()
			}
		}
	}

	#[test]
	fn can_be_created() {
		let mut setup = TestSetup::new();
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		let ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);

		assert_eq!(ext.env_info().number, 100);
	}

	#[test]
	fn can_return_block_hash_no_env() {
		let mut setup = TestSetup::new();
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		let mut ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);

		let hash = ext.blockhash(&"0000000000000000000000000000000000000000000000000000000000120000".parse::<U256>().unwrap());

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
			let mut last_hashes = (*env_info.last_hashes).clone();
			last_hashes.push(test_hash.clone());
			env_info.last_hashes = Arc::new(last_hashes);
		}
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		let mut ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);

		let hash = ext.blockhash(&"0000000000000000000000000000000000000000000000000000000000120000".parse::<U256>().unwrap());

		assert_eq!(test_hash, hash);
	}

	#[test]
	#[should_panic]
	fn can_call_fail_empty() {
		let mut setup = TestSetup::new();
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		let mut ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);

		let mut output = vec![];

		// this should panic because we have no balance on any account
		ext.call(
			&"0000000000000000000000000000000000000000000000000000000000120000".parse::<U256>().unwrap(),
			&Address::new(),
			&Address::new(),
			Some("0000000000000000000000000000000000000000000000000000000000150000".parse::<U256>().unwrap()),
			&[],
			&Address::new(),
			&mut output,
			CallType::Call
		);
	}

	#[test]
	fn can_log() {
		let log_data = vec![120u8, 110u8];
		let log_topics = vec![H256::from("af0fa234a6af46afa23faf23bcbc1c1cb4bcb7bcbe7e7e7ee3ee2edddddddddd")];

		let mut setup = TestSetup::new();
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		{
			let mut ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);
			ext.log(log_topics, &log_data).unwrap();
		}

		assert_eq!(setup.sub_state.logs.len(), 1);
	}

	#[test]
	fn can_suicide() {
		let refund_account = &Address::new();

		let mut setup = TestSetup::new();
		let state = &mut setup.state;
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;

		{
			let mut ext = Externalities::new(state, &setup.env_info, &setup.machine, 0, get_test_origin(), &mut setup.sub_state, OutputPolicy::InitContract(None), &mut tracer, &mut vm_tracer, false);
			ext.suicide(refund_account).unwrap();
		}

		assert_eq!(setup.sub_state.suicides.len(), 1);
	}
}
