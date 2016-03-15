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
extern crate rand;

use test::{Bencher, black_box};
use ethcore_util::numbers::*;

#[bench]
fn u256_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256([rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()]), |old, new| { old.overflowing_add(U256::from(new)).0 })
	});
}

#[bench]
fn u256_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256([rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()]), |old, new| { old.overflowing_sub(U256::from(new)).0 })
	});
}

#[bench]
fn u512_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(
			U512([
				rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(),
				rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()
			]),
			|old, new| {
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
		(0..n).fold(U512([0, 0, 0, 0, 0, 0, 0, 0]),
			|old, new| { old.overflowing_add(U512([new, new, new, new, new, new, new, new])).0 })
	});
}

#[bench]
fn u256_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256([rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()]), |old, new| { old.overflowing_mul(U256::from(new)).0 })
	});
}


#[bench]
fn u256_full_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(0..n).fold(U256([rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()]),
			|old, _new| {
				let U512(ref u512words) = old.full_mul(U256([rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>(), rand::random::<u64>()]));
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

