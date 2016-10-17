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
use client::{BlockChainClient, Client, ClientConfig};
use block::Block;
use ethereum;
use tests::helpers::*;
use devtools::*;
use spec::Genesis;
use ethjson;
use miner::Miner;
use io::IoChannel;

pub fn json_chain_test(json_data: &[u8], era: ChainEra) -> Vec<String> {
	init_log();
	let tests = ethjson::blockchain::Test::load(json_data).unwrap();
	let mut failed = Vec::new();

	for (name, blockchain) in tests.into_iter() {
		let mut fail = false;
		{
			let mut fail_unless = |cond: bool| if !cond && !fail {
				failed.push(name.clone());
				flushln!("FAIL");
				fail = true;
				true
			} else {false};

			flush!("   - {}...", name);

			let spec = {
				let genesis = Genesis::from(blockchain.genesis());
				let state = From::from(blockchain.pre_state.clone());
				let mut spec = match era {
					ChainEra::Frontier => ethereum::new_frontier_test(),
					ChainEra::Homestead => ethereum::new_homestead_test(),
					ChainEra::Eip150 => ethereum::new_eip150_test(),
					ChainEra::TransitionTest => ethereum::new_transition_test(),
				};
				spec.set_genesis_state(state);
				spec.overwrite_genesis_params(genesis);
				assert!(spec.is_state_root_valid());
				spec
			};

			let temp = RandomTempPath::new();
			{
				let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
				let client = Client::new(
					ClientConfig::default(),
					&spec,
					temp.as_path(),
					Arc::new(Miner::with_spec(&spec)),
					IoChannel::disconnected(),
					&db_config,
				).unwrap();
				for b in &blockchain.blocks_rlp() {
					if Block::is_good(&b) {
						let _ = client.import_block(b.clone());
						client.flush_queue();
						client.import_verified_blocks();
					}
				}
				fail_unless(client.chain_info().best_block_hash == blockchain.best_block.into());
			}
		}

		if !fail {
			flushln!("ok");
		}
	}

	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

mod frontier_era_tests {
	use tests::helpers::*;
	use super::json_chain_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_chain_test(json_data, ChainEra::Frontier)
	}

	declare_test!{BlockchainTests_bcBlockGasLimitTest, "BlockchainTests/bcBlockGasLimitTest"}
	declare_test!{BlockchainTests_bcForkBlockTest, "BlockchainTests/bcForkBlockTest"}
	declare_test!{BlockchainTests_bcForkStressTest, "BlockchainTests/bcForkStressTest"}
	declare_test!{BlockchainTests_bcForkUncle, "BlockchainTests/bcForkUncle"}
	declare_test!{BlockchainTests_bcGasPricerTest, "BlockchainTests/bcGasPricerTest"}
	declare_test!{BlockchainTests_bcInvalidHeaderTest, "BlockchainTests/bcInvalidHeaderTest"}
	// TODO [ToDr] Ignored because of incorrect JSON (https://github.com/ethereum/tests/pull/113)
	declare_test!{ignore => BlockchainTests_bcInvalidRLPTest, "BlockchainTests/bcInvalidRLPTest"}
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
}

mod transition_tests {
	use tests::helpers::*;
	use super::json_chain_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_chain_test(json_data, ChainEra::TransitionTest)
	}

	declare_test!{BlockchainTests_TestNetwork_bcSimpleTransitionTest, "BlockchainTests/TestNetwork/bcSimpleTransitionTest"}
	declare_test!{BlockchainTests_TestNetwork_bcTheDaoTest, "BlockchainTests/TestNetwork/bcTheDaoTest"}
	declare_test!{BlockchainTests_TestNetwork_bcEIP150Test, "BlockchainTests/TestNetwork/bcEIP150Test"}
}

mod eip150_blockchain_tests {
	use tests::helpers::*;
	use super::json_chain_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_chain_test(json_data, ChainEra::Eip150)
	}

	declare_test!{BlockchainTests_EIP150_bcBlockGasLimitTest, "BlockchainTests/EIP150/bcBlockGasLimitTest"}
	declare_test!{BlockchainTests_EIP150_bcForkStressTest, "BlockchainTests/EIP150/bcForkStressTest"}
	declare_test!{BlockchainTests_EIP150_bcGasPricerTest, "BlockchainTests/EIP150/bcGasPricerTest"}
	declare_test!{BlockchainTests_EIP150_bcInvalidHeaderTest, "BlockchainTests/EIP150/bcInvalidHeaderTest"}
	declare_test!{BlockchainTests_EIP150_bcInvalidRLPTest, "BlockchainTests/EIP150/bcInvalidRLPTest"}
	declare_test!{BlockchainTests_EIP150_bcMultiChainTest, "BlockchainTests/EIP150/bcMultiChainTest"}
	declare_test!{BlockchainTests_EIP150_bcRPC_API_Test, "BlockchainTests/EIP150/bcRPC_API_Test"}
	declare_test!{BlockchainTests_EIP150_bcStateTest, "BlockchainTests/EIP150/bcStateTest"}
	declare_test!{BlockchainTests_EIP150_bcTotalDifficultyTest, "BlockchainTests/EIP150/bcTotalDifficultyTest"}
	declare_test!{BlockchainTests_EIP150_bcUncleHeaderValiditiy, "BlockchainTests/EIP150/bcUncleHeaderValiditiy"}
	declare_test!{BlockchainTests_EIP150_bcUncleTest, "BlockchainTests/EIP150/bcUncleTest"}
	declare_test!{BlockchainTests_EIP150_bcValidBlockTest, "BlockchainTests/EIP150/bcValidBlockTest"}
	declare_test!{BlockchainTests_EIP150_bcWalletTest, "BlockchainTests/EIP150/bcWalletTest"}
}
