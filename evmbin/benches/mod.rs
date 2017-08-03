// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

#![feature(test)]

extern crate test;
extern crate ethcore;
extern crate evm;
extern crate ethcore_util;
extern crate rustc_hex;

use self::test::{Bencher, black_box};

use evm::run_vm;
use ethcore::vm::ActionParams;
use ethcore_util::U256;
use rustc_hex::FromHex;

#[bench]
fn simple_loop_usize(b: &mut Bencher) {
	simple_loop(U256::from(::std::usize::MAX), b)
}

#[bench]
fn simple_loop_u256(b: &mut Bencher) {
	simple_loop(!U256::zero(), b)
}

fn simple_loop(gas: U256, b: &mut Bencher) {
	let code = black_box(
		"606060405260005b620042408112156019575b6001016007565b600081905550600680602b6000396000f3606060405200".from_hex().unwrap()
	);

	b.iter(|| {
		let mut params = ActionParams::default();
		params.gas = gas;
		params.code = Some(code.clone());

		run_vm(params)
	});
}

#[bench]
fn rng_usize(b: &mut Bencher) {
	rng(U256::from(::std::usize::MAX), b)
}

#[bench]
fn rng_u256(b: &mut Bencher) {
	rng(!U256::zero(), b)
}

fn rng(gas: U256, b: &mut Bencher) {
	let code = black_box(
		"6060604052600360056007600b60005b62004240811215607f5767ffe7649d5eca84179490940267f47ed85c4b9a6379019367f8e5dd9a5c994bba9390930267f91d87e4b8b74e55019267ff97f6f3b29cda529290920267f393ada8dd75c938019167fe8d437c45bb3735830267f47d9a7b5428ffec019150600101600f565b838518831882186000555050505050600680609a6000396000f3606060405200".from_hex().unwrap()
	);

	b.iter(|| {
		let mut params = ActionParams::default();
		params.gas = gas;
		params.code = Some(code.clone());

		run_vm(params)
	});
}

