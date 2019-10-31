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

//! Benchmarking of calling builtin contract

use std::str::FromStr;

use account_state::State;
use parity_bytes::Bytes;
use ethcore::test_helpers::get_temp_state_db;
use ethereum_types::{H160, U256};
use criterion::{black_box, criterion_main, criterion_group, Criterion};
use machine::{test_helpers, Machine};
use machine::executive::CallCreateExecutive;
use machine::substate::Substate;
use trace::{NoopTracer, NoopVMTracer};
use trie_vm_factories::VmFactory;
use vm::{ActionParams, EnvInfo, Schedule};

const ECRECOVER: &str = "0000000000000000000000000000000000000001";
const SHA256: &str = "0000000000000000000000000000000000000002";
const SIGNED_DATA: &str = "hash000000000001v000000000000002r000000000000003s000000000000004";

fn single_builtin_pricing() -> Machine {
	test_helpers::load_machine(include_bytes!("../../res/ethereum/builtin_one_activation_bench.json"))
}

fn multiple_builtin_pricing() -> Machine {
	test_helpers::load_machine(include_bytes!("../../res/ethereum/builtin_multi_bench.json"))
}

fn builtin_params(address: H160, execute: bool) -> ActionParams {
	let mut params = ActionParams::default();
	params.code_address = address;
	params.gas = u64::max_value().into();
	if execute {
		params.data = Some(SIGNED_DATA.bytes().collect::<Bytes>());
	}
	params
}

fn single_activation(c: &mut Criterion) {
	let contract = H160::from_str(ECRECOVER).unwrap();
	let params = builtin_params(contract, false);

	let env_info = EnvInfo::default();
	let machine = single_builtin_pricing();
	let schedule = Schedule::default();
	let factory = VmFactory::default();
	let depth = 0;
	let stack_depth = 0;
	let parent_static_flag = false;

	let db = get_temp_state_db();
	let mut state = State::new(db, U256::from(0), Default::default());
	let mut substate = Substate::new();

    c.bench_function("single activation", move |b| {
        b.iter(|| black_box(CallCreateExecutive::new_call_raw(
				params.clone(),
				&env_info,
				&machine,
				&schedule,
				&factory,
				depth,
				stack_depth,
				parent_static_flag,
			).exec(&mut state, &mut substate, &mut NoopTracer, &mut NoopVMTracer))
		)
    });
}

fn ten_multiple_activations(c: &mut Criterion) {
	let contract = H160::from_str(ECRECOVER).unwrap();
	let params = builtin_params(contract, false);

	let env_info = EnvInfo::default();
	let machine = multiple_builtin_pricing();
	let schedule = Schedule::default();
	let factory = VmFactory::default();
	let depth = 0;
	let stack_depth = 0;
	let parent_static_flag = false;

	let db = get_temp_state_db();
	let mut state = State::new(db, U256::from(0), Default::default());
	let mut substate = Substate::new();

    c.bench_function("ten activations", move |b| {
        b.iter(|| black_box(CallCreateExecutive::new_call_raw(
				params.clone(),
				&env_info,
				&machine,
				&schedule,
				&factory,
				depth,
				stack_depth,
				parent_static_flag,
			).exec(&mut state, &mut substate, &mut NoopTracer, &mut NoopVMTracer))
		)
    });
}

fn fourty_multiple_activations(c: &mut Criterion) {
	let contract = H160::from_str(SHA256).unwrap();
	let params = builtin_params(contract, false);

	let env_info = EnvInfo::default();
	let machine = multiple_builtin_pricing();
	let schedule = Schedule::default();
	let factory = VmFactory::default();
	let depth = 0;
	let stack_depth = 0;
	let parent_static_flag = false;

	let db = get_temp_state_db();
	let mut state = State::new(db, U256::from(0), Default::default());
	let mut substate = Substate::new();

    c.bench_function("fourty activations", move |b| {
        b.iter(|| black_box(CallCreateExecutive::new_call_raw(
				params.clone(),
				&env_info,
				&machine,
				&schedule,
				&factory,
				depth,
				stack_depth,
				parent_static_flag,
			).exec(&mut state, &mut substate, &mut NoopTracer, &mut NoopVMTracer))
		)
    });
}

criterion_group!(benches, single_activation, ten_multiple_activations, fourty_multiple_activations);
criterion_main!(benches);
