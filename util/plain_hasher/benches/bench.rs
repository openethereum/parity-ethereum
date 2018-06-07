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

#![feature(test)]

extern crate test;
extern crate plain_hasher;

use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use test::{Bencher, black_box};
use plain_hasher::PlainHasher;

#[bench]
fn write_plain_hasher(b: &mut Bencher) {
	b.iter(|| {
		let n: u8 = black_box(100);
		(0..n).fold(PlainHasher::default(), |mut old, new| {
			let bb = black_box([new; 32]);
			old.write(&bb as &[u8]);
			old
		});
	});
}

#[bench]
fn write_default_hasher(b: &mut Bencher) {
	b.iter(|| {
		let n: u8 = black_box(100);
		(0..n).fold(DefaultHasher::default(), |mut old, new| {
			let bb = black_box([new; 32]);
			old.write(&bb as &[u8]);
			old
		});
	});
}
