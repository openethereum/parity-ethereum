// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

extern crate keccak_hash as hash;

pub type H256 = [u8; 32];

pub mod keccak_512 {
	use super::hash;

	pub use self::hash::{keccak_512 as write, keccak512 as inplace};

    pub unsafe fn unchecked(out: *mut u8, outlen: usize, input: *const u8, inputlen: usize) {
        // TODO(ordian): is this UB?
        let src = core::slice::from_raw_parts(input, inputlen);
        let dst = core::slice::from_raw_parts_mut(out, outlen);

		self::hash::keccak_512(src, dst);
	}
}

pub mod keccak_256 {
	use super::hash;

	pub use self::hash::{keccak_256 as write, keccak256 as inplace};

    pub unsafe fn unchecked(out: *mut u8, outlen: usize, input: *const u8, inputlen: usize) {
        // TODO(ordian): is this UB?
        let src = core::slice::from_raw_parts(input, inputlen);
        let dst = core::slice::from_raw_parts_mut(out, outlen);

		self::hash::keccak_256(src, dst);
	}
}
