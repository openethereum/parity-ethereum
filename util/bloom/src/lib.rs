// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{cmp, mem, f64};
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use siphasher::sip::SipHasher;

/// BitVec structure with journalling
/// Every time any of the blocks is getting set it's index is tracked
/// and can be then drained by `drain` method
struct BitVecJournal {
    elems: Vec<u64>,
    journal: HashSet<usize>,
}

impl BitVecJournal {
	pub fn new(size: usize) -> BitVecJournal {
		let extra = if size % 64 > 0  { 1 } else { 0 };
		BitVecJournal {
			elems: vec![0u64; size / 64 + extra],
			journal: HashSet::new(),
		}
	}

	pub fn from_parts(parts: &[u64]) -> BitVecJournal {
		BitVecJournal {
			elems: parts.to_vec(),
			journal: HashSet::new(),
		}
	}

	pub fn set(&mut self, index: usize) {
		let e_index = index / 64;
		let bit_index = index % 64;
		let val = self.elems.get_mut(e_index).unwrap();
		*val |= 1u64 << bit_index;
		self.journal.insert(e_index);
	}

	pub fn get(&self, index: usize) -> bool {
		let e_index = index / 64;
		let bit_index = index % 64;
		self.elems[e_index] & (1 << bit_index) != 0
	}

	pub fn drain(&mut self) -> Vec<(usize, u64)> {
		let journal = mem::replace(&mut self.journal, HashSet::new()).into_iter();
		journal.map(|idx| (idx, self.elems[idx])).collect::<Vec<(usize, u64)>>()
	}

	pub fn saturation(&self) -> f64 {
		self.elems.iter().fold(0u64, |acc, e| acc + e.count_ones() as u64) as f64 / (self.elems.len() * 64) as f64
	}
}

/// Bloom filter structure
pub struct Bloom {
	bitmap: BitVecJournal,
	bitmap_bits: u64,
	k_num: u32,
}

impl Bloom {
	/// Create a new bloom filter structure.
	/// bitmap_size is the size in bytes (not bits) that will be allocated in memory
	/// items_count is an estimation of the maximum number of items to store.
	pub fn new(bitmap_size: usize, items_count: usize) -> Bloom {
		assert!(bitmap_size > 0 && items_count > 0);
		let bitmap_bits = (bitmap_size as u64) * 8u64;
		let k_num = Bloom::optimal_k_num(bitmap_bits, items_count);
		let bitmap = BitVecJournal::new(bitmap_bits as usize);
		Bloom {
			bitmap: bitmap,
			bitmap_bits: bitmap_bits,
			k_num: k_num,
		}
	}

	/// Initializes bloom filter from saved state
	pub fn from_parts(parts: &[u64], k_num: u32) -> Bloom {
		let bitmap_size = parts.len() * 8;
		let bitmap_bits = (bitmap_size as u64) * 8u64;
		let bitmap = BitVecJournal::from_parts(parts);
		Bloom {
			bitmap: bitmap,
			bitmap_bits: bitmap_bits,
			k_num: k_num,
		}
	}

	/// Create a new bloom filter structure.
	/// items_count is an estimation of the maximum number of items to store.
	/// fp_p is the wanted rate of false positives, in ]0.0, 1.0[
	pub fn new_for_fp_rate(items_count: usize, fp_p: f64) -> Bloom {
		let bitmap_size = Bloom::compute_bitmap_size(items_count, fp_p);
		Bloom::new(bitmap_size, items_count)
	}

	/// Compute a recommended bitmap size for items_count items
	/// and a fp_p rate of false positives.
	/// fp_p obviously has to be within the ]0.0, 1.0[ range.
	pub fn compute_bitmap_size(items_count: usize, fp_p: f64) -> usize {
		assert!(items_count > 0);
		assert!(fp_p > 0.0 && fp_p < 1.0);
		let log2 = f64::consts::LN_2;
		let log2_2 = log2 * log2;
		((items_count as f64) * f64::ln(fp_p) / (-8.0 * log2_2)).ceil() as usize
	}

	/// Records the presence of an item.
	pub fn set<T>(&mut self, item: T)
		where T: Hash
	{
		let base_hash = Bloom::sip_hash(&item);
		for k_i in 0..self.k_num {
			let bit_offset = (Bloom::bloom_hash(base_hash, k_i) % self.bitmap_bits) as usize;
			self.bitmap.set(bit_offset);
		}
	}

