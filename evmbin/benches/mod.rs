// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
extern crate rustc_serialize;

use self::test::{Bencher, black_box};

use evm::run_vm;
use ethcore::action_params::ActionParams;
use ethcore_util::{U256, Uint};
use rustc_serialize::hex::FromHex;

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
		"606060405260005b620f42408112156019575b6001016007565b600081905550600680602b6000396000f3606060405200".from_hex().unwrap()
	);

	b.iter(|| {
		let mut params = ActionParams::default();
		params.gas = gas;
		params.code = Some(code.clone());

		run_vm(params)
	});
}

