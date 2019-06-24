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

//! benchmarking for EVM

#[macro_use]
extern crate criterion;
extern crate bit_set;
extern crate ethereum_types;
extern crate parking_lot;
extern crate parity_util_mem as mem;
extern crate vm;
extern crate evm;
extern crate keccak_hash as hash;
extern crate memory_cache;
extern crate parity_bytes as bytes;
extern crate rustc_hex;

use criterion::{Criterion, Bencher, black_box};
use std::str::FromStr;
use std::sync::Arc;
use ethereum_types::{U256, Address};
use vm::{ActionParams, Result, GasLeft, Ext};
use vm::tests::FakeExt;
use evm::Factory;
use rustc_hex::FromHex;

criterion_group!(
	basic,
	simple_loop_log0_usize,
	simple_loop_log0_u256,
	mem_gas_calculation_same_usize,
	mem_gas_calculation_same_u256,
	mem_gas_calculation_increasing_usize,
	mem_gas_calculation_increasing_u256,
	blockhash_mulmod_small,
	blockhash_mulmod_large,
);
criterion_main!(basic);

fn simple_loop_log0_usize(b: &mut Criterion) {
	b.bench_function("simple_loop_log0_usize", |b| {
		simple_loop_log0(U256::from(::std::usize::MAX), b);
	});
}

fn simple_loop_log0_u256(b: &mut Criterion) {
	b.bench_function("simple_loop_log0_u256", |b| {
		simple_loop_log0(!U256::zero(), b);
	});
}

fn simple_loop_log0(gas: U256, b: &mut Bencher) {
	let factory = Factory::default();
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = black_box(
		"62ffffff5b600190036000600fa0600357".from_hex().unwrap()
	);

	b.iter(|| {
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(Arc::new(code.clone()));

		let vm = factory.create(params, ext.schedule(), 0);

		result(vm.exec(&mut ext).ok().unwrap())
	});
}

fn mem_gas_calculation_same_usize(b: &mut Criterion) {
	b.bench_function("mem_gas_calculation_same_usize", |b| {
		mem_gas_calculation_same(U256::from(::std::usize::MAX), b);
	});
}

fn mem_gas_calculation_same_u256(b: &mut Criterion) {
	b.bench_function("mem_gas_calculation_same_u256", |b| {
		mem_gas_calculation_same(!U256::zero(), b);
	});
}

fn mem_gas_calculation_same(gas: U256, b: &mut Bencher) {
	let factory = Factory::default();
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

	b.iter(|| {
		let code = black_box(
			"6110006001556001546000555b610fff805560016000540380600055600c57".from_hex().unwrap()
		);

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(Arc::new(code.clone()));

		let vm = factory.create(params, ext.schedule(), 0);

		result(vm.exec(&mut ext).ok().unwrap())
	});
}

fn mem_gas_calculation_increasing_usize(b: &mut Criterion) {
	b.bench_function("mem_gas_calculation_increasing_usize", |b| {
		mem_gas_calculation_increasing(U256::from(::std::usize::MAX), b);
	});
}

fn mem_gas_calculation_increasing_u256(b: &mut Criterion) {
	b.bench_function("mem_gas_calculation_increasing_u256", |b| {
		mem_gas_calculation_increasing(!U256::zero(), b);
	});
}

fn mem_gas_calculation_increasing(gas: U256, b: &mut Bencher) {
	let factory = Factory::default();
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

	b.iter(|| {
		let code = black_box(
			"6110006001556001546000555b610fff60005401805560016000540380600055600c57".from_hex().unwrap()
		);

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(Arc::new(code.clone()));

		let vm = factory.create(params, ext.schedule(), 0);

		result(vm.exec(&mut ext).ok().unwrap())
	});
}

fn blockhash_mulmod_small(b: &mut Criterion) {
	b.bench_function("blockhash_mulmod_small", |b| {
		let factory = Factory::default();
		let mut ext = FakeExt::new();

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

		b.iter(|| {
			let code = black_box(
				"6080604052348015600f57600080fd5b5060005a90505b60c881111560de5760017effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80095060017effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80095060017effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80095060017effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80095060017effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8009505a90506016565b506035806100ed6000396000f3fe6080604052600080fdfea165627a7a72305820bde4a0ac6d0fac28fc879244baf8a6a0eda514bc95fb7ecbcaaebf2556e2687c0029".from_hex().unwrap()
			);

			let mut params = ActionParams::default();
			params.address = address.clone();
			params.gas = U256::from(4_000u64);
			params.code = Some(Arc::new(code.clone()));

			let vm = factory.create(params, ext.schedule(), 0);

			result(vm.exec(&mut ext).ok().unwrap())
		});
	});
}

fn blockhash_mulmod_large(b: &mut Criterion) {
	b.bench_function("blockhash_mulmod_large", |b| {
		let factory = Factory::default();
		let mut ext = FakeExt::new();

		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

		b.iter(|| {
			let code = black_box(
				"608060405234801561001057600080fd5b5060005a90505b60c8811115610177577efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff17efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08009507efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff17efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08009507efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff17efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08009507efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff17efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08009507efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff17efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08009505a9050610017565b506035806101866000396000f3fe6080604052600080fdfea165627a7a72305820dcaec306f67bb96f3044fff25c9af2ec66f01d0954d0656964f046f42f2780670029".from_hex().unwrap()
			);

			let mut params = ActionParams::default();
			params.address = address.clone();
			params.gas = U256::from(4_000u64);
			params.code = Some(Arc::new(code.clone()));

			let vm = factory.create(params, ext.schedule(), 0);

			result(vm.exec(&mut ext).ok().unwrap())
		});
	});
}

fn result(r: Result<evm::GasLeft>) -> U256 {
	match r {
		Ok(GasLeft::Known(gas_left)) => gas_left,
		Ok(GasLeft::NeedsReturn { gas_left,  .. }) => gas_left,
		_ => U256::zero(),
	}
}
