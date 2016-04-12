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
use tests::helpers::*;
use pod_state::*;
use state_diff::*;
use ethereum;
use ethjson;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	json_chain_test(json_data, ChainEra::Frontier)
}

pub fn json_chain_test(json_data: &[u8], era: ChainEra) -> Vec<String> {
	init_log();
	let tests = ethjson::state::Test::load(json_data).unwrap();
	let mut failed = Vec::new();
	let engine = match era {
		ChainEra::Frontier => ethereum::new_mainnet_like().engine,
		ChainEra::Homestead => ethereum::new_homestead_test().engine
	};

	for (name, test) in tests.into_iter() {
		let mut fail = false;
		{
			let mut fail_unless = |cond: bool| if !cond && !fail {
				failed.push(name.clone());
				flushln!("FAIL");
				fail = true;
				true
			} else {false};

			flush!("   - {}...", name);

			let transaction = test.transaction.into();
			let post_state_root = test.post_state_root.into();
			let env = test.env.into();
			let pre: PodState = test.pre_state.into();
			let post: PodState = test.post_state.into();
			let logs: Vec<LogEntry> = test.logs.into_iter().map(Into::into).collect();

			let calc_post = sec_trie_root(post.get().iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect());

			if fail_unless(post_state_root == calc_post) {
				println!("!!! {}: Trie root mismatch (got: {}, expect: {}):", name, calc_post, post_state_root);
				println!("!!! Post:\n{}", post);
			} else {
				let mut state_result = get_temp_state();
				let mut state = state_result.reference_mut();
				state.populate_from(pre);
				state.commit();
				let res = state.apply(&env, engine.deref(), &transaction, false);

				if fail_unless(state.root() == &post_state_root) {
					println!("!!! {}: State mismatch (got: {}, expect: {}):", name, state.root(), post_state_root);
					let our_post = state.to_pod();
					println!("Got:\n{}", our_post);
					println!("Expect:\n{}", post);
					println!("Diff ---expect -> +++got:\n{}", StateDiff::diff_pod(&post, &our_post));
				}

				if let Ok(r) = res {
					if fail_unless(logs == r.receipt.logs) {
						println!("!!! {}: Logs mismatch:", name);
						println!("Got:\n{:?}", r.receipt.logs);
						println!("Expect:\n{:?}", logs);
					}
				}
			}
		}

		if !fail {
			flushln!("ok");
		}
	}

	println!("!!! {:?} tests from failed.", failed.len());
	failed
}

declare_test!{StateTests_stBlockHashTest, "StateTests/stBlockHashTest"}
declare_test!{StateTests_stCallCodes, "StateTests/stCallCodes"}
declare_test!{StateTests_stCallCreateCallCodeTest, "StateTests/stCallCreateCallCodeTest"}
declare_test!{StateTests_stDelegatecallTest, "StateTests/stDelegatecallTest"}
declare_test!{StateTests_stExample, "StateTests/stExample"}
declare_test!{StateTests_stInitCodeTest, "StateTests/stInitCodeTest"}
declare_test!{StateTests_stLogTests, "StateTests/stLogTests"}
declare_test!{heavy => StateTests_stMemoryStressTest, "StateTests/stMemoryStressTest"}
declare_test!{heavy => StateTests_stMemoryTest, "StateTests/stMemoryTest"}
declare_test!{StateTests_stPreCompiledContracts, "StateTests/stPreCompiledContracts"}
declare_test!{heavy => StateTests_stQuadraticComplexityTest, "StateTests/stQuadraticComplexityTest"}
declare_test!{StateTests_stRecursiveCreate, "StateTests/stRecursiveCreate"}
declare_test!{StateTests_stRefundTest, "StateTests/stRefundTest"}
declare_test!{StateTests_stSolidityTest, "StateTests/stSolidityTest"}
declare_test!{StateTests_stSpecialTest, "StateTests/stSpecialTest"}
declare_test!{StateTests_stSystemOperationsTest, "StateTests/stSystemOperationsTest"}
declare_test!{StateTests_stTransactionTest, "StateTests/stTransactionTest"}
declare_test!{StateTests_stTransitionTest, "StateTests/stTransitionTest"}
declare_test!{StateTests_stWalletTest, "StateTests/stWalletTest"}


