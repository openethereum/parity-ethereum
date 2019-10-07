// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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
use criterion::{Criterion, criterion_group, criterion_main};
use ethash::{EthashParams, Ethash};
use ethereum_types::U256;
use ethcore::test_helpers::TestBlockChainClient;
use spec::new_constantinople_test_machine;
use tempdir::TempDir;

use ::verification::{
	FullFamilyParams,
	verification,
	test_helpers::TestBlockChain,
};

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

fn block_verification(c: &mut Criterion) {
	const PROOF: &str = "bytes from disk are ok";

	let ethash = build_ethash();

	// A fairly large block (32kb) with one uncle
	let rlp_8481476 = include_bytes!("./8481476-one-uncle.rlp").to_vec();
	// Parent of #8481476
	let rlp_8481475 = include_bytes!("./8481475.rlp").to_vec();
	// Parent of the uncle in #8481476
	let rlp_8481474 = include_bytes!("./8481474-parent-to-uncle.rlp").to_vec();

	// Phase 1 verification
	c.bench_function("verify_block_basic", |b| {
		let block = Unverified::from_rlp(rlp_8481476.clone()).expect(PROOF);
		b.iter(|| {
			assert!(verification::verify_block_basic(
				&block,
				&ethash,
				true
			).is_ok());
		})
	});

	// Phase 2 verification
	c.bench_function("verify_block_unordered", |b| {
		let block = Unverified::from_rlp(rlp_8481476.clone()).expect(PROOF);
		b.iter( || {
			assert!(verification::verify_block_unordered(
				block.clone(),
				&ethash,
				true
			).is_ok());
		})
	});

	// Phase 3 verification
	let block = Unverified::from_rlp(rlp_8481476.clone()).expect(PROOF);
	let preverified = verification::verify_block_unordered(block, &ethash, true).expect(PROOF);
	let parent = Unverified::from_rlp(rlp_8481475.clone()).expect(PROOF);

	// "partial" means we skip uncle and tx verification
	c.bench_function("verify_block_family (partial)", |b| {
		b.iter(|| {
			if let Err(e) = verification::verify_block_family::<TestBlockChainClient>(
				&preverified.header,
				&parent.header,
				&ethash,
				None
			) {
				panic!("verify_block_family (partial) ERROR: {:?}", e);
			}
		});
	});

	let mut block_provider = TestBlockChain::new();
	block_provider.insert(rlp_8481476.clone()); // block to verify
	block_provider.insert(rlp_8481475.clone()); // parent
	block_provider.insert(rlp_8481474.clone()); // uncle's parent

	let client = TestBlockChainClient::default();
	c.bench_function("verify_block_family (full)", |b| {
		b.iter(|| {
			let full = FullFamilyParams { block: &preverified, block_provider: &block_provider, client: &client };
			if let Err(e) = verification::verify_block_family::<TestBlockChainClient>(
				&preverified.header,
				&parent.header,
				&ethash,
				Some(full),
			) {
				panic!("verify_block_family (full) ERROR: {:?}", e)
			}
		});
	});
}

criterion_group!(benches, block_verification);
criterion_main!(benches);
