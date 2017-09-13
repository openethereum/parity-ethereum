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

use super::test_common::*;
use pod_state::PodState;
use trace;
use client::{EvmTestClient, EvmTestError, TransactResult};
use ethjson;
use transaction::SignedTransaction;
use vm::EnvInfo;

pub fn json_chain_test(json_data: &[u8]) -> Vec<String> {
	::ethcore_logger::init_log();
	let tests = ethjson::state::test::Test::load(json_data).unwrap();
	let mut failed = Vec::new();

	for (name, test) in tests.into_iter() {
		{
			let multitransaction = test.transaction;
			let env: EnvInfo = test.env.into();
			let pre: PodState = test.pre_state.into();

			for (spec_name, states) in test.post_states {
				let total = states.len();
				let spec = match EvmTestClient::spec_from_json(&spec_name) {
					Some(spec) => spec,
					None => {
						println!("   - {} | {:?} Ignoring tests because of missing spec", name, spec_name);
						continue;
					}
				};

				for (i, state) in states.into_iter().enumerate() {
					let info = format!("   - {} | {:?} ({}/{}) ...", name, spec_name, i + 1, total);

					let post_root: H256 = state.hash.into();
					let transaction: SignedTransaction = multitransaction.select(&state.indexes).into();

					let result = || -> Result<_, EvmTestError> {
						Ok(EvmTestClient::from_pod_state(spec, pre.clone())?
							.transact(&env, transaction, trace::NoopVMTracer))
					};
					match result() {
						Err(err) => {
							println!("{} !!! Unexpected internal error: {:?}", info, err);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Ok { state_root, .. }) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Err { state_root, ref error }) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							println!("{} !!! Execution error: {:?}", info, error);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Err { error, .. }) => {
							flushln!("{} ok ({:?})", info, error);
						},
						Ok(_) => {
							flushln!("{} ok", info);
						},
					}
				}
			}
		}

	}

	if !failed.is_empty() {
		println!("!!! {:?} tests failed.", failed.len());
	}
	failed
}

mod state_tests {
	use super::json_chain_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_chain_test(json_data)
	}

	declare_test!{GeneralStateTest_stAttackTest, "GeneralStateTests/stAttackTest/"}
	declare_test!{GeneralStateTest_stBadOpcodeTest, "GeneralStateTests/stBadOpcode/"}
	declare_test!{GeneralStateTest_stCallCodes, "GeneralStateTests/stCallCodes/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesCallCodeHomestead, "GeneralStateTests/stCallDelegateCodesCallCodeHomestead/"}
	declare_test!{GeneralStateTest_stCallDelegateCodesHomestead, "GeneralStateTests/stCallDelegateCodesHomestead/"}
	declare_test!{GeneralStateTest_stChangedEIP150, "GeneralStateTests/stChangedEIP150/"}
	declare_test!{GeneralStateTest_stCodeSizeLimit, "GeneralStateTests/stCodeSizeLimit/"}
	declare_test!{GeneralStateTest_stCreateTest, "GeneralStateTests/stCreateTest/"}
	declare_test!{GeneralStateTest_stDelegatecallTestHomestead, "GeneralStateTests/stDelegatecallTestHomestead/"}
	declare_test!{GeneralStateTest_stEIP150singleCodeGasPrices, "GeneralStateTests/stEIP150singleCodeGasPrices/"}
	declare_test!{GeneralStateTest_stEIP150Specific, "GeneralStateTests/stEIP150Specific/"}
	declare_test!{GeneralStateTest_stEIP158Specific, "GeneralStateTests/stEIP158Specific/"}
	declare_test!{GeneralStateTest_stExample, "GeneralStateTests/stExample/"}
	declare_test!{GeneralStateTest_stHomesteadSpecific, "GeneralStateTests/stHomesteadSpecific/"}
	declare_test!{GeneralStateTest_stInitCodeTest, "GeneralStateTests/stInitCodeTest/"}
	declare_test!{GeneralStateTest_stLogTests, "GeneralStateTests/stLogTests/"}
	declare_test!{GeneralStateTest_stMemExpandingEIP150Calls, "GeneralStateTests/stMemExpandingEIP150Calls/"}
	declare_test!{heavy => GeneralStateTest_stMemoryStressTest, "GeneralStateTests/stMemoryStressTest/"}
	declare_test!{GeneralStateTest_stMemoryTest, "GeneralStateTests/stMemoryTest/"}
	declare_test!{GeneralStateTest_stNonZeroCallsTest, "GeneralStateTests/stNonZeroCallsTest/"}
	declare_test!{GeneralStateTest_stPreCompiledContracts, "GeneralStateTests/stPreCompiledContracts/"}
	declare_test!{heavy => GeneralStateTest_stQuadraticComplexityTest, "GeneralStateTests/stQuadraticComplexityTest/"}
	declare_test!{GeneralStateTest_stRandom, "GeneralStateTests/stRandom/"}
	declare_test!{GeneralStateTest_stRecursiveCreate, "GeneralStateTests/stRecursiveCreate/"}
	declare_test!{GeneralStateTest_stRefundTest, "GeneralStateTests/stRefundTest/"}
	declare_test!{GeneralStateTest_stReturnDataTest, "GeneralStateTests/stReturnDataTest/"}
	declare_test!{GeneralStateTest_stRevertTest, "GeneralStateTests/stRevertTest/"}
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
}

