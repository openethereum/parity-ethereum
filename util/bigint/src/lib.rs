// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Efficient large, fixed-size big integers and hashes.

#![cfg_attr(asm_available, feature(asm))]

extern crate rand;
extern crate rustc_hex;
extern crate bigint;
extern crate libc;
extern crate plain_hasher;

#[cfg(feature="heapsizeof")]
#[macro_use]
extern crate heapsize;

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
	pub use ::bigint::*;
	pub use ::hash::*;
}
