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

use ethereum_types::U256;
use criterion::{Criterion, criterion_group, criterion_main};

use ::verification::verification;
use common_types::{
	verification::Unverified
};
use tempdir::TempDir;
use spec::new_constantinople_test_machine;
use ethash::{EthashParams, Ethash};


fn default_ethash_params() -> EthashParams {
	EthashParams {
		minimum_difficulty: U256::from(131072),
		difficulty_bound_divisor: U256::from(2048),
		difficulty_increment_divisor: 10,
		metropolis_difficulty_increment_divisor: 9,
		homestead_transition: 1150000,
		duration_limit: 13,
		block_reward: {
			let mut ret = BTreeMap::new();
			ret.insert(0, 0.into());
			ret
		},
		difficulty_hardfork_transition: u64::max_value(),
		difficulty_hardfork_bound_divisor: U256::from(0),
		bomb_defuse_transition: u64::max_value(),
		eip100b_transition: u64::max_value(),
		ecip1010_pause_transition: u64::max_value(),
		ecip1010_continue_transition: u64::max_value(),
		ecip1017_era_rounds: u64::max_value(),
		expip2_transition: u64::max_value(),
		expip2_duration_limit: 30,
		block_reward_contract: None,
		block_reward_contract_transition: 0,
		difficulty_bomb_delays: BTreeMap::new(),
		progpow_transition: u64::max_value(),
	}
}

fn build_unverified_block() -> Unverified {
	let rlp_bytes = include_bytes!("./8447676.rlp").to_vec();
	Unverified::from_rlp(rlp_bytes).expect("RLP bytes from disk are ok")
}

fn block_verification(c: &mut Criterion) {
	let machine = new_constantinople_test_machine();
	let ethparams = default_ethash_params();
	let tempdir = TempDir::new("").unwrap();
	let ethash = Ethash::new(tempdir.path(), ethparams, machine, None);
	let unverified_block = build_unverified_block();

	c.bench_function("verify_block_basic", |b| {
		b.iter(|| {
			verification::verify_block_basic(&unverified_block, &ethash, true)
		})
	});

	c.bench_function("verify_block_unordered", |b| {
		b.iter( || {
			let unverified_block = build_unverified_block();
			let _ = verification::verify_block_unordered(unverified_block, &ethash, true);
		})
	});
}

criterion_group!(benches, block_verification);
criterion_main!(benches);
