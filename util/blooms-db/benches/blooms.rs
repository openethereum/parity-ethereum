// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

#[macro_use]
extern crate criterion;
extern crate tempdir;
extern crate blooms_db;
extern crate ethbloom;

use std::iter;
use criterion::Criterion;
use tempdir::TempDir;
use blooms_db::Database;
use ethbloom::Bloom;

criterion_group!(
	blooms,
	bench_blooms_filter_1_million_ok,
	bench_blooms_filter_1_million_miss,
	bench_blooms_filter_1_million_miss_and_ok,
);
criterion_main!(blooms);

fn bench_blooms_filter_1_million_ok(c: &mut Criterion) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	c.bench_function("blooms_filter_1_million_ok", move |b| {
		b.iter(|| {
			let matches = database.filter(0, 999_999, Some(&bloom)).unwrap();
			assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
		})
	});
}

fn bench_blooms_filter_1_million_miss(c: &mut Criterion) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	let bad_bloom = Bloom::from_low_u64_be(0x0001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	c.bench_function("blooms_filter_1_million_miss", move |b| {
		b.iter(|| {
			let matches = database.filter(0, 999_999, Some(&bad_bloom)).unwrap();
			assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
		})
	});
}

fn bench_blooms_filter_1_million_miss_and_ok(c: &mut Criterion) {
	let tempdir = TempDir::new("").unwrap();
	let database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::zero())).unwrap();
	let bloom = Bloom::from_low_u64_be(0x001);
	let bad_bloom = Bloom::from_low_u64_be(0x0001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();

	c.bench_function("blooms_filter_1_million_miss_and_ok", move |b| {
		b.iter(|| {
			let matches = database.filter(0, 999_999, &vec![bad_bloom, bloom]).unwrap();
			assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
		})
	});
}
