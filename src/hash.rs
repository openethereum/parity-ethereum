use rustc_serialize::hex::*;
use error::EthcoreError;
use std::str::FromStr;

macro_rules! impl_hash {
	($from: ident, $size: expr) => {
		#[derive(PartialEq, Debug)]
		struct $from ([u8; $size]);

		impl FromStr for $from {
//			type Output = $from;
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
	}
}

impl_hash!(Hash64, 8);
impl_hash!(Hash128, 16);
impl_hash!(Address, 20);
impl_hash!(Hash256, 32);
//impl_hash!(Hash512, 64);

#[test]
fn it_works() {
	assert_eq!(Hash64::from_str("0123456789abcdef").unwrap(), Hash64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]));
}
