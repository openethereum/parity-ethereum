use super::test_common::*;
use state::*;
use executive::*;
use spec::*;
use engine::*;
use evm::{Schedule};
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

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();
	for (name, test) in json.as_object().unwrap() {
		::std::io::stdout().write(&name.as_bytes());
		::std::io::stdout().write(b"\n");
		::std::io::stdout().flush();
		//println!("name: {:?}", name);
		let mut fail = false;
		//let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.to_string()); fail = true };
		let mut fail_unless = |cond: bool, s: &str | if !cond && !fail { failed.push(name.to_string() + ": "+ s); fail = true };
	
		// test env
		let mut state = State::new_temp();

		test.find("pre").map(|pre| for (addr, s) in pre.as_object().unwrap() {
			let address = address_from_str(addr);
			let balance = u256_from_json(&s["balance"]);
			let code = bytes_from_json(&s["code"]);
			let nonce = u256_from_json(&s["nonce"]);

			state.new_contract(&address);
			state.add_balance(&address, &balance);
			state.init_code(&address, code);

			for (k, v) in s["storage"].as_object().unwrap() {
				let key = H256::from(&u256_from_str(k));
				let val = H256::from(&u256_from_json(v));
				state.set_storage(&address, key, val);
			}
		});

		let mut info = EnvInfo::new();

		test.find("env").map(|env| {
			info.author = address_from_json(&env["currentCoinbase"]);
			info.difficulty = u256_from_json(&env["currentDifficulty"]);
			info.gas_limit = u256_from_json(&env["currentGasLimit"]);
			info.number = u256_from_json(&env["currentNumber"]).low_u64();
			info.timestamp = u256_from_json(&env["currentTimestamp"]).low_u64();
		});

		let engine = TestEngine::new(0);

		// params
		let mut params = ActionParams::new();
		test.find("exec").map(|exec| {
			params.address = address_from_json(&exec["address"]);
			params.sender = address_from_json(&exec["caller"]);
			params.origin = address_from_json(&exec["origin"]);
			params.code = bytes_from_json(&exec["code"]);
			params.data = bytes_from_json(&exec["data"]);
			params.gas = u256_from_json(&exec["gas"]);
			params.gas_price = u256_from_json(&exec["gasPrice"]);
			params.value = u256_from_json(&exec["value"]);
		});

		let out_of_gas = test.find("callcreates").map(|calls| {
		}).is_none();
		
		let mut substate = Substate::new();

		// execute
		let res = {
			let mut ex = Executive::new(&mut state, &info, &engine);
			ex.call(&params, &mut substate, &mut [])
		};

		// then validate
		match res {
			Err(_) => fail_unless(out_of_gas, "didn't expect to run out of gas."),
			Ok(gas_left) => {
				fail_unless(!out_of_gas, "expected to run out of gas.");
				fail_unless(gas_left == u256_from_json(&test["gas"]), "gas_left is incorrect");
				println!("name: {}, gas_left : {:?}, expected: {:?}", name, gas_left, u256_from_json(&test["gas"]));
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
declare_test!{ExecutiveTests_vmSha3Test, "VMTests/vmSha3Test"}
declare_test!{ExecutiveTests_vmBitwiseLogicOperationTest, "VMTests/vmBitwiseLogicOperationTest"}
//declare_test!{ExecutiveTests_vmBlockInfoTest, "VMTests/vmBlockInfoTest"}
declare_test!{ExecutiveTests_vmEnvironmentalInfoTest, "VMTests/vmEnvironmentalInfoTest"}
