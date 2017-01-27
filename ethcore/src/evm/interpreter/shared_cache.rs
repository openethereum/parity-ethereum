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

use std::sync::Arc;
use util::{H256, HeapSizeOf, Mutex};
use util::sha3::*;
use util::cache::MemoryLruCache;
use bit_set::BitSet;
use super::super::instructions;

const DEFAULT_CACHE_SIZE: usize = 4 * 1024 * 1024;

// stub for a HeapSizeOf implementation.
struct Bits(Arc<BitSet>);

impl HeapSizeOf for Bits {
	fn heap_size_of_children(&self) -> usize {
		// dealing in bits here
		self.0.capacity() * 8
	}
}

/// Global cache for EVM interpreter
pub struct SharedCache {
	jump_destinations: Mutex<MemoryLruCache<H256, Bits>>,
}

impl SharedCache {
	/// Create a jump destinations cache with a maximum size in bytes
	/// to cache.
	pub fn new(max_size: usize) -> Self {
		SharedCache {
			jump_destinations: Mutex::new(MemoryLruCache::new(max_size)),
		}
	}

	/// Get jump destinations bitmap for a contract.
	pub fn jump_destinations(&self, code_hash: &H256, code: &[u8]) -> Arc<BitSet> {
		if code_hash == &SHA3_EMPTY {
			return Self::find_jump_destinations(code);
		}

		if let Some(d) = self.jump_destinations.lock().get_mut(code_hash) {
			return d.0.clone();
		}

		let d = Self::find_jump_destinations(code);
		self.jump_destinations.lock().insert(code_hash.clone(), Bits(d.clone()));

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
