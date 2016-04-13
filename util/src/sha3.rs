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

//! Wrapper around tiny-keccak crate.
extern crate sha3 as sha3_ext;

use std::mem::uninitialized;
use bytes::{BytesConvertable, Populatable};
use hash::{FixedHash, H256};
use self::sha3_ext::*;

/// Get the SHA3 (i.e. Keccak) hash of the empty bytes string.
pub const SHA3_EMPTY: H256 = H256([0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70]);


/// Types implementing this trait are sha3able.
///
/// ```
/// extern crate ethcore_util as util;
/// use std::str::FromStr;
/// use util::sha3::*;
/// use util::hash::*;
///
/// fn main() {
/// 	assert_eq!([0u8; 0].sha3(), H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
/// }
/// ```
pub trait Hashable {
	/// Calculate SHA3 of this object.
	fn sha3(&self) -> H256;

	/// Calculate SHA3 of this object and place result into dest.
	fn sha3_into(&self, dest: &mut [u8]) {
		self.sha3().copy_to(dest);
	}
}

impl<T> Hashable for T
    where T: BytesConvertable,
{
	fn sha3(&self) -> H256 {
		unsafe {
			let mut ret: H256 = uninitialized();
			self.sha3_into(ret.as_slice_mut());
			ret
		}
	}
	fn sha3_into(&self, dest: &mut [u8]) {
		unsafe {
			let input: &[u8] = self.bytes();
			sha3_256(dest.as_mut_ptr(), dest.len(), input.as_ptr(), input.len());
		}
	}
}

#[test]
fn sha3_empty() {
	assert_eq!([0u8; 0].sha3(), SHA3_EMPTY);
}
#[test]
fn sha3_as() {
	assert_eq!([0x41u8; 32].sha3(), From::from("59cad5948673622c1d64e2322488bf01619f7ff45789741b15a9f782ce9290a8"));
}
