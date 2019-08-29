#[macro_use]
extern crate criterion;
extern crate ethash;
extern crate rustc_hex;
extern crate tempdir;
extern crate common_types;

use criterion::Criterion;
use ethash::progpow;

use tempdir::TempDir;
use rustc_hex::FromHex;
use ethash::NodeCacheBuilder;
use ethash::compute::light_compute;
use common_types::engines::OptimizeFor;

fn bench_hashimoto_light(c: &mut Criterion) {
	let builder = NodeCacheBuilder::new(OptimizeFor::Memory, u64::max_value());
	let tempdir = TempDir::new("").unwrap();
	let light = builder.light(&tempdir.path(), 1);
	let h = FromHex::from_hex("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f").unwrap();
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

	let h = FromHex::from_hex("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f").unwrap();
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

	let h = FromHex::from_hex("c9149cc0386e689d789a1c2f3d5d169a61a6218ed30e74414dc736e442ef3d1f").unwrap();
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
