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

pub type NodeBytes = [u8; NODE_BYTES];
pub type NodeWords = [u32; NODE_WORDS];
pub type NodeDwords = [u64; NODE_DWORDS];

assert_eq_size!(node; Node, NodeBytes, NodeWords, NodeDwords);

#[repr(C)]
pub union Node {
	pub dwords: NodeDwords,
	pub words: NodeWords,
	pub bytes: NodeBytes,
}

impl Clone for Node {
	fn clone(&self) -> Self {
		unsafe { Node { bytes: *&self.bytes } }
	}
}

// We use `inline(always)` because I was experiencing an 100% slowdown and `perf` showed that these
// calls were taking up ~30% of the runtime. Adding these annotations fixes the issue. Remove at
// your peril, if and only if you have benchmarks to prove that this doesn't reintroduce the
// performance regression. It's not caused by the `debug_assert_eq!` either, your guess is as good
// as mine.
impl Node {
	#[inline(always)]
	pub fn as_bytes(&self) -> &NodeBytes {
		unsafe { &self.bytes }
	}

	#[inline(always)]
	pub fn as_bytes_mut(&mut self) -> &mut NodeBytes {
		unsafe { &mut self.bytes }
	}

	#[inline(always)]
	pub fn as_words(&self) -> &NodeWords {
		unsafe { &self.words }
	}

	#[inline(always)]
	pub fn as_words_mut(&mut self) -> &mut NodeWords {
		unsafe { &mut self.words }
	}

	#[inline(always)]
	pub fn as_dwords(&self) -> &NodeDwords {
		unsafe { &self.dwords }
	}

	#[inline(always)]
	pub fn as_dwords_mut(&mut self) -> &mut NodeDwords {
		unsafe { &mut self.dwords }
	}
}
