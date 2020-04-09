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
use super::test_common::*;
use pod::PodState;
use test_helpers::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess};
use types::transaction::SignedTransaction;
use vm::EnvInfo;
use super::SKIP_TESTS;

#[allow(dead_code)]
fn skip_test(subname: &str, chain: &String, number: usize, is_legacy: bool) -> bool {	
	trace!(target: "json-tests", "[state, skip_test] subname: '{}', chain: '{}', number: {} legacy:{}", subname, chain, number, is_legacy);
	let skip_set = if is_legacy {
		&SKIP_TESTS.legacy_state
	} else {
		&SKIP_TESTS.state
	};
	skip_set.iter().any(|state_test|{
		if let Some(subtest) = state_test.subtests.get(subname) {
			trace!(target: "json-tests", "[state, skip_test] Maybe skipping {:?} (legacy:{})", subtest, is_legacy);
			chain == &subtest.chain &&
			(
				subtest.subnumbers[0] == "*" ||
				subtest.subnumbers.contains(&number.to_string())
			)
		} else {
			false
		}
	})
}

#[allow(dead_code)]
pub fn json_chain_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], start_stop_hook: &mut H, is_legacy: bool) -> Vec<String> {
	let _ = ::env_logger::try_init();
	let tests = ethjson::test_helpers::state::Test::load(json_data)
		.expect(&format!("Could not parse JSON state test data from {}", path.display()));
	let mut failed = Vec::new();

	for (name, test) in tests.into_iter() {
		start_stop_hook(&name, HookType::OnStart);

		{
			let multitransaction = test.transaction;
			let env: EnvInfo = test.env.into();
			let pre: PodState = test.pre_state.into();

			for (spec_name, states) in test.post_states {
				let total = states.len();
				let spec = match EvmTestClient::fork_spec_from_json(&spec_name) {
					Some(spec) => spec,
					None => {
						panic!("Unimplemented chainspec '{:?}' in test '{}'", spec_name, name);
					}
				};

				for (i, state) in states.into_iter().enumerate() {
					let info = format!("   - {} | {:?} ({}/{}) ...", name, spec_name, i + 1, total);
					if skip_test(&name, &spec.name, i + 1, is_legacy) {
						println!("{}: SKIPPED", info);
						continue;
					}

					let post_root: H256 = state.hash.into();
					let transaction: SignedTransaction = multitransaction.select(&state.indexes).into();

					let result = || -> Result<_, EvmTestError> {
						Ok(EvmTestClient::from_pod_state(&spec, pre.clone())?
							.transact(&env, transaction, trace::NoopTracer, trace::NoopVMTracer))
					};
					match result() {
						Err(err) => {
							println!("{} !!! Unexpected internal error: {:?}", info, err);
							flushed_writeln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Ok(TransactSuccess { state_root, .. })) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							flushed_writeln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Err(TransactErr { state_root, ref error, .. })) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							println!("{} !!! Execution error: {:?}", info, error);
							flushed_writeln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Err(TransactErr { error, .. })) => {
							flushed_writeln!("{} ok ({:?})", info, error);
						},
						Ok(_) => {
							flushed_writeln!("{} ok", info);
						},
					}
				}
			}
		}

		start_stop_hook(&name, HookType::OnStop);
	}

	if !failed.is_empty() {
		println!("!!! {:?} tests failed.", failed.len());
	}
	failed
}

#[cfg(test)]
mod state_tests {
	use std::path::Path;

