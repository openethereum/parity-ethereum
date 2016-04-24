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

use standard::*;
use math::log2;
use error::UtilError;
use rand::Rng;
use rand::os::OsRng;
use bytes::{BytesConvertable,Populatable};
use from_json::*;
use bigint::uint::{Uint, U256};
use rustc_serialize::hex::ToHex;
use serde;

/// Trait for a fixed-size byte array to be used as the output of hash functions.
///
/// Note: types implementing `FixedHash` must be also `BytesConvertable`.
pub trait FixedHash: Sized + BytesConvertable + Populatable + FromStr + Default {
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
	/// When interpreting self as a bloom output, augment (bit-wise OR) with the a bloomed version of `b`.
	fn shift_bloomed<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: FixedHash;
	/// Same as `shift_bloomed` except that `self` is consumed and a new value returned.
	fn with_bloomed<T>(mut self, b: &T) -> Self where T: FixedHash { self.shift_bloomed(b); self }
	/// Bloom the current value using the bloom parameter `m`.
	fn bloom_part<T>(&self, m: usize) -> T where T: FixedHash;
	/// Check to see whether this hash, interpreted as a bloom, contains the value `b` when bloomed.
	fn contains_bloomed<T>(&self, b: &T) -> bool where T: FixedHash;
	/// Returns `true` if all bits set in `b` are also set in `self`.
	fn contains<'a>(&'a self, b: &'a Self) -> bool;
	/// Returns `true` if no bits are set.
	fn is_zero(&self) -> bool;
	/// Returns the lowest 8 bytes interpreted as a BigEndian integer.
	fn low_u64(&self) -> u64;
}

