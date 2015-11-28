use std::str::FromStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut, BitOr, BitAnd};
use rustc_serialize::hex::*;
use error::EthcoreError;
use rand::Rng;
use rand::os::OsRng;
use bytes::BytesConvertable;
use math::log2;

/// types implementing FixedHash must be also BytesConvertable
pub trait FixedHash: Sized + BytesConvertable {
	fn new() -> Self;
	fn random() -> Self;
	fn randomize(&mut self);
	fn size() -> usize;
	fn mut_bytes(&mut self) -> &mut [u8];
	fn shift_bloom<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: FixedHash;
	fn bloom_part<T>(&self, m: usize) -> T where T: FixedHash;
	fn contains_bloom<T>(&self, b: &T) -> bool where T: FixedHash;
	fn contains<'a>(&'a self, b: &'a Self) -> bool;
}

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[derive(Eq)]
		pub struct $from ([u8; $size]);

		impl BytesConvertable for $from {
			fn bytes(&self) -> &[u8] {
				&self.0
			}
		}

		impl FixedHash for $from {
			fn new() -> $from {
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

			fn mut_bytes(&mut self) -> &mut [u8] {
				&mut self.0
			}

			fn shift_bloom<'a, T>(&'a mut self, b: &T) -> &'a mut Self where T: FixedHash {
				let bp: Self = b.bloom_part($size);
				let new_self = &bp | self;

				// impl |= instead

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
					ret.mut_bytes()[m - 1 - index / 8] |= 1 << (index % 8);
				}

				ret
			}

			fn contains_bloom<T>(&self, b: &T) -> bool where T: FixedHash {
				let bp: Self = b.bloom_part($size);
				self.contains(&bp)
			}

			fn contains<'a>(&'a self, b: &'a Self) -> bool {
				&(b & self) == b
			}
		}

		impl FromStr for $from {
			type Err = EthcoreError;

			fn from_str(s: &str) -> Result<$from, EthcoreError> {
				let a = try!(s.from_hex());
				if a.len() != $size { return Err(EthcoreError::BadSize); }
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
				(self as &fmt::Debug).fmt(f)
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

	}
}

impl_hash!(H64, 8);
impl_hash!(H128, 16);
impl_hash!(Address, 20);
impl_hash!(H256, 32);
impl_hash!(H512, 64);
impl_hash!(H520, 65);
impl_hash!(H1024, 128);
impl_hash!(H2048, 256);

#[cfg(test)]
mod tests {
	use hash::*;
	use std::str::FromStr;

	#[test]
	fn hash() {
		let h = H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
		assert_eq!(H64::from_str("0123456789abcdef").unwrap(), h);
		assert_eq!(format!("{}", h), "0123456789abcdef");
		assert_eq!(format!("{:?}", h), "0123456789abcdef");
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
	fn shift_bloom() {
		use sha3::Hashable;
		
		let bloom = H2048::from_str("00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020000000000000000000000000000000000000000000008000000001000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let address = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
		let topic = H256::from_str("02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();

		let mut my_bloom = H2048::new();
		assert!(!my_bloom.contains_bloom(&address.sha3()));
		assert!(!my_bloom.contains_bloom(&topic.sha3()));

		my_bloom.shift_bloom(&address.sha3());
		assert!(my_bloom.contains_bloom(&address.sha3()));
		assert!(!my_bloom.contains_bloom(&topic.sha3()));
			
		my_bloom.shift_bloom(&topic.sha3());
		assert_eq!(my_bloom, bloom);
		assert!(my_bloom.contains_bloom(&address.sha3()));
		assert!(my_bloom.contains_bloom(&topic.sha3()));
	}
}