	use super::json_chain_test;
	use json_tests::HookType;

	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], h: &mut H) -> Vec<String> {
		json_chain_test(path, json_data, h, false)
	}

	declare_test!{GeneralStateTest_stArgsZeroOneBalance, "GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{GeneralStateTest_stAttackTest, "GeneralStateTests/stAttackTest/"}
	declare_test!{GeneralStateTest_stBadOpcodeTest, "GeneralStateTests/stBadOpcode/"}
	declare_test!{GeneralStateTest_stBugs, "GeneralStateTests/stBugs/"}
	declare_test!{GeneralStateTest_stCallCodes, "GeneralStateTests/stCallCodes/"}
	declare_test!{GeneralStateTest_stCallCreateCallCodeTest, "GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesHomestead, "GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{GeneralStateTest_stChainId, "GeneralStateTests/stChainId/"}
	declare_test!{GeneralStateTest_stChangedEIP150, "GeneralStateTests/stChangedEIP150/"}
	declare_test!{GeneralStateTest_stCodeCopyTest, "GeneralStateTests/stCodeCopyTest/"}
	declare_test!{GeneralStateTest_stCodeSizeLimit, "GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{GeneralStateTest_stCreate2Test, "GeneralStateTests/stCreate2/"}
	declare_test!{GeneralStateTest_stCreateTest, "GeneralStateTests/stCreateTest/"}
	declare_test!{GeneralStateTest_stDelegatecallTestHomestead, "GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{GeneralStateTest_stEIP150singleCodeGasPrices, "GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{GeneralStateTest_stEIP150Specific, "GeneralStateTests/stEIP150Specific/"}
	declare_test!{GeneralStateTest_stEIP158Specific, "GeneralStateTests/stEIP158Specific/"}
	declare_test!{GeneralStateTest_stEWASMTests, "GeneralStateTests/stEWASMTests/"}
	declare_test!{GeneralStateTest_stExample, "GeneralStateTests/stExample/"}
	declare_test!{GeneralStateTest_stExtCodeHash, "GeneralStateTests/stExtCodeHash/"}
	declare_test!{GeneralStateTest_stHomesteadSpecific, "GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{GeneralStateTest_stInitCodeTest, "GeneralStateTests/stInitCodeTest/"}
	declare_test!{GeneralStateTest_stLogTests, "GeneralStateTests/stLogTests/"}
	declare_test!{GeneralStateTest_stMemExpandingEIP150Calls, "GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{heavy => GeneralStateTest_stMemoryStressTest, "GeneralStateTests/stMemoryStressTest/"}
	declare_test!{GeneralStateTest_stMemoryTest, "GeneralStateTests/stMemoryTest/"}
	declare_test!{GeneralStateTest_stNonZeroCallsTest, "GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{GeneralStateTest_stPreCompiledContracts, "GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{GeneralStateTest_stPreCompiledContracts2, "GeneralStateTests/stPreCompiledContracts2/"}
	declare_test!{heavy => GeneralStateTest_stQuadraticComplexityTest, "GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{GeneralStateTest_stRandom, "GeneralStateTests/stRandom/"}
	declare_test!{GeneralStateTest_stRandom2, "GeneralStateTests/stRandom2/"}
	declare_test!{GeneralStateTest_stRecursiveCreate, "GeneralStateTests/stRecursiveCreate/"}
	declare_test!{GeneralStateTest_stRefundTest, "GeneralStateTests/stRefundTest/"}
	declare_test!{GeneralStateTest_stReturnDataTest, "GeneralStateTests/stReturnDataTest/"}
	// todo[dvdplm]:
	//      "RevertPrecompiledTouch_storage" contains 4 tests, only two fails
	//      "RevertPrecompiledTouchExactOOG" contains a ton of tests, only two fails
	//      "RevertPrecompiledTouch" has 4 tests, 2 failures
	//      Ignored in `currents.json`.
	//      Issues:
	//          https://github.com/OpenEthereum/open-ethereum/issues/11078
	//          https://github.com/OpenEthereum/open-ethereum/issues/11079
	//          https://github.com/OpenEthereum/open-ethereum/issues/11080
	declare_test!{GeneralStateTest_stRevertTest, "GeneralStateTests/stRevertTest/"}
	declare_test!{GeneralStateTest_stSelfBalance, "GeneralStateTests/stSelfBalance/"}
	declare_test!{GeneralStateTest_stShift, "GeneralStateTests/stShift/"}
	declare_test!{GeneralStateTest_stSLoadTest, "GeneralStateTests/stSLoadTest/"}
	declare_test!{GeneralStateTest_stSolidityTest, "GeneralStateTests/stSolidityTest/"}
	declare_test!{GeneralStateTest_stSpecialTest, "GeneralStateTests/stSpecialTest/"}
	declare_test!{GeneralStateTest_stSStoreTest, "GeneralStateTests/stSStoreTest/"}
	declare_test!{GeneralStateTest_stStackTests, "GeneralStateTests/stStackTests/"}
	declare_test!{GeneralStateTest_stStaticCall, "GeneralStateTests/stStaticCall/"}
	declare_test!{GeneralStateTest_stSystemOperationsTest, "GeneralStateTests/stSystemOperationsTest/"}
	declare_test!{GeneralStateTest_stTimeConsuming, "GeneralStateTests/stTimeConsuming/"}
	declare_test!{GeneralStateTest_stTransactionTest, "GeneralStateTests/stTransactionTest/"}
	declare_test!{GeneralStateTest_stTransitionTest, "GeneralStateTests/stTransitionTest/"}
	declare_test!{GeneralStateTest_stWalletTest, "GeneralStateTests/stWalletTest/"}
	declare_test!{GeneralStateTest_stZeroCallsRevert, "GeneralStateTests/stZeroCallsRevert/"}
	declare_test!{GeneralStateTest_stZeroCallsTest, "GeneralStateTests/stZeroCallsTest/"}
	declare_test!{GeneralStateTest_stZeroKnowledge, "GeneralStateTests/stZeroKnowledge/"}

	// Attempts to send a transaction that requires more than current balance:
	// Tx:
	// https://github.com/ethereum/tests/blob/726b161ba8a739691006cc1ba080672bb50a9d49/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_28000_96.json#L170
	// Balance:
	// https://github.com/ethereum/tests/blob/726b161ba8a739691006cc1ba080672bb50a9d49/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_28000_96.json#L126
	declare_test!{GeneralStateTest_stZeroKnowledge2, "GeneralStateTests/stZeroKnowledge2/"}
}

#[cfg(test)]
mod legacy_state_tests {
	use std::path::Path;

	use super::json_chain_test;
	use json_tests::HookType;

	fn do_json_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], h: &mut H) -> Vec<String> {
		json_chain_test(path, json_data, h, true)
	}
	declare_test!{Constantinople_GeneralStateTests_stArgsZeroOneBalance,"LegacyTests/Constantinople/GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{Constantinople_GeneralStateTests_stAttackTest,"LegacyTests/Constantinople/GeneralStateTests/stAttackTest/"}
	declare_test!{Constantinople_GeneralStateTests_stBadOpcode,"LegacyTests/Constantinople/GeneralStateTests/stBadOpcode/"}
	declare_test!{Constantinople_GeneralStateTests_stBugs,"LegacyTests/Constantinople/GeneralStateTests/stBugs/"}
	declare_test!{Constantinople_GeneralStateTests_stCallCodes,"LegacyTests/Constantinople/GeneralStateTests/stCallCodes/"}
	declare_test!{Constantinople_GeneralStateTests_stCallCreateCallCodeTest,"LegacyTests/Constantinople/GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{Constantinople_GeneralStateTests_stCallDelegateCodesCallCodeHomestead,"LegacyTests/Constantinople/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{Constantinople_GeneralStateTests_stCallDelegateCodesHomestead,"LegacyTests/Constantinople/GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{Constantinople_GeneralStateTests_stChangedEIP150,"LegacyTests/Constantinople/GeneralStateTests/stChangedEIP150/"}
	declare_test!{Constantinople_GeneralStateTests_stCodeCopyTest,"LegacyTests/Constantinople/GeneralStateTests/stCodeCopyTest/"}
	declare_test!{Constantinople_GeneralStateTests_stCodeSizeLimit,"LegacyTests/Constantinople/GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{Constantinople_GeneralStateTests_stCreate2,"LegacyTests/Constantinople/GeneralStateTests/stCreate2/"}
	declare_test!{Constantinople_GeneralStateTests_stCreateTest,"LegacyTests/Constantinople/GeneralStateTests/stCreateTest/"}
	declare_test!{Constantinople_GeneralStateTests_stDelegatecallTestHomestead,"LegacyTests/Constantinople/GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{Constantinople_GeneralStateTests_stEIP150singleCodeGasPrices,"LegacyTests/Constantinople/GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{Constantinople_GeneralStateTests_stEIP150Specific,"LegacyTests/Constantinople/GeneralStateTests/stEIP150Specific/"}
	declare_test!{Constantinople_GeneralStateTests_stEIP158Specific,"LegacyTests/Constantinople/GeneralStateTests/stEIP158Specific/"}
	declare_test!{Constantinople_GeneralStateTests_stEWASMTests,"LegacyTests/Constantinople/GeneralStateTests/stEWASMTests/"}
	declare_test!{Constantinople_GeneralStateTests_stExample,"LegacyTests/Constantinople/GeneralStateTests/stExample/"}
	declare_test!{Constantinople_GeneralStateTests_stExtCodeHash,"LegacyTests/Constantinople/GeneralStateTests/stExtCodeHash/"}
	declare_test!{Constantinople_GeneralStateTests_stHomesteadSpecific,"LegacyTests/Constantinople/GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{Constantinople_GeneralStateTests_stInitCodeTest,"LegacyTests/Constantinople/GeneralStateTests/stInitCodeTest/"}
	declare_test!{Constantinople_GeneralStateTests_stLogTests,"LegacyTests/Constantinople/GeneralStateTests/stLogTests/"}
	declare_test!{Constantinople_GeneralStateTests_stMemExpandingEIP150Calls,"LegacyTests/Constantinople/GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{Constantinople_GeneralStateTests_stMemoryStressTest,"LegacyTests/Constantinople/GeneralStateTests/stMemoryStressTest/"}
	declare_test!{Constantinople_GeneralStateTests_stMemoryTest,"LegacyTests/Constantinople/GeneralStateTests/stMemoryTest/"}
	declare_test!{Constantinople_GeneralStateTests_stNonZeroCallsTest,"LegacyTests/Constantinople/GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{Constantinople_GeneralStateTests_stPreCompiledContracts,"LegacyTests/Constantinople/GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{Constantinople_GeneralStateTests_stPreCompiledContracts2,"LegacyTests/Constantinople/GeneralStateTests/stPreCompiledContracts2/"}
	declare_test!{Constantinople_GeneralStateTests_stQuadraticComplexityTest,"LegacyTests/Constantinople/GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{Constantinople_GeneralStateTests_stRandom,"LegacyTests/Constantinople/GeneralStateTests/stRandom/"}
	declare_test!{Constantinople_GeneralStateTests_stRandom2,"LegacyTests/Constantinople/GeneralStateTests/stRandom2/"}
	declare_test!{Constantinople_GeneralStateTests_stRecursiveCreate,"LegacyTests/Constantinople/GeneralStateTests/stRecursiveCreate/"}
	declare_test!{Constantinople_GeneralStateTests_stRefundTest,"LegacyTests/Constantinople/GeneralStateTests/stRefundTest/"}
	declare_test!{Constantinople_GeneralStateTests_stReturnDataTest,"LegacyTests/Constantinople/GeneralStateTests/stReturnDataTest/"}
	declare_test!{Constantinople_GeneralStateTests_stRevertTest,"LegacyTests/Constantinople/GeneralStateTests/stRevertTest/"}
	declare_test!{Constantinople_GeneralStateTests_stShift,"LegacyTests/Constantinople/GeneralStateTests/stShift/"}
	declare_test!{Constantinople_GeneralStateTests_stSolidityTest,"LegacyTests/Constantinople/GeneralStateTests/stSolidityTest/"}
	declare_test!{Constantinople_GeneralStateTests_stSpecialTest,"LegacyTests/Constantinople/GeneralStateTests/stSpecialTest/"}
	declare_test!{Constantinople_GeneralStateTests_stSStoreTest,"LegacyTests/Constantinople/GeneralStateTests/stSStoreTest/"}
	declare_test!{Constantinople_GeneralStateTests_stStackTests,"LegacyTests/Constantinople/GeneralStateTests/stStackTests/"}
	declare_test!{Constantinople_GeneralStateTests_stStaticCall,"LegacyTests/Constantinople/GeneralStateTests/stStaticCall/"}
	declare_test!{Constantinople_GeneralStateTests_stSystemOperationsTest,"LegacyTests/Constantinople/GeneralStateTests/stSystemOperationsTest/"}
	declare_test!{Constantinople_GeneralStateTests_stTimeConsuming,"LegacyTests/Constantinople/GeneralStateTests/stTimeConsuming/"}
	declare_test!{Constantinople_GeneralStateTests_stTransactionTest,"LegacyTests/Constantinople/GeneralStateTests/stTransactionTest/"}
	declare_test!{Constantinople_GeneralStateTests_stTransitionTest,"LegacyTests/Constantinople/GeneralStateTests/stTransitionTest/"}
	declare_test!{Constantinople_GeneralStateTests_stWalletTest,"LegacyTests/Constantinople/GeneralStateTests/stWalletTest/"}
	declare_test!{Constantinople_GeneralStateTests_stZeroCallsRevert,"LegacyTests/Constantinople/GeneralStateTests/stZeroCallsRevert/"}
	declare_test!{Constantinople_GeneralStateTests_stZeroCallsTest,"LegacyTests/Constantinople/GeneralStateTests/stZeroCallsTest/"}
	declare_test!{Constantinople_GeneralStateTests_stZeroKnowledge,"LegacyTests/Constantinople/GeneralStateTests/stZeroKnowledge/"}
	declare_test!{Constantinople_GeneralStateTests_stZeroKnowledge2,"LegacyTests/Constantinople/GeneralStateTests/stZeroKnowledge2/"}
}