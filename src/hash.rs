use rustc_serialize::hex::*;
use error::EthcoreError;
use std::str::FromStr;
use std::fmt;

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		pub struct $from (pub [u8; $size]);

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
	}
}

impl_hash!(Hash64, 8);
impl_hash!(Hash128, 16);
impl_hash!(Address, 20);
impl_hash!(Hash256, 32);
impl_hash!(Hash512, 64);
impl_hash!(Hash520, 65);
impl_hash!(Hash1024, 128);
impl_hash!(Hash2048, 256);
impl_hash!(Hash4096, 512);

#[test]
fn it_works() {
	let h = Hash64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
	assert_eq!(Hash64::from_str("0123456789abcdef").unwrap(), h);
	assert_eq!(format!("{}", h), "0123456789abcdef");
	assert_eq!(format!("{:?}", h), "0123456789abcdef");
	assert!(h == h);
	assert!(h != Hash64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xee]));
	assert!(h != Hash64([0; 8]));
}