/// Return `s` without the `0x` at the beginning of it, if any.
pub fn clean_0x(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[derive(Eq)]
		/// Unformatted binary data of fixed length.
		pub struct $from (pub [u8; $size]);

		impl From<[u8; $size]> for $from {
			fn from(bytes: [u8; $size]) -> Self {
				$from(bytes)
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

			// TODO: remove once slice::clone_from_slice is stable
			#[inline]
			fn clone_from_slice(&mut self, src: &[u8]) -> usize {
				let min = ::std::cmp::min($size, src.len());
				let dst = &mut self.deref_mut()[.. min];
				let src = &src[.. min];
				for i in 0..min {
					dst[i] = src[i];
				}
				min
			}
			fn from_slice(src: &[u8]) -> Self {
				let mut r = Self::new();
				r.clone_from_slice(src);
				r
			}

			fn copy_to(&self, dest: &mut[u8]) {
				unsafe {
					let min = ::std::cmp::min($size, dest.len());
					::std::ptr::copy(self.0.as_ptr(), dest.as_mut_ptr(), min);
				}
			}

			fn shift_bloomed<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: FixedHash {
				let bp: Self = b.bloom_part($size);
				let new_self = &bp | self;

				// impl |= instead
				// TODO: that's done now!

				unsafe {
					use std::{mem, ptr};
					ptr::copy(new_self.0.as_ptr(), self.0.as_mut_ptr(), mem::size_of::<Self>());
				}

				self
			}

			fn bloom_part<T>(&self, m: usize) -> T where T: FixedHash {
				// numbers of bits
				// TODO: move it to some constant
				let p = 3;

				let bloom_bits = m * 8;
				let mask = bloom_bits - 1;
				let bloom_bytes = (log2(bloom_bits) + 7) / 8;
				//println!("bb: {}", bloom_bytes);

				// must be a power of 2
				assert_eq!(m & (m - 1), 0);
				// out of range
				assert!(p * bloom_bytes <= $size);

				// return type
				let mut ret = T::new();

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
					ret.as_slice_mut()[m - 1 - index / 8] |= 1 << (index % 8);
				}

				ret
			}

			fn contains_bloomed<T>(&self, b: &T) -> bool where T: FixedHash {
				let bp: Self = b.bloom_part($size);
				self.contains(&bp)
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
			type Err = UtilError;
			fn from_str(s: &str) -> Result<$from, UtilError> {
				let a = try!(s.from_hex());
				if a.len() != $size { return Err(UtilError::BadSize); }
				let mut ret = $from([0;$size]);
				for i in 0..$size {
					ret.0[i] = a[i];
				}
				Ok(ret)
			}
		}

		impl serde::Serialize for $from {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
			where S: serde::Serializer {
				let mut hex = "0x".to_owned();
				hex.push_str(self.to_hex().as_ref());
				serializer.serialize_str(hex.as_ref())
			}
		}

		impl serde::Deserialize for $from {
			fn deserialize<D>(deserializer: &mut D) -> Result<$from, D::Error>
			where D: serde::Deserializer {
				struct HashVisitor;

				impl serde::de::Visitor for HashVisitor {
					type Value = $from;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: serde::Error {
						// 0x + len
						if value.len() != 2 + $size * 2 {
							return Err(serde::Error::custom("Invalid length."));
						}

						value[2..].from_hex().map(|ref v| $from::from_slice(v)).map_err(|_| serde::Error::custom("Invalid hex value."))
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: serde::Error {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize(HashVisitor)
			}
		}

		impl FromJson for $from {
			fn from_json(json: &Json) -> Self {
				match *json {
					Json::String(ref s) => {
						match s.len() % 2 {
							0 => FromStr::from_str(clean_0x(s)).unwrap(),
							_ => FromStr::from_str(&("0".to_owned() + &(clean_0x(s).to_owned()))[..]).unwrap()
						}
					},
					_ => Default::default(),
				}
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
				unsafe {
					use std::{mem, ptr};
					let mut ret: $from = mem::uninitialized();
					ptr::copy(self.0.as_ptr(), ret.0.as_mut_ptr(), mem::size_of::<$from>());
					ret
				}
			}
		}

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
				unsafe {
					use std::mem;
					let mut ret: $from = mem::uninitialized();
					for i in 0..$size {
						ret.0[i] = self.0[i] | rhs.0[i];
					}
					ret
				}
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
				unsafe {
					use std::mem;
					let mut ret: $from = mem::uninitialized();
					for i in 0..$size {
						ret.0[i] = self.0[i] & rhs.0[i];
					}
					ret
				}
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
				unsafe {
					use std::mem;
					let mut ret: $from = mem::uninitialized();
					for i in 0..$size {
						ret.0[i] = self.0[i] ^ rhs.0[i];
					}
					ret
				}
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

			/// Construct new instance equal to the bloomed value of `b`.
			pub fn from_bloomed<T>(b: &T) -> Self where T: FixedHash { b.bloom_part($size) }
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

		impl<'_> From<&'_ str> for $from {
			fn from(s: &'_ str) -> $from {
				use std::str::FromStr;
				if s.len() % 2 == 1 {
					$from::from_str(&("0".to_owned() + &(clean_0x(s).to_owned()))[..]).unwrap_or_else(|_| $from::new())
				} else {
					$from::from_str(clean_0x(s)).unwrap_or_else(|_| $from::new())
				}
			}
		}
	}
}

impl From<U256> for H256 {
	fn from(value: U256) -> H256 {
		unsafe {
			let mut ret: H256 = ::std::mem::uninitialized();
			value.to_bytes(&mut ret);
			ret
		}
	}
}

impl<'_> From<&'_ U256> for H256 {
	fn from(value: &'_ U256) -> H256 {
		unsafe {
			let mut ret: H256 = ::std::mem::uninitialized();
			value.to_bytes(&mut ret);
			ret
		}
	}
}

impl From<H256> for U256 {
	fn from(value: H256) -> U256 {
		U256::from(value.bytes())
	}
}

impl<'_> From<&'_ H256> for U256 {
	fn from(value: &'_ H256) -> U256 {
		U256::from(value.bytes())
	}
}

impl From<H256> for Address {
	fn from(value: H256) -> Address {
		unsafe {
			let mut ret: Address = ::std::mem::uninitialized();
			::std::ptr::copy(value.as_ptr().offset(12), ret.as_mut_ptr(), 20);
			ret
		}
	}
}

impl From<H256> for H64 {
	fn from(value: H256) -> H64 {
		unsafe {
			let mut ret: H64 = ::std::mem::uninitialized();
			::std::ptr::copy(value.as_ptr().offset(20), ret.as_mut_ptr(), 8);
			ret
		}
	}
}
/*
impl<'_> From<&'_ H256> for Address {
	fn from(value: &'_ H256) -> Address {
		unsafe {
			let mut ret: Address = ::std::mem::uninitialized();
			::std::ptr::copy(value.as_ptr().offset(12), ret.as_mut_ptr(), 20);
			ret
		}
	}
}
*/
impl From<Address> for H256 {
	fn from(value: Address) -> H256 {
		unsafe {
			let mut ret = H256::new();
			::std::ptr::copy(value.as_ptr(), ret.as_mut_ptr().offset(12), 20);
			ret
		}
	}
}

impl<'_> From<&'_ Address> for H256 {
	fn from(value: &'_ Address) -> H256 {
		unsafe {
			let mut ret = H256::new();
			::std::ptr::copy(value.as_ptr(), ret.as_mut_ptr().offset(12), 20);
			ret
		}
	}
}

/// Convert string `s` to an `H256`. Will panic if `s` is not 64 characters long or if any of
/// those characters are not 0-9, a-z or A-Z.
pub fn h256_from_hex(s: &str) -> H256 {
	use std::str::FromStr;
	H256::from_str(s).unwrap()
}

/// Convert `n` to an `H256`, setting the rightmost 8 bytes.
pub fn h256_from_u64(n: u64) -> H256 {
	use bigint::uint::U256;
	H256::from(&U256::from(n))
}

/// Convert string `s` to an `Address`. Will panic if `s` is not 40 characters long or if any of
/// those characters are not 0-9, a-z or A-Z.
pub fn address_from_hex(s: &str) -> Address {
	use std::str::FromStr;
	Address::from_str(s).unwrap()
}

/// Convert `n` to an `Address`, setting the rightmost 8 bytes.
pub fn address_from_u64(n: u64) -> Address {
	let h256 = h256_from_u64(n);
	From::from(h256)
}

impl_hash!(H32, 4);
impl_hash!(H64, 8);
impl_hash!(H128, 16);
impl_hash!(Address, 20);
impl_hash!(H256, 32);
impl_hash!(H264, 33);
impl_hash!(H512, 64);
impl_hash!(H520, 65);
impl_hash!(H1024, 128);
impl_hash!(H2048, 256);

/// Constant address for point 0. Often used as a default.
pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
/// Constant 256-bit datum for 0. Often used as a default.
pub static ZERO_H256: H256 = H256([0x00; 32]);

#[cfg(test)]
mod tests {
	use hash::*;
	use bigint::uint::*;
	use std::str::FromStr;

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
	fn shift_bloomed() {
		use sha3::Hashable;

		let bloom = H2048::from_str("00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020000000000000000000000000000000000000000000008000000001000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let address = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
		let topic = H256::from_str("02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();

		let mut my_bloom = H2048::new();
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

	#[test]
	fn from_and_to_address() {
		let address = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
		let h = H256::from(address.clone());
		let a = Address::from(h);
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
		// too short.
		assert_eq!(H64::from(0), H64::from("0x34567890abcdef"));
	}

	#[test]
	fn from_and_to_u256() {
		let u: U256 = x!(0x123456789abcdef0u64);
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
