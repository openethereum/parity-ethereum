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

use super::HookType;
use client::{
    Balance, BlockChainClient, BlockId, ChainInfo, Client, ClientConfig, EvmTestClient,
    ImportBlock, Nonce, StateOrBlock,
};
use ethereum_types::{H256, U256};
use ethjson;
use io::IoChannel;
use log::warn;
use miner::Miner;
use rustc_hex::ToHex;
use spec::Genesis;
use std::{path::Path, sync::Arc};
use test_helpers;
use verification::{queue::kind::blocks::Unverified, VerifierType};

fn check_poststate(
    client: &Arc<Client>,
    test_name: &str,
    post_state: ethjson::blockchain::State,
) -> bool {
    let mut success = true;

    for (address, expected) in post_state {
        if let Some(expected_balance) = expected.balance {
            let expected_balance: U256 = expected_balance.into();
            let current_balance = client
                .balance(
                    &address.clone().into(),
                    StateOrBlock::Block(BlockId::Latest),
                )
                .unwrap();
            if expected_balance != current_balance {
                warn!(target: "json-tests", "{} – Poststate {:?} balance mismatch current={} expected={}",
					test_name, address, current_balance, expected_balance);
                success = false;
            }
        }

        if let Some(expected_nonce) = expected.nonce {
            let expected_nonce: U256 = expected_nonce.into();
            let current_nonce = client
                .nonce(&address.clone().into(), BlockId::Latest)
                .unwrap();
            if expected_nonce != current_nonce {
                warn!(target: "json-tests", "{} – Poststate {:?} nonce mismatch current={} expected={}",
					test_name, address, current_nonce, expected_nonce);
                success = false;
            }
        }

        if let Some(expected_code) = expected.code {
            let expected_code: String = expected_code.to_hex();
            let current_code = match client.code(
                &address.clone().into(),
                StateOrBlock::Block(BlockId::Latest),
            ) {
                Some(Some(code)) => code.to_hex(),
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
                uint_position.0.to_big_endian(position.as_mut());

                let mut expected_value = H256::default();
                uint_expected_value.0.to_big_endian(expected_value.as_mut());

                let current_value = client
                    .storage_at(
                        &address.clone().into(),
                        &position,
                        StateOrBlock::Block(BlockId::Latest),
                    )
                    .unwrap();

                if current_value != expected_value {
                    let position: &[u8] = position.as_ref();
                    let current_value: &[u8] = current_value.as_ref();
                    let expected_value: &[u8] = expected_value.as_ref();
                    warn!(target: "json-tests", "{} – Poststate {:?} state {} mismatch actual={} expected={}",
						test_name, address, position.to_hex(), current_value.to_hex(),
						expected_value.to_hex());
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
    }
    success
}

pub fn json_chain_test<H: FnMut(&str, HookType)>(
    test: &ethjson::test::ChainTests,
    path: &Path,
    json_data: &[u8],
    start_stop_hook: &mut H,
) -> Vec<String> {
    let _ = ::env_logger::try_init();
    let tests = ethjson::blockchain::Test::load(json_data).expect(&format!(
        "Could not parse JSON chain test data from {}",
        path.display()
    ));
    let mut failed = Vec::new();

    for (name, blockchain) in tests.into_iter() {
        if !super::debug_include_test(&name) {
            continue;
        }

        let skip_test = test
            .skip
            .iter()
            .any(|block_test| block_test.names.contains(&name));

        if skip_test {
            info!("   SKIPPED {:?} {:?}", name, blockchain.network);
            continue;
        }

        let mut fail = false;
        {
            let mut fail_unless = |cond: bool| {
                if !cond && !fail {
                    failed.push(name.clone());
                    flushln!("FAIL");
                    fail = true;
                    true
                } else {
                    false
                }
            };

            let spec = {
                let mut spec = match EvmTestClient::spec_from_json(&blockchain.network) {
                    Some(spec) => spec,
                    None => {
                        info!(
                            "   SKIPPED {:?} {:?} - Unimplemented chainspec ",
                            name, blockchain.network
                        );
                        continue;
                    }
                };

                let genesis = Genesis::from(blockchain.genesis());
                let state = From::from(blockchain.pre_state.clone());
                spec.set_genesis_state(state)
                    .expect("Failed to overwrite genesis state");
                spec.overwrite_genesis_params(genesis);
                spec
            };

            start_stop_hook(&name, HookType::OnStart);

            {
                let db = test_helpers::new_db();
                let mut config = ClientConfig::default();
                if ethjson::blockchain::Engine::NoProof == blockchain.engine {
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
                )
                .expect("Failed to instantiate a new Client");

                for b in blockchain.blocks_rlp() {
                    let bytes_len = b.len();
                    let block = Unverified::from_rlp(b);
                    match block {
                        Ok(block) => {
                            let num = block.header.number();
                            trace!(target: "json-tests", "{} – Importing {} bytes. Block #{}", name, bytes_len, num);
                            let res = client.import_block(block);
                            if let Err(e) = res {
                                warn!(target: "json-tests", "{} – Error importing block #{}: {:?}", name, num, e);
                            }
                            client.flush_queue();
                            client.import_verified_blocks();
                        }
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
                        && post_state_success,
                );
            }
        }

        if fail {
            flushln!("   - chain: {}...FAILED", name);
        } else {
            flushln!("   - chain: {}...OK", name);
        }

        start_stop_hook(&name, HookType::OnStop);
    }

    failed
}
