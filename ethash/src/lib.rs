//! Ethash implementation
//! See https://github.com/ethereum/wiki/wiki/Ethash
extern crate tiny_keccak;
mod sizes;
mod compute;

use compute::Light;
pub use compute::{quick_get_difficulty, H256, ProofOfWork, ETHASH_EPOCH_LENGTH};

use std::collections::HashMap;
use std::sync::RwLock;

/// Lighy/Full cache manager
pub struct EthashManager {
	lights: RwLock<HashMap<u64, Light>>,
}

impl EthashManager {
	/// Create a new new instance of ethash manager
	pub fn new() -> EthashManager {
		EthashManager { 
			lights: RwLock::new(HashMap::new())
		}
	}

	/// Calculate the light client data
	/// `block_number` - Block number to check
	/// `light` - The light client handler
	/// `header_hash` - The header hash to pack into the mix
	/// `nonce` - The nonce to pack into the mix
	pub fn compute_light(&self, block_number: u64, header_hash: &H256, nonce: u64) -> ProofOfWork {
		let epoch = block_number / ETHASH_EPOCH_LENGTH;
		if !self.lights.read().unwrap().contains_key(&epoch) {
			let mut lights = self.lights.write().unwrap(); // obtain write lock
			if !lights.contains_key(&epoch) {
				let light = Light::new(block_number);
				lights.insert(epoch, light);
			}
		}
		self.lights.read().unwrap().get(&epoch).unwrap().compute(header_hash, nonce)
	}
}
