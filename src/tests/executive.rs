use super::test_common::*;
use state::*;
use executive::*;
use spec::*;
use engine::*;
use evm;
use evm::{Schedule, Ext, Factory};
use ethereum;

struct TestEngine {
	spec: Spec,
	stack_limit: usize
}

impl TestEngine {
	fn new(stack_limit: usize) -> TestEngine {
		TestEngine {
			spec: ethereum::new_frontier_test(),
			stack_limit: stack_limit 
		}
	}
}

impl Engine for TestEngine {
	fn name(&self) -> &str { "TestEngine" }
	fn spec(&self) -> &Spec { &self.spec }
	fn schedule(&self, _env_info: &EnvInfo) -> Schedule { 
		let mut schedule = Schedule::new_frontier();
		schedule.stack_limit = self.stack_limit; 
		schedule
	}
}

struct CallCreate {
	data: Bytes,
	destination: Address,
	_gas_limit: U256,
	value: U256
}

/// Tiny wrapper around executive externalities.
/// Stores callcreates.
struct TestExt<'a> {
	ext: Externalities<'a>,
	callcreates: Vec<CallCreate>
}

impl<'a> TestExt<'a> {
	fn new(ext: Externalities<'a>) -> TestExt {
		TestExt {
			ext: ext,
			callcreates: vec![]
		}
	}
}

impl<'a> Ext for TestExt<'a> {
	fn sload(&self, key: &H256) -> H256 {
		self.ext.sload(key)
	}

