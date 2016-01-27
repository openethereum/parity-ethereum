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
			let mut spec = ethereum::new_frontier_like_test();
			let s = PodState::from_json(test.find("pre").unwrap());
			spec.set_genesis_state(s);
			spec.overwrite_genesis(test.find("genesisBlockHeader").unwrap());
			assert!(spec.is_state_root_valid());

			let mut dir = env::temp_dir();
			dir.push(H32::random().hex());
			{
				let client = Client::new(spec, &dir, IoChannel::disconnected()).unwrap();
				for (b, is_valid) in blocks.into_iter() {
					let mut hash = H256::new();
					if Block::is_good(&b) {
						if let Ok(h) = client.import_block(b.clone()) {
							hash = h;
						}
					}
					client.flush_queue();
					let imported_ok = client.import_verified_blocks(&IoChannel::disconnected()) > 0;
					assert_eq!(imported_ok, is_valid);
					if imported_ok {
						flushln!("Imported {}; best block {}", hash, client.chain_info().best_block_hash);
					}
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

declare_test!{BlockchainTests_bcBlockGasLimitTest, "BlockchainTests/bcBlockGasLimitTest"}
declare_test!{BlockchainTests_bcForkBlockTest, "BlockchainTests/bcForkBlockTest"}
declare_test!{BlockchainTests_bcForkStressTest, "BlockchainTests/bcForkStressTest"}				// STILL FAILS
declare_test!{BlockchainTests_bcForkUncle, "BlockchainTests/bcForkUncle"}
declare_test!{BlockchainTests_bcGasPricerTest, "BlockchainTests/bcGasPricerTest"}
declare_test!{BlockchainTests_bcInvalidHeaderTest, "BlockchainTests/bcInvalidHeaderTest"}
declare_test!{BlockchainTests_bcInvalidRLPTest, "BlockchainTests/bcInvalidRLPTest"}
declare_test!{BlockchainTests_bcMultiChainTest, "BlockchainTests/bcMultiChainTest"}
declare_test!{BlockchainTests_bcRPC_API_Test, "BlockchainTests/bcRPC_API_Test"}
declare_test!{BlockchainTests_bcStateTest, "BlockchainTests/bcStateTest"}
declare_test!{BlockchainTests_bcTotalDifficultyTest, "BlockchainTests/bcTotalDifficultyTest"}
declare_test!{BlockchainTests_bcUncleHeaderValiditiy, "BlockchainTests/bcUncleHeaderValiditiy"}
declare_test!{BlockchainTests_bcUncleTest, "BlockchainTests/bcUncleTest"}
declare_test!{BlockchainTests_bcValidBlockTest, "BlockchainTests/bcValidBlockTest"}
declare_test!{BlockchainTests_bcWalletTest, "BlockchainTests/bcWalletTest"}
