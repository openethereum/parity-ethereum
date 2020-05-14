// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::path::Path;
use std::sync::Arc;
use client::{Client, ClientConfig};
use client_traits::{ImportBlock, ChainInfo, StateOrBlock, Balance, Nonce, BlockChainClient};
use spec::Genesis;
use ethjson::{
	test_helpers::blockchain,
	spec::State
};
use miner::Miner;
use io::IoChannel;
use test_helpers::{self, EvmTestClient};
use types::{
	verification::Unverified,
	ids::BlockId,
	client_types::StateResult
};
use verification::{VerifierType, queue::kind::BlockLike};
use super::{HookType, SKIP_TESTS};
use rustc_hex::ToHex;
use ethereum_types::{U256, H256};

#[allow(dead_code)]
fn skip_test(name: &String, is_legacy: bool) -> bool {
	let skip_set = if is_legacy {
		&SKIP_TESTS.legacy_block
	} else {
		&SKIP_TESTS.block
	};
	skip_set.iter()
		.any(|block_test| block_test.subtests.contains(name))
}

fn check_poststate(client: &Arc<Client>, test_name: &str, post_state: State) -> bool {
	let mut success = true;

	for (address, expected) in post_state {

		if let Some(expected_balance) = expected.balance {
			let expected_balance : U256 = expected_balance.into();
			let current_balance = client.balance(&address.into(), StateOrBlock::Block(BlockId::Latest)).unwrap();
			if expected_balance != current_balance {
				warn!(target: "json-tests", "{} – Poststate {:?} balance mismatch current={} expected={}",
					test_name, address, current_balance, expected_balance);
				success = false;
			}
		}

		if let Some(expected_nonce) = expected.nonce {
			let expected_nonce : U256 = expected_nonce.into();
			let current_nonce = client.nonce(&address.into(), BlockId::Latest).unwrap();
			if expected_nonce != current_nonce {
				warn!(target: "json-tests", "{} – Poststate {:?} nonce mismatch current={} expected={}",
					test_name, address, current_nonce, expected_nonce);
				success = false;
			}
		}

		if let Some(expected_code) = expected.code {
			let expected_code : String = expected_code.to_hex();
			let current_code = match client.code(&address.into(), StateOrBlock::Block(BlockId::Latest)) {
				StateResult::Some(Some(code)) => code.to_hex(),
				_ => "".to_string(),
			};
			if current_code != expected_code {
				warn!(target: "json-tests", "{} – Poststate {:?} code mismatch current={} expected={}",
					test_name, address, current_code, expected_code);
				success = false;
			}
		}

		if let Some(expected_storage) = expected.storage {
			for (uint_position, uint_expected_value) in expected_storage.iter() {

				let mut position = H256::default();
				uint_position.0.to_big_endian(position.as_fixed_bytes_mut());

				let mut expected_value = H256::default();
				uint_expected_value.0.to_big_endian(expected_value.as_fixed_bytes_mut());

				let current_value = client.storage_at(&address.into(), &position, StateOrBlock::Block(BlockId::Latest)).unwrap();

				if current_value != expected_value {
					warn!(target: "json-tests", "{} – Poststate {:?} state {} mismatch actual={} expected={}",
						test_name, address, position.as_bytes().to_hex::<String>(), current_value.as_bytes().to_hex::<String>(),
						expected_value.as_bytes().to_hex::<String>());
					success = false;
				}
			}
		}

		if expected.builtin.is_some() {
			warn!(target: "json-tests", "{} – Poststate {:?} builtin not supported", test_name, address);
			success = false;
		}
		if expected.constructor.is_some() {
		    warn!(target: "json-tests", "{} – Poststate {:?} constructor not supported", test_name, address);
			success = false;
		}
		if expected.version.is_some() {
			warn!(target: "json-tests", "{} – Poststate {:?} version not supported", test_name, address);
			success = false;
		}
	}
	success
}

