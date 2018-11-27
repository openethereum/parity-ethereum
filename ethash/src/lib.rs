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

extern crate either;
extern crate ethereum_types;
extern crate memmap;
extern crate parking_lot;
extern crate primal;

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
use ethereum_types::{U256, U512};
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

/// Convert an Ethash boundary to its original difficulty. Basically just `f(x) = 2^256 / x`.
pub fn boundary_to_difficulty(boundary: &ethereum_types::H256) -> U256 {
	difficulty_to_boundary_aux(&**boundary)
}

/// Convert an Ethash difficulty to the target boundary. Basically just `f(x) = 2^256 / x`.
pub fn difficulty_to_boundary(difficulty: &U256) -> ethereum_types::H256 {
	difficulty_to_boundary_aux(difficulty).into()
}

fn difficulty_to_boundary_aux<T: Into<U512>>(difficulty: T) -> ethereum_types::U256 {
	let difficulty = difficulty.into();

	assert!(!difficulty.is_zero());

	if difficulty == U512::one() {
		U256::max_value()
	} else {
		// difficulty > 1, so result should never overflow 256 bits
		U256::from((U512::one() << 256) / difficulty)
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

#[test]
fn test_difficulty_to_boundary() {
	use ethereum_types::H256;
	use std::str::FromStr;

	assert_eq!(difficulty_to_boundary(&U256::from(1)), H256::from(U256::max_value()));
	assert_eq!(difficulty_to_boundary(&U256::from(2)), H256::from_str("8000000000000000000000000000000000000000000000000000000000000000").unwrap());
	assert_eq!(difficulty_to_boundary(&U256::from(4)), H256::from_str("4000000000000000000000000000000000000000000000000000000000000000").unwrap());
	assert_eq!(difficulty_to_boundary(&U256::from(32)), H256::from_str("0800000000000000000000000000000000000000000000000000000000000000").unwrap());
}

#[test]
fn test_difficulty_to_boundary_regression() {
	use ethereum_types::H256;

	// the last bit was originally being truncated when performing the conversion
	// https://github.com/paritytech/parity-ethereum/issues/8397
	for difficulty in 1..9 {
		assert_eq!(U256::from(difficulty), boundary_to_difficulty(&difficulty_to_boundary(&difficulty.into())));
		assert_eq!(H256::from(difficulty), difficulty_to_boundary(&boundary_to_difficulty(&difficulty.into())));
		assert_eq!(U256::from(difficulty), boundary_to_difficulty(&boundary_to_difficulty(&difficulty.into()).into()));
		assert_eq!(H256::from(difficulty), difficulty_to_boundary(&difficulty_to_boundary(&difficulty.into()).into()));
	}
}

#[test]
#[should_panic]
fn test_difficulty_to_boundary_panics_on_zero() {
	difficulty_to_boundary(&U256::from(0));
}

#[test]
#[should_panic]
fn test_boundary_to_difficulty_panics_on_zero() {
	boundary_to_difficulty(&ethereum_types::H256::from(0));
}
