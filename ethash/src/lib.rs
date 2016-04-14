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

//! Ethash implementation
//! See https://github.com/ethereum/wiki/wiki/Ethash
extern crate primal;
extern crate sha3;
#[macro_use]
extern crate log;
mod compute;

use std::mem;
use compute::Light;
pub use compute::{ETHASH_EPOCH_LENGTH, H256, ProofOfWork, SeedHashCompute, quick_get_difficulty};

use std::sync::{Arc, Mutex};

struct LightCache {
	recent_epoch: Option<u64>,
	recent: Option<Arc<Light>>,
	prev_epoch: Option<u64>,
	prev: Option<Arc<Light>>,
}

/// Light/Full cache manager.
pub struct EthashManager {
	cache: Mutex<LightCache>,
}

impl EthashManager {
	/// Create a new new instance of ethash manager
	pub fn new() -> EthashManager {
		EthashManager {
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
			let mut lights = self.cache.lock().unwrap();
			let light = match lights.recent_epoch.clone() {
				Some(ref e) if *e == epoch => lights.recent.clone(),
				_ => match lights.prev_epoch.clone() {
					Some(e) if e == epoch => {
						// swap
						let t = lights.prev_epoch;
						lights.prev_epoch = lights.recent_epoch;
						lights.recent_epoch = t;
						let t = lights.prev.clone();
						lights.prev = lights.recent.clone();
						lights.recent = t;
						lights.recent.clone()
					}
					_ => None,
				},
			};
			match light {
				None => {
					let light = match Light::from_file(block_number) {
						Ok(light) => Arc::new(light),
						Err(e) => {
							debug!("Light cache file not found for {}:{}", block_number, e);
							let light = Light::new(block_number);
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
	let ethash = EthashManager::new();
	let hash = [0u8; 32];
	ethash.compute_light(1, &hash, 1);
	ethash.compute_light(50000, &hash, 1);
	assert_eq!(ethash.cache.lock().unwrap().recent_epoch.unwrap(), 1);
	assert_eq!(ethash.cache.lock().unwrap().prev_epoch.unwrap(), 0);
	ethash.compute_light(1, &hash, 1);
	assert_eq!(ethash.cache.lock().unwrap().recent_epoch.unwrap(), 0);
	assert_eq!(ethash.cache.lock().unwrap().prev_epoch.unwrap(), 1);
	ethash.compute_light(70000, &hash, 1);
	assert_eq!(ethash.cache.lock().unwrap().recent_epoch.unwrap(), 2);
	assert_eq!(ethash.cache.lock().unwrap().prev_epoch.unwrap(), 0);
}
