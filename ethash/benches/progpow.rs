// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate criterion;

#[macro_use]
extern crate hex_literal;

extern crate common_types;
extern crate ethash;
extern crate tempdir;

use criterion::Criterion;
use ethash::progpow;

use tempdir::TempDir;
use ethash::NodeCacheBuilder;
use ethash::compute::light_compute;
use common_types::engines::OptimizeFor;

fn bench_hashimoto_light(c: &mut Criterion) {
	let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
	let tempdir = TempDir::new("").unwrap();
	let light = builder.light(&tempdir.path(), 1);
	let h = hex!("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f");
	let mut hash = [0; 32];
	hash.copy_from_slice(&h);

	c.bench_function("hashimoto_light", move |b| {
		b.iter(|| light_compute(&light, &hash, 0))
	});
}

fn bench_progpow_light(c: &mut Criterion) {
	let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
	let tempdir = TempDir::new("").unwrap();
	let cache = builder.new_cache(tempdir.into_path(), 0);

	let h = hex!("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f");
	let mut hash = [0; 32];
	hash.copy_from_slice(&h);

	c.bench_function("progpow_light", move |b| {
		b.iter(|| {
			let c_dag = progpow::generate_cdag(cache.as_ref());
			progpow::progpow(
				hash,
				0,
				0,
				cache.as_ref(),
				&c_dag,
			);
		})
	});
}

fn bench_progpow_optimal_light(c: &mut Criterion) {
	let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
	let tempdir = TempDir::new("").unwrap();
	let cache = builder.new_cache(tempdir.into_path(), 0);
	let c_dag = progpow::generate_cdag(cache.as_ref());

	let h = hex!("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f");
	let mut hash = [0; 32];
	hash.copy_from_slice(&h);

	c.bench_function("progpow_optimal_light", move |b| {
		b.iter(|| {
			progpow::progpow(
				hash,
				0,
				0,
				cache.as_ref(),
				&c_dag,
			);
		})
	});
}

fn bench_keccak_f800_long(c: &mut Criterion) {
	c.bench_function("keccak_f800_long(0, 0, 0)", |b| {
		b.iter(|| progpow::keccak_f800_long([0; 32], 0, [0; 8]))
	});
}

criterion_group!(benches,
	bench_hashimoto_light,
	bench_progpow_light,
	bench_progpow_optimal_light,
	bench_keccak_f800_long,
);
criterion_main!(benches);
