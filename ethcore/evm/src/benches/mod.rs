// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! benchmarking for EVM
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

extern crate test;

use self::test::{Bencher, black_box};

use bigint::prelude::U256;
use bigint::hash::H256;
use util::*;
use vm::ActionParams;
use evm::{self, Factory, VMType};
use evm::tests::FakeExt;

#[bench]
fn simple_loop_log0_usize(b: &mut Bencher) {
	simple_loop_log0(U256::from(::std::usize::MAX), b)
}

#[bench]
fn simple_loop_log0_u256(b: &mut Bencher) {
	simple_loop_log0(!U256::zero(), b)
}

fn simple_loop_log0(gas: U256, b: &mut Bencher) {
	let mut vm = Factory::new(VMType::Interpreter).create(gas);
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = black_box(
		"62ffffff5b600190036000600fa0600357".from_hex().unwrap()
	);

	b.iter(|| {
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(code.clone());

		result(vm.exec(params, &mut ext))
	});
}

#[bench]
fn mem_gas_calculation_same_usize(b: &mut Bencher) {
	mem_gas_calculation_same(U256::from(::std::usize::MAX), b)
}

#[bench]
fn mem_gas_calculation_same_u256(b: &mut Bencher) {
	mem_gas_calculation_same(!U256::zero(), b)
}

fn mem_gas_calculation_same(gas: U256, b: &mut Bencher) {
	let mut vm = Factory::new(VMType::Interpreter).create(gas);
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

	b.iter(|| {
		let code = black_box(
			"6110006001556001546000555b610fff805560016000540380600055600c57".from_hex().unwrap()
		);

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(code.clone());

		result(vm.exec(params, &mut ext))
	});
}

#[bench]
fn mem_gas_calculation_increasing_usize(b: &mut Bencher) {
	mem_gas_calculation_increasing(U256::from(::std::usize::MAX), b)
}

#[bench]
fn mem_gas_calculation_increasing_u256(b: &mut Bencher) {
	mem_gas_calculation_increasing(!U256::zero(), b)
}

fn mem_gas_calculation_increasing(gas: U256, b: &mut Bencher) {
	let mut vm = Factory::new(VMType::Interpreter).create(gas);
	let mut ext = FakeExt::new();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

	b.iter(|| {
		let code = black_box(
			"6110006001556001546000555b610fff60005401805560016000540380600055600c57".from_hex().unwrap()
		);

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = gas;
		params.code = Some(code.clone());

		result(vm.exec(params, &mut ext))
	});
}

fn result(r: evm::Result<evm::GasLeft>) -> U256 {
	match r {
		Ok(evm::GasLeft::Known(v)) => v,
		Ok(evm::GasLeft::NeedsReturn(v, _)) => v,
		_ => U256::zero(),
	}
}
