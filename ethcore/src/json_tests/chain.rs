// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::path::Path;
use std::sync::Arc;
use client::{EvmTestClient, Client, ClientConfig, ChainInfo, ImportBlock};
use spec::Genesis;
use ethjson;
use miner::Miner;
use io::IoChannel;
use test_helpers;
use verification::queue::kind::blocks::Unverified;
use super::SKIP_TEST_STATE;
use super::HookType;

/// Run chain jsontests on a given folder.
pub fn run_test_path<H: FnMut(&str, HookType)>(p: &Path, skip: &[&'static str], h: &mut H) {
	::json_tests::test_common::run_test_path(p, skip, json_chain_test, h)
}

/// Run chain jsontests on a given file.
pub fn run_test_file<H: FnMut(&str, HookType)>(p: &Path, h: &mut H) {
	::json_tests::test_common::run_test_file(p, json_chain_test, h)
}

fn skip_test(name: &String) -> bool {
	SKIP_TEST_STATE.block.iter().any(|block_test|block_test.subtests.contains(name))
}

pub fn json_chain_test<H: FnMut(&str, HookType)>(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String> {
	::ethcore_logger::init_log();
	let tests = ethjson::blockchain::Test::load(json_data).unwrap();
	let mut failed = Vec::new();

	for (name, blockchain) in tests.into_iter() {
		if skip_test(&name) {
			println!("   - {} | {:?} Ignoring tests because in skip list", name, blockchain.network);
			continue;
		}
		start_stop_hook(&name, HookType::OnStart);

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
					Some(spec) => spec,
					None => {
						println!("   - {} | {:?} Ignoring tests because of missing spec", name, blockchain.network);
						continue;
					}
				};

				spec.engine = match blockchain.engine {
					ethjson::blockchain::Engine::NoProof => Arc::new(no_proof_engine::NoProofEthashEngine(spec.engine)),
					ethjson::blockchain::Engine::Ethash => spec.engine,
				};
				let genesis = Genesis::from(blockchain.genesis());
				let state = From::from(blockchain.pre_state.clone());
				spec.set_genesis_state(state).expect("Failed to overwrite genesis state");
				spec.overwrite_genesis_params(genesis);
				assert!(spec.is_state_root_valid());
				spec
			};

			{
				let db = test_helpers::new_db();
				let mut config = ClientConfig::default();
				config.history = 8;
				let client = Client::new(
					config,
					&spec,
					db,
					Arc::new(Miner::new_for_tests(&spec, None)),
					IoChannel::disconnected(),
				).unwrap();
				for b in blockchain.blocks_rlp() {
					if let Ok(block) = Unverified::from_rlp(b) {
						let _ = client.import_block(block);
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

		start_stop_hook(&name, HookType::OnStop);
	}

	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

mod no_proof_engine {
	use ethereum::ethash::{Seal};
	use engines::{self, Engine, EthEngine};
	use machine::EthereumMachine;
	use header::{Header, BlockNumber, ExtendedHeader};
	use error::{BlockError, Error};
	use unexpected::OutOfBounds;
	use std::collections::BTreeMap;
	use std::sync::Arc;
	use block::ExecutedBlock;

	/// Run engine but skip some validation for 'NoProof' configuration of json tests.
	#[derive(Clone)]
	pub struct NoProofEthashEngine(pub Arc<EthEngine>);

	impl Engine<EthereumMachine> for NoProofEthashEngine {
		/// No difficulty handle, skip some test.
		fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
			// check the seal fields.
			let _ = Seal::parse_seal(header.seal())?;

			if header.gas_limit() > &0x7fffffffffffffffu64.into() {
				return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: None, max: Some(0x7fffffffffffffffu64.into()), found: header.gas_limit().clone() })));
			}

			Ok(())
		}

		/// No nonce handle, skip test.
		fn verify_block_unordered(&self, _header: &Header) -> Result<(), Error> {
			Ok(())
		}

		// Dumb composition derive.

		fn name(&self) -> &str { self.0.name() }
		fn machine(&self) -> &EthereumMachine { self.0.machine() }
		fn seal_fields(&self, header: &Header) -> usize { self.0.seal_fields(header) }
		fn extra_info(&self, header: &Header) -> BTreeMap<String, String> { self.0.extra_info(header) }
		fn maximum_uncle_count(&self, block: BlockNumber) -> usize { self.0.maximum_uncle_count(block) }
		fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
			self.0.populate_from_parent(header, parent)
		}
		fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
			self.0.on_close_block(block)
		}
		fn verify_local_seal(&self, header: &Header) -> Result<(), Error> {
			self.0.verify_local_seal(header)
		}
		fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
			self.0.verify_block_family(header, parent)
		}
		fn epoch_verifier<'a>(&self, header: &Header, proof: &'a [u8]) -> engines::ConstructedVerifier<'a, EthereumMachine> {
			self.0.epoch_verifier(header, proof)
		}
		fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
			self.0.snapshot_components()
		}
		fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> engines::ForkChoice {
			self.0.fork_choice(new, current)
		}
}


}

