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

//! Canonical hash trie definitions and helper functions.

/// The size of each CHT.
pub const SIZE: u64 = 2048;

/// Convert a block number to a CHT number.
/// Returns `None` for `block_num` == 0, `Some` otherwise.
pub fn block_to_cht_number(block_num: u64) -> Option<u64> {
	match block_num {
		0 => None,
		n => Some((n - 1) / SIZE),
	}
}

/// Get the starting block of a given CHT.
/// CHT 0 includes block 1...SIZE,
/// CHT 1 includes block SIZE + 1 ... 2*SIZE
/// More generally: CHT N includes block (1 + N*SIZE)...((N+1)*SIZE).
/// This is because the genesis hash is assumed to be known
/// and including it would be redundant.
pub fn start_number(cht_num: u64) -> u64 {
	(cht_num * SIZE) + 1
}

#[cfg(test)]
mod tests {
	#[test]
	fn block_to_cht_number() {
		assert!(::cht::block_to_cht_number(0).is_none());
		assert_eq!(::cht::block_to_cht_number(1).unwrap(), 0);
		assert_eq!(::cht::block_to_cht_number(::cht::SIZE + 1).unwrap(), 1);
		assert_eq!(::cht::block_to_cht_number(::cht::SIZE).unwrap(), 0);
	}

	#[test]
	fn start_number() {
		assert_eq!(::cht::start_number(0), 1);
		assert_eq!(::cht::start_number(1), ::cht::SIZE + 1);
		assert_eq!(::cht::start_number(2), ::cht::SIZE * 2 + 1);
	}
}
