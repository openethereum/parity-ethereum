use super::test_common::*;
use state::*;
use pod_state::*;
use state_diff::*;
use ethereum;

fn flush(s: String) {
	::std::io::stdout().write(s.as_bytes()).unwrap();
	::std::io::stdout().flush().unwrap();
}

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();

	let engine = ethereum::new_frontier_test().to_engine().unwrap();

	flush(format!("\n"));

	for (name, test) in json.as_object().unwrap() {
		let mut fail = false;
		{
			let mut fail_unless = |cond: bool| if !cond && !fail {
				failed.push(name.to_string());
				flush(format!("FAIL\n"));
				fail = true;
				true
			} else {false};

			flush(format!("   - {}...", name));

			let t = Transaction::from_json(&test["transaction"]);
			let env = EnvInfo::from_json(&test["env"]);
			let _out = Bytes::from_json(&test["out"]);
			let post_state_root = xjson!(&test["postStateRoot"]);
			let pre = PodState::from_json(&test["pre"]);
			let post = PodState::from_json(&test["post"]);
			let logs: Vec<_> = test["logs"].as_array().unwrap().iter().map(&LogEntry::from_json).collect();

			//println!("Transaction: {:?}", t);
			//println!("Env: {:?}", env);

			{
				let mut s = State::new_temp();
				s.populate_from(post.clone());
				s.commit();
				assert_eq!(&post_state_root, s.root());
			}

			let mut s = State::new_temp();
			s.populate_from(pre);
			s.commit();
			let res = s.apply(&env, engine.deref(), &t);

			if fail_unless(s.root() == &post_state_root) {
				println!("!!! {}: State mismatch (got: {}, expect: {}):", name, s.root(), post_state_root);
				let our_post = s.to_pod();
				println!("Got:\n{}", our_post);
				println!("Expect:\n{}", post);
				println!("Diff ---expect -> +++got:\n{}", StateDiff::diff_pod(&post, &our_post));
			}

			if let Ok(r) = res {
				if fail_unless(logs == r.logs) {
					println!("!!! {}: Logs mismatch:", name);
					println!("Got:\n{:?}", r.logs);
					println!("Expect:\n{:?}", logs);
				}
			}
		}
		if !fail {
			flush(format!("ok\n"));
		}
		// TODO: Add extra APIs for output
		//if fail_unless(out == r.)
	}
	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

declare_test!{StateTests_stBlockHashTest, "StateTests/stBlockHashTest"}
declare_test!{StateTests_stCallCodes, "StateTests/stCallCodes"}
declare_test_ignore!{StateTests_stCallCreateCallCodeTest, "StateTests/stCallCreateCallCodeTest"}	//<< Out of stack
declare_test!{StateTests_stDelegatecallTest, "StateTests/stDelegatecallTest"}						//<< FAIL - gas too high
declare_test!{StateTests_stExample, "StateTests/stExample"}
declare_test!{StateTests_stInitCodeTest, "StateTests/stInitCodeTest"}
declare_test!{StateTests_stLogTests, "StateTests/stLogTests"}
declare_test!{StateTests_stMemoryStressTest, "StateTests/stMemoryStressTest"}
declare_test!{StateTests_stMemoryTest, "StateTests/stMemoryTest"}
declare_test!{StateTests_stPreCompiledContracts, "StateTests/stPreCompiledContracts"}
declare_test_ignore!{StateTests_stQuadraticComplexityTest, "StateTests/stQuadraticComplexityTest"}	//<< Too long
declare_test_ignore!{StateTests_stRecursiveCreate, "StateTests/stRecursiveCreate"}					//<< Out of stack 
declare_test!{StateTests_stRefundTest, "StateTests/stRefundTest"}
declare_test!{StateTests_stSolidityTest, "StateTests/stSolidityTest"}
declare_test_ignore!{StateTests_stSpecialTest, "StateTests/stSpecialTest"}							//<< Signal 11
declare_test_ignore!{StateTests_stSystemOperationsTest, "StateTests/stSystemOperationsTest"}		//<< Signal 11
declare_test!{StateTests_stTransactionTest, "StateTests/stTransactionTest"}
declare_test!{StateTests_stTransitionTest, "StateTests/stTransitionTest"}
declare_test!{StateTests_stWalletTest, "StateTests/stWalletTest"}
