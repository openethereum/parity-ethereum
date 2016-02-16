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

use super::test_common::*;
use state::*;
use executive::*;
use spec::*;
use engine::*;
use evm;
use evm::{ContractCreateResult, Ext, Factory, MessageCallResult, Schedule, VMType};
use ethereum;
use externalities::*;
use substate::*;
use tests::helpers::*;

struct TestEngineFrontier {
	vm_factory: Factory,
	spec: Spec,
	max_depth: usize,
}

impl TestEngineFrontier {
	fn new(max_depth: usize, vm_type: VMType) -> TestEngineFrontier {
		TestEngineFrontier {
			vm_factory: Factory::new(vm_type),
			spec: ethereum::new_frontier_test(),
			max_depth: max_depth,
		}
	}
}

impl Engine for TestEngineFrontier {
	fn name(&self) -> &str {
		"TestEngine"
	}
	fn spec(&self) -> &Spec {
		&self.spec
	}
	fn vm_factory(&self) -> &Factory {
		&self.vm_factory
	}
	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		let mut schedule = Schedule::new_frontier();
		schedule.max_depth = self.max_depth;
		schedule
	}
}

struct CallCreate {
	data: Bytes,
	destination: Option<Address>,
	gas_limit: U256,
	value: U256,
}

/// Tiny wrapper around executive externalities.
/// Stores callcreates.
struct TestExt<'a> {
	ext: Externalities<'a>,
	callcreates: Vec<CallCreate>,
	contract_address: Address,
}

impl<'a> TestExt<'a> {
	fn new(state: &'a mut State,
	       info: &'a EnvInfo,
	       engine: &'a Engine,
	       depth: usize,
	       origin_info: OriginInfo,
	       substate: &'a mut Substate,
	       output: OutputPolicy<'a>,
	       address: Address)
	       -> Self {
		TestExt {
			contract_address: contract_address(&address, &state.nonce(&address)),
			ext: Externalities::new(state, info, engine, depth, origin_info, substate, output),
			callcreates: vec![],
		}
	}
}

impl<'a> Ext for TestExt<'a> {
	fn storage_at(&self, key: &H256) -> H256 {
		self.ext.storage_at(key)
	}

	fn set_storage(&mut self, key: H256, value: H256) {
		self.ext.set_storage(key, value)
	}

