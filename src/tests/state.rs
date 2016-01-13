use super::test_common::*;
use state::*;
use ethereum;

pub fn hashmap_h256_h256_from_json(json: &Json) -> HashMap<H256, H256> {
	json.as_object().unwrap().iter().fold(HashMap::new(), |mut m, (key, value)| {
		m.insert(H256::from(&u256_from_str(key)), H256::from(&u256_from_json(value)));
		m
	})
}

pub fn map_h256_h256_from_json(json: &Json) -> BTreeMap<H256, H256> {
	json.as_object().unwrap().iter().fold(BTreeMap::new(), |mut m, (key, value)| {
		m.insert(H256::from(&u256_from_str(key)), H256::from(&u256_from_json(value)));
		m
	})
}

/// Translate the JSON object into a hash map of account information ready for insertion into State.
pub fn pod_map_from_json(json: &Json) -> BTreeMap<Address, PodAccount> {
	json.as_object().unwrap().iter().fold(BTreeMap::new(), |mut state, (address, acc)| {
		let balance = acc.find("balance").map(&u256_from_json);
		let nonce = acc.find("nonce").map(&u256_from_json);
		let storage = acc.find("storage").map(&map_h256_h256_from_json);;
		let code = acc.find("code").map(&bytes_from_json);
		if balance.is_some() || nonce.is_some() || storage.is_some() || code.is_some() {
			state.insert(address_from_hex(address), PodAccount{
				balance: balance.unwrap_or(U256::zero()),
				nonce: nonce.unwrap_or(U256::zero()),
				storage: storage.unwrap_or(BTreeMap::new()),
				code: code.unwrap_or(Vec::new())
			});
		}
		state
	})
}

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
		let pre = pod_map_from_json(&test["pre"]);
		let post = pod_map_from_json(&test["post"]);
		// TODO: read test["logs"]

		println!("Transaction: {:?}", t);
		println!("Env: {:?}", env);

		let mut s = State::new_temp();
		s.populate_from(pre);

		s.apply(&env, engine.deref(), &t).unwrap();
		let our_post = s.to_pod_map();

		if fail_unless(s.root() == &post_state_root) {
			println!("DIFF:\n{:?}", pod_map_diff(&post, &our_post));
		}

		// TODO: Compare logs.
	}
	for f in failed.iter() {
		println!("FAILED: {:?}", f);
	}
	failed
}

declare_test!{StateTests_stExample, "StateTests/stExample"}
