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

#![cfg_attr(feature = "benches", feature(test))]

extern crate primal;
extern crate parking_lot;
extern crate either;
extern crate memmap;

#[macro_use]
extern crate crunchy;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate tempdir;

mod compute;
mod seed_compute;
mod cache;
mod keccak;
mod shared;

pub use cache::{NodeCacheBuilder, OptimizeFor};
pub use compute::{ProofOfWork, quick_get_difficulty, slow_hash_block_number};
use compute::Light;
use keccak::H256;
use parking_lot::Mutex;
pub use seed_compute::SeedHashCompute;
pub use shared::ETHASH_EPOCH_LENGTH;
use std::mem;
use std::path::{Path, PathBuf};

use std::sync::Arc;

struct LightCache {
	recent_epoch: Option<u64>,
	recent: Option<Arc<Light>>,
	prev_epoch: Option<u64>,
	prev: Option<Arc<Light>>,
}

/// Light/Full cache manager.
pub struct EthashManager {
	nodecache_builder: NodeCacheBuilder,
	cache: Mutex<LightCache>,
	cache_dir: PathBuf,
}

impl EthashManager {
	/// Create a new new instance of ethash manager
	pub fn new<T: Into<Option<OptimizeFor>>>(cache_dir: &Path, optimize_for: T) -> EthashManager {
		EthashManager {
			cache_dir: cache_dir.to_path_buf(),
			nodecache_builder: NodeCacheBuilder::new(optimize_for.into().unwrap_or_default()),
			cache: Mutex::new(LightCache {
				recent_epoch: None,
				recent: None,
				prev_epoch: None,
				prev: None,
			}),
		}
	}

	/// Calculate the light client data
	/// `block_number` - Block number to check
	/// `light` - The light client handler
	/// `header_hash` - The header hash to pack into the mix
	/// `nonce` - The nonce to pack into the mix
	pub fn compute_light(&self, block_number: u64, header_hash: &H256, nonce: u64) -> ProofOfWork {
		let epoch = block_number / ETHASH_EPOCH_LENGTH;
		let light = {
			let mut lights = self.cache.lock();
			let light = match lights.recent_epoch.clone() {
				Some(ref e) if *e == epoch => lights.recent.clone(),
				_ => match lights.prev_epoch.clone() {
					Some(e) if e == epoch => {
						// don't swap if recent is newer.
						if lights.recent_epoch > lights.prev_epoch {
							None
						} else {
							// swap
							let t = lights.prev_epoch;
							lights.prev_epoch = lights.recent_epoch;
							lights.recent_epoch = t;
							let t = lights.prev.clone();
							lights.prev = lights.recent.clone();
							lights.recent = t;
							lights.recent.clone()
						}
					}
					_ => None,
				},
			};
			match light {
				None => {
					let light = match self.nodecache_builder.light_from_file(
						&self.cache_dir,
						block_number,
					) {
						Ok(light) => Arc::new(light),
						Err(e) => {
							debug!("Light cache file not found for {}:{}", block_number, e);
							let mut light = self.nodecache_builder.light(
								&self.cache_dir,
								block_number,
							);
							if let Err(e) = light.to_file() {
								warn!("Light cache file write error: {}", e);
							}
							Arc::new(light)
						}
					};
					lights.prev_epoch = mem::replace(&mut lights.recent_epoch, Some(epoch));
					lights.prev = mem::replace(&mut lights.recent, Some(light.clone()));
					light
				}
				Some(light) => light,
			}
		};
		light.compute(header_hash, nonce)
	}
}

#[test]
fn test_lru() {
	use tempdir::TempDir;

	let tempdir = TempDir::new("").unwrap();
	let ethash = EthashManager::new(tempdir.path(), None);
	let hash = [0u8; 32];
	ethash.compute_light(1, &hash, 1);
	ethash.compute_light(50000, &hash, 1);
	assert_eq!(ethash.cache.lock().recent_epoch.unwrap(), 1);
	assert_eq!(ethash.cache.lock().prev_epoch.unwrap(), 0);
	ethash.compute_light(1, &hash, 1);
	assert_eq!(ethash.cache.lock().recent_epoch.unwrap(), 0);
	assert_eq!(ethash.cache.lock().prev_epoch.unwrap(), 1);
	ethash.compute_light(70000, &hash, 1);
	assert_eq!(ethash.cache.lock().recent_epoch.unwrap(), 2);
	assert_eq!(ethash.cache.lock().prev_epoch.unwrap(), 0);
}

#[cfg(feature = "benches")]
mod benchmarks {
	extern crate test;

	use self::test::Bencher;
	use cache::{NodeCacheBuilder, OptimizeFor};
	use compute::{Light, light_compute};

	const HASH: [u8; 32] = [0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe,
	                        0xe4, 0x0a, 0xb3, 0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f,
	                        0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94, 0x05, 0x52, 0x7d, 0x72];
	const NONCE: u64 = 0xd7b3ac70a301a249;

	#[bench]
	fn bench_light_compute_memmap(b: &mut Bencher) {
		use std::env;

		let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
		let light = builder.light(&env::temp_dir(), 486382);

		b.iter(|| light_compute(&light, &HASH, NONCE));
	}

	#[bench]
	fn bench_light_compute_memory(b: &mut Bencher) {
		use std::env;

		let builder = NodeCacheBuilder::new(OptimizeFor::Cpu);
		let light = builder.light(&env::temp_dir(), 486382);

		b.iter(|| light_compute(&light, &HASH, NONCE));
	}

	#[bench]
	#[ignore]
	fn bench_light_new_round_trip_memmap(b: &mut Bencher) {
		use std::env;

		b.iter(|| {
			let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
			let light = builder.light(&env::temp_dir(), 486382);
			light_compute(&light, &HASH, NONCE);
		});
	}

	#[bench]
	#[ignore]
	fn bench_light_new_round_trip_memory(b: &mut Bencher) {
		use std::env;

		b.iter(|| {
			let builder = NodeCacheBuilder::new(OptimizeFor::Cpu);
			let light = builder.light(&env::temp_dir(), 486382);
			light_compute(&light, &HASH, NONCE);
		});
	}

	#[bench]
	fn bench_light_from_file_round_trip_memory(b: &mut Bencher) {
		use std::env;

		let dir = env::temp_dir();
		let height = 486382;
		{
			let builder = NodeCacheBuilder::new(OptimizeFor::Cpu);
			let mut dummy = builder.light(&dir, height);
			dummy.to_file().unwrap();
		}

		b.iter(|| {
			let builder = NodeCacheBuilder::new(OptimizeFor::Cpu);
			let light = builder.light_from_file(&dir, 486382).unwrap();
			light_compute(&light, &HASH, NONCE);
		});
	}

	#[bench]
	fn bench_light_from_file_round_trip_memmap(b: &mut Bencher) {
		use std::env;

		let dir = env::temp_dir();
		let height = 486382;

		{
			let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
			let mut dummy = builder.light(&dir, height);
			dummy.to_file().unwrap();
		}

		b.iter(|| {
			let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
			let light = builder.light_from_file(&dir, 486382).unwrap();
			light_compute(&light, &HASH, NONCE);
		});
	}
}
