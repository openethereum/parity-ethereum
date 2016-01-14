//! General hash types, a fixed-size raw-data type used as the output of hash functions.

use std::str::FromStr;
use std::fmt;
use std::ops;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut, Deref, DerefMut, BitOr, BitOrAssign, BitAnd, BitXor};
use std::cmp::{PartialOrd, Ordering};
use rustc_serialize::hex::*;
use error::UtilError;
use rand::Rng;
use rand::os::OsRng;
use bytes::{BytesConvertable,Populatable};
use math::log2;
use uint::U256;

/// Trait for a fixed-size byte array to be used as the output of hash functions.
///
/// Note: types implementing `FixedHash` must be also `BytesConvertable`.
pub trait FixedHash: Sized + BytesConvertable + Populatable {
	fn new() -> Self;
	/// Synonym for `new()`. Prefer to new as it's more readable.
	fn zero() -> Self;
	fn random() -> Self;
	fn randomize(&mut self);
	fn size() -> usize;
	fn from_slice(src: &[u8]) -> Self;
	fn clone_from_slice(&mut self, src: &[u8]) -> usize;
	fn copy_to(&self, dest: &mut [u8]);
	fn shift_bloomed<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: FixedHash;
	fn with_bloomed<T>(mut self, b: &T) -> Self where T: FixedHash { self.shift_bloomed(b); self }
	fn bloom_part<T>(&self, m: usize) -> T where T: FixedHash;
	fn contains_bloomed<T>(&self, b: &T) -> bool where T: FixedHash;
	fn contains<'a>(&'a self, b: &'a Self) -> bool;
	fn is_zero(&self) -> bool;
}

fn clean_0x(s: &str) -> &str {
	if s.len() >= 2 && &s[0..2] == "0x" {
		&s[2..]
	} else {
		s
	}
}

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[derive(Eq)]
		pub struct $from (pub [u8; $size]);

		impl BytesConvertable for $from {
			fn bytes(&self) -> &[u8] {
				&self.0
			}
		}

		impl Deref for $from {
			type Target = [u8];

			#[inline]
			fn deref(&self) -> &[u8] {
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

			fn size() -> usize {
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

		impl fmt::Debug for $from {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				for i in self.0.iter() {
					try!(write!(f, "{:02x}", i));
				}
				Ok(())
			}
		}
		impl fmt::Display for $from {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				for i in self.0[0..3].iter() {
					try!(write!(f, "{:02x}", i));
				}
				write!(f, "…{:02x}", self.0.last().unwrap())
			}
		}

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

			fn index<'a>(&'a self, index: usize) -> &'a u8 {
				&self.0[index]
			}
		}
		impl IndexMut<usize> for $from {
			fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut u8 {
				&mut self.0[index]
			}
		}
		impl Index<ops::Range<usize>> for $from {
			type Output = [u8];

			fn index<'a>(&'a self, index: ops::Range<usize>) -> &'a [u8] {
				&self.0[index]
			}
		}
		impl IndexMut<ops::Range<usize>> for $from {
			fn index_mut<'a>(&'a mut self, index: ops::Range<usize>) -> &'a mut [u8] {
				&mut self.0[index]
			}
		}
		impl Index<ops::RangeFull> for $from {
			type Output = [u8];

			fn index<'a>(&'a self, _index: ops::RangeFull) -> &'a [u8] {
				&self.0
			}
		}
		impl IndexMut<ops::RangeFull> for $from {
			fn index_mut<'a>(&'a mut self, _index: ops::RangeFull) -> &'a mut [u8] {
				&mut self.0
			}
		}

		/// BitOr on references
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

		/// Moving BitOr
		impl BitOr for $from {
			type Output = $from;

			fn bitor(self, rhs: Self) -> Self::Output {
				&self | &rhs
			}
		}

		/// Moving BitOrAssign
		impl<'a> BitOrAssign<&'a $from> for $from {
			fn bitor_assign(&mut self, rhs: &'a Self) {
				for i in 0..$size {
					self.0[i] = self.0[i] | rhs.0[i];
				}
			}
		}

		/// BitAnd on references
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

		/// Moving BitAnd
		impl BitAnd for $from {
			type Output = $from;

			fn bitand(self, rhs: Self) -> Self::Output {
				&self & &rhs
			}
		}

		/// BitXor on references
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

		/// Moving BitXor
		impl BitXor for $from {
			type Output = $from;

			fn bitxor(self, rhs: Self) -> Self::Output {
				&self ^ &rhs
			}
		}
		impl $from {
			pub fn hex(&self) -> String {
				format!("{:?}", self)
			}

			pub fn from_bloomed<T>(b: &T) -> Self where T: FixedHash { b.bloom_part($size) }
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
					$from::from_str(&("0".to_string() + &(clean_0x(s).to_string()))[..]).unwrap_or($from::new())
				} else {
					$from::from_str(clean_0x(s)).unwrap_or($from::new())
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

pub fn h256_from_hex(s: &str) -> H256 {
	use std::str::FromStr;
	H256::from_str(s).unwrap()
}

pub fn h256_from_u64(n: u64) -> H256 {
	use uint::U256;
	H256::from(&U256::from(n))
}

pub fn address_from_hex(s: &str) -> Address {
	use std::str::FromStr;
	Address::from_str(s).unwrap()
}

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
	use std::str::FromStr;

	#[test]
	fn hash() {
		let h = H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
		assert_eq!(H64::from_str("0123456789abcdef").unwrap(), h);
		assert_eq!(format!("{}", h), "012345…ef");
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
}

