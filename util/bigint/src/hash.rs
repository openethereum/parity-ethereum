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

//! General hash types, a fixed-size raw-data type used as the output of hash functions.

use std::{ops, fmt, cmp};
use std::cmp::{min, Ordering};
use std::ops::{Deref, DerefMut, BitXor, BitAnd, BitOr, IndexMut, Index};
use std::hash::{Hash, Hasher, BuildHasherDefault};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use rand::Rng;
use rand::os::OsRng;
use rustc_serialize::hex::{FromHex, FromHexError};
use uint::{Uint, U256};

/// Trait for a fixed-size byte array to be used as the output of hash functions.
pub trait FixedHash: Sized {
	/// Create a new, zero-initialised, instance.
	fn new() -> Self;
	/// Synonym for `new()`. Prefer to new as it's more readable.
	fn zero() -> Self;
	/// Create a new, cryptographically random, instance.
	fn random() -> Self;
	/// Assign self have a cryptographically random value.
	fn randomize(&mut self);
	/// Get the size of this object in bytes.
	fn len() -> usize;
	/// Convert a slice of bytes of length `len()` to an instance of this type.
	fn from_slice(src: &[u8]) -> Self;
	/// Assign self to be of the same value as a slice of bytes of length `len()`.
	fn clone_from_slice(&mut self, src: &[u8]) -> usize;
	/// Copy the data of this object into some mutable slice of length `len()`.
	fn copy_to(&self, dest: &mut [u8]);
	/// Returns `true` if all bits set in `b` are also set in `self`.
	fn contains<'a>(&'a self, b: &'a Self) -> bool;
	/// Returns `true` if no bits are set.
	fn is_zero(&self) -> bool;
	/// Returns the lowest 8 bytes interpreted as a BigEndian integer.
	fn low_u64(&self) -> u64;
}

