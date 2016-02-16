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
extern crate sha3;
extern crate lru_cache;
#[macro_use]
extern crate log;
mod sizes;
mod compute;

use lru_cache::LruCache;
use compute::Light;
pub use compute::{ETHASH_EPOCH_LENGTH, H256, ProofOfWork, quick_get_difficulty};

use std::sync::{Arc, Mutex};

/// Lighy/Full cache manager
pub struct EthashManager {
	lights: Mutex<LruCache<u64, Arc<Light>>>,
}

impl EthashManager {
	/// Create a new new instance of ethash manager
	pub fn new() -> EthashManager {
		EthashManager { lights: Mutex::new(LruCache::new(2)) }
	}

	/// Calculate the light client data
	/// `block_number` - Block number to check
	/// `light` - The light client handler
	/// `header_hash` - The header hash to pack into the mix
	/// `nonce` - The nonce to pack into the mix
	pub fn compute_light(&self, block_number: u64, header_hash: &H256, nonce: u64) -> ProofOfWork {
		let epoch = block_number / ETHASH_EPOCH_LENGTH;
		let light = {
			let mut lights = self.lights.lock().unwrap();
			match lights.get_mut(&epoch).map(|l| l.clone()) {
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
					lights.insert(epoch, light.clone());
					light
				}
				Some(light) => light,
			}
		};
		light.compute(header_hash, nonce)
	}
}
