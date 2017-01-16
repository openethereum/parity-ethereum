// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
pub fn block_to_cht_number(block_num: u64) -> u64 {
	(block_num + 1) / SIZE
}

/// Get the starting block of a given CHT.
pub fn start_number(cht_num: u64) -> u64 {
	(cht_num * SIZE) + 1
}