/// Return `s` without the `0x` at the beginning of it, if any.
pub fn clean_0x(s: &str) -> &str {
	if s.starts_with("0x") {
		&s[2..]
	} else {
		s
	}
}

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[repr(C)]
		/// Unformatted binary data of fixed length.
		pub struct $from (pub [u8; $size]);


		impl From<[u8; $size]> for $from {
			fn from(bytes: [u8; $size]) -> Self {
				$from(bytes)
			}
		}

		impl From<$from> for [u8; $size] {
			fn from(s: $from) -> Self {
				s.0
			}
		}

		impl Deref for $from {
			type Target = [u8];

			#[inline]
			fn deref(&self) -> &[u8] {
				&self.0
			}
		}

		impl AsRef<[u8]> for $from {
			#[inline]
			fn as_ref(&self) -> &[u8] {
				&self.0
			}
		}

		impl DerefMut for $from {
			#[inline]
			fn deref_mut(&mut self) -> &mut [u8] {
				&mut self.0
			}
		}

		impl FixedHash for $from {
			fn new() -> $from {
				$from([0; $size])
			}

			fn zero() -> $from {
				$from([0; $size])
			}

			fn random() -> $from {
				let mut hash = $from::new();
				hash.randomize();
				hash
			}

			fn randomize(&mut self) {
				let mut rng = OsRng::new().unwrap();
				rng.fill_bytes(&mut self.0);
			}

			fn len() -> usize {
				$size
			}

			#[inline]
			fn clone_from_slice(&mut self, src: &[u8]) -> usize {
				let min = cmp::min($size, src.len());
				self.0[..min].copy_from_slice(&src[..min]);
				min
			}

			fn from_slice(src: &[u8]) -> Self {
				let mut r = Self::new();
				r.clone_from_slice(src);
				r
			}

			fn copy_to(&self, dest: &mut[u8]) {
				let min = cmp::min($size, dest.len());
				dest[..min].copy_from_slice(&self.0[..min]);
			}

			fn contains<'a>(&'a self, b: &'a Self) -> bool {
				&(b & self) == b
			}

			fn is_zero(&self) -> bool {
				self.eq(&Self::new())
			}

			fn low_u64(&self) -> u64 {
				let mut ret = 0u64;
				for i in 0..min($size, 8) {
					ret |= (self.0[$size - 1 - i] as u64) << (i * 8);
				}
				ret
			}
		}

		impl FromStr for $from {
			type Err = FromHexError;

			fn from_str(s: &str) -> Result<$from, FromHexError> {
				let a = try!(s.from_hex());
				if a.len() != $size {
					return Err(FromHexError::InvalidHexLength);
				}

				let mut ret = [0;$size];
				ret.copy_from_slice(&a);
				Ok($from(ret))
			}
		}

		impl fmt::Debug for $from {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				for i in &self.0[..] {
					try!(write!(f, "{:02x}", i));
				}
				Ok(())
			}
		}

		impl fmt::Display for $from {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				for i in &self.0[0..2] {
					try!(write!(f, "{:02x}", i));
				}
				try!(write!(f, "…"));
				for i in &self.0[$size - 2..$size] {
					try!(write!(f, "{:02x}", i));
				}
				Ok(())
			}
		}

		impl Copy for $from {}
		#[cfg_attr(feature="dev", allow(expl_impl_clone_on_copy))]
		impl Clone for $from {
			fn clone(&self) -> $from {
				let mut ret = $from::new();
				ret.0.copy_from_slice(&self.0);
				ret
			}
		}

		impl Eq for $from {}

		impl PartialEq for $from {
			fn eq(&self, other: &Self) -> bool {
				for i in 0..$size {
					if self.0[i] != other.0[i] {
						return false;
					}
				}
				true
			}
		}

		impl Ord for $from {
			fn cmp(&self, other: &Self) -> Ordering {
				for i in 0..$size {
					if self.0[i] > other.0[i] {
						return Ordering::Greater;
					} else if self.0[i] < other.0[i] {
						return Ordering::Less;
					}
				}
				Ordering::Equal
			}
		}

		impl PartialOrd for $from {
			fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
				Some(self.cmp(other))
			}
		}

		impl Hash for $from {
			fn hash<H>(&self, state: &mut H) where H: Hasher {
				state.write(&self.0);
				state.finish();
			}
		}

		impl Index<usize> for $from {
			type Output = u8;

			fn index(&self, index: usize) -> &u8 {
				&self.0[index]
			}
		}
		impl IndexMut<usize> for $from {
			fn index_mut(&mut self, index: usize) -> &mut u8 {
				&mut self.0[index]
			}
		}
		impl Index<ops::Range<usize>> for $from {
			type Output = [u8];

			fn index(&self, index: ops::Range<usize>) -> &[u8] {
				&self.0[index]
			}
		}
		impl IndexMut<ops::Range<usize>> for $from {
			fn index_mut(&mut self, index: ops::Range<usize>) -> &mut [u8] {
				&mut self.0[index]
			}
		}
		impl Index<ops::RangeFull> for $from {
			type Output = [u8];

			fn index(&self, _index: ops::RangeFull) -> &[u8] {
				&self.0
			}
		}
		impl IndexMut<ops::RangeFull> for $from {
			fn index_mut(&mut self, _index: ops::RangeFull) -> &mut [u8] {
				&mut self.0
			}
		}

		/// `BitOr` on references
		impl<'a> BitOr for &'a $from {
			type Output = $from;

			fn bitor(self, rhs: Self) -> Self::Output {
				let mut ret: $from = $from::default();
				for i in 0..$size {
					ret.0[i] = self.0[i] | rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitOr`
		impl BitOr for $from {
			type Output = $from;

			fn bitor(self, rhs: Self) -> Self::Output {
				&self | &rhs
			}
		}

		/// `BitAnd` on references
		impl <'a> BitAnd for &'a $from {
			type Output = $from;

			fn bitand(self, rhs: Self) -> Self::Output {
				let mut ret: $from = $from::default();
				for i in 0..$size {
					ret.0[i] = self.0[i] & rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitAnd`
		impl BitAnd for $from {
			type Output = $from;

			fn bitand(self, rhs: Self) -> Self::Output {
				&self & &rhs
			}
		}

		/// `BitXor` on references
		impl <'a> BitXor for &'a $from {
			type Output = $from;

			fn bitxor(self, rhs: Self) -> Self::Output {
				let mut ret: $from = $from::default();
				for i in 0..$size {
					ret.0[i] = self.0[i] ^ rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitXor`
		impl BitXor for $from {
			type Output = $from;

			fn bitxor(self, rhs: Self) -> Self::Output {
				&self ^ &rhs
			}
		}

		impl $from {
			/// Get a hex representation.
			pub fn hex(&self) -> String {
				format!("{:?}", self)
			}
		}

		impl Default for $from {
			fn default() -> Self { $from::new() }
		}

		impl From<u64> for $from {
			fn from(mut value: u64) -> $from {
				let mut ret = $from::new();
				for i in 0..8 {
					if i < $size {
						ret.0[$size - i - 1] = (value & 0xff) as u8;
						value >>= 8;
					}
				}
				ret
			}
		}

		impl From<&'static str> for $from {
			fn from(s: &'static str) -> $from {
				let s = clean_0x(s);
				if s.len() % 2 == 1 {
					$from::from_str(&("0".to_owned() + s)).unwrap()
				} else {
					$from::from_str(s).unwrap()
				}
			}
		}

		impl<'a> From<&'a [u8]> for $from {
			fn from(s: &'a [u8]) -> $from {
				$from::from_slice(s)
			}
		}
	}
}

impl From<U256> for H256 {
	fn from(value: U256) -> H256 {
		let mut ret = H256::new();
		value.to_big_endian(&mut ret);
		ret
	}
}

impl<'a> From<&'a U256> for H256 {
	fn from(value: &'a U256) -> H256 {
		let mut ret: H256 = H256::new();
		value.to_big_endian(&mut ret);
		ret
	}
}

impl From<H256> for U256 {
	fn from(value: H256) -> U256 {
		U256::from(&value)
	}
}

impl<'a> From<&'a H256> for U256 {
	fn from(value: &'a H256) -> U256 {
		U256::from(value.as_ref() as &[u8])
	}
}

impl From<H256> for H160 {
	fn from(value: H256) -> H160 {
		let mut ret = H160::new();
		ret.0.copy_from_slice(&value[12..32]);
		ret
	}
}

impl From<H256> for H64 {
	fn from(value: H256) -> H64 {
		let mut ret = H64::new();
		ret.0.copy_from_slice(&value[20..28]);
		ret
	}
}

impl From<H160> for H256 {
	fn from(value: H160) -> H256 {
		let mut ret = H256::new();
		ret.0[12..32].copy_from_slice(&value);
		ret
	}
}

impl<'a> From<&'a H160> for H256 {
	fn from(value: &'a H160) -> H256 {
		let mut ret = H256::new();
		ret.0[12..32].copy_from_slice(value);
		ret
	}
}

impl_hash!(H32, 4);
impl_hash!(H64, 8);
impl_hash!(H128, 16);
impl_hash!(H160, 20);
impl_hash!(H256, 32);
impl_hash!(H264, 33);
impl_hash!(H512, 64);
impl_hash!(H520, 65);
impl_hash!(H1024, 128);
impl_hash!(H2048, 256);

known_heap_size!(0, H32, H64, H128, H160, H256, H264, H512, H520, H1024, H2048);
// Specialized HashMap and HashSet

/// Hasher that just takes 8 bytes of the provided value.
/// May only be used for keys which are 32 bytes.
pub struct PlainHasher {
	prefix: [u8; 8],
	_marker: [u64; 0], // for alignment
}

impl Default for PlainHasher {
	#[inline]
	fn default() -> PlainHasher {
		PlainHasher {
			prefix: [0; 8],
			_marker: [0; 0],
		}
	}
}

impl Hasher for PlainHasher {
	#[inline]
	fn finish(&self) -> u64 {
		unsafe { ::std::mem::transmute(self.prefix) }
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		debug_assert!(bytes.len() == 32);

		for quarter in bytes.chunks(8) {
			for (x, y) in self.prefix.iter_mut().zip(quarter) {
				*x ^= *y
			}
		}
	}
}

/// Specialized version of `HashMap` with H256 keys and fast hashing function.
pub type H256FastMap<T> = HashMap<H256, T, BuildHasherDefault<PlainHasher>>;
/// Specialized version of `HashSet` with H256 keys and fast hashing function.
pub type H256FastSet = HashSet<H256, BuildHasherDefault<PlainHasher>>;

#[cfg(test)]
mod tests {
	use hash::*;
	use uint::*;
	use std::str::FromStr;

	#[test]
	fn hasher_alignment() {
		use std::mem::align_of;
		assert_eq!(align_of::<u64>(), align_of::<PlainHasher>());
	}

	#[test]
	#[cfg_attr(feature="dev", allow(eq_op))]
	fn hash() {
		let h = H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
		assert_eq!(H64::from_str("0123456789abcdef").unwrap(), h);
		assert_eq!(format!("{}", h), "0123…cdef");
		assert_eq!(format!("{:?}", h), "0123456789abcdef");
		assert_eq!(h.hex(), "0123456789abcdef");
		assert!(h == h);
		assert!(h != H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xee]));
		assert!(h != H64([0; 8]));
	}

	#[test]
	fn hash_bitor() {
		let a = H64([1; 8]);
		let b = H64([2; 8]);
		let c = H64([3; 8]);

		// borrow
		assert_eq!(&a | &b, c);

		// move
		assert_eq!(a | b, c);
	}

	#[test]
	fn from_and_to_address() {
		let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		let h = H256::from(address.clone());
		let a = H160::from(h);
		assert_eq!(address, a);
	}

	#[test]
	fn from_u64() {
		assert_eq!(H128::from(0x1234567890abcdef), H128::from_str("00000000000000001234567890abcdef").unwrap());
		assert_eq!(H64::from(0x1234567890abcdef), H64::from_str("1234567890abcdef").unwrap());
		assert_eq!(H32::from(0x1234567890abcdef), H32::from_str("90abcdef").unwrap());
	}

	#[test]
	fn from_str() {
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("0x1234567890abcdef"));
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("1234567890abcdef"));
		assert_eq!(H64::from(0x234567890abcdef), H64::from("0x234567890abcdef"));
	}

	#[test]
	fn from_and_to_u256() {
		let u: U256 = 0x123456789abcdef0u64.into();
		let h = H256::from(u);
		assert_eq!(H256::from(u), H256::from("000000000000000000000000000000000000000000000000123456789abcdef0"));
		let h_ref = H256::from(&u);
		assert_eq!(h, h_ref);
		let r_ref: U256 = From::from(&h);
		assert_eq!(r_ref, u);
		let r: U256 = From::from(h);
		assert_eq!(r, u);
	}
}