#[allow(dead_code)]
pub fn json_chain_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], start_stop_hook: &mut H, is_legacy: bool) -> Vec<String> {
	let _ = ::env_logger::try_init();
	let tests = blockchain::Test::load(json_data)
		.expect(&format!("Could not parse JSON chain test data from {}", path.display()));
	let mut failed = Vec::new();

	for (name, blockchain) in tests.into_iter() {
		if skip_test(&name, is_legacy) {
			println!("   - {} | {:?}: SKIPPED", name, blockchain.network);
			continue;
		}

		start_stop_hook(&name, HookType::OnStart);

		let mut fail = false;
		{
			let mut fail_unless = |cond: bool| {
				if !cond && !fail {
					failed.push(name.clone());
					flushed_writeln!("FAIL");
					fail = true;
					true
				} else {
					false
				}
			};

			flushed_write!("   - {}...", name);

			let spec = {
				let mut spec = match EvmTestClient::fork_spec_from_json(&blockchain.network) {
					Some(spec) => spec,
					None => {
						panic!("Unimplemented chainspec '{:?}' in test '{}'", blockchain.network, name);
					}
				};

				let genesis = Genesis::from(blockchain.genesis());
				let state = From::from(blockchain.pre_state.clone());
				spec.set_genesis_state(state).expect("Failed to overwrite genesis state");
				spec.overwrite_genesis_params(genesis);
				spec
			};

			{
				let db = test_helpers::new_db();
				let mut config = ClientConfig::default();
				if ethjson::test_helpers::blockchain::Engine::NoProof == blockchain.engine {
					config.verifier_type = VerifierType::CanonNoSeal;
					config.check_seal = false;
				}
				config.history = 8;
				config.queue.verifier_settings.num_verifiers = 1;
				let client = Client::new(
					config,
					&spec,
					db,
					Arc::new(Miner::new_for_tests(&spec, None)),
					IoChannel::disconnected(),
				).expect("Failed to instantiate a new Client");

				for b in blockchain.blocks_rlp() {
					let bytes_len = b.len();
					let block = Unverified::from_rlp(b);
					match block {
						Ok(block) => {
							let num = block.header.number();
							let hash = block.hash();
							trace!(target: "json-tests", "{} – Importing {} bytes. Block #{}/{}", name, bytes_len, num, hash);
							let res = client.import_block(block);
							if let Err(e) = res {
								warn!(target: "json-tests", "{} – Error importing block #{}/{}: {:?}", name, num, hash, e);
							}
							client.flush_queue();
						},
						Err(decoder_err) => {
							warn!(target: "json-tests", "Error decoding test block: {:?} ({} bytes)", decoder_err, bytes_len);
						}
					}
				}

				let post_state_success = if let Some(post_state) = blockchain.post_state.clone() {
					check_poststate(&client, &name, post_state)
				} else {
					true
				};

				fail_unless(
					client.chain_info().best_block_hash == blockchain.best_block.into()
					&& post_state_success
				);
			}
		}

		if !fail {
			flushed_writeln!("OK");
		} else {
			flushed_writeln!("FAILED");
		}

		start_stop_hook(&name, HookType::OnStop);
	}

	if failed.len() > 0 {
		println!("!!! {:?} tests failed.", failed.len());
	}
	failed
}

#[cfg(test)]
mod block_tests {
	use std::path::Path;