declare_test!{StateTests_RandomTests_st201503121803PYTHON, "StateTests/RandomTests/st201503121803PYTHON"}
declare_test!{StateTests_RandomTests_st201503121806PYTHON, "StateTests/RandomTests/st201503121806PYTHON"}
declare_test!{StateTests_RandomTests_st201503121848GO, "StateTests/RandomTests/st201503121848GO"}
declare_test!{StateTests_RandomTests_st201503121849GO, "StateTests/RandomTests/st201503121849GO"}
declare_test!{StateTests_RandomTests_st201503121850GO, "StateTests/RandomTests/st201503121850GO"}
declare_test!{StateTests_RandomTests_st201503121851GO, "StateTests/RandomTests/st201503121851GO"}
declare_test!{StateTests_RandomTests_st201503121953GO, "StateTests/RandomTests/st201503121953GO"}
declare_test!{StateTests_RandomTests_st201503122023GO, "StateTests/RandomTests/st201503122023GO"}
declare_test!{StateTests_RandomTests_st201503122023PYTHON, "StateTests/RandomTests/st201503122023PYTHON"}
declare_test!{StateTests_RandomTests_st201503122027GO, "StateTests/RandomTests/st201503122027GO"}
declare_test!{StateTests_RandomTests_st201503122054GO, "StateTests/RandomTests/st201503122054GO"}
declare_test!{StateTests_RandomTests_st201503122055GO, "StateTests/RandomTests/st201503122055GO"}
declare_test!{StateTests_RandomTests_st201503122115CPPJIT, "StateTests/RandomTests/st201503122115CPPJIT"}
declare_test!{StateTests_RandomTests_st201503122115GO, "StateTests/RandomTests/st201503122115GO"}
declare_test!{StateTests_RandomTests_st201503122123GO, "StateTests/RandomTests/st201503122123GO"}
declare_test!{StateTests_RandomTests_st201503122124GO, "StateTests/RandomTests/st201503122124GO"}
declare_test!{StateTests_RandomTests_st201503122128PYTHON, "StateTests/RandomTests/st201503122128PYTHON"}
declare_test!{StateTests_RandomTests_st201503122140GO, "StateTests/RandomTests/st201503122140GO"}
declare_test!{StateTests_RandomTests_st201503122159GO, "StateTests/RandomTests/st201503122159GO"}
declare_test!{StateTests_RandomTests_st201503122204GO, "StateTests/RandomTests/st201503122204GO"}
declare_test!{StateTests_RandomTests_st201503122212GO, "StateTests/RandomTests/st201503122212GO"}
declare_test!{StateTests_RandomTests_st201503122231GO, "StateTests/RandomTests/st201503122231GO"}
declare_test!{StateTests_RandomTests_st201503122238GO, "StateTests/RandomTests/st201503122238GO"}
declare_test!{StateTests_RandomTests_st201503122252GO, "StateTests/RandomTests/st201503122252GO"}
declare_test!{StateTests_RandomTests_st201503122316GO, "StateTests/RandomTests/st201503122316GO"}
declare_test!{StateTests_RandomTests_st201503122324GO, "StateTests/RandomTests/st201503122324GO"}
declare_test!{StateTests_RandomTests_st201503122358GO, "StateTests/RandomTests/st201503122358GO"}
declare_test!{StateTests_RandomTests_st201503130002GO, "StateTests/RandomTests/st201503130002GO"}
declare_test!{StateTests_RandomTests_st201503130005GO, "StateTests/RandomTests/st201503130005GO"}
declare_test!{StateTests_RandomTests_st201503130007GO, "StateTests/RandomTests/st201503130007GO"}
declare_test!{StateTests_RandomTests_st201503130010GO, "StateTests/RandomTests/st201503130010GO"}
declare_test!{StateTests_RandomTests_st201503130023PYTHON, "StateTests/RandomTests/st201503130023PYTHON"}
declare_test!{StateTests_RandomTests_st201503130059GO, "StateTests/RandomTests/st201503130059GO"}
declare_test!{StateTests_RandomTests_st201503130101GO, "StateTests/RandomTests/st201503130101GO"}
declare_test!{StateTests_RandomTests_st201503130109GO, "StateTests/RandomTests/st201503130109GO"}
declare_test!{StateTests_RandomTests_st201503130117GO, "StateTests/RandomTests/st201503130117GO"}
declare_test!{StateTests_RandomTests_st201503130122GO, "StateTests/RandomTests/st201503130122GO"}
declare_test!{StateTests_RandomTests_st201503130156GO, "StateTests/RandomTests/st201503130156GO"}
declare_test!{StateTests_RandomTests_st201503130156PYTHON, "StateTests/RandomTests/st201503130156PYTHON"}
declare_test!{StateTests_RandomTests_st201503130207GO, "StateTests/RandomTests/st201503130207GO"}
declare_test!{StateTests_RandomTests_st201503130219CPPJIT, "StateTests/RandomTests/st201503130219CPPJIT"}
declare_test!{StateTests_RandomTests_st201503130219GO, "StateTests/RandomTests/st201503130219GO"}
declare_test!{StateTests_RandomTests_st201503130243GO, "StateTests/RandomTests/st201503130243GO"}
declare_test!{StateTests_RandomTests_st201503130246GO, "StateTests/RandomTests/st201503130246GO"}
declare_test!{StateTests_RandomTests_st201503130321GO, "StateTests/RandomTests/st201503130321GO"}
declare_test!{StateTests_RandomTests_st201503130322GO, "StateTests/RandomTests/st201503130322GO"}
declare_test!{StateTests_RandomTests_st201503130332GO, "StateTests/RandomTests/st201503130332GO"}
declare_test!{StateTests_RandomTests_st201503130359GO, "StateTests/RandomTests/st201503130359GO"}
declare_test!{StateTests_RandomTests_st201503130405GO, "StateTests/RandomTests/st201503130405GO"}
declare_test!{StateTests_RandomTests_st201503130408GO, "StateTests/RandomTests/st201503130408GO"}
declare_test!{StateTests_RandomTests_st201503130411GO, "StateTests/RandomTests/st201503130411GO"}
declare_test!{StateTests_RandomTests_st201503130431GO, "StateTests/RandomTests/st201503130431GO"}
declare_test!{StateTests_RandomTests_st201503130437GO, "StateTests/RandomTests/st201503130437GO"}
declare_test!{StateTests_RandomTests_st201503130450GO, "StateTests/RandomTests/st201503130450GO"}
declare_test!{StateTests_RandomTests_st201503130512CPPJIT, "StateTests/RandomTests/st201503130512CPPJIT"}
declare_test!{StateTests_RandomTests_st201503130512GO, "StateTests/RandomTests/st201503130512GO"}
declare_test!{StateTests_RandomTests_st201503130615GO, "StateTests/RandomTests/st201503130615GO"}
declare_test!{StateTests_RandomTests_st201503130705GO, "StateTests/RandomTests/st201503130705GO"}
declare_test!{StateTests_RandomTests_st201503130733CPPJIT, "StateTests/RandomTests/st201503130733CPPJIT"}
declare_test!{StateTests_RandomTests_st201503130733GO, "StateTests/RandomTests/st201503130733GO"}
declare_test!{StateTests_RandomTests_st201503130747GO, "StateTests/RandomTests/st201503130747GO"}
declare_test!{StateTests_RandomTests_st201503130751GO, "StateTests/RandomTests/st201503130751GO"}
declare_test!{StateTests_RandomTests_st201503130752PYTHON, "StateTests/RandomTests/st201503130752PYTHON"}
declare_test!{StateTests_RandomTests_st201503130757PYTHON, "StateTests/RandomTests/st201503130757PYTHON"}
declare_test!{StateTests_RandomTests_st201503131658GO, "StateTests/RandomTests/st201503131658GO"}
declare_test!{StateTests_RandomTests_st201503131739GO, "StateTests/RandomTests/st201503131739GO"}
declare_test!{StateTests_RandomTests_st201503131755CPPJIT, "StateTests/RandomTests/st201503131755CPPJIT"}
declare_test!{StateTests_RandomTests_st201503131755GO, "StateTests/RandomTests/st201503131755GO"}
declare_test!{StateTests_RandomTests_st201503132001CPPJIT, "StateTests/RandomTests/st201503132001CPPJIT"}
declare_test!{StateTests_RandomTests_st201503132127PYTHON, "StateTests/RandomTests/st201503132127PYTHON"}
declare_test!{StateTests_RandomTests_st201503132201CPPJIT, "StateTests/RandomTests/st201503132201CPPJIT"}
declare_test!{StateTests_RandomTests_st201503132201GO, "StateTests/RandomTests/st201503132201GO"}
declare_test!{StateTests_RandomTests_st201503132202PYTHON, "StateTests/RandomTests/st201503132202PYTHON"}
declare_test!{StateTests_RandomTests_st201503140002PYTHON, "StateTests/RandomTests/st201503140002PYTHON"}
declare_test!{StateTests_RandomTests_st201503140240PYTHON, "StateTests/RandomTests/st201503140240PYTHON"}
declare_test!{StateTests_RandomTests_st201503140522PYTHON, "StateTests/RandomTests/st201503140522PYTHON"}
declare_test!{StateTests_RandomTests_st201503140756PYTHON, "StateTests/RandomTests/st201503140756PYTHON"}
declare_test!{StateTests_RandomTests_st201503141144PYTHON, "StateTests/RandomTests/st201503141144PYTHON"}
declare_test!{StateTests_RandomTests_st201503141510PYTHON, "StateTests/RandomTests/st201503141510PYTHON"}
declare_test!{StateTests_RandomTests_st201503150427PYTHON, "StateTests/RandomTests/st201503150427PYTHON"}
declare_test!{StateTests_RandomTests_st201503150716PYTHON, "StateTests/RandomTests/st201503150716PYTHON"}
declare_test!{StateTests_RandomTests_st201503151450PYTHON, "StateTests/RandomTests/st201503151450PYTHON"}
declare_test!{StateTests_RandomTests_st201503151516PYTHON, "StateTests/RandomTests/st201503151516PYTHON"}
declare_test!{StateTests_RandomTests_st201503151753PYTHON, "StateTests/RandomTests/st201503151753PYTHON"}
declare_test!{StateTests_RandomTests_st201503152057PYTHON, "StateTests/RandomTests/st201503152057PYTHON"}
declare_test!{StateTests_RandomTests_st201503152241PYTHON, "StateTests/RandomTests/st201503152241PYTHON"}
declare_test!{StateTests_RandomTests_st201503160014PYTHON, "StateTests/RandomTests/st201503160014PYTHON"}
declare_test!{StateTests_RandomTests_st201503160733PYTHON, "StateTests/RandomTests/st201503160733PYTHON"}
declare_test!{StateTests_RandomTests_st201503170051PYTHON, "StateTests/RandomTests/st201503170051PYTHON"}
declare_test!{StateTests_RandomTests_st201503170433PYTHON, "StateTests/RandomTests/st201503170433PYTHON"}
declare_test!{StateTests_RandomTests_st201503170523PYTHON, "StateTests/RandomTests/st201503170523PYTHON"}
declare_test!{StateTests_RandomTests_st201503171108PYTHON, "StateTests/RandomTests/st201503171108PYTHON"}
declare_test!{StateTests_RandomTests_st201503181223GO, "StateTests/RandomTests/st201503181223GO"}
declare_test!{StateTests_RandomTests_st201503181225GO, "StateTests/RandomTests/st201503181225GO"}
declare_test!{StateTests_RandomTests_st201503181226CPPJIT, "StateTests/RandomTests/st201503181226CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181227CPPJIT, "StateTests/RandomTests/st201503181227CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181227GO, "StateTests/RandomTests/st201503181227GO"}
declare_test!{StateTests_RandomTests_st201503181229GO, "StateTests/RandomTests/st201503181229GO"}
declare_test!{StateTests_RandomTests_st201503181230CPPJIT, "StateTests/RandomTests/st201503181230CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181230GO, "StateTests/RandomTests/st201503181230GO"}
declare_test!{StateTests_RandomTests_st201503181231GO, "StateTests/RandomTests/st201503181231GO"}
declare_test!{StateTests_RandomTests_st201503181232CPPJIT, "StateTests/RandomTests/st201503181232CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181232GO, "StateTests/RandomTests/st201503181232GO"}
declare_test!{StateTests_RandomTests_st201503181233GO, "StateTests/RandomTests/st201503181233GO"}
declare_test!{StateTests_RandomTests_st201503181234CPPJIT, "StateTests/RandomTests/st201503181234CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181234GO, "StateTests/RandomTests/st201503181234GO"}
declare_test!{StateTests_RandomTests_st201503181235CPPJIT, "StateTests/RandomTests/st201503181235CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181235GO, "StateTests/RandomTests/st201503181235GO"}
declare_test!{StateTests_RandomTests_st201503181236GO, "StateTests/RandomTests/st201503181236GO"}
declare_test!{StateTests_RandomTests_st201503181237GO, "StateTests/RandomTests/st201503181237GO"}
declare_test!{StateTests_RandomTests_st201503181239GO, "StateTests/RandomTests/st201503181239GO"}
declare_test!{StateTests_RandomTests_st201503181241CPPJIT, "StateTests/RandomTests/st201503181241CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181241GO, "StateTests/RandomTests/st201503181241GO"}
declare_test!{StateTests_RandomTests_st201503181243GO, "StateTests/RandomTests/st201503181243GO"}
declare_test!{StateTests_RandomTests_st201503181244GO, "StateTests/RandomTests/st201503181244GO"}
declare_test!{StateTests_RandomTests_st201503181245CPPJIT, "StateTests/RandomTests/st201503181245CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181245GO, "StateTests/RandomTests/st201503181245GO"}
declare_test!{StateTests_RandomTests_st201503181246CPPJIT, "StateTests/RandomTests/st201503181246CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181246GO, "StateTests/RandomTests/st201503181246GO"}
declare_test!{StateTests_RandomTests_st201503181247GO, "StateTests/RandomTests/st201503181247GO"}
declare_test!{StateTests_RandomTests_st201503181248GO, "StateTests/RandomTests/st201503181248GO"}
declare_test!{StateTests_RandomTests_st201503181249GO, "StateTests/RandomTests/st201503181249GO"}
declare_test!{StateTests_RandomTests_st201503181250CPPJIT, "StateTests/RandomTests/st201503181250CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181250GO, "StateTests/RandomTests/st201503181250GO"}
declare_test!{StateTests_RandomTests_st201503181251GO, "StateTests/RandomTests/st201503181251GO"}
declare_test!{StateTests_RandomTests_st201503181252CPPJIT, "StateTests/RandomTests/st201503181252CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181253GO, "StateTests/RandomTests/st201503181253GO"}
declare_test!{StateTests_RandomTests_st201503181255CPPJIT, "StateTests/RandomTests/st201503181255CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181255GO, "StateTests/RandomTests/st201503181255GO"}
declare_test!{StateTests_RandomTests_st201503181257GO, "StateTests/RandomTests/st201503181257GO"}
declare_test!{StateTests_RandomTests_st201503181258CPPJIT, "StateTests/RandomTests/st201503181258CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181258GO, "StateTests/RandomTests/st201503181258GO"}
declare_test!{StateTests_RandomTests_st201503181301CPPJIT, "StateTests/RandomTests/st201503181301CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181301GO, "StateTests/RandomTests/st201503181301GO"}
declare_test!{StateTests_RandomTests_st201503181303GO, "StateTests/RandomTests/st201503181303GO"}
declare_test!{StateTests_RandomTests_st201503181304GO, "StateTests/RandomTests/st201503181304GO"}
declare_test!{StateTests_RandomTests_st201503181305GO, "StateTests/RandomTests/st201503181305GO"}
declare_test!{StateTests_RandomTests_st201503181306GO, "StateTests/RandomTests/st201503181306GO"}
declare_test!{StateTests_RandomTests_st201503181307CPPJIT, "StateTests/RandomTests/st201503181307CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181307GO, "StateTests/RandomTests/st201503181307GO"}
declare_test!{StateTests_RandomTests_st201503181308GO, "StateTests/RandomTests/st201503181308GO"}
declare_test!{StateTests_RandomTests_st201503181309GO, "StateTests/RandomTests/st201503181309GO"}
declare_test!{StateTests_RandomTests_st201503181310GO, "StateTests/RandomTests/st201503181310GO"}
declare_test!{StateTests_RandomTests_st201503181311GO, "StateTests/RandomTests/st201503181311GO"}
declare_test!{StateTests_RandomTests_st201503181313GO, "StateTests/RandomTests/st201503181313GO"}
declare_test!{StateTests_RandomTests_st201503181314GO, "StateTests/RandomTests/st201503181314GO"}
declare_test!{StateTests_RandomTests_st201503181315CPPJIT, "StateTests/RandomTests/st201503181315CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181315GO, "StateTests/RandomTests/st201503181315GO"}
declare_test!{StateTests_RandomTests_st201503181316CPPJIT, "StateTests/RandomTests/st201503181316CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181316PYTHON, "StateTests/RandomTests/st201503181316PYTHON"}
declare_test!{StateTests_RandomTests_st201503181317GO, "StateTests/RandomTests/st201503181317GO"}
declare_test!{StateTests_RandomTests_st201503181318CPPJIT, "StateTests/RandomTests/st201503181318CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181318GO, "StateTests/RandomTests/st201503181318GO"}
declare_test!{StateTests_RandomTests_st201503181319GO, "StateTests/RandomTests/st201503181319GO"}
declare_test!{StateTests_RandomTests_st201503181319PYTHON, "StateTests/RandomTests/st201503181319PYTHON"}
declare_test!{StateTests_RandomTests_st201503181322GO, "StateTests/RandomTests/st201503181322GO"}
declare_test!{StateTests_RandomTests_st201503181323CPPJIT, "StateTests/RandomTests/st201503181323CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181323GO, "StateTests/RandomTests/st201503181323GO"}
declare_test!{StateTests_RandomTests_st201503181324GO, "StateTests/RandomTests/st201503181324GO"}
declare_test!{StateTests_RandomTests_st201503181325GO, "StateTests/RandomTests/st201503181325GO"}
declare_test!{StateTests_RandomTests_st201503181326CPPJIT, "StateTests/RandomTests/st201503181326CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181326GO, "StateTests/RandomTests/st201503181326GO"}
declare_test!{StateTests_RandomTests_st201503181327GO, "StateTests/RandomTests/st201503181327GO"}
declare_test!{StateTests_RandomTests_st201503181329CPPJIT, "StateTests/RandomTests/st201503181329CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181329GO, "StateTests/RandomTests/st201503181329GO"}
declare_test!{StateTests_RandomTests_st201503181330GO, "StateTests/RandomTests/st201503181330GO"}
declare_test!{StateTests_RandomTests_st201503181332GO, "StateTests/RandomTests/st201503181332GO"}
declare_test!{StateTests_RandomTests_st201503181333GO, "StateTests/RandomTests/st201503181333GO"}
declare_test!{StateTests_RandomTests_st201503181334GO, "StateTests/RandomTests/st201503181334GO"}
declare_test!{StateTests_RandomTests_st201503181336CPPJIT, "StateTests/RandomTests/st201503181336CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181337GO, "StateTests/RandomTests/st201503181337GO"}
declare_test!{StateTests_RandomTests_st201503181338GO, "StateTests/RandomTests/st201503181338GO"}
declare_test!{StateTests_RandomTests_st201503181339CPPJIT, "StateTests/RandomTests/st201503181339CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181339GO, "StateTests/RandomTests/st201503181339GO"}
declare_test!{StateTests_RandomTests_st201503181340GO, "StateTests/RandomTests/st201503181340GO"}
declare_test!{StateTests_RandomTests_st201503181341CPPJIT, "StateTests/RandomTests/st201503181341CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181342CPPJIT, "StateTests/RandomTests/st201503181342CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181342GO, "StateTests/RandomTests/st201503181342GO"}
declare_test!{StateTests_RandomTests_st201503181345GO, "StateTests/RandomTests/st201503181345GO"}
declare_test!{StateTests_RandomTests_st201503181346GO, "StateTests/RandomTests/st201503181346GO"}
declare_test!{StateTests_RandomTests_st201503181347CPPJIT, "StateTests/RandomTests/st201503181347CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181347GO, "StateTests/RandomTests/st201503181347GO"}
declare_test!{StateTests_RandomTests_st201503181347PYTHON, "StateTests/RandomTests/st201503181347PYTHON"}
declare_test!{StateTests_RandomTests_st201503181350CPPJIT, "StateTests/RandomTests/st201503181350CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181352GO, "StateTests/RandomTests/st201503181352GO"}
declare_test!{StateTests_RandomTests_st201503181353GO, "StateTests/RandomTests/st201503181353GO"}
declare_test!{StateTests_RandomTests_st201503181354CPPJIT, "StateTests/RandomTests/st201503181354CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181354GO, "StateTests/RandomTests/st201503181354GO"}
declare_test!{StateTests_RandomTests_st201503181355GO, "StateTests/RandomTests/st201503181355GO"}
declare_test!{StateTests_RandomTests_st201503181356CPPJIT, "StateTests/RandomTests/st201503181356CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181357CPPJIT, "StateTests/RandomTests/st201503181357CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181358CPPJIT, "StateTests/RandomTests/st201503181358CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181358GO, "StateTests/RandomTests/st201503181358GO"}
declare_test!{StateTests_RandomTests_st201503181359GO, "StateTests/RandomTests/st201503181359GO"}
declare_test!{StateTests_RandomTests_st201503181402CPPJIT, "StateTests/RandomTests/st201503181402CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181403GO, "StateTests/RandomTests/st201503181403GO"}
declare_test!{StateTests_RandomTests_st201503181406CPPJIT, "StateTests/RandomTests/st201503181406CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181406GO, "StateTests/RandomTests/st201503181406GO"}
declare_test!{StateTests_RandomTests_st201503181410GO, "StateTests/RandomTests/st201503181410GO"}
declare_test!{StateTests_RandomTests_st201503181412CPPJIT, "StateTests/RandomTests/st201503181412CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181413GO, "StateTests/RandomTests/st201503181413GO"}
declare_test!{StateTests_RandomTests_st201503181415GO, "StateTests/RandomTests/st201503181415GO"}
declare_test!{StateTests_RandomTests_st201503181416GO, "StateTests/RandomTests/st201503181416GO"}
declare_test!{StateTests_RandomTests_st201503181417CPPJIT, "StateTests/RandomTests/st201503181417CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181417GO, "StateTests/RandomTests/st201503181417GO"}
declare_test!{StateTests_RandomTests_st201503181418CPPJIT, "StateTests/RandomTests/st201503181418CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181422GO, "StateTests/RandomTests/st201503181422GO"}
declare_test!{StateTests_RandomTests_st201503181423CPPJIT, "StateTests/RandomTests/st201503181423CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181424GO, "StateTests/RandomTests/st201503181424GO"}
declare_test!{StateTests_RandomTests_st201503181426CPPJIT, "StateTests/RandomTests/st201503181426CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181426GO, "StateTests/RandomTests/st201503181426GO"}
declare_test!{StateTests_RandomTests_st201503181428GO, "StateTests/RandomTests/st201503181428GO"}
declare_test!{StateTests_RandomTests_st201503181430CPPJIT, "StateTests/RandomTests/st201503181430CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181435GO, "StateTests/RandomTests/st201503181435GO"}
declare_test!{StateTests_RandomTests_st201503181436GO, "StateTests/RandomTests/st201503181436GO"}
declare_test!{StateTests_RandomTests_st201503181437CPPJIT, "StateTests/RandomTests/st201503181437CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181437GO, "StateTests/RandomTests/st201503181437GO"}
declare_test!{StateTests_RandomTests_st201503181438CPPJIT, "StateTests/RandomTests/st201503181438CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181438GO, "StateTests/RandomTests/st201503181438GO"}
declare_test!{StateTests_RandomTests_st201503181439CPPJIT, "StateTests/RandomTests/st201503181439CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181439GO, "StateTests/RandomTests/st201503181439GO"}
declare_test!{StateTests_RandomTests_st201503181439PYTHON, "StateTests/RandomTests/st201503181439PYTHON"}
declare_test!{StateTests_RandomTests_st201503181440GO, "StateTests/RandomTests/st201503181440GO"}
declare_test!{StateTests_RandomTests_st201503181441GO, "StateTests/RandomTests/st201503181441GO"}
declare_test!{StateTests_RandomTests_st201503181442GO, "StateTests/RandomTests/st201503181442GO"}
declare_test!{StateTests_RandomTests_st201503181445CPPJIT, "StateTests/RandomTests/st201503181445CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181446GO, "StateTests/RandomTests/st201503181446GO"}
declare_test!{StateTests_RandomTests_st201503181447GO, "StateTests/RandomTests/st201503181447GO"}
declare_test!{StateTests_RandomTests_st201503181450GO, "StateTests/RandomTests/st201503181450GO"}
declare_test!{StateTests_RandomTests_st201503181451CPPJIT, "StateTests/RandomTests/st201503181451CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181453GO, "StateTests/RandomTests/st201503181453GO"}
declare_test!{StateTests_RandomTests_st201503181455GO, "StateTests/RandomTests/st201503181455GO"}
declare_test!{StateTests_RandomTests_st201503181456CPPJIT, "StateTests/RandomTests/st201503181456CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181457GO, "StateTests/RandomTests/st201503181457GO"}
declare_test!{StateTests_RandomTests_st201503181458GO, "StateTests/RandomTests/st201503181458GO"}
declare_test!{StateTests_RandomTests_st201503181459GO, "StateTests/RandomTests/st201503181459GO"}
declare_test!{StateTests_RandomTests_st201503181500GO, "StateTests/RandomTests/st201503181500GO"}
declare_test!{StateTests_RandomTests_st201503181501GO, "StateTests/RandomTests/st201503181501GO"}
declare_test!{StateTests_RandomTests_st201503181503GO, "StateTests/RandomTests/st201503181503GO"}
declare_test!{StateTests_RandomTests_st201503181504GO, "StateTests/RandomTests/st201503181504GO"}
declare_test!{StateTests_RandomTests_st201503181505GO, "StateTests/RandomTests/st201503181505GO"}
declare_test!{StateTests_RandomTests_st201503181506CPPJIT, "StateTests/RandomTests/st201503181506CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181507GO, "StateTests/RandomTests/st201503181507GO"}
declare_test!{StateTests_RandomTests_st201503181509CPPJIT, "StateTests/RandomTests/st201503181509CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181509GO, "StateTests/RandomTests/st201503181509GO"}
declare_test!{StateTests_RandomTests_st201503181510GO, "StateTests/RandomTests/st201503181510GO"}
declare_test!{StateTests_RandomTests_st201503181511GO, "StateTests/RandomTests/st201503181511GO"}
declare_test!{StateTests_RandomTests_st201503181512GO, "StateTests/RandomTests/st201503181512GO"}
declare_test!{StateTests_RandomTests_st201503181513CPPJIT, "StateTests/RandomTests/st201503181513CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181513GO, "StateTests/RandomTests/st201503181513GO"}
declare_test!{StateTests_RandomTests_st201503181514CPPJIT, "StateTests/RandomTests/st201503181514CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181514GO, "StateTests/RandomTests/st201503181514GO"}
declare_test!{StateTests_RandomTests_st201503181517CPPJIT, "StateTests/RandomTests/st201503181517CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181517GO, "StateTests/RandomTests/st201503181517GO"}
declare_test!{StateTests_RandomTests_st201503181519CPPJIT, "StateTests/RandomTests/st201503181519CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181519GO, "StateTests/RandomTests/st201503181519GO"}
declare_test!{StateTests_RandomTests_st201503181520CPPJIT, "StateTests/RandomTests/st201503181520CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181520GO, "StateTests/RandomTests/st201503181520GO"}
declare_test!{StateTests_RandomTests_st201503181521GO, "StateTests/RandomTests/st201503181521GO"}
declare_test!{StateTests_RandomTests_st201503181522GO, "StateTests/RandomTests/st201503181522GO"}
declare_test!{StateTests_RandomTests_st201503181524CPPJIT, "StateTests/RandomTests/st201503181524CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181524GO, "StateTests/RandomTests/st201503181524GO"}
declare_test!{StateTests_RandomTests_st201503181526GO, "StateTests/RandomTests/st201503181526GO"}
declare_test!{StateTests_RandomTests_st201503181527GO, "StateTests/RandomTests/st201503181527GO"}
declare_test!{StateTests_RandomTests_st201503181528CPPJIT, "StateTests/RandomTests/st201503181528CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181528GO, "StateTests/RandomTests/st201503181528GO"}
declare_test!{StateTests_RandomTests_st201503181528PYTHON, "StateTests/RandomTests/st201503181528PYTHON"}
declare_test!{StateTests_RandomTests_st201503181529GO, "StateTests/RandomTests/st201503181529GO"}
declare_test!{StateTests_RandomTests_st201503181531CPPJIT, "StateTests/RandomTests/st201503181531CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181533GO, "StateTests/RandomTests/st201503181533GO"}
declare_test!{StateTests_RandomTests_st201503181534CPPJIT, "StateTests/RandomTests/st201503181534CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181534GO, "StateTests/RandomTests/st201503181534GO"}
declare_test!{StateTests_RandomTests_st201503181536CPPJIT, "StateTests/RandomTests/st201503181536CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181536GO, "StateTests/RandomTests/st201503181536GO"}
declare_test!{StateTests_RandomTests_st201503181537GO, "StateTests/RandomTests/st201503181537GO"}
declare_test!{StateTests_RandomTests_st201503181538GO, "StateTests/RandomTests/st201503181538GO"}
declare_test!{StateTests_RandomTests_st201503181539GO, "StateTests/RandomTests/st201503181539GO"}
declare_test!{StateTests_RandomTests_st201503181540CPPJIT, "StateTests/RandomTests/st201503181540CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181540PYTHON, "StateTests/RandomTests/st201503181540PYTHON"}
declare_test!{StateTests_RandomTests_st201503181543GO, "StateTests/RandomTests/st201503181543GO"}
declare_test!{StateTests_RandomTests_st201503181544CPPJIT, "StateTests/RandomTests/st201503181544CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181544GO, "StateTests/RandomTests/st201503181544GO"}
declare_test!{StateTests_RandomTests_st201503181547GO, "StateTests/RandomTests/st201503181547GO"}
declare_test!{StateTests_RandomTests_st201503181548CPPJIT, "StateTests/RandomTests/st201503181548CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181548GO, "StateTests/RandomTests/st201503181548GO"}
declare_test!{StateTests_RandomTests_st201503181551GO, "StateTests/RandomTests/st201503181551GO"}
declare_test!{StateTests_RandomTests_st201503181552CPPJIT, "StateTests/RandomTests/st201503181552CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181553GO, "StateTests/RandomTests/st201503181553GO"}
declare_test!{StateTests_RandomTests_st201503181555CPPJIT, "StateTests/RandomTests/st201503181555CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181555GO, "StateTests/RandomTests/st201503181555GO"}
declare_test!{StateTests_RandomTests_st201503181557GO, "StateTests/RandomTests/st201503181557GO"}
declare_test!{StateTests_RandomTests_st201503181559GO, "StateTests/RandomTests/st201503181559GO"}
declare_test!{StateTests_RandomTests_st201503181601CPPJIT, "StateTests/RandomTests/st201503181601CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181601GO, "StateTests/RandomTests/st201503181601GO"}
declare_test!{StateTests_RandomTests_st201503181602GO, "StateTests/RandomTests/st201503181602GO"}
declare_test!{StateTests_RandomTests_st201503181603GO, "StateTests/RandomTests/st201503181603GO"}
declare_test!{StateTests_RandomTests_st201503181604GO, "StateTests/RandomTests/st201503181604GO"}
declare_test!{StateTests_RandomTests_st201503181605GO, "StateTests/RandomTests/st201503181605GO"}
declare_test!{StateTests_RandomTests_st201503181606GO, "StateTests/RandomTests/st201503181606GO"}
declare_test!{StateTests_RandomTests_st201503181607GO, "StateTests/RandomTests/st201503181607GO"}
declare_test!{StateTests_RandomTests_st201503181608CPPJIT, "StateTests/RandomTests/st201503181608CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181608GO, "StateTests/RandomTests/st201503181608GO"}
declare_test!{StateTests_RandomTests_st201503181609GO, "StateTests/RandomTests/st201503181609GO"}
declare_test!{StateTests_RandomTests_st201503181610CPPJIT, "StateTests/RandomTests/st201503181610CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181610GO, "StateTests/RandomTests/st201503181610GO"}
declare_test!{StateTests_RandomTests_st201503181611CPPJIT, "StateTests/RandomTests/st201503181611CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181611GO, "StateTests/RandomTests/st201503181611GO"}
declare_test!{StateTests_RandomTests_st201503181612GO, "StateTests/RandomTests/st201503181612GO"}
declare_test!{StateTests_RandomTests_st201503181614CPPJIT, "StateTests/RandomTests/st201503181614CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181614GO, "StateTests/RandomTests/st201503181614GO"}
declare_test!{StateTests_RandomTests_st201503181616CPPJIT, "StateTests/RandomTests/st201503181616CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181616GO, "StateTests/RandomTests/st201503181616GO"}
declare_test!{StateTests_RandomTests_st201503181617GO, "StateTests/RandomTests/st201503181617GO"}
declare_test!{StateTests_RandomTests_st201503181618GO, "StateTests/RandomTests/st201503181618GO"}
declare_test!{StateTests_RandomTests_st201503181619GO, "StateTests/RandomTests/st201503181619GO"}
declare_test!{StateTests_RandomTests_st201503181620CPPJIT, "StateTests/RandomTests/st201503181620CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181620GO, "StateTests/RandomTests/st201503181620GO"}
declare_test!{StateTests_RandomTests_st201503181621GO, "StateTests/RandomTests/st201503181621GO"}
declare_test!{StateTests_RandomTests_st201503181625GO, "StateTests/RandomTests/st201503181625GO"}
declare_test!{StateTests_RandomTests_st201503181626CPPJIT, "StateTests/RandomTests/st201503181626CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181626GO, "StateTests/RandomTests/st201503181626GO"}
declare_test!{StateTests_RandomTests_st201503181627GO, "StateTests/RandomTests/st201503181627GO"}
declare_test!{StateTests_RandomTests_st201503181628GO, "StateTests/RandomTests/st201503181628GO"}
declare_test!{StateTests_RandomTests_st201503181629GO, "StateTests/RandomTests/st201503181629GO"}
declare_test!{StateTests_RandomTests_st201503181630CPPJIT, "StateTests/RandomTests/st201503181630CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181630GO, "StateTests/RandomTests/st201503181630GO"}
declare_test!{StateTests_RandomTests_st201503181630PYTHON, "StateTests/RandomTests/st201503181630PYTHON"}
declare_test!{StateTests_RandomTests_st201503181632GO, "StateTests/RandomTests/st201503181632GO"}
declare_test!{StateTests_RandomTests_st201503181634CPPJIT, "StateTests/RandomTests/st201503181634CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181635GO, "StateTests/RandomTests/st201503181635GO"}
declare_test!{StateTests_RandomTests_st201503181636GO, "StateTests/RandomTests/st201503181636GO"}
declare_test!{StateTests_RandomTests_st201503181638GO, "StateTests/RandomTests/st201503181638GO"}
declare_test!{StateTests_RandomTests_st201503181639CPPJIT, "StateTests/RandomTests/st201503181639CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181641GO, "StateTests/RandomTests/st201503181641GO"}
declare_test!{StateTests_RandomTests_st201503181645GO, "StateTests/RandomTests/st201503181645GO"}
declare_test!{StateTests_RandomTests_st201503181646GO, "StateTests/RandomTests/st201503181646GO"}
declare_test!{StateTests_RandomTests_st201503181647CPPJIT, "StateTests/RandomTests/st201503181647CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181649CPPJIT, "StateTests/RandomTests/st201503181649CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181650GO, "StateTests/RandomTests/st201503181650GO"}
declare_test!{StateTests_RandomTests_st201503181652CPPJIT, "StateTests/RandomTests/st201503181652CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181653GO, "StateTests/RandomTests/st201503181653GO"}
declare_test!{StateTests_RandomTests_st201503181654GO, "StateTests/RandomTests/st201503181654GO"}
declare_test!{StateTests_RandomTests_st201503181655CPPJIT, "StateTests/RandomTests/st201503181655CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181655GO, "StateTests/RandomTests/st201503181655GO"}
declare_test!{StateTests_RandomTests_st201503181656CPPJIT, "StateTests/RandomTests/st201503181656CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181656GO, "StateTests/RandomTests/st201503181656GO"}
declare_test!{StateTests_RandomTests_st201503181657GO, "StateTests/RandomTests/st201503181657GO"}
declare_test!{StateTests_RandomTests_st201503181658GO, "StateTests/RandomTests/st201503181658GO"}
declare_test!{StateTests_RandomTests_st201503181700GO, "StateTests/RandomTests/st201503181700GO"}
declare_test!{StateTests_RandomTests_st201503181702GO, "StateTests/RandomTests/st201503181702GO"}
declare_test!{StateTests_RandomTests_st201503181703CPPJIT, "StateTests/RandomTests/st201503181703CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181703GO, "StateTests/RandomTests/st201503181703GO"}
declare_test!{StateTests_RandomTests_st201503181704GO, "StateTests/RandomTests/st201503181704GO"}
declare_test!{StateTests_RandomTests_st201503181706GO, "StateTests/RandomTests/st201503181706GO"}
declare_test!{StateTests_RandomTests_st201503181709GO, "StateTests/RandomTests/st201503181709GO"}
declare_test!{StateTests_RandomTests_st201503181711CPPJIT, "StateTests/RandomTests/st201503181711CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181711GO, "StateTests/RandomTests/st201503181711GO"}
declare_test!{StateTests_RandomTests_st201503181713CPPJIT, "StateTests/RandomTests/st201503181713CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181713GO, "StateTests/RandomTests/st201503181713GO"}
declare_test!{StateTests_RandomTests_st201503181714GO, "StateTests/RandomTests/st201503181714GO"}
declare_test!{StateTests_RandomTests_st201503181715CPPJIT, "StateTests/RandomTests/st201503181715CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181715GO, "StateTests/RandomTests/st201503181715GO"}
declare_test!{StateTests_RandomTests_st201503181716GO, "StateTests/RandomTests/st201503181716GO"}
declare_test!{StateTests_RandomTests_st201503181717GO, "StateTests/RandomTests/st201503181717GO"}
declare_test!{StateTests_RandomTests_st201503181720CPPJIT, "StateTests/RandomTests/st201503181720CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181722GO, "StateTests/RandomTests/st201503181722GO"}
declare_test!{StateTests_RandomTests_st201503181723CPPJIT, "StateTests/RandomTests/st201503181723CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181723GO, "StateTests/RandomTests/st201503181723GO"}
declare_test!{StateTests_RandomTests_st201503181724CPPJIT, "StateTests/RandomTests/st201503181724CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181724GO, "StateTests/RandomTests/st201503181724GO"}
declare_test!{StateTests_RandomTests_st201503181725GO, "StateTests/RandomTests/st201503181725GO"}
declare_test!{StateTests_RandomTests_st201503181728GO, "StateTests/RandomTests/st201503181728GO"}
declare_test!{StateTests_RandomTests_st201503181729GO, "StateTests/RandomTests/st201503181729GO"}
declare_test!{StateTests_RandomTests_st201503181730GO, "StateTests/RandomTests/st201503181730GO"}
declare_test!{StateTests_RandomTests_st201503181731CPPJIT, "StateTests/RandomTests/st201503181731CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181732GO, "StateTests/RandomTests/st201503181732GO"}
declare_test!{StateTests_RandomTests_st201503181734CPPJIT, "StateTests/RandomTests/st201503181734CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181734GO, "StateTests/RandomTests/st201503181734GO"}
declare_test!{StateTests_RandomTests_st201503181735GO, "StateTests/RandomTests/st201503181735GO"}
declare_test!{StateTests_RandomTests_st201503181737CPPJIT, "StateTests/RandomTests/st201503181737CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181737GO, "StateTests/RandomTests/st201503181737GO"}
declare_test!{StateTests_RandomTests_st201503181738CPPJIT, "StateTests/RandomTests/st201503181738CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181738GO, "StateTests/RandomTests/st201503181738GO"}
declare_test!{StateTests_RandomTests_st201503181739GO, "StateTests/RandomTests/st201503181739GO"}
declare_test!{StateTests_RandomTests_st201503181740CPPJIT, "StateTests/RandomTests/st201503181740CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181740GO, "StateTests/RandomTests/st201503181740GO"}
declare_test!{StateTests_RandomTests_st201503181742CPPJIT, "StateTests/RandomTests/st201503181742CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181743GO, "StateTests/RandomTests/st201503181743GO"}
declare_test!{StateTests_RandomTests_st201503181744GO, "StateTests/RandomTests/st201503181744GO"}
declare_test!{StateTests_RandomTests_st201503181745CPPJIT, "StateTests/RandomTests/st201503181745CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181746GO, "StateTests/RandomTests/st201503181746GO"}
declare_test!{StateTests_RandomTests_st201503181747GO, "StateTests/RandomTests/st201503181747GO"}
declare_test!{StateTests_RandomTests_st201503181748GO, "StateTests/RandomTests/st201503181748GO"}
declare_test!{StateTests_RandomTests_st201503181749GO, "StateTests/RandomTests/st201503181749GO"}
declare_test!{StateTests_RandomTests_st201503181750CPPJIT, "StateTests/RandomTests/st201503181750CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181750GO, "StateTests/RandomTests/st201503181750GO"}
declare_test!{StateTests_RandomTests_st201503181752GO, "StateTests/RandomTests/st201503181752GO"}
declare_test!{StateTests_RandomTests_st201503181753CPPJIT, "StateTests/RandomTests/st201503181753CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181754CPPJIT, "StateTests/RandomTests/st201503181754CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181754GO, "StateTests/RandomTests/st201503181754GO"}
declare_test!{StateTests_RandomTests_st201503181755CPPJIT, "StateTests/RandomTests/st201503181755CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181755GO, "StateTests/RandomTests/st201503181755GO"}
declare_test!{StateTests_RandomTests_st201503181756GO, "StateTests/RandomTests/st201503181756GO"}
declare_test!{StateTests_RandomTests_st201503181757CPPJIT, "StateTests/RandomTests/st201503181757CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181757GO, "StateTests/RandomTests/st201503181757GO"}
declare_test!{StateTests_RandomTests_st201503181759GO, "StateTests/RandomTests/st201503181759GO"}
declare_test!{StateTests_RandomTests_st201503181800GO, "StateTests/RandomTests/st201503181800GO"}
declare_test!{StateTests_RandomTests_st201503181801GO, "StateTests/RandomTests/st201503181801GO"}
declare_test!{StateTests_RandomTests_st201503181802GO, "StateTests/RandomTests/st201503181802GO"}
declare_test!{StateTests_RandomTests_st201503181803CPPJIT, "StateTests/RandomTests/st201503181803CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181803GO, "StateTests/RandomTests/st201503181803GO"}
declare_test!{StateTests_RandomTests_st201503181804GO, "StateTests/RandomTests/st201503181804GO"}
declare_test!{StateTests_RandomTests_st201503181806GO, "StateTests/RandomTests/st201503181806GO"}
declare_test!{StateTests_RandomTests_st201503181808GO, "StateTests/RandomTests/st201503181808GO"}
declare_test!{StateTests_RandomTests_st201503181809CPPJIT, "StateTests/RandomTests/st201503181809CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181812CPPJIT, "StateTests/RandomTests/st201503181812CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181812GO, "StateTests/RandomTests/st201503181812GO"}
declare_test!{StateTests_RandomTests_st201503181814CPPJIT, "StateTests/RandomTests/st201503181814CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181815GO, "StateTests/RandomTests/st201503181815GO"}
declare_test!{StateTests_RandomTests_st201503181816CPPJIT, "StateTests/RandomTests/st201503181816CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181817CPPJIT, "StateTests/RandomTests/st201503181817CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181819GO, "StateTests/RandomTests/st201503181819GO"}
declare_test!{StateTests_RandomTests_st201503181821GO, "StateTests/RandomTests/st201503181821GO"}
declare_test!{StateTests_RandomTests_st201503181822GO, "StateTests/RandomTests/st201503181822GO"}
declare_test!{StateTests_RandomTests_st201503181823GO, "StateTests/RandomTests/st201503181823GO"}
declare_test!{StateTests_RandomTests_st201503181824GO, "StateTests/RandomTests/st201503181824GO"}
declare_test!{StateTests_RandomTests_st201503181825GO, "StateTests/RandomTests/st201503181825GO"}
declare_test!{StateTests_RandomTests_st201503181829GO, "StateTests/RandomTests/st201503181829GO"}
declare_test!{StateTests_RandomTests_st201503181830CPPJIT, "StateTests/RandomTests/st201503181830CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181833GO, "StateTests/RandomTests/st201503181833GO"}
declare_test!{StateTests_RandomTests_st201503181834CPPJIT, "StateTests/RandomTests/st201503181834CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181834GO, "StateTests/RandomTests/st201503181834GO"}
declare_test!{StateTests_RandomTests_st201503181837GO, "StateTests/RandomTests/st201503181837GO"}
declare_test!{StateTests_RandomTests_st201503181840GO, "StateTests/RandomTests/st201503181840GO"}
declare_test!{StateTests_RandomTests_st201503181842GO, "StateTests/RandomTests/st201503181842GO"}
declare_test!{StateTests_RandomTests_st201503181843GO, "StateTests/RandomTests/st201503181843GO"}
declare_test!{StateTests_RandomTests_st201503181844GO, "StateTests/RandomTests/st201503181844GO"}
declare_test!{StateTests_RandomTests_st201503181845GO, "StateTests/RandomTests/st201503181845GO"}
declare_test!{StateTests_RandomTests_st201503181846GO, "StateTests/RandomTests/st201503181846GO"}
declare_test!{StateTests_RandomTests_st201503181847GO, "StateTests/RandomTests/st201503181847GO"}
declare_test!{StateTests_RandomTests_st201503181848GO, "StateTests/RandomTests/st201503181848GO"}
declare_test!{StateTests_RandomTests_st201503181849GO, "StateTests/RandomTests/st201503181849GO"}
declare_test!{StateTests_RandomTests_st201503181850GO, "StateTests/RandomTests/st201503181850GO"}
declare_test!{StateTests_RandomTests_st201503181851CPPJIT, "StateTests/RandomTests/st201503181851CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181851GO, "StateTests/RandomTests/st201503181851GO"}
declare_test!{StateTests_RandomTests_st201503181852CPPJIT, "StateTests/RandomTests/st201503181852CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181854GO, "StateTests/RandomTests/st201503181854GO"}
declare_test!{StateTests_RandomTests_st201503181855CPPJIT, "StateTests/RandomTests/st201503181855CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181857PYTHON, "StateTests/RandomTests/st201503181857PYTHON"}
declare_test!{StateTests_RandomTests_st201503181859GO, "StateTests/RandomTests/st201503181859GO"}
declare_test!{StateTests_RandomTests_st201503181900GO, "StateTests/RandomTests/st201503181900GO"}
declare_test!{StateTests_RandomTests_st201503181903GO, "StateTests/RandomTests/st201503181903GO"}
declare_test!{StateTests_RandomTests_st201503181904GO, "StateTests/RandomTests/st201503181904GO"}
declare_test!{StateTests_RandomTests_st201503181906GO, "StateTests/RandomTests/st201503181906GO"}
declare_test!{StateTests_RandomTests_st201503181907GO, "StateTests/RandomTests/st201503181907GO"}
declare_test!{StateTests_RandomTests_st201503181910GO, "StateTests/RandomTests/st201503181910GO"}
declare_test!{StateTests_RandomTests_st201503181915GO, "StateTests/RandomTests/st201503181915GO"}
declare_test!{StateTests_RandomTests_st201503181919CPPJIT, "StateTests/RandomTests/st201503181919CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181919PYTHON, "StateTests/RandomTests/st201503181919PYTHON"}
declare_test!{StateTests_RandomTests_st201503181920GO, "StateTests/RandomTests/st201503181920GO"}
declare_test!{StateTests_RandomTests_st201503181922GO, "StateTests/RandomTests/st201503181922GO"}
declare_test!{StateTests_RandomTests_st201503181926GO, "StateTests/RandomTests/st201503181926GO"}
declare_test!{StateTests_RandomTests_st201503181929GO, "StateTests/RandomTests/st201503181929GO"}
declare_test!{StateTests_RandomTests_st201503181931CPPJIT, "StateTests/RandomTests/st201503181931CPPJIT"}
declare_test!{StateTests_RandomTests_st201503181931GO, "StateTests/RandomTests/st201503181931GO"}
declare_test!{StateTests_RandomTests_st201503181931PYTHON, "StateTests/RandomTests/st201503181931PYTHON"}
declare_test!{StateTests_RandomTests_st201503191646GO, "StateTests/RandomTests/st201503191646GO"}
declare_test!{StateTests_RandomTests_st201503200837JS, "StateTests/RandomTests/st201503200837JS"}
declare_test!{StateTests_RandomTests_st201503200838JS, "StateTests/RandomTests/st201503200838JS"}
declare_test!{StateTests_RandomTests_st201503200841JS, "StateTests/RandomTests/st201503200841JS"}
declare_test!{StateTests_RandomTests_st201503200848JS, "StateTests/RandomTests/st201503200848JS"}
declare_test!{StateTests_RandomTests_st201503240609JS, "StateTests/RandomTests/st201503240609JS"}
declare_test!{StateTests_RandomTests_st201503302200JS, "StateTests/RandomTests/st201503302200JS"}
declare_test!{StateTests_RandomTests_st201503302202JS, "StateTests/RandomTests/st201503302202JS"}
declare_test!{StateTests_RandomTests_st201503302206JS, "StateTests/RandomTests/st201503302206JS"}
declare_test!{StateTests_RandomTests_st201503302208JS, "StateTests/RandomTests/st201503302208JS"}
declare_test!{StateTests_RandomTests_st201503302210JS, "StateTests/RandomTests/st201503302210JS"}
declare_test!{StateTests_RandomTests_st201503302211JS, "StateTests/RandomTests/st201503302211JS"}
declare_test!{StateTests_RandomTests_st201504011535GO, "StateTests/RandomTests/st201504011535GO"}
declare_test!{StateTests_RandomTests_st201504011536GO, "StateTests/RandomTests/st201504011536GO"}
declare_test!{StateTests_RandomTests_st201504011547GO, "StateTests/RandomTests/st201504011547GO"}
declare_test!{StateTests_RandomTests_st201504011916JS, "StateTests/RandomTests/st201504011916JS"}
declare_test!{StateTests_RandomTests_st201504012130JS, "StateTests/RandomTests/st201504012130JS"}
declare_test!{StateTests_RandomTests_st201504012259JS, "StateTests/RandomTests/st201504012259JS"}
declare_test!{StateTests_RandomTests_st201504012359JS, "StateTests/RandomTests/st201504012359JS"}
declare_test!{StateTests_RandomTests_st201504020305JS, "StateTests/RandomTests/st201504020305JS"}
declare_test!{StateTests_RandomTests_st201504020400JS, "StateTests/RandomTests/st201504020400JS"}
declare_test!{StateTests_RandomTests_st201504020428JS, "StateTests/RandomTests/st201504020428JS"}
declare_test!{StateTests_RandomTests_st201504020431JS, "StateTests/RandomTests/st201504020431JS"}
declare_test!{StateTests_RandomTests_st201504020444JS, "StateTests/RandomTests/st201504020444JS"}
declare_test!{StateTests_RandomTests_st201504020538JS, "StateTests/RandomTests/st201504020538JS"}
declare_test!{StateTests_RandomTests_st201504020639JS, "StateTests/RandomTests/st201504020639JS"}
declare_test!{StateTests_RandomTests_st201504020836JS, "StateTests/RandomTests/st201504020836JS"}
declare_test!{StateTests_RandomTests_st201504020910JS, "StateTests/RandomTests/st201504020910JS"}
declare_test!{StateTests_RandomTests_st201504021057JS, "StateTests/RandomTests/st201504021057JS"}
declare_test!{StateTests_RandomTests_st201504021104JS, "StateTests/RandomTests/st201504021104JS"}
declare_test!{StateTests_RandomTests_st201504021237CPPJIT, "StateTests/RandomTests/st201504021237CPPJIT"}
declare_test!{StateTests_RandomTests_st201504021237GO, "StateTests/RandomTests/st201504021237GO"}
declare_test!{StateTests_RandomTests_st201504021237JS, "StateTests/RandomTests/st201504021237JS"}
declare_test!{StateTests_RandomTests_st201504021237PYTHON, "StateTests/RandomTests/st201504021237PYTHON"}
declare_test!{StateTests_RandomTests_st201504021949JS, "StateTests/RandomTests/st201504021949JS"}
declare_test!{StateTests_RandomTests_st201504022003CPPJIT, "StateTests/RandomTests/st201504022003CPPJIT"}
declare_test!{StateTests_RandomTests_st201504022124JS, "StateTests/RandomTests/st201504022124JS"}
declare_test!{StateTests_RandomTests_st201504030138JS, "StateTests/RandomTests/st201504030138JS"}
declare_test!{StateTests_RandomTests_st201504030646JS, "StateTests/RandomTests/st201504030646JS"}
declare_test!{StateTests_RandomTests_st201504030709JS, "StateTests/RandomTests/st201504030709JS"}
declare_test!{StateTests_RandomTests_st201504031133JS, "StateTests/RandomTests/st201504031133JS"}
declare_test!{StateTests_RandomTests_st201504031446JS, "StateTests/RandomTests/st201504031446JS"}
declare_test!{StateTests_RandomTests_st201504031841JS, "StateTests/RandomTests/st201504031841JS"}
declare_test!{StateTests_RandomTests_st201504041605JS, "StateTests/RandomTests/st201504041605JS"}
declare_test!{StateTests_RandomTests_st201504042052JS, "StateTests/RandomTests/st201504042052JS"}
declare_test!{StateTests_RandomTests_st201504042226CPPJIT, "StateTests/RandomTests/st201504042226CPPJIT"}
declare_test!{StateTests_RandomTests_st201504042355CPPJIT, "StateTests/RandomTests/st201504042355CPPJIT"}
declare_test!{StateTests_RandomTests_st201504050059JS, "StateTests/RandomTests/st201504050059JS"}
declare_test!{StateTests_RandomTests_st201504050733JS, "StateTests/RandomTests/st201504050733JS"}
declare_test!{StateTests_RandomTests_st201504051540JS, "StateTests/RandomTests/st201504051540JS"}
declare_test!{StateTests_RandomTests_st201504051944CPPJIT, "StateTests/RandomTests/st201504051944CPPJIT"}
declare_test!{StateTests_RandomTests_st201504052008CPPJIT, "StateTests/RandomTests/st201504052008CPPJIT"}
declare_test!{StateTests_RandomTests_st201504052014GO, "StateTests/RandomTests/st201504052014GO"}
declare_test!{StateTests_RandomTests_st201504052031CPPJIT, "StateTests/RandomTests/st201504052031CPPJIT"}
declare_test!{StateTests_RandomTests_st201504060057CPPJIT, "StateTests/RandomTests/st201504060057CPPJIT"}
declare_test!{StateTests_RandomTests_st201504060418CPPJIT, "StateTests/RandomTests/st201504060418CPPJIT"}
declare_test!{StateTests_RandomTests_st201504061106CPPJIT, "StateTests/RandomTests/st201504061106CPPJIT"}
declare_test!{StateTests_RandomTests_st201504061134CPPJIT, "StateTests/RandomTests/st201504061134CPPJIT"}
declare_test!{StateTests_RandomTests_st201504062033CPPJIT, "StateTests/RandomTests/st201504062033CPPJIT"}
declare_test!{StateTests_RandomTests_st201504062046CPPJIT, "StateTests/RandomTests/st201504062046CPPJIT"}
declare_test!{StateTests_RandomTests_st201504062314CPPJIT, "StateTests/RandomTests/st201504062314CPPJIT"}
declare_test!{StateTests_RandomTests_st201504070746JS, "StateTests/RandomTests/st201504070746JS"}
declare_test!{StateTests_RandomTests_st201504070816CPPJIT, "StateTests/RandomTests/st201504070816CPPJIT"}
declare_test!{StateTests_RandomTests_st201504070836CPPJIT, "StateTests/RandomTests/st201504070836CPPJIT"}
declare_test!{StateTests_RandomTests_st201504070839CPPJIT, "StateTests/RandomTests/st201504070839CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071041CPPJIT, "StateTests/RandomTests/st201504071041CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071056CPPJIT, "StateTests/RandomTests/st201504071056CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071621CPPJIT, "StateTests/RandomTests/st201504071621CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071653CPPJIT, "StateTests/RandomTests/st201504071653CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071750CPPJIT, "StateTests/RandomTests/st201504071750CPPJIT"}
declare_test!{StateTests_RandomTests_st201504071905CPPJIT, "StateTests/RandomTests/st201504071905CPPJIT"}
declare_test!{StateTests_RandomTests_st201504080454CPPJIT, "StateTests/RandomTests/st201504080454CPPJIT"}
declare_test!{StateTests_RandomTests_st201504080457CPPJIT, "StateTests/RandomTests/st201504080457CPPJIT"}
declare_test!{StateTests_RandomTests_st201504080650CPPJIT, "StateTests/RandomTests/st201504080650CPPJIT"}
declare_test!{StateTests_RandomTests_st201504080840CPPJIT, "StateTests/RandomTests/st201504080840CPPJIT"}
declare_test!{StateTests_RandomTests_st201504080948CPPJIT, "StateTests/RandomTests/st201504080948CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081100CPPJIT, "StateTests/RandomTests/st201504081100CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081134CPPJIT, "StateTests/RandomTests/st201504081134CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081138CPPJIT, "StateTests/RandomTests/st201504081138CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081611CPPJIT, "StateTests/RandomTests/st201504081611CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081841JAVA, "StateTests/RandomTests/st201504081841JAVA"}
declare_test!{StateTests_RandomTests_st201504081842JAVA, "StateTests/RandomTests/st201504081842JAVA"}
declare_test!{StateTests_RandomTests_st201504081843JAVA, "StateTests/RandomTests/st201504081843JAVA"}
declare_test!{StateTests_RandomTests_st201504081928CPPJIT, "StateTests/RandomTests/st201504081928CPPJIT"}
declare_test!{StateTests_RandomTests_st201504081953JAVA, "StateTests/RandomTests/st201504081953JAVA"}
declare_test!{StateTests_RandomTests_st201504081954JAVA, "StateTests/RandomTests/st201504081954JAVA"}
declare_test!{StateTests_RandomTests_st201504081955JAVA, "StateTests/RandomTests/st201504081955JAVA"}
declare_test!{StateTests_RandomTests_st201504081956JAVA, "StateTests/RandomTests/st201504081956JAVA"}
declare_test!{StateTests_RandomTests_st201504081957JAVA, "StateTests/RandomTests/st201504081957JAVA"}
declare_test!{StateTests_RandomTests_st201504082000JAVA, "StateTests/RandomTests/st201504082000JAVA"}
declare_test!{StateTests_RandomTests_st201504082001JAVA, "StateTests/RandomTests/st201504082001JAVA"}
declare_test!{StateTests_RandomTests_st201504082002JAVA, "StateTests/RandomTests/st201504082002JAVA"}
declare_test!{StateTests_RandomTests_st201504090553CPPJIT, "StateTests/RandomTests/st201504090553CPPJIT"}
declare_test!{StateTests_RandomTests_st201504090657CPPJIT, "StateTests/RandomTests/st201504090657CPPJIT"}
declare_test!{StateTests_RandomTests_st201504091403CPPJIT, "StateTests/RandomTests/st201504091403CPPJIT"}
declare_test!{StateTests_RandomTests_st201504091641CPPJIT, "StateTests/RandomTests/st201504091641CPPJIT"}
declare_test!{StateTests_RandomTests_st201504092303CPPJIT, "StateTests/RandomTests/st201504092303CPPJIT"}
declare_test!{StateTests_RandomTests_st201504100125CPPJIT, "StateTests/RandomTests/st201504100125CPPJIT"}
declare_test!{StateTests_RandomTests_st201504100215CPPJIT, "StateTests/RandomTests/st201504100215CPPJIT"}
declare_test!{StateTests_RandomTests_st201504100226PYTHON, "StateTests/RandomTests/st201504100226PYTHON"}
declare_test!{StateTests_RandomTests_st201504100308CPPJIT, "StateTests/RandomTests/st201504100308CPPJIT"}
declare_test!{StateTests_RandomTests_st201504100337CPPJIT, "StateTests/RandomTests/st201504100337CPPJIT"}
declare_test!{StateTests_RandomTests_st201504100341CPPJIT, "StateTests/RandomTests/st201504100341CPPJIT"}
declare_test!{StateTests_RandomTests_st201504101009CPPJIT, "StateTests/RandomTests/st201504101009CPPJIT"}
declare_test!{StateTests_RandomTests_st201504101150CPPJIT, "StateTests/RandomTests/st201504101150CPPJIT"}
declare_test!{StateTests_RandomTests_st201504101223CPPJIT, "StateTests/RandomTests/st201504101223CPPJIT"}
declare_test!{StateTests_RandomTests_st201504101338CPPJIT, "StateTests/RandomTests/st201504101338CPPJIT"}
declare_test!{StateTests_RandomTests_st201504101754PYTHON, "StateTests/RandomTests/st201504101754PYTHON"}
declare_test!{StateTests_RandomTests_st201504111554CPPJIT, "StateTests/RandomTests/st201504111554CPPJIT"}
declare_test!{StateTests_RandomTests_st201504130653JS, "StateTests/RandomTests/st201504130653JS"}
declare_test!{StateTests_RandomTests_st201504131821CPPJIT, "StateTests/RandomTests/st201504131821CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140229CPPJIT, "StateTests/RandomTests/st201504140229CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140236CPPJIT, "StateTests/RandomTests/st201504140236CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140359CPPJIT, "StateTests/RandomTests/st201504140359CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140750CPPJIT, "StateTests/RandomTests/st201504140750CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140818CPPJIT, "StateTests/RandomTests/st201504140818CPPJIT"}
declare_test!{StateTests_RandomTests_st201504140900CPPJIT, "StateTests/RandomTests/st201504140900CPPJIT"}
declare_test!{StateTests_RandomTests_st201504150854CPPJIT, "StateTests/RandomTests/st201504150854CPPJIT"}
declare_test!{StateTests_RandomTests_st201504151057CPPJIT, "StateTests/RandomTests/st201504151057CPPJIT"}
declare_test!{StateTests_RandomTests_st201504202124CPPJIT, "StateTests/RandomTests/st201504202124CPPJIT"}
declare_test!{StateTests_RandomTests_st201504210245CPPJIT, "StateTests/RandomTests/st201504210245CPPJIT"}
declare_test!{StateTests_RandomTests_st201504210957CPPJIT, "StateTests/RandomTests/st201504210957CPPJIT"}
declare_test!{StateTests_RandomTests_st201504211739CPPJIT, "StateTests/RandomTests/st201504211739CPPJIT"}
declare_test!{StateTests_RandomTests_st201504212038CPPJIT, "StateTests/RandomTests/st201504212038CPPJIT"}
declare_test!{StateTests_RandomTests_st201504230729CPPJIT, "StateTests/RandomTests/st201504230729CPPJIT"}
declare_test!{StateTests_RandomTests_st201504231639CPPJIT, "StateTests/RandomTests/st201504231639CPPJIT"}
declare_test!{StateTests_RandomTests_st201504231710CPPJIT, "StateTests/RandomTests/st201504231710CPPJIT"}
declare_test!{StateTests_RandomTests_st201504231742CPPJIT, "StateTests/RandomTests/st201504231742CPPJIT"}
declare_test!{StateTests_RandomTests_st201504232350CPPJIT, "StateTests/RandomTests/st201504232350CPPJIT"}
declare_test!{StateTests_RandomTests_st201504240140CPPJIT, "StateTests/RandomTests/st201504240140CPPJIT"}
declare_test!{StateTests_RandomTests_st201504240220CPPJIT, "StateTests/RandomTests/st201504240220CPPJIT"}
declare_test!{StateTests_RandomTests_st201504240351CPPJIT, "StateTests/RandomTests/st201504240351CPPJIT"}
declare_test!{StateTests_RandomTests_st201504240817CPPJIT, "StateTests/RandomTests/st201504240817CPPJIT"}
declare_test!{StateTests_RandomTests_st201504241118CPPJIT, "StateTests/RandomTests/st201504241118CPPJIT"}
declare_test!{StateTests_RandomTests_st201505021810CPPJIT, "StateTests/RandomTests/st201505021810CPPJIT"}
declare_test!{StateTests_RandomTests_st201505050557JS, "StateTests/RandomTests/st201505050557JS"}
declare_test!{StateTests_RandomTests_st201505050929GO, "StateTests/RandomTests/st201505050929GO"}
declare_test!{StateTests_RandomTests_st201505050942PYTHON, "StateTests/RandomTests/st201505050942PYTHON"}
declare_test!{StateTests_RandomTests_st201505051004PYTHON, "StateTests/RandomTests/st201505051004PYTHON"}
declare_test!{StateTests_RandomTests_st201505051016PYTHON, "StateTests/RandomTests/st201505051016PYTHON"}
declare_test!{StateTests_RandomTests_st201505051114GO, "StateTests/RandomTests/st201505051114GO"}
declare_test!{StateTests_RandomTests_st201505051238GO, "StateTests/RandomTests/st201505051238GO"}
declare_test!{StateTests_RandomTests_st201505051249GO, "StateTests/RandomTests/st201505051249GO"}
declare_test!{StateTests_RandomTests_st201505051558PYTHON, "StateTests/RandomTests/st201505051558PYTHON"}
declare_test!{StateTests_RandomTests_st201505051611PYTHON, "StateTests/RandomTests/st201505051611PYTHON"}
declare_test!{StateTests_RandomTests_st201505051648JS, "StateTests/RandomTests/st201505051648JS"}
declare_test!{StateTests_RandomTests_st201505051710GO, "StateTests/RandomTests/st201505051710GO"}
declare_test!{StateTests_RandomTests_st201505052013GO, "StateTests/RandomTests/st201505052013GO"}
declare_test!{StateTests_RandomTests_st201505052102JS, "StateTests/RandomTests/st201505052102JS"}
declare_test!{StateTests_RandomTests_st201505052235GO, "StateTests/RandomTests/st201505052235GO"}
declare_test!{StateTests_RandomTests_st201505052238JS, "StateTests/RandomTests/st201505052238JS"}
declare_test!{StateTests_RandomTests_st201505052242PYTHON, "StateTests/RandomTests/st201505052242PYTHON"}
declare_test!{StateTests_RandomTests_st201505052343PYTHON, "StateTests/RandomTests/st201505052343PYTHON"}
declare_test!{StateTests_RandomTests_st201505060120GO, "StateTests/RandomTests/st201505060120GO"}
declare_test!{StateTests_RandomTests_st201505060121GO, "StateTests/RandomTests/st201505060121GO"}
declare_test!{StateTests_RandomTests_st201505060136PYTHON, "StateTests/RandomTests/st201505060136PYTHON"}
declare_test!{StateTests_RandomTests_st201505060646JS, "StateTests/RandomTests/st201505060646JS"}
declare_test!{StateTests_RandomTests_st201505252314CPPJIT, "StateTests/RandomTests/st201505252314CPPJIT"}
declare_test!{StateTests_RandomTests_st201505272131CPPJIT, "StateTests/RandomTests/st201505272131CPPJIT"}
declare_test!{StateTests_RandomTests_st201506040034GO, "StateTests/RandomTests/st201506040034GO"}
declare_test!{StateTests_RandomTests_st201506040157GO, "StateTests/RandomTests/st201506040157GO"}
declare_test!{StateTests_RandomTests_st201506052130GO, "StateTests/RandomTests/st201506052130GO"}
declare_test!{StateTests_RandomTests_st201506060929GO, "StateTests/RandomTests/st201506060929GO"}
declare_test!{StateTests_RandomTests_st201506061255GO, "StateTests/RandomTests/st201506061255GO"}
declare_test!{StateTests_RandomTests_st201506062331GO, "StateTests/RandomTests/st201506062331GO"}
declare_test!{StateTests_RandomTests_st201506070548GO, "StateTests/RandomTests/st201506070548GO"}
declare_test!{StateTests_RandomTests_st201506071050GO, "StateTests/RandomTests/st201506071050GO"}
declare_test!{StateTests_RandomTests_st201506071624GO, "StateTests/RandomTests/st201506071624GO"}
declare_test!{StateTests_RandomTests_st201506071819GO, "StateTests/RandomTests/st201506071819GO"}
declare_test!{StateTests_RandomTests_st201506072007GO, "StateTests/RandomTests/st201506072007GO"}
declare_test!{StateTests_RandomTests_st201506080556GO, "StateTests/RandomTests/st201506080556GO"}
declare_test!{StateTests_RandomTests_st201506080721GO, "StateTests/RandomTests/st201506080721GO"}
declare_test!{StateTests_RandomTests_st201506091836GO, "StateTests/RandomTests/st201506091836GO"}
declare_test!{StateTests_RandomTests_st201506092032GO, "StateTests/RandomTests/st201506092032GO"}
declare_test!{StateTests_RandomTests_st201506101359JS, "StateTests/RandomTests/st201506101359JS"}
declare_test!{StateTests_RandomTests_st201507030359GO, "StateTests/RandomTests/st201507030359GO"}
