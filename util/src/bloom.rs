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

//! Bloom operations.

use std::mem;
use std::ops::DerefMut;
use {H64, H160, H256, H512, H520, H2048, FixedHash};

/// Returns log2.
pub fn log2(x: usize) -> u32 {
	if x <= 1 {
		return 0;
	}

	let n = x.leading_zeros();
	mem::size_of::<usize>() as u32 * 8 - n
}

/// Bloom operations.
pub trait Bloomable: Sized + Default + DerefMut<Target = [u8]> {
	/// When interpreting self as a bloom output, augment (bit-wise OR) with the a bloomed version of `b`.
	fn shift_bloomed<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: Bloomable;

	/// Same as `shift_bloomed` except that `self` is consumed and a new value returned.
	fn with_bloomed<T>(mut self, b: &T) -> Self where T: Bloomable {
		self.shift_bloomed(b);
		self
	}

	/// Construct new instance equal to the bloomed value of `b`.
	fn from_bloomed<T>(b: &T) -> Self where T: Bloomable;

	/// Bloom the current value using the bloom parameter `m`.
	fn bloom_part<T>(&self, m: usize) -> T where T: Bloomable;

	/// Check to see whether this hash, interpreted as a bloom, contains the value `b` when bloomed.
	fn contains_bloomed<T>(&self, b: &T) -> bool where T: Bloomable;
}

macro_rules! impl_bloomable_for_hash {
	($name: ident, $size: expr) => {
		impl Bloomable for $name {
			fn shift_bloomed<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: Bloomable {
				let bp: Self = b.bloom_part($size);
				let new_self = &bp | self;

				self.0 = new_self.0;
				self
			}

			fn bloom_part<T>(&self, m: usize) -> T where T: Bloomable + Default {
				// numbers of bits
				// TODO: move it to some constant
				let p = 3;

				let bloom_bits = m * 8;
				let mask = bloom_bits - 1;
				let bloom_bytes = (log2(bloom_bits) + 7) / 8;

				// must be a power of 2
				assert_eq!(m & (m - 1), 0);
				// out of range
				assert!(p * bloom_bytes <= $size);

				// return type
				let mut ret = T::default();

				// 'ptr' to out slice
				let mut ptr = 0;

				// set p number of bits,
				// p is equal 3 according to yellowpaper
				for _ in 0..p {
					let mut index = 0 as usize;
					for _ in 0..bloom_bytes {
						index = (index << 8) | self.0[ptr] as usize;
						ptr += 1;
					}
					index &= mask;
					ret[m - 1 - index / 8] |= 1 << (index % 8);
				}

				ret
			}

			fn contains_bloomed<T>(&self, b: &T) -> bool where T: Bloomable {
				let bp: Self = b.bloom_part($size);
				self.contains(&bp)
			}

			fn from_bloomed<T>(b: &T) -> Self where T: Bloomable {
				b.bloom_part($size)
			}
		}
	}
}

impl_bloomable_for_hash!(H64, 8);
impl_bloomable_for_hash!(H160, 20);
impl_bloomable_for_hash!(H256, 32);
impl_bloomable_for_hash!(H512, 64);
impl_bloomable_for_hash!(H520, 65);
impl_bloomable_for_hash!(H2048, 256);

#[cfg(test)]
mod tests {
	use {H160, H256, H2048};
	use sha3::Hashable;
	use super::Bloomable;

	#[test]
	fn shift_bloomed() {
		let bloom: H2048 = "00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020000000000000000000000000000000000000000000008000000001000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into();
		let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		let topic: H256 = "02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc".into();

		let mut my_bloom = H2048::default();
		assert!(!my_bloom.contains_bloomed(&address.sha3()));
		assert!(!my_bloom.contains_bloomed(&topic.sha3()));

		my_bloom.shift_bloomed(&address.sha3());
		assert!(my_bloom.contains_bloomed(&address.sha3()));
		assert!(!my_bloom.contains_bloomed(&topic.sha3()));

		my_bloom.shift_bloomed(&topic.sha3());
		assert_eq!(my_bloom, bloom);
		assert!(my_bloom.contains_bloomed(&address.sha3()));
		assert!(my_bloom.contains_bloomed(&topic.sha3()));
	}
}

