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

//! Efficient large, fixed-size big integers and hashes.

#![cfg_attr(asm_available, feature(asm))]

extern crate rand;
extern crate rustc_serialize;
#[macro_use] extern crate heapsize;

pub mod uint;
pub mod hash;

/// A prelude module for re-exporting all the types defined in this crate.
///
/// ```rust
/// use ethcore_bigint::prelude::*;
///
/// let x: U256 = U256::zero();
/// let y = x + 1.into();
/// ```
pub mod prelude {
	pub use ::uint::*;
	pub use ::hash::*;
}