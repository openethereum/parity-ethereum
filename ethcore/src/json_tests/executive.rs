// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::path::Path;
use std::sync::Arc;
use super::test_common::*;
use state::{Backend as StateBackend, State, Substate};
use executive::*;
use evm::{VMType, Finalize};
use vm::{
	self, ActionParams, CallType, Schedule, Ext,
	ContractCreateResult, EnvInfo, MessageCallResult,
	CreateContractAddress, ReturnData,
};
use externalities::*;
use test_helpers::get_temp_state;
use ethjson;
use trace::{Tracer, NoopTracer};
use trace::{VMTracer, NoopVMTracer};
use bytes::Bytes;
use ethtrie;
use rlp::RlpStream;
use hash::keccak;
use machine::Machine;
use ethereum_types::BigEndianHash;

use super::HookType;

/// Run executive jsontests on a given folder.
pub fn run_test_path<H: FnMut(&str, HookType)>(p: &Path, skip: &[&'static str], h: &mut H) {
	::json_tests::test_common::run_test_path(p, skip, do_json_test, h)
}

/// Run executive jsontests on a given file.
pub fn run_test_file<H: FnMut(&str, HookType)>(p: &Path, h: &mut H) {
	::json_tests::test_common::run_test_file(p, do_json_test, h)
}

#[derive(Debug, PartialEq, Clone)]
struct CallCreate {
	data: Bytes,
	destination: Option<Address>,
	gas_limit: U256,
	value: U256
}

impl From<ethjson::vm::Call> for CallCreate {
	fn from(c: ethjson::vm::Call) -> Self {
		let dst: Option<ethjson::hash::Address> = c.destination.into();
		CallCreate {
			data: c.data.into(),
			destination: dst.map(Into::into),
			gas_limit: c.gas_limit.into(),
			value: c.value.into()
		}
	}
}

/// Tiny wrapper around executive externalities.
/// Stores callcreates.
struct TestExt<'a, T: 'a, V: 'a, B: 'a>
	where T: Tracer, V: VMTracer, B: StateBackend
{
	ext: Externalities<'a, T, V, B>,
	callcreates: Vec<CallCreate>,
	nonce: U256,
	sender: Address,
}

impl<'a, T: 'a, V: 'a, B: 'a> TestExt<'a, T, V, B>
	where T: Tracer, V: VMTracer, B: StateBackend,
{
	fn new(
		state: &'a mut State<B>,
		info: &'a EnvInfo,
		machine: &'a Machine,
		schedule: &'a Schedule,
		depth: usize,
		origin_info: &'a OriginInfo,
		substate: &'a mut Substate,
		output: OutputPolicy,
		address: Address,
		tracer: &'a mut T,
		vm_tracer: &'a mut V,
	) -> ethtrie::Result<Self> {
		let static_call = false;
		Ok(TestExt {
			nonce: state.nonce(&address)?,
			ext: Externalities::new(state, info, machine, schedule, depth, 0, origin_info, substate, output, tracer, vm_tracer, static_call),
			callcreates: vec![],
			sender: address,
		})
	}
}

impl<'a, T: 'a, V: 'a, B: 'a> Ext for TestExt<'a, T, V, B>
	where T: Tracer, V: VMTracer, B: StateBackend
{
	fn storage_at(&self, key: &H256) -> vm::Result<H256> {
		self.ext.storage_at(key)
	}

	fn initial_storage_at(&self, key: &H256) -> vm::Result<H256> {
		self.ext.initial_storage_at(key)
	}

	fn set_storage(&mut self, key: H256, value: H256) -> vm::Result<()> {
		self.ext.set_storage(key, value)
	}

	fn exists(&self, address: &Address) -> vm::Result<bool> {
		self.ext.exists(address)
	}

	fn exists_and_not_null(&self, address: &Address) -> vm::Result<bool> {
		self.ext.exists_and_not_null(address)
	}

	fn balance(&self, address: &Address) -> vm::Result<U256> {
		self.ext.balance(address)
	}

	fn origin_balance(&self) -> vm::Result<U256> {
		self.ext.origin_balance()
	}

	fn blockhash(&mut self, number: &U256) -> H256 {
		self.ext.blockhash(number)
	}

	fn create(
		&mut self,
		gas: &U256,
		value: &U256,
		code: &[u8],
		_code_version: &U256,
		address: CreateContractAddress,
		_trap: bool
	) -> Result<ContractCreateResult, vm::TrapKind> {
		self.callcreates.push(CallCreate {
			data: code.to_vec(),
			destination: None,
			gas_limit: *gas,
			value: *value
		});
		let contract_address = contract_address(address, &self.sender, &self.nonce, &code).0;
		Ok(ContractCreateResult::Created(contract_address, *gas))
	}

	fn call(
		&mut self,
		gas: &U256,
		_sender_address: &Address,
		receive_address: &Address,
		value: Option<U256>,
		data: &[u8],
		_code_address: &Address,
		_call_type: CallType,
		_trap: bool
	) -> Result<MessageCallResult, vm::TrapKind> {
		self.callcreates.push(CallCreate {
			data: data.to_vec(),
			destination: Some(receive_address.clone()),
			gas_limit: *gas,
			value: value.unwrap()
		});
		Ok(MessageCallResult::Success(*gas, ReturnData::empty()))
	}

	fn extcode(&self, address: &Address) -> vm::Result<Option<Arc<Bytes>>>  {
		self.ext.extcode(address)
	}

	fn extcodesize(&self, address: &Address) -> vm::Result<Option<usize>> {
		self.ext.extcodesize(address)
	}

	fn extcodehash(&self, address: &Address) -> vm::Result<Option<H256>> {
		self.ext.extcodehash(address)
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> vm::Result<()> {
		self.ext.log(topics, data)
	}

	fn ret(self, gas: &U256, data: &ReturnData, apply_state: bool) -> Result<U256, vm::Error> {
		self.ext.ret(gas, data, apply_state)
	}

	fn suicide(&mut self, refund_address: &Address) -> vm::Result<()> {
		self.ext.suicide(refund_address)
	}

	fn schedule(&self) -> &Schedule {
		self.ext.schedule()
	}

	fn env_info(&self) -> &EnvInfo {
		self.ext.env_info()
	}

	fn depth(&self) -> usize {
		0
	}

	fn is_static(&self) -> bool {
		false
	}

	fn add_sstore_refund(&mut self, value: usize) {
		self.ext.add_sstore_refund(value)
	}

	fn sub_sstore_refund(&mut self, value: usize) {
		self.ext.sub_sstore_refund(value)
	}
}

fn do_json_test<H: FnMut(&str, HookType)>(json_data: &[u8], h: &mut H) -> Vec<String> {
	let vms = VMType::all();
	vms
		.iter()
		.flat_map(|vm| do_json_test_for(vm, json_data, h))
		.collect()
}

fn do_json_test_for<H: FnMut(&str, HookType)>(vm_type: &VMType, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String> {
	let tests = ethjson::vm::Test::load(json_data).unwrap();
	let mut failed = Vec::new();

	for (name, vm) in tests.into_iter() {
		start_stop_hook(&format!("{}-{}", name, vm_type), HookType::OnStart);

		info!(target: "jsontests", "name: {:?}", name);
		let mut fail = false;

		let mut fail_unless = |cond: bool, s: &str | if !cond && !fail {
			failed.push(format!("[{}] {}: {}", vm_type, name, s));
			fail = true
		};

		macro_rules! try_fail {
			($e: expr) => {
				match $e {
					Ok(x) => x,
					Err(e) => {
						let msg = format!("Internal error: {}", e);
						fail_unless(false, &msg);
						continue
					}
				}
			}
		}

		let out_of_gas = vm.out_of_gas();
		let mut state = get_temp_state();
		state.populate_from(From::from(vm.pre_state.clone()));
		let info: EnvInfo = From::from(vm.env);
		let machine = {
			let mut machine = ::ethereum::new_frontier_test_machine();
			machine.set_schedule_creation_rules(Box::new(move |s, _| s.max_depth = 1));
			machine
		};

		let params = ActionParams::from(vm.transaction);

		let mut substate = Substate::new();
		let mut tracer = NoopTracer;
		let mut vm_tracer = NoopVMTracer;
		let vm_factory = state.vm_factory();
		let origin_info = OriginInfo::from(&params);

		// execute
		let (res, callcreates) = {
			let schedule = machine.schedule(info.number);
			let mut ex = try_fail!(TestExt::new(
				&mut state,
				&info,
				&machine,
				&schedule,
				0,
				&origin_info,
				&mut substate,
				OutputPolicy::Return,
				params.address.clone(),
				&mut tracer,
				&mut vm_tracer,
			));
			let mut evm = vm_factory.create(params, &schedule, 0).expect("Current tests are all of version 0; factory always return Some; qed");
			let res = evm.exec(&mut ex).ok().expect("TestExt never trap; resume error never happens; qed");
			// a return in finalize will not alter callcreates
			let callcreates = ex.callcreates.clone();
			(res.finalize(ex), callcreates)
		};

		let output = match &res {
			Ok(res) => res.return_data.to_vec(),
			Err(_) => Vec::new(),
		};

		let log_hash = {
			let mut rlp = RlpStream::new_list(substate.logs.len());
			for l in &substate.logs {
				rlp.append(l);
			}
			keccak(&rlp.drain())
		};

		match res {
			Err(_) => fail_unless(out_of_gas, "didn't expect to run out of gas."),
			Ok(res) => {
				fail_unless(!out_of_gas, "expected to run out of gas.");
				fail_unless(Some(res.gas_left) == vm.gas_left.map(Into::into), "gas_left is incorrect");
				let vm_output: Option<Vec<u8>> = vm.output.map(Into::into);
				fail_unless(Some(output) == vm_output, "output is incorrect");
				fail_unless(Some(log_hash) == vm.logs.map(|h| h.0), "logs are incorrect");

				for (address, account) in vm.post_state.unwrap().into_iter() {
					let address = address.into();
					let code: Vec<u8> = account.code.into();
					let found_code = try_fail!(state.code(&address));
					let found_balance = try_fail!(state.balance(&address));
					let found_nonce = try_fail!(state.nonce(&address));

					fail_unless(found_code.as_ref().map_or_else(|| code.is_empty(), |c| &**c == &code), "code is incorrect");
					fail_unless(found_balance == account.balance.into(), "balance is incorrect");
					fail_unless(found_nonce == account.nonce.into(), "nonce is incorrect");
					for (k, v) in account.storage {
						let key: U256 = k.into();
						let value: U256 = v.into();
						let found_storage = try_fail!(state.storage_at(&address, &BigEndianHash::from_uint(&key)));
						fail_unless(found_storage == BigEndianHash::from_uint(&value), "storage is incorrect");
					}
				}

				let calls: Option<Vec<CallCreate>> = vm.calls.map(|c| c.into_iter().map(From::from).collect());
				fail_unless(Some(callcreates) == calls, "callcreates does not match");
			}
		};

		start_stop_hook(&format!("{}-{}", name, vm_type), HookType::OnStop);
	}

	for f in &failed {
		error!("FAILED: {:?}", f);
	}

	failed
}

declare_test!{ExecutiveTests_vmArithmeticTest, "VMTests/vmArithmeticTest"}
declare_test!{ExecutiveTests_vmBitwiseLogicOperationTest, "VMTests/vmBitwiseLogicOperation"}
declare_test!{ExecutiveTests_vmBlockInfoTest, "VMTests/vmBlockInfoTest"}
 // TODO [todr] Fails with Signal 11 when using JIT
declare_test!{ExecutiveTests_vmEnvironmentalInfoTest, "VMTests/vmEnvironmentalInfo"}
declare_test!{ExecutiveTests_vmIOandFlowOperationsTest, "VMTests/vmIOandFlowOperations"}
declare_test!{ExecutiveTests_vmLogTest, "VMTests/vmLogTest"}
declare_test!{heavy => ExecutiveTests_vmPerformance, "VMTests/vmPerformance"}
declare_test!{ExecutiveTests_vmPushDupSwapTest, "VMTests/vmPushDupSwapTest"}
declare_test!{ExecutiveTests_vmRandomTest, "VMTests/vmRandomTest"}
declare_test!{ExecutiveTests_vmSha3Test, "VMTests/vmSha3Test"}
declare_test!{ExecutiveTests_vmSystemOperationsTest, "VMTests/vmSystemOperations"}
declare_test!{ExecutiveTests_vmTests, "VMTests/vmTests"}
