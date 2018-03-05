// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;
use client::{EvmTestClient, Client, ClientConfig, ChainInfo, ImportBlock};
use block::Block;
use spec::Genesis;
use ethjson;
use miner::Miner;
use io::IoChannel;

pub fn json_chain_test(json_data: &[u8]) -> Vec<String> {
	::ethcore_logger::init_log();
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
				let mut spec = match EvmTestClient::spec_from_json(&blockchain.network) {
					Some(spec) => (*spec).clone(),
					None => {
						println!("   - {} | {:?} Ignoring tests because of missing spec", name, blockchain.network);
						continue;
					}
				};

				let genesis = Genesis::from(blockchain.genesis());
				let state = From::from(blockchain.pre_state.clone());
				spec.set_genesis_state(state).expect("Failed to overwrite genesis state");
				spec.overwrite_genesis_params(genesis);
				assert!(spec.is_state_root_valid());
				spec
			};

			{
				let db = Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap_or(0)));
				let mut config = ClientConfig::default();
				config.history = 8;
				let client = Client::new(
					config,
					&spec,
					db,
					Arc::new(Miner::with_spec(&spec)),
					IoChannel::disconnected(),
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

mod block_tests {
	use super::json_chain_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_chain_test(json_data)
	}

	declare_test!{BlockchainTests_bcBlockGasLimitTest, "BlockchainTests/bcBlockGasLimitTest"}
	declare_test!{BlockchainTests_bcExploitTest, "BlockchainTests/bcExploitTest"}
	declare_test!{BlockchainTests_bcForgedTest, "BlockchainTests/bcForgedTest"}
	declare_test!{BlockchainTests_bcForkStressTest, "BlockchainTests/bcForkStressTest"}
	declare_test!{BlockchainTests_bcGasPricerTest, "BlockchainTests/bcGasPricerTest"}
	declare_test!{BlockchainTests_bcInvalidHeaderTest, "BlockchainTests/bcInvalidHeaderTest"}
	declare_test!{BlockchainTests_bcMultiChainTest, "BlockchainTests/bcMultiChainTest"}
	declare_test!{BlockchainTests_bcRandomBlockhashTest, "BlockchainTests/bcRandomBlockhashTest"}
	declare_test!{BlockchainTests_bcTotalDifficultyTest, "BlockchainTests/bcTotalDifficultyTest"}
	declare_test!{BlockchainTests_bcUncleHeaderValidity, "BlockchainTests/bcUncleHeaderValidity"}
	declare_test!{BlockchainTests_bcUncleTest, "BlockchainTests/bcUncleTest"}
	declare_test!{BlockchainTests_bcValidBlockTest, "BlockchainTests/bcValidBlockTest"}
	declare_test!{BlockchainTests_bcWalletTest, "BlockchainTests/bcWalletTest"}

	declare_test!{BlockchainTests_GeneralStateTest_stAttackTest, "BlockchainTests/GeneralStateTests/stAttackTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stBadOpcodeTest, "BlockchainTests/GeneralStateTests/stBadOpcode/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallCodes, "BlockchainTests/GeneralStateTests/stCallCodes/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stChangedEIP150, "BlockchainTests/GeneralStateTests/stChangedEIP150/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCodeSizeLimit, "BlockchainTests/GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCreateTest, "BlockchainTests/GeneralStateTests/stCreateTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stDelegatecallTestHomestead, "BlockchainTests/GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP150singleCodeGasPrices, "BlockchainTests/GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP150Specific, "BlockchainTests/GeneralStateTests/stEIP150Specific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP158Specific, "BlockchainTests/GeneralStateTests/stEIP158Specific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stExample, "BlockchainTests/GeneralStateTests/stExample/"}
	declare_test!{BlockchainTests_GeneralStateTest_stHomesteadSpecific, "BlockchainTests/GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stInitCodeTest, "BlockchainTests/GeneralStateTests/stInitCodeTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stLogTests, "BlockchainTests/GeneralStateTests/stLogTests/"}
	declare_test!{BlockchainTests_GeneralStateTest_stMemExpandingEIP150Calls, "BlockchainTests/GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{heavy => BlockchainTests_GeneralStateTest_stMemoryStressTest, "BlockchainTests/GeneralStateTests/stMemoryStressTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stMemoryTest, "BlockchainTests/GeneralStateTests/stMemoryTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stNonZeroCallsTest, "BlockchainTests/GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stPreCompiledContracts, "BlockchainTests/GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{heavy => BlockchainTests_GeneralStateTest_stQuadraticComplexityTest, "BlockchainTests/GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRandom, "BlockchainTests/GeneralStateTests/stRandom/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRecursiveCreate, "BlockchainTests/GeneralStateTests/stRecursiveCreate/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRefundTest, "BlockchainTests/GeneralStateTests/stRefundTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stReturnDataTest, "BlockchainTests/GeneralStateTests/stReturnDataTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRevertTest, "BlockchainTests/GeneralStateTests/stRevertTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSolidityTest, "BlockchainTests/GeneralStateTests/stSolidityTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSpecialTest, "BlockchainTests/GeneralStateTests/stSpecialTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stStackTests, "BlockchainTests/GeneralStateTests/stStackTests/"}
	declare_test!{BlockchainTests_GeneralStateTest_stStaticCall, "BlockchainTests/GeneralStateTests/stStaticCall/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSystemOperationsTest, "BlockchainTests/GeneralStateTests/stSystemOperationsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stTransactionTest, "BlockchainTests/GeneralStateTests/stTransactionTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stTransitionTest, "BlockchainTests/GeneralStateTests/stTransitionTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stWalletTest, "BlockchainTests/GeneralStateTests/stWalletTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroCallsRevert, "BlockchainTests/GeneralStateTests/stZeroCallsRevert/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroCallsTest, "BlockchainTests/GeneralStateTests/stZeroCallsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroKnowledge, "BlockchainTests/GeneralStateTests/stZeroKnowledge/"}

	declare_test!{BlockchainTests_TransitionTests_bcEIP158ToByzantium, "BlockchainTests/TransitionTests/bcEIP158ToByzantium/"}
	declare_test!{BlockchainTests_TransitionTests_bcFrontierToHomestead, "BlockchainTests/TransitionTests/bcFrontierToHomestead/"}
	declare_test!{BlockchainTests_TransitionTests_bcHomesteadToDao, "BlockchainTests/TransitionTests/bcHomesteadToDao/"}
	declare_test!{BlockchainTests_TransitionTests_bcHomesteadToEIP150, "BlockchainTests/TransitionTests/bcHomesteadToEIP150/"}
}

