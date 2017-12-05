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

extern crate hash;

pub type H256 = [u8; 32];

pub mod keccak_512 {
	use super::hash;

	pub use self::hash::keccak_512 as unchecked;

	pub fn write(input: &[u8], output: &mut [u8]) {
		unsafe { hash::keccak_512(output.as_mut_ptr(), output.len(), input.as_ptr(), input.len()) };
	}

	pub fn inplace(input: &mut [u8]) {
		// This is safe since `sha3_*` uses an internal buffer and copies the result to the output. This
		// means that we can reuse the input buffer for both input and output.
		unsafe { hash::keccak_512(input.as_mut_ptr(), input.len(), input.as_ptr(), input.len()) };
	}
}

pub mod keccak_256 {
	use super::hash;

	pub use self::hash::keccak_256 as unchecked;

	#[allow(dead_code)]
	pub fn write(input: &[u8], output: &mut [u8]) {
		unsafe { hash::keccak_256(output.as_mut_ptr(), output.len(), input.as_ptr(), input.len()) };
	}

	pub fn inplace(input: &mut [u8]) {
		// This is safe since `sha3_*` uses an internal buffer and copies the result to the output. This
		// means that we can reuse the input buffer for both input and output.
		unsafe { hash::keccak_256(input.as_mut_ptr(), input.len(), input.as_ptr(), input.len()) };
	}
}
