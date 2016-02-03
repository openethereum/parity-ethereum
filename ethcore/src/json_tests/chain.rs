use super::test_common::*;
use client::{BlockChainClient,Client};
use pod_state::*;
use block::Block;
use ethereum;
use tests::helpers::*;

pub fn json_chain_test(json_data: &[u8], era: ChainEra) -> Vec<String> {
	init_log();
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
			let mut spec = match era {
				ChainEra::Frontier => ethereum::new_frontier_test(),
				ChainEra::Homestead => ethereum::new_homestead_test(),
			};
			let s = PodState::from_json(test.find("pre").unwrap());
			spec.set_genesis_state(s);
			spec.overwrite_genesis(test.find("genesisBlockHeader").unwrap());
			assert!(spec.is_state_root_valid());
			let genesis_hash = spec.genesis_header().hash();
			assert_eq!(genesis_hash, H256::from_json(&test.find("genesisBlockHeader").unwrap()["hash"]));

			let temp = RandomTempPath::new();
			{
				let client = Client::new(spec, temp.as_path(), IoChannel::disconnected()).unwrap();
				assert_eq!(client.chain_info().best_block_hash, genesis_hash);
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
		}
		if !fail {
			flush(format!("ok\n"));
		}
	}
	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	json_chain_test(json_data, ChainEra::Frontier)
}

declare_test!{BlockchainTests_bcBlockGasLimitTest, "BlockchainTests/bcBlockGasLimitTest"}
declare_test!{BlockchainTests_bcForkBlockTest, "BlockchainTests/bcForkBlockTest"}
declare_test!{BlockchainTests_bcForkStressTest, "BlockchainTests/bcForkStressTest"}
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

declare_test!{BlockchainTests_RandomTests_bl10251623GO, "BlockchainTests/RandomTests/bl10251623GO"}
declare_test!{BlockchainTests_RandomTests_bl201507071825GO, "BlockchainTests/RandomTests/bl201507071825GO"}
