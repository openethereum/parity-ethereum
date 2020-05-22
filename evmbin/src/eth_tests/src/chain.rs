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
use ethcore::{
	log::{warn,trace,info},
	client::{Client, ClientConfig},
	client_traits::{ImportBlock, ChainInfo, StateOrBlock, Balance, Nonce, BlockChainClient},
	spec::Genesis,
	miner::Miner,
	io::IoChannel,
	test_helpers::{self, EvmTestClient},
	types::{
		verification::Unverified,
		ids::BlockId,
		client_types::StateResult
	},
	verification::{VerifierType, queue::kind::BlockLike},
	ethereum_types::{U256, H256}
};

use super::json::{self,blockchain};
use ethjson::spec::State;

use super::{HookType};
use rustc_hex::ToHex;

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
pub fn json_chain_test<H: FnMut(&str, HookType)>(test: &super::runner::ChainTests, path: &Path, json_data: &[u8], start_stop_hook: &mut H, is_legacy: bool) -> Vec<String> {
	let _ = ::env_logger::try_init();
	let tests = blockchain::Test::load(json_data)
		.expect(&format!("Could not parse JSON chain test data from {}", path.display()));
	let mut failed = Vec::new();

	for (name, blockchain) in tests.into_iter() {

		let skip_test = test.skip.iter().any(|block_test| block_test.names.contains(&name));
		if skip_test {
			info!("   SKIPPED {:?} {:?}", name, blockchain.network);
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
				if json::blockchain::Engine::NoProof == blockchain.engine {
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

		if fail {
			flushed_writeln!("   - chain: {}...FAILED", name);
		} else {
			flushed_writeln!("   - chain: {}...OK", name);
		}

		start_stop_hook(&name, HookType::OnStop);
	}

	failed
}