#[cfg(test)]
mod block_tests {
	use super::json_chain_test;
	use json_tests::HookType;

	fn do_json_test<H: FnMut(&str, HookType)>(json_data: &[u8], h: &mut H) -> Vec<String> {
		json_chain_test(json_data, h)
	}

	declare_test!{BlockchainTests_bcBlockGasLimitTest, "BlockchainTests/bcBlockGasLimitTest"}
	declare_test!{BlockchainTests_bcExploitTest, "BlockchainTests/bcExploitTest"}
	declare_test!{BlockchainTests_bcForgedTest, "BlockchainTests/bcForgedTest"}
	declare_test!{BlockchainTests_bcForkStressTest, "BlockchainTests/bcForkStressTest"}
	declare_test!{BlockchainTests_bcGasPricerTest, "BlockchainTests/bcGasPricerTest"}
	declare_test!{BlockchainTests_bcInvalidHeaderTest, "BlockchainTests/bcInvalidHeaderTest"}
	declare_test!{BlockchainTests_bcMultiChainTest, "BlockchainTests/bcMultiChainTest"}
	declare_test!{BlockchainTests_bcRandomBlockhashTest, "BlockchainTests/bcRandomBlockhashTest"}
	declare_test!{BlockchainTests_bcStateTest, "BlockchainTests/bcStateTests"}
	declare_test!{BlockchainTests_bcTotalDifficultyTest, "BlockchainTests/bcTotalDifficultyTest"}
	declare_test!{BlockchainTests_bcUncleHeaderValidity, "BlockchainTests/bcUncleHeaderValidity"}
	declare_test!{BlockchainTests_bcUncleTest, "BlockchainTests/bcUncleTest"}
	declare_test!{BlockchainTests_bcValidBlockTest, "BlockchainTests/bcValidBlockTest"}
	declare_test!{BlockchainTests_bcWalletTest, "BlockchainTests/bcWalletTest"}

	declare_test!{BlockchainTests_GeneralStateTest_stArgsZeroOneBalance, "BlockchainTests/GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{BlockchainTests_GeneralStateTest_stAttackTest, "BlockchainTests/GeneralStateTests/stAttackTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stBadOpcodeTest, "BlockchainTests/GeneralStateTests/stBadOpcode/"}
	declare_test!{BlockchainTests_GeneralStateTest_stBugsTest, "BlockchainTests/GeneralStateTests/stBugs/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallCodes, "BlockchainTests/GeneralStateTests/stCallCodes/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallCreateCallCodeTest, "BlockchainTests/GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCallDelegateCodesHomestead, "BlockchainTests/GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{BlockchainTests_GeneralStateTest_stChangedEIP150, "BlockchainTests/GeneralStateTests/stChangedEIP150/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCodeSizeLimit, "BlockchainTests/GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{BlockchainTests_GeneralStateTest_stCreate2, "BlockchainTests/GeneralStateTests/stCreate2/"}
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
	declare_test!{BlockchainTests_GeneralStateTest_stStackTests, "BlockchainTests/GeneralStateTests/stStackTests/"}
	declare_test!{BlockchainTests_GeneralStateTest_stStaticCall, "BlockchainTests/GeneralStateTests/stStaticCall/"}
	declare_test!{BlockchainTests_GeneralStateTest_stSystemOperationsTest, "BlockchainTests/GeneralStateTests/stSystemOperationsTest/"}
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
}