	fn sstore(&mut self, key: H256, value: H256) {
		self.ext.sstore(key, value)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.ext.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		self.ext.blockhash(number)
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> (U256, Option<Address>) {
		// in call and create we need to check if we exited with insufficient balance or max limit reached.
		// in case of reaching max depth, we should store callcreates. Otherwise, ignore.
		let res = self.ext.create(gas, value, code);
		let ext = &self.ext;
		match res {
			// just record call create
			(gas_left, Some(address)) => {
				self.callcreates.push(CallCreate {
					data: code.to_vec(),
					destination: address.clone(),
					_gas_limit: *gas,
					value: *value
				});
				(gas_left, Some(address))
			},
			// creation failed only due to reaching max_depth
			(gas_left, None) if ext.state.balance(&ext.params.address) >= *value => {
				let address = contract_address(&ext.params.address, &ext.state.nonce(&ext.params.address));
				self.callcreates.push(CallCreate {
					data: code.to_vec(),
					// TODO: address is not stored here?
					destination: Address::new(),
					_gas_limit: *gas,
					value: *value
				});
				(gas_left, Some(address))
			},
			other => other
		}
	}

	fn call(&mut self, 
			gas: &U256, 
			call_gas: &U256, 
			receive_address: &Address, 
			value: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> Result<(U256, bool), evm::Error> {
		let res = self.ext.call(gas, call_gas, receive_address, value, data, code_address, output);
		let ext = &self.ext;
		if let &Ok(_some) = &res {
			if ext.state.balance(&ext.params.address) >= *value {
				self.callcreates.push(CallCreate {
					data: data.to_vec(),
					destination: receive_address.clone(),
					_gas_limit: *call_gas,
					value: *value
				});
			}
		}
		res
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
		self.ext.extcode(address)
	}
	
	fn log(&mut self, topics: Vec<H256>, data: Bytes) {
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
}

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();
	for (name, test) in json.as_object().unwrap() {
		println!("name: {:?}", name);
		// sync io is usefull when something crashes in jit
		//::std::io::stdout().write(&name.as_bytes());
		//::std::io::stdout().write(b"\n");
		//::std::io::stdout().flush();
		let mut fail = false;
		//let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.to_string()); fail = true };
		let mut fail_unless = |cond: bool, s: &str | if !cond && !fail { failed.push(name.to_string() + ": "+ s); fail = true };
	
		// test env
		let mut state = State::new_temp();

		test.find("pre").map(|pre| for (addr, s) in pre.as_object().unwrap() {
			let address = Address::from(addr.as_ref());
			let balance = xjson!(&s["balance"]);
			let code = xjson!(&s["code"]);
			let _nonce: U256 = xjson!(&s["nonce"]);

			state.new_contract(&address);
			state.add_balance(&address, &balance);
			state.init_code(&address, code);
			BTreeMap::from_json(&s["storage"]).into_iter().foreach(|(k, v)| state.set_storage(&address, k, v));
		});

		let mut info = EnvInfo::new();

		test.find("env").map(|env| {
			info.author = xjson!(&env["currentCoinbase"]);
			info.difficulty = xjson!(&env["currentDifficulty"]);
			info.gas_limit = xjson!(&env["currentGasLimit"]);
			info.number = xjson!(&env["currentNumber"]);
			info.timestamp = xjson!(&env["currentTimestamp"]);
		});

		let engine = TestEngine::new(0);

		// params
		let mut params = ActionParams::new();
		test.find("exec").map(|exec| {
			params.address = xjson!(&exec["address"]);
			params.sender = xjson!(&exec["caller"]);
			params.origin = xjson!(&exec["origin"]);
			params.code = xjson!(&exec["code"]);
			params.data = xjson!(&exec["data"]);
			params.gas = xjson!(&exec["gas"]);
			params.gas_price = xjson!(&exec["gasPrice"]);
			params.value = xjson!(&exec["value"]);
		});

		let out_of_gas = test.find("callcreates").map(|_calls| {
		}).is_none();
		
		let mut substate = Substate::new();
		let mut output = vec![];

		// execute
		let (res, callcreates) = {
			let ex = Externalities::new(&mut state, &info, &engine, 0, &params, &mut substate, OutputPolicy::Return(BytesRef::Flexible(&mut output)));
			let mut test_ext = TestExt::new(ex);
			let evm = Factory::create();
			let res = evm.exec(&params, &mut test_ext);
			(res, test_ext.callcreates)
		};

		// then validate
		match res {
			Err(_) => fail_unless(out_of_gas, "didn't expect to run out of gas."),
			Ok(gas_left) => {
				//println!("name: {}, gas_left : {:?}, expected: {:?}", name, gas_left, U256::from(&test["gas"]));
				fail_unless(!out_of_gas, "expected to run out of gas.");
				fail_unless(gas_left == xjson!(&test["gas"]), "gas_left is incorrect");
				fail_unless(output == Bytes::from_json(&test["out"]), "output is incorrect");


				test.find("post").map(|pre| for (addr, s) in pre.as_object().unwrap() {
					let address = Address::from(addr.as_ref());

					fail_unless(state.code(&address).unwrap_or(vec![]) == Bytes::from_json(&s["code"]), "code is incorrect");
					fail_unless(state.balance(&address) == xjson!(&s["balance"]), "balance is incorrect");
					fail_unless(state.nonce(&address) == xjson!(&s["nonce"]), "nonce is incorrect");
					BTreeMap::from_json(&s["storage"]).iter().foreach(|(k, v)| fail_unless(&state.storage_at(&address, &k) == v, "storage is incorrect"));
				});

				let cc = test["callcreates"].as_array().unwrap();
				fail_unless(callcreates.len() == cc.len(), "callcreates does not match");
				for i in 0..cc.len() {
					let is = &callcreates[i];
					let expected = &cc[i];
					fail_unless(is.data == Bytes::from_json(&expected["data"]), "callcreates data is incorrect");
					fail_unless(is.destination == xjson!(&expected["destination"]), "callcreates destination is incorrect");
					fail_unless(is.value == xjson!(&expected["value"]), "callcreates value is incorrect");

					// TODO: call_gas is calculated in externalities and is not exposed to TestExt.
					// maybe move it to it's own function to simplify calculation?
					//println!("name: {:?}, is {:?}, expected: {:?}", name, is.gas_limit, U256::from(&expected["gasLimit"]));
					//fail_unless(is.gas_limit == U256::from(&expected["gasLimit"]), "callcreates gas_limit is incorrect");
				}
			}
		}
	}


	for f in failed.iter() {
		println!("FAILED: {:?}", f);
	}

	//assert!(false);
	failed
}

declare_test!{ExecutiveTests_vmArithmeticTest, "VMTests/vmArithmeticTest"}
declare_test!{ExecutiveTests_vmBitwiseLogicOperationTest, "VMTests/vmBitwiseLogicOperationTest"}
// this one crashes with some vm internal error. Separately they pass.
declare_test_ignore!{ExecutiveTests_vmBlockInfoTest, "VMTests/vmBlockInfoTest"}
declare_test!{ExecutiveTests_vmEnvironmentalInfoTest, "VMTests/vmEnvironmentalInfoTest"}
declare_test!{ExecutiveTests_vmIOandFlowOperationsTest, "VMTests/vmIOandFlowOperationsTest"}
// this one take way too long.
declare_test_ignore!{ExecutiveTests_vmInputLimits, "VMTests/vmInputLimits"}
declare_test!{ExecutiveTests_vmLogTest, "VMTests/vmLogTest"}
declare_test!{ExecutiveTests_vmPerformanceTest, "VMTests/vmPerformanceTest"}
declare_test!{ExecutiveTests_vmPushDupSwapTest, "VMTests/vmPushDupSwapTest"}
declare_test!{ExecutiveTests_vmSha3Test, "VMTests/vmSha3Test"}
declare_test!{ExecutiveTests_vmSystemOperationsTest, "VMTests/vmSystemOperationsTest"}
declare_test!{ExecutiveTests_vmtests, "VMTests/vmtests"}
