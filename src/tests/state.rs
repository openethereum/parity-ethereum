use super::test_common::*;
use state::*;
use executive::*;
use ethereum;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();

	let engine = ethereum::new_frontier_test().to_engine().unwrap();

	for (name, test) in json.as_object().unwrap() {
		let mut fail = false;
		let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.to_string()); fail = true; true } else {false};

		let t = Transaction::from_json(&test["transaction"]);
		let env = EnvInfo::from_json(&test["env"]);
		let _out = bytes_from_json(&test["out"]);
		let post_state_root = h256_from_json(&test["postStateRoot"]);
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
		let r = s.apply(&env, engine.deref(), &t).unwrap();

		if fail_unless(&r.state_root == &post_state_root) {
			println!("!!! {}: State mismatch (got: {}, expect: {}):", name, r.state_root, post_state_root);
			let our_post = s.to_pod();
			println!("Got:\n{}", our_post);
			println!("Expect:\n{}", post);
			println!("Diff ---expect -> +++got:\n{}", pod_state_diff(&post, &our_post));
		}

		if fail_unless(logs == r.logs) {
			println!("!!! {}: Logs mismatch:", name);
			println!("Got:\n{:?}", r.logs);
			println!("Expect:\n{:?}", logs);
		}

		// TODO: Add extra APIs for output
		//if fail_unless(out == r.)
	}
	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

declare_test!{StateTests_stExample, "StateTests/stExample"}
declare_test!{StateTests_stLogTests, "StateTests/stLogTests"}
