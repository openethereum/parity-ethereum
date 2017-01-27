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

//! benchmarking for bigint
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

#![feature(test)]
#![feature(asm)]

extern crate test;
extern crate ethcore_util;

use test::{Bencher, black_box};
use ethcore_util::{U256, U512, Uint, U128};

#[bench]
fn u256_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let zero = black_box(U256::zero());
		(0..n).fold(zero, |old, new| { old.overflowing_add(U256::from(black_box(new))).0 })
	});
}

#[bench]
fn u256_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let max = black_box(U256::max_value());
		(0..n).fold(max, |old, new| { old.overflowing_sub(U256::from(black_box(new))).0 })
	});
}

#[bench]
fn u512_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let max = black_box(U512::max_value());
		(0..n).fold(
			max,
			|old, new| {
				let new = black_box(new);
				let p = new % 2;
				old.overflowing_sub(U512([p, p, p, p, p, p, p, new])).0
			}
		)
	});
}

#[bench]
fn u512_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let zero = black_box(U512::zero());
		(0..n).fold(zero,
			|old, new| {
				let new = black_box(new);
				old.overflowing_add(U512([new, new, new, new, new, new, new, new])).0
			})
	});
}

#[bench]
fn u256_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let one = black_box(U256::one());
		(0..n).fold(one, |old, new| { old.overflowing_mul(U256::from(black_box(new))).0 })
	});
}


#[bench]
fn u256_full_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let one = black_box(U256::one());
		(0..n).fold(one,
			|old, new| {
				let new = black_box(new);
				let U512(ref u512words) = old.full_mul(U256([new, new, new, new]));
				U256([u512words[0], u512words[2], u512words[2], u512words[3]])
			})
	});
}


#[bench]
fn u128_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U128([12345u64, 0u64]), |old, new| { old.overflowing_mul(U128::from(new)).0 })
	});
}

