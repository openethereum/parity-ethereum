use std::mem::uninitialized;
use tiny_keccak::keccak_256;
use bytes::BytesConvertable;
use hash::{FixedHash, H256};

pub trait Hashable {
	fn sha3(&self) -> H256;
	fn sha3_into(&self, dest: &mut [u8]);
}

impl<T> Hashable for T where T: BytesConvertable {
	fn sha3(&self) -> H256 {
		unsafe {
			let mut ret: H256 = uninitialized();
			keccak_256(self.bytes(), ret.mut_bytes());
			ret
		}
	}
	fn sha3_into(&self, dest: &mut [u8]) {
		keccak_256(self.bytes(), dest);
	}
}

#[test]
fn sha3_empty() {
	use std::str::FromStr;
	assert_eq!([0u8; 0].sha3(), H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
}
#[test]
fn sha3_as() {
	use std::str::FromStr;
	assert_eq!([0x41u8; 32].sha3(), H256::from_str("59cad5948673622c1d64e2322488bf01619f7ff45789741b15a9f782ce9290a8").unwrap());
}

