// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::path::Path;
use super::test_common::*;
use pod::PodState;
use trace;
use ethjson;
use test_helpers::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess};
use types::transaction::SignedTransaction;
use vm::EnvInfo;
use super::SKIP_TESTS;
use super::HookType;

#[allow(dead_code)]
fn skip_test(subname: &str, chain: &String, number: usize) -> bool {
	trace!(target: "json-tests", "[state, skip_test] subname: '{}', chain: '{}', number: {}", subname, chain, number);
	SKIP_TESTS.state.iter().any(|state_test|{
		if let Some(subtest) = state_test.subtests.get(subname) {
			trace!(target: "json-tests", "[state, skip_test] Maybe skipping {:?}", subtest);
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
pub fn json_chain_test<H: FnMut(&str, HookType)>(path: &Path, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String> {
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
						println!("   - {} | {:?} Ignoring tests because of missing chainspec", name, spec_name);
						continue;
					}
				};

				for (i, state) in states.into_iter().enumerate() {
					let info = format!("   - {} | {:?} ({}/{}) ...", name, spec_name, i + 1, total);
					if skip_test(&name, &spec.name, i + 1) {
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
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Ok(TransactSuccess { state_root, .. })) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Err(TransactErr { state_root, ref error, .. })) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							println!("{} !!! Execution error: {:?}", info, error);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(Err(TransactErr { error, .. })) => {
							flushln!("{} ok ({:?})", info, error);
						},
						Ok(_) => {
							flushln!("{} ok", info);
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
		json_chain_test(path, json_data, h)
	}

	declare_test!{GeneralStateTest_stArgsZeroOneBalance, "GeneralStateTests/stArgsZeroOneBalance/"}
	declare_test!{GeneralStateTest_stAttackTest, "GeneralStateTests/stAttackTest/"}
	declare_test!{GeneralStateTest_stBadOpcodeTest, "GeneralStateTests/stBadOpcode/"}
	declare_test!{GeneralStateTest_stBugs, "GeneralStateTests/stBugs/"}
	declare_test!{GeneralStateTest_stCallCodes, "GeneralStateTests/stCallCodes/"}
	declare_test!{GeneralStateTest_stCallCreateCallCodeTest, "GeneralStateTests/stCallCreateCallCodeTest/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesHomestead, "GeneralStateTests/stCallDelegateCodesHomestead/"}
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
	//          https://github.com/paritytech/parity-ethereum/issues/11078
	//          https://github.com/paritytech/parity-ethereum/issues/11079
	//          https://github.com/paritytech/parity-ethereum/issues/11080
	declare_test!{GeneralStateTest_stRevertTest, "GeneralStateTests/stRevertTest/"}
	declare_test!{GeneralStateTest_stSStoreTest, "GeneralStateTests/stSStoreTest/"}
	declare_test!{GeneralStateTest_stShift, "GeneralStateTests/stShift/"}
	declare_test!{GeneralStateTest_stSolidityTest, "GeneralStateTests/stSolidityTest/"}
	declare_test!{GeneralStateTest_stSpecialTest, "GeneralStateTests/stSpecialTest/"}
	declare_test!{GeneralStateTest_stStackTests, "GeneralStateTests/stStackTests/"}
	declare_test!{GeneralStateTest_stStaticCall, "GeneralStateTests/stStaticCall/"}
	declare_test!{GeneralStateTest_stSystemOperationsTest, "GeneralStateTests/stSystemOperationsTest/"}
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
