// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use shared;
use keccak::{keccak_256, H256};

use std::cell::Cell;

pub struct SeedHashCompute {
	prev_epoch: Cell<u64>,
	prev_seedhash: Cell<H256>,
}

impl SeedHashCompute {
	#[inline]
	pub fn new() -> SeedHashCompute {
		SeedHashCompute {
			prev_epoch: Cell::new(0),
			prev_seedhash: Cell::new([0u8; 32]),
		}
	}

	#[inline]
	fn reset_cache(&self) {
		self.prev_epoch.set(0);
		self.prev_seedhash.set([0u8; 32]);
	}

	#[inline]
	pub fn hash_block_number(&self, block_number: u64) -> H256 {
		self.hash_epoch(shared::epoch(block_number))
	}

	#[inline]
	pub fn hash_epoch(&self, epoch: u64) -> H256 {
		if epoch < self.prev_epoch.get() {
			// can't build on previous hash if requesting an older block
			self.reset_cache();
		}
		if epoch > self.prev_epoch.get() {
			let seed_hash = SeedHashCompute::resume_compute_seedhash(
				self.prev_seedhash.get(),
				self.prev_epoch.get(),
				epoch,
			);
			self.prev_seedhash.set(seed_hash);
			self.prev_epoch.set(epoch);
		}
		self.prev_seedhash.get()
	}

	#[inline]
	pub fn resume_compute_seedhash(mut hash: H256, start_epoch: u64, end_epoch: u64) -> H256 {
		for _ in start_epoch..end_epoch {
			keccak_256::inplace(&mut hash);
		}
		hash
	}
}

#[cfg(test)]
mod tests {
	use super::SeedHashCompute;

	#[test]
	fn test_seed_compute_once() {
		let seed_compute = SeedHashCompute::new();
		let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
		assert_eq!(seed_compute.hash_block_number(486382), hash);
	}

	#[test]
	fn test_seed_compute_zero() {
		let seed_compute = SeedHashCompute::new();
		assert_eq!(seed_compute.hash_block_number(0), [0u8; 32]);
	}

	#[test]
	fn test_seed_compute_after_older() {
		let seed_compute = SeedHashCompute::new();
		// calculating an older value first shouldn't affect the result
		let _ = seed_compute.hash_block_number(50000);
		let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
		assert_eq!(seed_compute.hash_block_number(486382), hash);
	}

	#[test]
	fn test_seed_compute_after_newer() {
		let seed_compute = SeedHashCompute::new();
		// calculating an newer value first shouldn't affect the result
		let _ = seed_compute.hash_block_number(972764);
		let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
		assert_eq!(seed_compute.hash_block_number(486382), hash);
	}

}
