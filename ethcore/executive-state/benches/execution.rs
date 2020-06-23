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

//! Benchmark transaction execution of two blocks from mainnet, both of average
//! size (~35kb as RLP), one with 229 transactions (#8481475, Constantinople era)
//! and the other with 139 transactions (9532543, Istanbul era). Note that the
//! benchmark here is almost completely CPU-bound and does not involve IO at all,
//! so be careful not to draw too many conclusions from the results.

use std::time::{Duration, Instant};
use criterion::{Criterion, criterion_group, criterion_main};

use account_state::{CleanupMode, State};
use common_types::{
	header::Header,
	transaction::SignedTransaction,
	verification::Unverified
};
use ethcore::test_helpers::new_temp_db;
use ethcore_db as db;
use ethereum_types::U256;
use executive_state::ExecutiveState;
use spec::{new_constantinople_test_machine, new_istanbul_test_machine};
use state_db::StateDB;
use tempfile::Builder;

fn build_state() -> State<StateDB> {
	let db_path = Builder::new().prefix("execution-bench").tempdir().unwrap();
	let db = new_temp_db(&db_path.path());
	let journal_db = journaldb::new(db.key_value().clone(), journaldb::Algorithm::OverlayRecent, db::COL_STATE);
	let state_db = StateDB::new(journal_db, 25 * 1024 * 1024);
	State::new(state_db, U256::zero(), Default::default())
}

fn setup_state_for_block(state: &mut State<StateDB>, block: Unverified) -> Vec<SignedTransaction> {
	block.transactions
		.into_iter()
		.map(|tx| tx.verify_unordered().expect("tx is from known-good block"))
		.inspect(|tx| {
			// Ensure we have enough cash to execute the transaction
			let gas_cost = tx.gas * tx.gas_price;
			state.add_balance(&tx.sender(), &(tx.value + gas_cost), CleanupMode::ForceCreate).unwrap();
			// Fix up the nonce such that the state has the expected nonce
			if state.nonce(&tx.sender()).unwrap() == U256::zero() {
				for _ in 0..tx.nonce.as_usize() {
					state.inc_nonce(&tx.sender()).unwrap();
				}
			}
		})
		.collect::<Vec<_>>()

}
fn build_env_info(header: &Header) -> vm::EnvInfo {
	vm::EnvInfo {
		number: header.number(),
		author: *header.author(),
		timestamp: header.timestamp(),
		difficulty: *header.difficulty(),
		gas_limit: *header.gas_limit() * 10,
		last_hashes: std::sync::Default::default(),
		gas_used: *header.gas_used(),
	}
}

macro_rules! bench_tx_apply {
	($b: expr, $state: expr, $env_info: expr, $machine: expr, $signed_txs: expr, tracing => $tracing: expr ) => {
		$b.iter_custom(|iters| {
			let mut dur = Duration::new(0, 0);
			for _ in 0..iters {
				$state.checkpoint();
				let start = Instant::now();
				for tx in &$signed_txs {
					let outcome = $state.apply(&$env_info, &$machine, tx, $tracing);
					assert!(outcome.is_ok())
				}
				dur += start.elapsed();
				$state.revert_to_checkpoint();
			}
			dur
		})
	}
}

fn execute_8481475(c: &mut Criterion) {
	// Block from the Constantinople era; 202 transactions, 32k RLP
	let constantinople_block = Unverified::from_rlp(include_bytes!("./8481475.rlp").to_vec()).unwrap();
	let mut state = build_state();
	let env_info = build_env_info(&constantinople_block.header);
	let signed_txs = setup_state_for_block(&mut state, constantinople_block);

	let machine = new_constantinople_test_machine();
	c.bench_function("Block 8481475, apply txs (Costantinople, tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => true);
	});

	c.bench_function("Block 8481475, apply txs (Costantinople, no tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => false);
	});

	let machine = new_istanbul_test_machine();
	c.bench_function("Block 8481475, apply txs (Istanbul, tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => true);
	});

	c.bench_function("Block 8481475, apply txs (Istanbul, no tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => false);
	});
}

fn execute_9532543(c: &mut Criterion) {
	// Block from the Istanbul era; 139 transactions, 38k RLP
	let istanbul_block = Unverified::from_rlp(include_bytes!("./9532543.rlp").to_vec()).unwrap();
	let mut state = build_state();
	let env_info = build_env_info(&istanbul_block.header);
	let signed_txs = setup_state_for_block(&mut state, istanbul_block);

	let machine = new_constantinople_test_machine();
	c.bench_function("Block 9532543, apply txs (Constantinople, tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => true);
	});

	c.bench_function("Block 9532543, apply txs (Constantinople, no tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => false);
	});

	let machine = new_istanbul_test_machine();
	c.bench_function("Block 9532543, apply txs (Istanbul, tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => true);
	});

	c.bench_function("Block 9532543, apply txs (Istanbul, no tracing)", |b| {
		bench_tx_apply!(b, state, env_info, machine, signed_txs, tracing => false);
	});
}

criterion_group!(benches, execute_8481475, execute_9532543);
criterion_main!(benches);
