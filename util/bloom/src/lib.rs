// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{cmp, mem, f64};
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use std::f64::consts::LN_2;
use siphasher::sip::SipHasher;

/// BitVec structure with journalling
/// Every time any of the blocks is getting set it's index is tracked
/// and can be then drained by `drain` method
struct BitVecJournal {
    elems: Vec<u64>,
    journal: HashSet<usize>,
}

impl BitVecJournal {
	fn new(size: usize) -> BitVecJournal {
		let extra = if size % 64 > 0  { 1 } else { 0 };
		BitVecJournal {
			elems: vec![0u64; size / 64 + extra],
			journal: HashSet::new(),
		}
	}

	fn from_parts(parts: Vec<u64>) -> BitVecJournal {
		BitVecJournal {
			elems: parts,
			journal: HashSet::new(),
		}
	}

	fn set(&mut self, index: usize) {
		let e_index = index / 64;
		let bit_index = index % 64;
		let val = self.elems.get_mut(e_index).unwrap();
		*val |= 1u64 << bit_index;
		self.journal.insert(e_index);
	}

	fn get(&self, index: usize) -> bool {
		let e_index = index / 64;
		let bit_index = index % 64;
		self.elems[e_index] & (1 << bit_index) != 0
	}

	fn drain(&mut self) -> Vec<(usize, u64)> {
		let journal = mem::replace(&mut self.journal, HashSet::new()).into_iter();
		journal.map(|idx| (idx, self.elems[idx])).collect::<Vec<(usize, u64)>>()
	}

	fn saturation(&self) -> f64 {
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
	/// `bitmap_size` is the size in bytes (not bits) that will be allocated in memory
	/// `items_count` is an estimation of the maximum number of items to store.
	fn new(bitmap_size: u64, item_count: u64) -> Bloom {
		assert!(bitmap_size > 0 && item_count > 0);
		let bitmap_bits = bitmap_size * 8;
		let k_num = Bloom::optimal_k_num(bitmap_bits, item_count);
		let bitmap = BitVecJournal::new(bitmap_bits as usize);
		Bloom {
			bitmap,
			bitmap_bits,
			k_num,
		}
	}

	/// The legacy accounts bloom filter used non-optimal parameters that cannot
	/// be calculated with the facilities in this crate, hence this method that
	/// allows the instantiation of a non-optimal filter so that older databases
	/// can continue to work. DO NOT USE FOR OTHER PURPOSES.
	pub fn from_parts_legacy(parts: Vec<u64>, k_num: u32) -> Bloom {
		let bitmap_bits = parts.len() as u64 * 64 ;
		let bitmap = BitVecJournal::from_parts(parts);
		Bloom { bitmap, bitmap_bits, k_num }
	}

	/// Initializes a bloom filter from saved state
	pub fn from_parts(parts: Vec<u64>, item_count: u64) -> Bloom {
		let bitmap_size = parts.len() * 8;
		let bitmap_bits = (bitmap_size as u64) * 8u64;
		let bitmap = BitVecJournal::from_parts(parts);
		let k_num = Self::optimal_k_num(bitmap_bits, item_count);
		Bloom { bitmap, bitmap_bits, k_num }
	}

	/// Create a new bloom filter structure.
	/// `item_count` is an estimation of the maximum number of items to store.
	/// `fp_p` is the desired false positives rate, in ]0.0, 1.0[
	pub fn new_for_fp_rate(item_count: u64, fp_p: f64) -> Bloom {
		let bitmap_size = Bloom::compute_bitmap_size(item_count, fp_p);
		Bloom::new(bitmap_size, item_count)
	}

	/// Compute a recommended Bloom bitmap size in bytes for `items_count` items
	/// and a fp_p rate of false positives.
	/// `fp_p` obviously has to be within the ]0.0, 1.0[ range.
	pub fn compute_bitmap_size(item_count: u64, fp_p: f64) -> u64 {
		assert!(item_count > 0);
		assert!(fp_p > 0.0 && fp_p < 1.0);
		let bitmap_size = ((item_count as f64) * f64::ln(fp_p) / (-8.0 * LN_2 * LN_2)).ceil() as u64;
		// Round up to nearest multiple of 8 because we need to use this to index u64s
		((bitmap_size + 7) / 8) * 8
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

	/// The optimal number of hash functions for a given bitmap size and item
	/// count is calculated as `bits-per-item * ln(2)`.
	fn optimal_k_num(bitmap_bits: u64, item_count: u64) -> u32 {
		let m = bitmap_bits as f64;
		let n = item_count as f64;
		let k_num = (m / n * LN_2).ceil() as u32;
		cmp::max(k_num, 1)
	}

	fn sip_hash<T>(item: &T) -> u64
		where T: Hash
	{
		let mut sip = SipHasher::new();
		item.hash(&mut sip);
		sip.finish()
	}

	fn bloom_hash(base_hash: u64, k_i: u32) -> u64 {
		if k_i < 2 {
			base_hash
		} else {
			base_hash.wrapping_add((k_i as u64).wrapping_mul(base_hash) % 0xffff_ffff_ffff_ffc5)
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
		// Set up bloom a with 512 bits and 120 estimated items stored; we'll get a `k` of 3…
		let initial = vec![0u64; 8];
		let mut bloom = Bloom::from_parts(initial, 120);
		// …which will cause this particular key…
		bloom.set(&vec![5u8, 4]);
		let drain = bloom.drain_journal();
		// …to set one bit in two different entries.
		assert_eq!(2, drain.entries.len())
	}

	#[test]
	fn saturation() {
		// Set up bloom a with 512 bits and 120 estimated items stored; we'll get a `k` of 3…
		let initial = vec![0u64; 8];
		let mut bloom = Bloom::from_parts(initial, 120);
		// …which will cause this particular key to set one bit in two different entries.
		bloom.set(&vec![5u8, 4]);

		let full = bloom.saturation();
		// 2 bits touched, over 8 entries where each entry has 64 bits, so 2/8/64 = 0.00390625
		assert!(full >= 0.0039f64 && full <= 0.004f64);
	}

	#[test]
	fn test_compute_bitmap_size() {
		use std::f64::consts::LN_2;
		let bitmap_size = Bloom::compute_bitmap_size(10_000_000, 0.01);
		// ~12Mbytes
		let expected_size_in_bits = (-(10_000_000 as f64 * f64::ln(0.01)) / ( LN_2 * LN_2)).ceil() as u64;
		// rounded up to nearest multiple of 8
		let expected_size_in_bytes = (((expected_size_in_bits / 8) + 7) / 8) * 8;
		assert_eq!(bitmap_size, expected_size_in_bytes);
		let bloom = Bloom::new( bitmap_size,10_000_000);
		assert_eq!(bloom.number_of_hash_functions(), 7);
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
		let bloom = Bloom::from_parts(stored_state, k_num);

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
