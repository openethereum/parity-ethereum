use std::env;
use super::test_common::*;
use client::{BlockChainClient,Client};
use pod_state::*;
use block::Block;
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

			let blocks: Vec<(Bytes, bool)> = test["blocks"].as_array().unwrap().iter().map(|e| (xjson!(&e["rlp"]), e.find("blockHeader").is_some())).collect();
			let mut spec = ethereum::new_homestead_test();
			let s = PodState::from_json(test.find("pre").unwrap());
			spec.set_genesis_state(s);
			spec.overwrite_genesis(test.find("genesisBlockHeader").unwrap());
			assert!(spec.is_state_root_valid());

			let mut dir = env::temp_dir();
			dir.push(H32::random().hex());
			{
				let client = Client::new(spec, &dir, IoChannel::disconnected()).unwrap();
				for (b, is_valid) in blocks.into_iter() {
					if Block::is_good(&b) {
						let _ = client.import_block(b.clone());
					}
					client.flush_queue();
					let imported_ok = client.import_verified_blocks(&IoChannel::disconnected()) > 0;
					assert_eq!(imported_ok, is_valid);
				}
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

declare_test!{BlockchainTests_Homestead_bcBlockGasLimitTest, "BlockchainTests/Homestead/bcBlockGasLimitTest"}
declare_test!{BlockchainTests_Homestead_bcForkStressTest, "BlockchainTests/Homestead/bcForkStressTest"}
declare_test!{BlockchainTests_Homestead_bcGasPricerTest, "BlockchainTests/Homestead/bcGasPricerTest"}
declare_test!{BlockchainTests_Homestead_bcInvalidHeaderTest, "BlockchainTests/Homestead/bcInvalidHeaderTest"}
declare_test!{BlockchainTests_Homestead_bcInvalidRLPTest, "BlockchainTests/Homestead/bcInvalidRLPTest"}
declare_test!{BlockchainTests_Homestead_bcMultiChainTest, "BlockchainTests/Homestead/bcMultiChainTest"}
declare_test!{BlockchainTests_Homestead_bcRPC_API_Test, "BlockchainTests/Homestead/bcRPC_API_Test"}
declare_test!{BlockchainTests_Homestead_bcStateTest, "BlockchainTests/Homestead/bcStateTest"}
declare_test!{BlockchainTests_Homestead_bcTotalDifficultyTest, "BlockchainTests/Homestead/bcTotalDifficultyTest"}
declare_test!{BlockchainTests_Homestead_bcUncleHeaderValiditiy, "BlockchainTests/Homestead/bcUncleHeaderValiditiy"}
declare_test!{BlockchainTests_Homestead_bcUncleTest, "BlockchainTests/Homestead/bcUncleTest"}
declare_test!{BlockchainTests_Homestead_bcValidBlockTest, "BlockchainTests/Homestead/bcValidBlockTest"}
declare_test!{BlockchainTests_Homestead_bcWalletTest, "BlockchainTests/Homestead/bcWalletTest"}
