use std::str::FromStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use rustc_serialize::hex::*;
use error::EthcoreError;
use rand::Rng;
use rand::os::OsRng;
use bytes::BytesConvertable;

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[derive(Eq)]
		pub struct $from (pub [u8; $size]);

		impl $from {
			pub fn new() -> $from {
				$from([0; $size])
			}
			pub fn random() -> $from {
				let mut hash = $from::new();
				hash.randomize();
				hash
			}
			pub fn randomize(&mut self) {
				let mut rng = OsRng::new().unwrap();
				rng.fill_bytes(&mut self.0);
			}

			pub fn mut_bytes(&mut self) -> &mut [u8; $size] {
				&mut self.0
			}
		}

		impl BytesConvertable for $from {
			fn bytes(&self) -> &[u8] {
				&self.0
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
				*self
			}
		}
		impl Copy for $from {}

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
impl_hash!(H4096, 512);

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
