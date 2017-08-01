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

use std::mem;
use primal::is_prime;

pub const DATASET_BYTES_INIT: u64 = 1 << 30;
pub const DATASET_BYTES_GROWTH: u64 = 1 << 23;
pub const CACHE_BYTES_INIT: u64 = 1 << 24;
pub const CACHE_BYTES_GROWTH: u64 = 1 << 17;

pub const ETHASH_EPOCH_LENGTH: u64 = 30000;
pub const ETHASH_CACHE_ROUNDS: usize = 3;
pub const ETHASH_MIX_BYTES: usize = 128;
pub const ETHASH_ACCESSES: usize = 64;
pub const ETHASH_DATASET_PARENTS: u32 = 256;
pub const NODE_DWORDS: usize = NODE_WORDS / 2;
pub const NODE_WORDS: usize = NODE_BYTES / 4;
pub const NODE_BYTES: usize = 64;

pub fn epoch(block_number: u64) -> u64 {
	block_number / ETHASH_EPOCH_LENGTH
}

static CHARS: &'static [u8] = b"0123456789abcdef";
pub fn to_hex(bytes: &[u8]) -> String {
	let mut v = Vec::with_capacity(bytes.len() * 2);
	for &byte in bytes.iter() {
		v.push(CHARS[(byte >> 4) as usize]);
		v.push(CHARS[(byte & 0xf) as usize]);
	}

	unsafe { String::from_utf8_unchecked(v) }
}

pub fn get_cache_size(block_number: u64) -> usize {
	// TODO: Memoise
	let mut sz: u64 = CACHE_BYTES_INIT + CACHE_BYTES_GROWTH * (block_number / ETHASH_EPOCH_LENGTH);
	sz = sz - NODE_BYTES as u64;
	while !is_prime(sz / NODE_BYTES as u64) {
		sz = sz - 2 * NODE_BYTES as u64;
	}
	sz as usize
}

pub fn get_data_size(block_number: u64) -> usize {
	// TODO: Memoise
	let mut sz: u64 = DATASET_BYTES_INIT + DATASET_BYTES_GROWTH * (block_number / ETHASH_EPOCH_LENGTH);
	sz = sz - ETHASH_MIX_BYTES as u64;
	while !is_prime(sz / ETHASH_MIX_BYTES as u64) {
		sz = sz - 2 * ETHASH_MIX_BYTES as u64;
	}
	sz as usize
}

pub struct Node {
	pub bytes: [u8; NODE_BYTES],
}

impl Default for Node {
	fn default() -> Self {
		Node { bytes: [0u8; NODE_BYTES] }
	}
}

impl Clone for Node {
	fn clone(&self) -> Self {
		Node { bytes: *&self.bytes }
	}
}

impl Node {
	pub fn as_words(&self) -> &[u32; NODE_WORDS] {
		debug_assert_eq!(mem::size_of::<Self>(), mem::size_of::<[u32; NODE_WORDS]>());
		unsafe { mem::transmute(&self.bytes) }
	}

	pub fn as_words_mut(&mut self) -> &mut [u32; NODE_WORDS] {
		debug_assert_eq!(mem::size_of::<Self>(), mem::size_of::<[u32; NODE_WORDS]>());
		unsafe { mem::transmute(&mut self.bytes) }
	}

	pub fn as_dwords(&self) -> &[u64; NODE_DWORDS] {
		debug_assert_eq!(mem::size_of::<Self>(), mem::size_of::<[u64; NODE_DWORDS]>());
		unsafe { mem::transmute(&self.bytes) }
	}

	pub fn as_dwords_mut(&mut self) -> &mut [u64; NODE_DWORDS] {
		debug_assert_eq!(mem::size_of::<Self>(), mem::size_of::<[u64; NODE_DWORDS]>());
		unsafe { mem::transmute(&mut self.bytes) }
	}
}
