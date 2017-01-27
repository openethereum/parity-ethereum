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

//! Wrapper around tiny-keccak crate as well as common hash constants.
extern crate sha3 as sha3_ext;

use std::io;
use tiny_keccak::Keccak;
use hash::{H256, FixedHash};
use self::sha3_ext::*;

/// Get the SHA3 (i.e. Keccak) hash of the empty bytes string.
pub const SHA3_EMPTY: H256 = H256( [0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70] );

/// The SHA3 of the RLP encoding of empty data.
pub const SHA3_NULL_RLP: H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );

/// The SHA3 of the RLP encoding of empty list.
pub const SHA3_EMPTY_LIST_RLP: H256 = H256( [0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a, 0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47] );

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

impl<T> Hashable for T where T: AsRef<[u8]> {
	fn sha3(&self) -> H256 {
		let mut ret: H256 = H256::zero();
		self.sha3_into(&mut *ret);
		ret
	}
	fn sha3_into(&self, dest: &mut [u8]) {
		let input: &[u8] = self.as_ref();

		unsafe {
			sha3_256(dest.as_mut_ptr(), dest.len(), input.as_ptr(), input.len());
		}
	}
}

/// Calculate SHA3 of given stream.
pub fn sha3(r: &mut io::BufRead) -> Result<H256, io::Error> {
	let mut output = [0u8; 32];
	let mut input = [0u8; 1024];
	let mut sha3 = Keccak::new_keccak256();

	// read file
	loop {
		let some = r.read(&mut input)?;
		if some == 0 {
			break;
		}
		sha3.update(&input[0..some]);
	}

	sha3.finalize(&mut output);
	Ok(output.into())
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::io::{Write, BufReader};
	use super::*;

	#[test]
	fn sha3_empty() {
		assert_eq!([0u8; 0].sha3(), SHA3_EMPTY);
	}
	#[test]
	fn sha3_as() {
		assert_eq!([0x41u8; 32].sha3(), From::from("59cad5948673622c1d64e2322488bf01619f7ff45789741b15a9f782ce9290a8"));
	}

	#[test]
	fn should_sha3_a_file() {
		// given
		use devtools::RandomTempPath;
		let path = RandomTempPath::new();
		// Prepare file
		{
			let mut file = fs::File::create(&path).unwrap();
			file.write_all(b"something").unwrap();
		}

		let mut file = BufReader::new(fs::File::open(&path).unwrap());
		// when
		let hash = sha3(&mut file).unwrap();

		// then
		assert_eq!(format!("{:?}", hash), "68371d7e884c168ae2022c82bd837d51837718a7f7dfb7aa3f753074a35e1d87");
	}
}