	fn exists(&self, address: &Address) -> bool {
		self.ext.exists(address)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.ext.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		self.ext.blockhash(number)
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> ContractCreateResult {
		self.callcreates.push(CallCreate {
			data: code.to_vec(),
			destination: None,
			gas_limit: *gas,
			value: *value,
		});
		ContractCreateResult::Created(self.contract_address.clone(), *gas)
	}

	fn call(&mut self,
	        gas: &U256,
	        _sender_address: &Address,
	        receive_address: &Address,
	        value: Option<U256>,
	        data: &[u8],
	        _code_address: &Address,
	        _output: &mut [u8])
	        -> MessageCallResult {
		self.callcreates.push(CallCreate {
			data: data.to_vec(),
			destination: Some(receive_address.clone()),
			gas_limit: *gas,
			value: value.unwrap(),
		});
		MessageCallResult::Success(*gas)
	}

	fn extcode(&self, address: &Address) -> Bytes {
		self.ext.extcode(address)
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) {
		self.ext.log(topics, data)
	}

	fn ret(&mut self, gas: &U256, data: &[u8]) -> Result<U256, evm::Error> {
		self.ext.ret(gas, data)
	}

	fn suicide(&mut self, refund_address: &Address) {
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

	fn inc_sstore_clears(&mut self) {
		self.ext.inc_sstore_clears()
	}
}

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let vms = VMType::all();
	vms.iter()
	   .flat_map(|vm| do_json_test_for(vm, json_data))
	   .collect()
}

fn do_json_test_for(vm: &VMType, json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();
	for (name, test) in json.as_object().unwrap() {
		println!("name: {:?}", name);
		// sync io is usefull when something crashes in jit
		// ::std::io::stdout().write(&name.as_bytes());
		// ::std::io::stdout().write(b"\n");
		// ::std::io::stdout().flush();
		let mut fail = false;
		// let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.to_string()); fail = true };
		let mut fail_unless = |cond: bool, s: &str| {
			if !cond && !fail {
				failed.push(format!("[{}] {}: {}", vm, name, s));
				fail = true
			}
		};

		// test env
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();

		test.find("pre").map(|pre| {
			for (addr, s) in pre.as_object().unwrap() {
				let address = Address::from(addr.as_ref());
				let balance = xjson!(&s["balance"]);
				let code = xjson!(&s["code"]);
				let _nonce: U256 = xjson!(&s["nonce"]);

				state.new_contract(&address, balance);
				state.init_code(&address, code);
				BTreeMap::from_json(&s["storage"]).into_iter().foreach(|(k, v)| state.set_storage(&address, k, v));
			}
		});

		let info = test.find("env")
		               .map(|env| EnvInfo::from_json(env))
		               .unwrap_or_default();

		let engine = TestEngineFrontier::new(1, vm.clone());

		// params
		let mut params = ActionParams::default();
		test.find("exec").map(|exec| {
			params.address = xjson!(&exec["address"]);
			params.sender = xjson!(&exec["caller"]);
			params.origin = xjson!(&exec["origin"]);
			params.code = xjson!(&exec["code"]);
			params.data = xjson!(&exec["data"]);
			params.gas = xjson!(&exec["gas"]);
			params.gas_price = xjson!(&exec["gasPrice"]);
			params.value = ActionValue::Transfer(xjson!(&exec["value"]));
		});

		let out_of_gas = test.find("callcreates")
		                     .map(|_calls| {
			                    })
		                     .is_none();

		let mut substate = Substate::new();
		let mut output = vec![];

		// execute
		let (res, callcreates) = {
			let mut ex = TestExt::new(&mut state,
			                          &info,
			                          &engine,
			                          0,
			                          OriginInfo::from(&params),
			                          &mut substate,
			                          OutputPolicy::Return(BytesRef::Flexible(&mut output)),
			                          params.address.clone());
			let evm = engine.vm_factory().create();
			let res = evm.exec(params, &mut ex);
			(res, ex.callcreates)
		};

		// then validate
		match res {
			Err(_) => fail_unless(out_of_gas, "didn't expect to run out of gas."),
			Ok(gas_left) => {
				// println!("name: {}, gas_left : {:?}", name, gas_left);
				fail_unless(!out_of_gas, "expected to run out of gas.");
				fail_unless(gas_left == xjson!(&test["gas"]), "gas_left is incorrect");
				fail_unless(output == Bytes::from_json(&test["out"]), "output is incorrect");

				test.find("post").map(|pre| {
					for (addr, s) in pre.as_object().unwrap() {
						let address = Address::from(addr.as_ref());

						fail_unless(state.code(&address).unwrap_or_else(|| vec![]) == Bytes::from_json(&s["code"]), "code is incorrect");
						fail_unless(state.balance(&address) == xjson!(&s["balance"]), "balance is incorrect");
						fail_unless(state.nonce(&address) == xjson!(&s["nonce"]), "nonce is incorrect");
						BTreeMap::from_json(&s["storage"])
							.iter()
							.foreach(|(k, v)| fail_unless(&state.storage_at(&address, &k) == v, "storage is incorrect"));
					}
				});

				let cc = test["callcreates"].as_array().unwrap();
				fail_unless(callcreates.len() == cc.len(), "callcreates does not match");
				for i in 0..cc.len() {
					let callcreate = &callcreates[i];
					let expected = &cc[i];
					fail_unless(callcreate.data == Bytes::from_json(&expected["data"]), "callcreates data is incorrect");
					fail_unless(callcreate.destination == xjson!(&expected["destination"]), "callcreates destination is incorrect");
					fail_unless(callcreate.value == xjson!(&expected["value"]), "callcreates value is incorrect");
					fail_unless(callcreate.gas_limit == xjson!(&expected["gasLimit"]), "callcreates gas_limit is incorrect");
				}
			}
		}
	}


	for f in &failed {
		println!("FAILED: {:?}", f);
	}

	// assert!(false);
	failed
}

declare_test!{ExecutiveTests_vmArithmeticTest, "VMTests/vmArithmeticTest"}
declare_test!{ExecutiveTests_vmBitwiseLogicOperationTest, "VMTests/vmBitwiseLogicOperationTest"}
declare_test!{ExecutiveTests_vmBlockInfoTest, "VMTests/vmBlockInfoTest"}
// TODO [todr] Fails with Signal 11 when using JIT
declare_test!{ExecutiveTests_vmEnvironmentalInfoTest, "VMTests/vmEnvironmentalInfoTest"}
declare_test!{ExecutiveTests_vmIOandFlowOperationsTest, "VMTests/vmIOandFlowOperationsTest"}
declare_test!{heavy => ExecutiveTests_vmInputLimits, "VMTests/vmInputLimits"}
declare_test!{ExecutiveTests_vmLogTest, "VMTests/vmLogTest"}
declare_test!{ExecutiveTests_vmPerformanceTest, "VMTests/vmPerformanceTest"}
declare_test!{ExecutiveTests_vmPushDupSwapTest, "VMTests/vmPushDupSwapTest"}
declare_test!{ExecutiveTests_vmSha3Test, "VMTests/vmSha3Test"}
declare_test!{ExecutiveTests_vmSystemOperationsTest, "VMTests/vmSystemOperationsTest"}
declare_test!{ExecutiveTests_vmtests, "VMTests/vmtests"}
