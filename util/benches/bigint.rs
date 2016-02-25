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

//! benchmarking for rlp
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

#![feature(test)]
#![feature(asm)]

extern crate test;
extern crate ethcore_util;

use test::{Bencher, black_box};
use ethcore_util::uint::*;

#[bench]
fn u256_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256::from(1234599u64), |old, new| { old.overflowing_add(U256::from(new)).0 })
	});
}

#[bench]
fn u256_uber_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256::from(1234599u64), |old, new| { old.uber_add(U256::from(new)).0 })
	});
}

#[bench]
fn u256_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256::from(::std::u64::MAX), |old, new| { old.overflowing_sub(U256::from(new)).0 })
	});
}

#[bench]
fn u256_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256([12345u64, 0u64, 0u64, 0u64]), |old, new| { old.overflowing_mul(U256::from(new)).0 })
	});
}


#[bench]
fn u128_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U128([12345u64, 0u64]), |old, new| { old.overflowing_mul(U128::from(new)).0 })
	});
}

