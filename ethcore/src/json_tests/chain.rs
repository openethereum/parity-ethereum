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

			let mut spec = match era {
				ChainEra::Frontier => ethereum::new_frontier_test(),
				ChainEra::Homestead => ethereum::new_homestead_test(),
			};

			let genesis = Genesis::from(blockchain.genesis());
			let state = From::from(blockchain.pre_state.clone());
			spec.set_genesis_state(state);
			spec.overwrite_genesis_params(genesis);
			assert!(spec.is_state_root_valid());

			let temp = RandomTempPath::new();
			{
				let client = Client::new(ClientConfig::default(), spec, temp.as_path(), IoChannel::disconnected());
				for b in &blockchain.blocks_rlp() {
					if Block::is_good(&b) {
						let _ = client.import_block(b.clone());
						client.flush_queue();
						client.import_verified_blocks(&IoChannel::disconnected());
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
