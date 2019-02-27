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

#[macro_use]
extern crate criterion;
extern crate ethash;

use criterion::Criterion;
use ethash::{NodeCacheBuilder, OptimizeFor};

const HASH: [u8; 32] = [0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe,
	                    0xe4, 0x0a, 0xb3, 0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f,
	                    0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94, 0x05, 0x52, 0x7d, 0x72];
const NONCE: u64 = 0xd7b3ac70a301a249;

criterion_group!(
	basic,
	bench_light_compute_memmap,
	bench_light_compute_memory,
	bench_light_new_round_trip_memmap,
	bench_light_new_round_trip_memory,
	bench_light_from_file_round_trip_memory,
	bench_light_from_file_round_trip_memmap
);
criterion_main!(basic);

fn bench_light_compute_memmap(b: &mut Criterion) {
	use std::env;

	let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
	let light = builder.light(&env::temp_dir(), 486382);

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| light.compute(&HASH, NONCE, u64::max_value())));
}

fn bench_light_compute_memory(b: &mut Criterion) {
	use std::env;

	let builder = NodeCacheBuilder::new(OptimizeFor::Cpu, u64::max_value());
	let light = builder.light(&env::temp_dir(), 486382);

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| light.compute(&HASH, NONCE, u64::max_value())));
}

fn bench_light_new_round_trip_memmap(b: &mut Criterion) {
	use std::env;

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| {
		let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
		let light = builder.light(&env::temp_dir(), 486382);
		light.compute(&HASH, NONCE, u64::max_value());
	}));
}

fn bench_light_new_round_trip_memory(b: &mut Criterion) {
	use std::env;

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| {
		let builder = NodeCacheBuilder::new(OptimizeFor::Cpu, u64::max_value());
		let light = builder.light(&env::temp_dir(), 486382);
		light.compute(&HASH, NONCE, u64::max_value());
	}));
}

fn bench_light_from_file_round_trip_memory(b: &mut Criterion) {
	use std::env;

	let dir = env::temp_dir();
	let height = 486382;
	{
		let builder = NodeCacheBuilder::new(OptimizeFor::Cpu, u64::max_value());
		let mut dummy = builder.light(&dir, height);
		dummy.to_file().unwrap();
	}

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| {
		let builder = NodeCacheBuilder::new(OptimizeFor::Cpu, u64::max_value());
		let light = builder.light_from_file(&dir, 486382).unwrap();
		light.compute(&HASH, NONCE, u64::max_value());
	}));
}

fn bench_light_from_file_round_trip_memmap(b: &mut Criterion) {
	use std::env;

	let dir = env::temp_dir();
	let height = 486382;

	{
		let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
		let mut dummy = builder.light(&dir, height);
		dummy.to_file().unwrap();
	}

	b.bench_function("bench_light_compute_memmap", move |b| b.iter(|| {
		let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
		let light = builder.light_from_file(&dir, 486382).unwrap();
		light.compute(&HASH, NONCE, u64::max_value());
	}));
}
