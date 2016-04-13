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

use util::numbers::H256;
use chainfilter::BloomIndex;

/// Represents location of block bloom in extras database.
#[derive(Debug, PartialEq)]
pub struct BlocksBloomLocation {
	/// Unique hash of BlocksBloom
	pub hash: H256,
	/// Index within BlocksBloom
	pub index: usize,
}

/// Should be used to localize blocks blooms in extras database.
pub struct BloomIndexer {
	index_size: usize,
	levels: u8,
}

impl BloomIndexer {
	/// Plain constructor.
	pub fn new(index_size: usize, levels: u8) -> Self {
		BloomIndexer {
			index_size: index_size,
			levels: levels,
		}
	}

	/// Calculates bloom's position in database.
	pub fn location(&self, bloom_index: &BloomIndex) -> BlocksBloomLocation {
		use std::{mem, ptr};

		let hash = unsafe {
			let mut hash: H256 = mem::zeroed();
			ptr::copy(&[bloom_index.index / self.index_size] as *const usize as *const u8, hash.as_mut_ptr(), 8);
			hash[8] = bloom_index.level;
			hash.reverse();
			hash
		};

		BlocksBloomLocation {
			hash: hash,
			index: bloom_index.index % self.index_size,
		}
	}

	/// Returns index size.
	pub fn index_size(&self) -> usize {
		self.index_size
	}

	/// Returns number of cache levels.
	pub fn levels(&self) -> u8 {
		self.levels
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::{FixedHash, H256};
	use chainfilter::BloomIndex;
	use blockchain::bloom_indexer::{BlocksBloomLocation, BloomIndexer};

	#[test]
	fn test_bloom_indexer() {
		let bi = BloomIndexer::new(16, 3);

		let index = BloomIndex::new(0, 0);
		assert_eq!(bi.location(&index),
		           BlocksBloomLocation {
			           hash: H256::new(),
			           index: 0,
		           });

		let index = BloomIndex::new(1, 0);
		assert_eq!(bi.location(&index),
		           BlocksBloomLocation {
			           hash: H256::from_str("0000000000000000000000000000000000000000000000010000000000000000").unwrap(),
			           index: 0,
		           });

		let index = BloomIndex::new(0, 299_999);
		assert_eq!(bi.location(&index),
		           BlocksBloomLocation {
			           hash: H256::from_str("000000000000000000000000000000000000000000000000000000000000493d").unwrap(),
			           index: 15,
		           });
	}
}
