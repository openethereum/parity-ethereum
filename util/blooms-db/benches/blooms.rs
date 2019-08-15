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

#![feature(test)]

extern crate test;
extern crate tempdir;
extern crate blooms_db;
extern crate ethbloom;

use std::iter;
use test::Bencher;
use tempdir::TempDir;
use blooms_db::Database;
use ethbloom::Bloom;

#[bench]
fn blooms_filter_1_million_ok(b: &mut Bencher) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	b.iter(|| {
		let matches = database.filter(0, 999_999, Some(&bloom)).unwrap();
		assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
	});
}

#[bench]
fn blooms_filter_1_million_miss(b: &mut Bencher) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	let bad_bloom = Bloom::from_low_u64_be(0x0001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	b.iter(|| {
		let matches = database.filter(0, 999_999, Some(&bad_bloom)).unwrap();
		assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
	});
}

#[bench]
fn blooms_filter_1_million_miss_and_ok(b: &mut Bencher) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	let bad_bloom = Bloom::from_low_u64_be(0x0001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	b.iter(|| {
		let matches = database.filter(0, 999_999, &vec![bad_bloom, bloom]).unwrap();
		assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
	});
}
