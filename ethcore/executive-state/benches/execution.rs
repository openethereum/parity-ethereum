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

//! Benchmark transaction execution

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
use tempdir::TempDir;

fn build_state() -> State<StateDB> {
	let db_path = TempDir::new("execution-bench").unwrap();
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
		last_hashes: std::sync::Arc::new(vec![]),
		gas_used: *header.gas_used(),
	}
}

fn tx_constantinople_execution(c: &mut Criterion) {
	// Block from the Constantinople era; 202 transactions, 32k RLP
	let constantinople_block = Unverified::from_rlp(include_bytes!("./8481475.rlp").to_vec()).unwrap();
	let mut state = build_state();
	let env_info = build_env_info(&constantinople_block.header);
	let signed_txs = setup_state_for_block(&mut state, constantinople_block);
	let machine = new_constantinople_test_machine();

	c.bench_function("apply transactions with tracing (Constantinople)", |b| {
		b.iter_custom(|iters| {
			let mut dur = Duration::new(0, 0);
			for _ in 0..iters {
				state.checkpoint();
				let start = Instant::now();
				for tx in &signed_txs {
					let outcome = state.apply(&env_info, &machine, tx, true);
					assert!(outcome.is_ok())
				}
				dur += start.elapsed();
				state.revert_to_checkpoint();
			}
			dur
		})
	});

	c.bench_function("apply transactions without tracing (Constantinople)", |b| {
		b.iter_custom(|iters| {
			let mut dur = Duration::new(0, 0);
			for _ in 0..iters {
				state.checkpoint();
				let start = Instant::now();
				for tx in &signed_txs {
					let outcome = state.apply(&env_info, &machine, tx, false);
					assert!(outcome.is_ok())
				}
				dur += start.elapsed();
				state.revert_to_checkpoint();
			}
			dur
		})
	});
}

fn tx_istanbul_execution(c: &mut Criterion) {
	// Block from the Istanbul era; 139 transactions, 38k RLP
	let istanbul_block = Unverified::from_rlp(include_bytes!("./9532543.rlp").to_vec()).unwrap();
	let mut state = build_state();
	let env_info = build_env_info(&istanbul_block.header);
	let signed_txs = setup_state_for_block(&mut state, istanbul_block);
	let machine = new_istanbul_test_machine();

	c.bench_function("apply transactions with tracing (Istanbul)", |b| {
		b.iter_custom(|iters| {
			let mut dur = Duration::new(0, 0);
			for _ in 0..iters {
				state.checkpoint();
				let start = Instant::now();
				for tx in &signed_txs {
					let outcome = state.apply(&env_info, &machine, tx, true);
					assert!(outcome.is_ok())
				}
				dur += start.elapsed();
				state.revert_to_checkpoint();
			}
			dur
		})
	});

	c.bench_function("apply transactions without tracing (Istanbul)", |b| {
		b.iter_custom(|iters| {
			let mut dur = Duration::new(0, 0);
			for _ in 0..iters {
				state.checkpoint();
				let start = Instant::now();
				for tx in &signed_txs {
					let outcome = state.apply(&env_info, &machine, tx, false);
					assert!(outcome.is_ok())
				}
				dur += start.elapsed();
				state.revert_to_checkpoint();
			}
			dur
		})
	});
}

criterion_group!(benches, tx_constantinople_execution, tx_istanbul_execution);
criterion_main!(benches);
