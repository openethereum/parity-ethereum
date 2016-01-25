use std::env;
use super::test_common::*;
use client::{BlockChainClient,Client};
use pod_state::*;
use ethereum;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();

	for (name, test) in json.as_object().unwrap() {
		let mut fail = false;
		{
			let mut fail_unless = |cond: bool| if !cond && !fail {
				failed.push(name.clone());
				flush(format!("FAIL\n"));
				fail = true;
				true
			} else {false};

			flush(format!("   - {}...", name));

			let blocks: Vec<Bytes> = test["blocks"].as_array().unwrap().iter().map(|e| xjson!(&e["rlp"])).collect();
			let mut spec = ethereum::new_frontier_like_test();
			spec.set_genesis_state(PodState::from_json(test.find("pre").unwrap()));
			spec.overwrite_genesis(test.find("genesisBlockHeader").unwrap());
			assert!(spec.is_state_root_valid());

			let mut dir = env::temp_dir();
			dir.push(H32::random().hex());
			{
				let client = Client::new(spec, &dir, IoChannel::disconnected()).unwrap();
				blocks.into_iter().foreach(|b| {
					client.import_block(b).unwrap();
				});
				client.flush_queue();
				client.import_verified_blocks(&IoChannel::disconnected());
				fail_unless(client.chain_info().best_block_hash == H256::from_json(&test["lastblockhash"]));
			}
			fs::remove_dir_all(&dir).unwrap();
		}
		if !fail {
			flush(format!("ok\n"));
		}
	}
	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

// Fails. TODO: figure out why.
//declare_test!{BlockchainTests_bcStateTest, "BlockchainTests/bcStateTest"}

declare_test!{BlockchainTests_bcForkBlockTest, "BlockchainTests/bcForkBlockTest"}
