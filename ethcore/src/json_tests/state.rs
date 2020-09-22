// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use super::{test_common::*, HookType};
use client::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess};
use ethjson;
use pod_state::PodState;
use std::path::Path;
use trace;
use types::transaction::SignedTransaction;
use vm::EnvInfo;

fn skip_test(
    test: &ethjson::test::StateTests,
    subname: &str,
    chain: &String,
    number: usize,
) -> bool {
    trace!(target: "json-tests", "[state, skip_test] subname: '{}', chain: '{}', number: {}", subname, chain, number);
    test.skip.iter().any(|state_test| {
        if let Some(subtest) = state_test.names.get(subname) {
            trace!(target: "json-tests", "[state, skip_test] Maybe skipping {:?}", subtest);
            chain == &subtest.chain
                && (subtest.subnumbers[0] == "*"
                    || subtest.subnumbers.contains(&number.to_string()))
        } else {
            false
        }
    })
}

pub fn json_chain_test<H: FnMut(&str, HookType)>(
    state_test: &ethjson::test::StateTests,
    path: &Path,
    json_data: &[u8],
    start_stop_hook: &mut H,
) -> Vec<String> {
    let _ = ::env_logger::try_init();
    let tests = ethjson::state::test::Test::load(json_data).expect(&format!(
        "Could not parse JSON state test data from {}",
        path.display()
    ));
    let mut failed = Vec::new();

    for (name, test) in tests.into_iter() {
        if !super::debug_include_test(&name) {
            continue;
        }

        start_stop_hook(&name, HookType::OnStart);

        {
            let multitransaction = test.transaction;
            let env: EnvInfo = test.env.into();
            let pre: PodState = test.pre_state.into();

            for (spec_name, states) in test.post_states {
                let total = states.len();
                let spec = match EvmTestClient::spec_from_json(&spec_name) {
                    Some(spec) => spec,
                    None => {
                        panic!(
                            "Unimplemented chainspec '{:?}' in test '{}'",
                            spec_name, name
                        );
                    }
                };

                for (i, state) in states.into_iter().enumerate() {
                    let info = format!(
                        "   - state: {} | {:?} ({}/{}) ...",
                        name,
                        spec_name,
                        i + 1,
                        total
                    );
                    if skip_test(&state_test, &name, &spec.name, i + 1) {
                        println!("{}: SKIPPED", info);
                        continue;
                    }

                    let post_root: H256 = state.hash.into();
                    let transaction: SignedTransaction =
                        multitransaction.select(&state.indexes).into();

                    let result = || -> Result<_, EvmTestError> {
                        Ok(EvmTestClient::from_pod_state(&spec, pre.clone())?.transact(
                            &env,
                            transaction,
                            trace::NoopTracer,
                            trace::NoopVMTracer,
                        ))
                    };
                    match result() {
                        Err(err) => {
                            println!("{} !!! Unexpected internal error: {:?}", info, err);
                            flushln!("{} fail", info);
                            failed.push(name.clone());
                        }
                        Ok(Ok(TransactSuccess { state_root, .. })) if state_root != post_root => {
                            println!(
                                "{} !!! State mismatch (got: {}, expect: {}",
                                info, state_root, post_root
                            );
                            flushln!("{} fail", info);
                            failed.push(name.clone());
                        }
                        Ok(Err(TransactErr {
                            state_root,
                            ref error,
                            ..
                        })) if state_root != post_root => {
                            println!(
                                "{} !!! State mismatch (got: {}, expect: {}",
                                info, state_root, post_root
                            );
                            println!("{} !!! Execution error: {:?}", info, error);
                            flushln!("{} fail", info);
                            failed.push(name.clone());
                        }
                        Ok(Err(TransactErr { error, .. })) => {
                            flushln!("{} ok ({:?})", info, error);
                        }
                        Ok(_) => {
                            flushln!("{} ok", info);
                        }
                    }
                }
            }
        }

        start_stop_hook(&name, HookType::OnStop);
    }

    failed
}
