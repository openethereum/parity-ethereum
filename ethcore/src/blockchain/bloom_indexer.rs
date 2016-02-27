use util::hash::H256;
use chainfilter::BloomIndex;
use extras::BlocksBloomLocation;

pub struct BloomIndexer {
	index_size: usize,
	levels: u8,
}

impl BloomIndexer {
	pub fn new(index_size: usize, levels: u8) -> Self {
		BloomIndexer {
			index_size: index_size,
			levels: levels
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
			index: bloom_index.index % self.index_size
		}
	}

	pub fn index_size(&self) -> usize {
		self.index_size
	}

	pub fn levels(&self) -> u8 {
		self.levels
	}
}
