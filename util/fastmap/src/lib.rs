// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Provides a `H256FastMap` type with H256 keys and fast hashing function.

extern crate ethereum_types;
extern crate plain_hasher;

use ethereum_types::H256;
use std::hash;
use std::collections::{HashMap, HashSet};
use plain_hasher::PlainHasher;

/// Specialized version of `HashMap` with H256 keys and fast hashing function.
pub type H256FastMap<T> = HashMap<H256, T, hash::BuildHasherDefault<PlainHasher>>;
/// Specialized version of HashSet with H256 values and fast hashing function.
pub type H256FastSet = HashSet<H256, hash::BuildHasherDefault<PlainHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_works() {
        let mut h = H256FastMap::default();
        h.insert(H256::from_low_u64_be(123), "abc");
    }
}
