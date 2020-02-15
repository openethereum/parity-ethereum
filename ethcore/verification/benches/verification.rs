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

//! benchmarking for verification

use std::collections::BTreeMap;

use common_types::verification::Unverified;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use ethash::{EthashParams, Ethash};
use ethereum_types::U256;
use ethcore::test_helpers::TestBlockChainClient;
use spec::new_constantinople_test_machine;
use tempdir::TempDir;

use ::verification::{
	verification,
	test_helpers::TestBlockChain,
};

/// Proof
const PROOF: &str = "bytes from disk are ok";
/// A fairly large block (32kb) with one uncle
const RLP_8481476: &[u8] = include_bytes!("./8481476-one-uncle.rlp");
/// Parent of #8481476
const RLP_8481475: &[u8] = include_bytes!("./8481475.rlp");
/// Parent of the uncle in #8481476
const RLP_8481474: &[u8] = include_bytes!("./8481474-parent-to-uncle.rlp");

// These are current production values. Needed when using real blocks.
fn ethash_params() -> EthashParams {
	EthashParams {
		minimum_difficulty: U256::from(131072),
		difficulty_bound_divisor: U256::from(2048),
		difficulty_increment_divisor: 10,
		metropolis_difficulty_increment_divisor: 9,
		duration_limit: 13,
		homestead_transition: 1150000,
		difficulty_hardfork_transition: u64::max_value(),
		difficulty_hardfork_bound_divisor: U256::from(2048),
		bomb_defuse_transition: u64::max_value(),
		eip100b_transition: 4370000,
		ecip1010_pause_transition: u64::max_value(),
		ecip1010_continue_transition: u64::max_value(),
		ecip1017_era_rounds: u64::max_value(),
		block_reward: {
			let mut m = BTreeMap::<u64, U256>::new();
			m.insert(0, 5000000000000000000u64.into());
			m.insert(4370000, 3000000000000000000u64.into());
			m.insert(7280000, 2000000000000000000u64.into());
			m
		},
		expip2_transition: u64::max_value(),
		expip2_duration_limit: 30,
		block_reward_contract_transition: 0,
		block_reward_contract: None,
		difficulty_bomb_delays: {
			let mut m = BTreeMap::new();
			m.insert(4370000, 3000000);
			m.insert(7280000, 2000000);
			m
		},
		progpow_transition: u64::max_value()
	}
}

fn build_ethash() -> Ethash {
	let machine = new_constantinople_test_machine();
	let ethash_params = ethash_params();
	let cache_dir = TempDir::new("").unwrap();
	Ethash::new(
		cache_dir.path(),
		ethash_params,
		machine,
		None
	)
}

fn build_block(rlp: &[u8]) -> Unverified {
	Unverified::from_rlp(rlp.to_vec()).expect("bytes from disk are ok; qed")
}

fn block_verification(c: &mut Criterion) {
	let ethash = build_ethash();

	// Phase 1 verification
	c.bench_function("verify_block_basic", |b| {
		b.iter_batched(|| build_block(RLP_8481476), |block| {
				assert!(verification::verify_block_basic(
					block,
					&ethash,
					true
				).is_ok())
			},
			BatchSize::SmallInput
		)
	});

	// Phase 2 verification
	c.bench_function("verify_block_unordered", |b| {
		b.iter_batched(|| build_block(RLP_8481476), |block| {
				assert!(verification::verify_block_unordered(
					block,
					&ethash,
					true
				).is_ok())
			},
			BatchSize::SmallInput
		)
	});

	// Phase 3 verification
	let parent = build_block(RLP_8481475);

	let mut block_provider = TestBlockChain::new();
	block_provider.insert(RLP_8481476.to_vec()); // block to verify
	block_provider.insert(RLP_8481475.to_vec()); // parent
	block_provider.insert(RLP_8481474.to_vec()); // uncle's parent

	let client = TestBlockChainClient::default();
	c.bench_function("verify_block_family (full)", |b| {
		b.iter_batched(
			|| {
				verification::verify_block_unordered(build_block(RLP_8481476), &ethash, true).expect(PROOF)
			},
			|preverified| {
				assert!(verification::verify_block_family::<TestBlockChainClient>(
					&parent.header,
					&ethash,
					preverified,
					&block_provider,
					&client
				).is_ok())
			},
			BatchSize::SmallInput
		)
	});
}

criterion_group!(benches, block_verification);
criterion_main!(benches);
