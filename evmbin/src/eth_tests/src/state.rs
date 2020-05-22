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
use super::json;

use ethcore::{
	log::trace,
	pod::PodState,
	machine,
	ethereum_types::{H256},
	test_helpers::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess},
	types::transaction::SignedTransaction,
	vm::EnvInfo
};

#[allow(dead_code)]
fn skip_test(test: &super::runner::StateTests, subname: &str, chain: &String, number: usize) -> bool {
	trace!(target: "json-tests", "[state, skip_test] subname: '{}', chain: '{}', number: {}", subname, chain, number);
	test.skip.iter().any(|state_test|{
		if let Some(subtest) = state_test.names.get(subname) {
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
pub fn json_chain_test<H: FnMut(&str, HookType)>(state_test: &super::runner::StateTests, path: &Path, json_data: &[u8], start_stop_hook: &mut H, is_legacy: bool) -> Vec<String> {
	let _ = ::env_logger::try_init();
	let tests = json::state::Test::load(json_data)
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
					let info = format!("   - state: {} | {:?} ({}/{}) ...", name, spec_name, i + 1, total);
					if skip_test(&state_test, &name, &spec.name, i + 1) {
						println!("{}: SKIPPED", info);
						continue;
					}

					let post_root: H256 = state.hash.into();
					let transaction: SignedTransaction = multitransaction.select(&state.indexes).into();

					let result = || -> Result<_, EvmTestError> {
						Ok(EvmTestClient::from_pod_state(&spec, pre.clone())?
							.transact(&env, transaction, ethcore::trace::NoopTracer, ethcore::trace::NoopVMTracer))
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

	failed
}