	use super::json_chain_test;
	use json_tests::HookType;

	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], h: &mut H) -> Vec<String> {
		json_chain_test(path, json_data, h, false)
	}

	declare_test!{BlockchainTests_InvalidBlocks_bcBlockGasLimitTest, "BlockchainTests/InvalidBlocks/bcBlockGasLimitTest/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcForgedTest, "BlockchainTests/InvalidBlocks/bcForgedTest/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcInvalidHeaderTest, "BlockchainTests/InvalidBlocks/bcInvalidHeaderTest/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcMultiChainTest, "BlockchainTests/InvalidBlocks/bcMultiChainTest/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcUncleHeaderValidity, "BlockchainTests/InvalidBlocks/bcUncleHeaderValidity/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcUncleSpecialTests, "BlockchainTests/InvalidBlocks/bcUncleSpecialTests/"}
	declare_test!{BlockchainTests_InvalidBlocks_bcUncleTest, "BlockchainTests/InvalidBlocks/bcUncleTest/"}

	declare_test!{BlockchainTests_ValidBlocks_bcBlockGasLimitTest, "BlockchainTests/ValidBlocks/bcBlockGasLimitTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcExploitTest, "BlockchainTests/ValidBlocks/bcExploitTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcForkStressTest, "BlockchainTests/ValidBlocks/bcForkStressTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcGasPricerTest, "BlockchainTests/ValidBlocks/bcGasPricerTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcMultiChainTest, "BlockchainTests/ValidBlocks/bcMultiChainTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcRandomBlockhashTest, "BlockchainTests/ValidBlocks/bcRandomBlockhashTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcStateTests, "BlockchainTests/ValidBlocks/bcStateTests/"}
	declare_test!{BlockchainTests_ValidBlocks_bcTotalDifficultyTest, "BlockchainTests/ValidBlocks/bcTotalDifficultyTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcUncleSpecialTests, "BlockchainTests/ValidBlocks/bcUncleSpecialTests/"}
	declare_test!{BlockchainTests_ValidBlocks_bcUncleTest, "BlockchainTests/ValidBlocks/bcUncleTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcValidBlockTest, "BlockchainTests/ValidBlocks/bcValidBlockTest/"}
	declare_test!{BlockchainTests_ValidBlocks_bcWalletTest, "BlockchainTests/ValidBlocks/bcWalletTest/"}

	declare_test!{BlockchainTests_GeneralStateTest_stArgsZeroOneBalance, "BlockchainTests/GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{BlockchainTests_GeneralStateTest_stAttackTest, "BlockchainTests/GeneralStateTests/stAttackTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stBadOpcodeTest, "BlockchainTests/GeneralStateTests/stBadOpcode/"}
	declare_test!{BlockchainTests_GeneralStateTest_stBugsTest, "BlockchainTests/GeneralStateTests/stBugs/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallCodes, "BlockchainTests/GeneralStateTests/stCallCodes/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallCreateCallCodeTest, "BlockchainTests/GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stChangedEIP150, "BlockchainTests/GeneralStateTests/stChangedEIP150/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCopyCodeTest, "BlockchainTests/GeneralStateTests/stCodeCopyTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCodeSizeLimit, "BlockchainTests/GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCreate2, "BlockchainTests/GeneralStateTests/stCreate2/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCreateTest, "BlockchainTests/GeneralStateTests/stCreateTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stDelegatecallTestHomestead, "BlockchainTests/GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP150singleCodeGasPrices, "BlockchainTests/GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP150Specific, "BlockchainTests/GeneralStateTests/stEIP150Specific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stEIP158Specific, "BlockchainTests/GeneralStateTests/stEIP158Specific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stExample, "BlockchainTests/GeneralStateTests/stExample/"}
	declare_test!{BlockchainTests_GeneralStateTest_stExtCodeHash, "BlockchainTests/GeneralStateTests/stExtCodeHash/"}
	declare_test!{BlockchainTests_GeneralStateTest_stHomesteadSpecific, "BlockchainTests/GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{BlockchainTests_GeneralStateTest_stInitCodeTest, "BlockchainTests/GeneralStateTests/stInitCodeTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stLogTests, "BlockchainTests/GeneralStateTests/stLogTests/"}
	declare_test!{BlockchainTests_GeneralStateTest_stMemExpandingEIP150Calls, "BlockchainTests/GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{heavy => BlockchainTests_GeneralStateTest_stMemoryStressTest, "BlockchainTests/GeneralStateTests/stMemoryStressTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stMemoryTest, "BlockchainTests/GeneralStateTests/stMemoryTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stNonZeroCallsTest, "BlockchainTests/GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stPreCompiledContracts, "BlockchainTests/GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{BlockchainTests_GeneralStateTest_stPreCompiledContracts2, "BlockchainTests/GeneralStateTests/stPreCompiledContracts2/"}
	declare_test!{heavy => BlockchainTests_GeneralStateTest_stQuadraticComplexityTest, "BlockchainTests/GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRandom, "BlockchainTests/GeneralStateTests/stRandom/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRandom2, "BlockchainTests/GeneralStateTests/stRandom2/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRecursiveCreate, "BlockchainTests/GeneralStateTests/stRecursiveCreate/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRefundTest, "BlockchainTests/GeneralStateTests/stRefundTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stReturnDataTest, "BlockchainTests/GeneralStateTests/stReturnDataTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stRevertTest, "BlockchainTests/GeneralStateTests/stRevertTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stShift, "BlockchainTests/GeneralStateTests/stShift/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSolidityTest, "BlockchainTests/GeneralStateTests/stSolidityTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSpecialTest, "BlockchainTests/GeneralStateTests/stSpecialTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSStoreTest, "BlockchainTests/GeneralStateTests/stSStoreTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stStackTests, "BlockchainTests/GeneralStateTests/stStackTests/"}
	declare_test!{BlockchainTests_GeneralStateTest_stStaticCall, "BlockchainTests/GeneralStateTests/stStaticCall/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSystemOperationsTest, "BlockchainTests/GeneralStateTests/stSystemOperationsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stTimeConsuming, "BlockchainTests/GeneralStateTests/stTimeConsuming/"}
	declare_test!{BlockchainTests_GeneralStateTest_stTransactionTest, "BlockchainTests/GeneralStateTests/stTransactionTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stTransitionTest, "BlockchainTests/GeneralStateTests/stTransitionTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stWalletTest, "BlockchainTests/GeneralStateTests/stWalletTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroCallsRevert, "BlockchainTests/GeneralStateTests/stZeroCallsRevert/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroCallsTest, "BlockchainTests/GeneralStateTests/stZeroCallsTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroKnowledge, "BlockchainTests/GeneralStateTests/stZeroKnowledge/"}
	declare_test!{BlockchainTests_GeneralStateTest_stZeroKnowledge2, "BlockchainTests/GeneralStateTests/stZeroKnowledge2/"}

	declare_test!{BlockchainTests_TransitionTests_bcEIP158ToByzantium, "BlockchainTests/TransitionTests/bcEIP158ToByzantium/"}
	declare_test!{BlockchainTests_TransitionTests_bcFrontierToHomestead, "BlockchainTests/TransitionTests/bcFrontierToHomestead/"}
	declare_test!{BlockchainTests_TransitionTests_bcHomesteadToDao, "BlockchainTests/TransitionTests/bcHomesteadToDao/"}
	declare_test!{BlockchainTests_TransitionTests_bcHomesteadToEIP150, "BlockchainTests/TransitionTests/bcHomesteadToEIP150/"}
	declare_test!{BlockchainTests_TransitionTests_bcByzantiumToConstantinopleFix, "BlockchainTests/TransitionTests/bcByzantiumToConstantinopleFix/"}

	declare_test!{BlockchainTests_RandomStateTest391, "BlockchainTests/randomStatetest391.json"}
}

/// Legacy tests, still keeping it to check if there is any regression in blocks < Instambul HF
#[cfg(test)]
mod block_tests_legacy {
	use std::path::Path;

	use super::json_chain_test;
	use json_tests::HookType;

	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], h: &mut H) -> Vec<String> {
		json_chain_test(path, json_data, h, true)
	}

	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stArgsZeroOneBalance, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stAttackTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stAttackTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stBadOpcode, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stBadOpcode/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stBugs, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stBugs/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCallCodes, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCallCodes/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCallCreateCallCodeTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCallDelegateCodesCallCodeHomestead, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCallDelegateCodesHomestead, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stChangedEIP150, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stChangedEIP150/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCodeCopyTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCodeCopyTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCodeSizeLimit, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCreate2, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCreate2/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stCreateTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stCreateTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stDelegatecallTestHomestead, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stEIP150singleCodeGasPrices, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stEIP150Specific, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stEIP150Specific/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stEIP158Specific, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stEIP158Specific/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stExample, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stExample/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stExtCodeHash, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stExtCodeHash/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stHomesteadSpecific, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stInitCodeTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stInitCodeTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stLogTests, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stLogTests/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stMemExpandingEIP150Calls, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stMemoryStressTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stMemoryStressTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stMemoryTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stMemoryTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stNonZeroCallsTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stPreCompiledContracts, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stPreCompiledContracts2, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stPreCompiledContracts2/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stQuadraticComplexityTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stRandom, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stRandom/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stRandom2, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stRandom2/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stRecursiveCreate, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stRecursiveCreate/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stRefundTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stRefundTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stReturnDataTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stReturnDataTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stRevertTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stRevertTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stShift, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stShift/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stSolidityTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stSolidityTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stSpecialTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stSpecialTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stSStoreTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stSStoreTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stStackTests, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stStackTests/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stStaticCall, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stStaticCall/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stSystemOperationsTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stSystemOperationsTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stTimeConsuming, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stTimeConsuming/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stTransactionTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stTransactionTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stTransitionTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stTransitionTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stWalletTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stWalletTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stZeroCallsRevert, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stZeroCallsRevert/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stZeroCallsTest, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stZeroCallsTest/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stZeroKnowledge, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stZeroKnowledge/"}
	declare_test!{Constantinople_BlockchainTests_GeneralStateTests_stZeroKnowledge2, "LegacyTests/Constantinople/BlockchainTests/GeneralStateTests/stZeroKnowledge2/"}
}