	/// Check if an item is present in the set.
	/// There can be false positives, but no false negatives.
	pub fn check<T>(&self, item: T) -> bool
		where T: Hash
	{
		let base_hash = Bloom::sip_hash(&item);
		for k_i in 0..self.k_num {
			let bit_offset = (Bloom::bloom_hash(base_hash, k_i) % self.bitmap_bits) as usize;
			if !self.bitmap.get(bit_offset) {
				return false;
			}
		}
		true
	}

	/// Return the number of bits in the filter
	pub fn number_of_bits(&self) -> u64 {
		self.bitmap_bits
	}

	/// Return the number of hash functions used for `check` and `set`
	pub fn number_of_hash_functions(&self) -> u32 {
		self.k_num
	}

	fn optimal_k_num(bitmap_bits: u64, items_count: usize) -> u32 {
		let m = bitmap_bits as f64;
		let n = items_count as f64;
		let k_num = (m / n * f64::ln(2.0f64)).ceil() as u32;
		cmp::max(k_num, 1)
	}

	fn sip_hash<T>(item: &T) -> u64
		where T: Hash
	{
		let mut sip = SipHasher::new();
		item.hash(&mut sip);
		let hash = sip.finish();
		hash
	}

	fn bloom_hash(base_hash: u64, k_i: u32) -> u64 {
		if k_i < 2 {
			base_hash
		} else {
			base_hash.wrapping_add((k_i as u64).wrapping_mul(base_hash) % 0xffffffffffffffc5)
		}
	}

	/// Drains the bloom journal returning the updated bloom part
	pub fn drain_journal(&mut self) -> BloomJournal {
		BloomJournal {
			entries: self.bitmap.drain(),
			hash_functions: self.k_num,
		}
	}

	/// Returns the ratio of set bits in the bloom filter to the total bits
	pub fn saturation(&self) -> f64 {
		self.bitmap.saturation()
	}
}

/// Bloom journal
/// Returns the tuple of (bloom part index, bloom part value) where each one is representing
/// an index of bloom parts that was updated since the last drain
pub struct BloomJournal {
    pub hash_functions: u32,
    pub entries: Vec<(usize, u64)>,
}

#[cfg(test)]
mod tests {
	use super::Bloom;
	use std::collections::HashSet;

	#[test]
	fn get_set() {
		let mut bloom = Bloom::new(10, 80);
		let key = vec![115u8, 99];
		assert!(!bloom.check(&key));
		bloom.set(&key);
		assert!(bloom.check(&key));
	}

	#[test]
	fn journalling() {
		let initial = vec![0u64; 8];
		let mut bloom = Bloom::from_parts(&initial, 3);
		bloom.set(&vec![5u8, 4]);
		let drain = bloom.drain_journal();

		assert_eq!(2, drain.entries.len())
	}

	#[test]
	fn saturation() {
		let initial = vec![0u64; 8];
		let mut bloom = Bloom::from_parts(&initial, 3);
		bloom.set(&vec![5u8, 4]);

		let full = bloom.saturation();
		// 2/8/64 = 0.00390625
		assert!(full >= 0.0039f64 && full <= 0.004f64);
	}

	#[test]
	fn hash_backward_compatibility_for_new() {
		let ss = vec!["you", "should", "not", "break", "hash", "backward", "compatibility"];
		let mut bloom = Bloom::new(16, 8);
		for s in ss.iter() {
			bloom.set(&s);
		}

		let drained_elems: HashSet<u64> = bloom.drain_journal().entries.into_iter().map(|t| t.1).collect();
		let expected: HashSet<u64> = [2094615114573771027u64, 244675582389208413u64].iter().cloned().collect();
		assert_eq!(drained_elems, expected);
		assert_eq!(bloom.k_num, 12);
	}

	#[test]
	fn hash_backward_compatibility_for_from_parts() {
		let stored_state = vec![2094615114573771027u64, 244675582389208413u64];
		let k_num = 12;
		let bloom = Bloom::from_parts(&stored_state, k_num);

		let ss = vec!["you", "should", "not", "break", "hash", "backward", "compatibility"];
		let tt = vec!["this", "doesnot", "exist"];
		for s in ss.iter() {
			assert!(bloom.check(&s));
		}
		for s in tt.iter() {
			assert!(!bloom.check(&s));
		}

	}
}
