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

use std::sync::Arc;
use lru_cache::LruCache;
use util::{H256, Mutex};
use util::sha3::*;
use bit_set::BitSet;
use super::super::instructions;

const INITIAL_CAPACITY: usize = 32;
const DEFAULT_CACHE_SIZE: usize = 4 * 1024 * 1024;

/// Global cache for EVM interpreter
pub struct SharedCache {
	jump_destinations: Mutex<LruCache<H256, Arc<BitSet>>>,
	max_size: usize,
	cur_size: Mutex<usize>,
}

impl SharedCache {
	/// Create a jump destinations cache with a maximum size in bytes
	/// to cache.
	pub fn new(max_size: usize) -> Self {
		SharedCache {
			jump_destinations: Mutex::new(LruCache::new(INITIAL_CAPACITY)),
			max_size: max_size * 8, // dealing with bits here.
			cur_size: Mutex::new(0),
		}
	}

	/// Get jump destinations bitmap for a contract.
	pub fn jump_destinations(&self, code_hash: &H256, code: &[u8]) -> Arc<BitSet> {
		if code_hash == &SHA3_EMPTY {
			return Self::find_jump_destinations(code);
		}

		if let Some(d) = self.jump_destinations.lock().get_mut(code_hash) {
			return d.clone();
		}

		let d = Self::find_jump_destinations(code);

		{
			let mut cur_size = self.cur_size.lock();
			*cur_size += d.capacity();

			let mut jump_dests = self.jump_destinations.lock();
			let cap = jump_dests.capacity();

			// grow the cache as necessary; it operates on amount of items
			// but we're working based on memory usage.
			if jump_dests.len() == cap && *cur_size < self.max_size {
				jump_dests.set_capacity(cap * 2);
			}

			// account for any element displaced from the cache.
			if let Some(lru) = jump_dests.insert(code_hash.clone(), d.clone()) {
				*cur_size -= lru.capacity();
			}

			// remove elements until we are below the memory target.
			while *cur_size > self.max_size {
				match jump_dests.remove_lru() {
					Some((_, v)) => *cur_size -= v.capacity(),
					_ => break,
				}
			}
		}

		d
	}

	fn find_jump_destinations(code: &[u8]) -> Arc<BitSet> {
		let mut jump_dests = BitSet::with_capacity(code.len());
		let mut position = 0;

		while position < code.len() {
			let instruction = code[position];

			if instruction == instructions::JUMPDEST {
				jump_dests.insert(position);
			} else if instructions::is_push(instruction) {
				position += instructions::get_push_bytes(instruction);
			}
			position += 1;
		}

		jump_dests.shrink_to_fit();
		Arc::new(jump_dests)
	}
}

impl Default for SharedCache {
	fn default() -> Self {
		SharedCache::new(DEFAULT_CACHE_SIZE)
	}
}


#[test]
fn test_find_jump_destinations() {
	use util::FromHex;
	// given
	let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5b01600055".from_hex().unwrap();

	// when
	let valid_jump_destinations = SharedCache::find_jump_destinations(&code);

	// then
	assert!(valid_jump_destinations.contains(66));
}
