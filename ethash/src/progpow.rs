// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use compute::{FNV_PRIME, calculate_dag_item};
use keccak::H256;
use shared::{ETHASH_ACCESSES, ETHASH_MIX_BYTES, Node};

const PROGPOW_LANES: usize = 32;
const PROGPOW_REGS: usize = 16;
const PROGPOW_CACHE_WORDS: usize = 4 * 1024;
const PROGPOW_CNT_MEM: usize = ETHASH_ACCESSES;
const PROGPOW_CNT_CACHE: usize = 8;
const PROGPOW_CNT_MATH: usize = 8;
const PROGPOW_MIX_BYTES: usize = 2 * ETHASH_MIX_BYTES;
const PROGPOW_PERIOD_LENGTH: usize = 50; // blocks per progpow epoch (N)

const FNV_HASH: u32 = 0x811c9dc5;

const KECCAKF_RNDC: [u32; 24] = [
	0x00000001, 0x00008082, 0x0000808a, 0x80008000, 0x0000808b, 0x80000001,
	0x80008081, 0x00008009, 0x0000008a, 0x00000088, 0x80008009, 0x8000000a,
	0x8000808b, 0x0000008b, 0x00008089, 0x00008003, 0x00008002, 0x00000080,
	0x0000800a, 0x8000000a, 0x80008081, 0x00008080, 0x80000001, 0x80008008
];

const KECCAKF_ROTC: [u32; 24] = [
	1,  3,  6,  10, 15, 21, 28, 36, 45, 55, 2,  14,
	27, 41, 56, 8,  25, 43, 62, 18, 39, 61, 20, 44
];

const KECCAKF_PILN: [usize; 24] = [
	10, 7,  11, 17, 18, 3, 5,  16, 8,  21, 24, 4,
	15, 23, 19, 13, 12, 2, 20, 14, 22, 9,  6,  1
];

fn keccak_f800_round(st: &mut [u32; 25], r: usize) {
	// Theta
	let mut bc = [0u32; 5];
	for i in 0..bc.len() {
		bc[i] = st[i] ^ st[i + 5] ^ st[i + 10] ^ st[i + 15] ^ st[i + 20];
	}

	for i in 0..bc.len() {
		let t = bc[(i + 4) % 5] ^ bc[(i + 1) % 5].rotate_left(1);
		for j in (0..st.len()).step_by(5) {
			st[j + i] ^= t;
		}
	}

	// Rho Pi
	let mut t = st[1];
	for i in 0..KECCAKF_ROTC.len() {
		let j = KECCAKF_PILN[i];
		bc[0] = st[j];
		st[j] = t.rotate_left(KECCAKF_ROTC[i]);
		t = bc[0];
	}

	// Chi
	for j in (0..st.len()).step_by(5) {
		for i in 0..bc.len() {
			bc[i] = st[j + i];
		}
		for i in 0..bc.len() {
			st[j + i] ^= (!bc[(i + 1) % 5]) & bc[(i + 2) % 5];
		}
	}

	// Iota
	st[0] ^= KECCAKF_RNDC[r];
}

fn keccak_f800_short(header_hash: H256, nonce: u64, result: [u32; 8]) -> u64 {
	let mut st = [0u32; 25];

	for i in 0..8 {
		st[i] = (header_hash[4 * i] as u32) +
			((header_hash[4 * i + 1] as u32) << 8) +
			((header_hash[4 * i + 2] as u32) << 16) +
			((header_hash[4 * i + 3] as u32) << 24);
	}

	st[8] = nonce as u32;
	st[9] = (nonce >> 32) as u32;

	for i in 0..8 {
		st[10 + i] = result[i];
	}

	for r in 0..21 {
		keccak_f800_round(&mut st, r);
	}
	keccak_f800_round(&mut st, 21);

	(st[0] as u64) << 32 | st[1] as u64
}

pub fn keccak_f800_long(header_hash: H256, nonce: u64, result: [u32; 8]) -> H256 {
	let mut st = [0u32; 25];

	for i in 0..8 {
		st[i] = (header_hash[4 * i] as u32) +
			((header_hash[4 * i + 1] as u32) << 8) +
			((header_hash[4 * i + 2] as u32) << 16) +
			((header_hash[4 * i + 3] as u32) << 24);
	}

	st[8] = nonce as u32;
	st[9] = (nonce >> 32) as u32;

	for i in 0..8 {
		st[10 + i] = result[i];
	}

	for r in 0..21 {
		keccak_f800_round(&mut st, r);
	}
	keccak_f800_round(&mut st, 21);

	let res: [u32; 8] = [st[0], st[1], st[2], st[3], st[4], st[5], st[6], st[7]];
	// transmute to little endian bytes
	unsafe { ::std::mem::transmute(res) }
}

fn fnv1a_hash(h: &mut u32, d: u32) -> u32 {
	*h = (*h ^ d).wrapping_mul(FNV_PRIME);
	*h
}

struct Kiss99 {
	z: u32,
	w: u32,
	jsr: u32,
	jcong: u32,
}

impl Kiss99 {
	fn new(z: u32, w: u32, jsr: u32, jcong: u32) -> Kiss99 {
		Kiss99 { z, w, jsr, jcong }
	}

	fn next_u32(&mut self) -> u32 {
		self.z = 36969u32.wrapping_mul(self.z & 65535).wrapping_add(self.z >> 16);
		self.w = 18000u32.wrapping_mul(self.w & 65535).wrapping_add(self.w >> 16);
		let mwc = (self.z << 16).wrapping_add(self.w);
		self.jsr ^= self.jsr << 17;
		self.jsr ^= self.jsr >> 13;
		self.jsr ^= self.jsr << 5;
		self.jcong = 69069u32.wrapping_mul(self.jcong).wrapping_add(1234567);

		(mwc ^ self.jcong).wrapping_add(self.jsr)
	}
}

fn fill_mix(seed: u64, lane_id: u32) -> [u32; PROGPOW_REGS] {
	// Use FNV to expand the per-warp seed to per-lane
	// Use KISS to expand the per-lane seed to fill mix
	let mut fnv_hash = FNV_HASH;
	let mut rnd = Kiss99::new(
		fnv1a_hash(&mut fnv_hash, seed as u32),
		fnv1a_hash(&mut fnv_hash, (seed >> 32) as u32),
		fnv1a_hash(&mut fnv_hash, lane_id),
		fnv1a_hash(&mut fnv_hash, lane_id),
	);

	let mut mix = [0; PROGPOW_REGS];
	for i in 0..mix.len() {
		mix[i] = rnd.next_u32();
	}

	mix
}

// Merge new data from b into the value in a. Assuming A has high entropy only
// do ops that retain entropy even if B is low entropy (IE don't do A&B)
fn merge(a: &mut u32, b: u32, r: u32) {
	match r % 4 {
		0 => *a = a.wrapping_mul(33).wrapping_add(b),
		1 => *a = (*a ^ b).wrapping_mul(33),
		2 => *a = a.rotate_left((r >> 16) % 32) ^ b,
		3 => *a = a.rotate_right((r >> 16) % 32) ^ b,
		_ => unreachable!(),
	}
}

fn math(a: u32, b: u32, r: u32) -> u32 {
	match r % 11 {
		0 => a.wrapping_add(b),
		1 => a.wrapping_mul(b),
		2 => ((a as u64).wrapping_mul(b as u64) >> 32) as u32,
		3 => a.min(b),
		4 => a.rotate_left(b),
		5 => a.rotate_right(b),
		6 => a & b,
		7 => a | b,
		8 => a ^ b,
		9 => a.leading_zeros() + b.leading_zeros(),
		10 => a.count_ones() + b.count_ones(),
		_ => unreachable!(),
	}
}

fn progpow_init(seed: u64) -> (Kiss99, [u32; PROGPOW_REGS]) {
	let mut fnv_hash = FNV_HASH;
	let mut rnd = Kiss99::new(
		fnv1a_hash(&mut fnv_hash, seed as u32),
		fnv1a_hash(&mut fnv_hash, (seed >> 32) as u32),
		fnv1a_hash(&mut fnv_hash, seed as u32),
		fnv1a_hash(&mut fnv_hash, (seed >> 32) as u32),
	);

	// Create a random sequence of mix destinations for merge() guaranteeing
	// every location is touched once. Uses Fisher–Yates shuffle
	let mut mix_seq = [0u32; PROGPOW_REGS];
	for i in 0..mix_seq.len() {
		mix_seq[i] = i as u32;
	}

	for i in (1..mix_seq.len()).rev() {
		let j = rnd.next_u32() as usize % (i + 1);
		mix_seq.swap(i, j);
	}

	(rnd, mix_seq)
}

fn progpow_loop<F>(
	seed: u64,
	loop_: usize,
	mix: &mut [[u32; PROGPOW_REGS]; PROGPOW_LANES],
	c_dag: &[u32; PROGPOW_CACHE_WORDS],
	lookup: &F,
	data_size: usize,
) where F: Fn(usize) -> Node {
	let g_offset = mix[loop_ % PROGPOW_LANES][0] as usize % data_size;
	let g_offset = g_offset * PROGPOW_LANES;

	let mut node = lookup(2 * g_offset);

	// Lanes can execute in parallel and will be convergent
	for l in 0..mix.len() {
		let index = 2 * (g_offset + l);

		if l != 0 && index % 16 == 0 {
			node = lookup(index);
		}

		// Global load to sequential locations
		let data64 =
			(node.as_words()[(index + 1) % 16] as u64) << 32 |
			node.as_words()[index % 16] as u64;

		// Initialize the seed and mix destination sequence
		let (mut rnd, mut mix_seq) = progpow_init(seed);
		let mut mix_seq_cnt = 0;

		let mix_src = |rnd: &mut Kiss99| rnd.next_u32() as usize % PROGPOW_REGS;
		let mut mix_dst = || {
			let ret = mix_seq[mix_seq_cnt % PROGPOW_REGS];
			mix_seq_cnt += 1;
			ret as usize
		};

		for i in 0..(PROGPOW_CNT_CACHE.max(PROGPOW_CNT_MATH)) {
			if i < PROGPOW_CNT_CACHE {
				// Cached memory access lanes access random location
				let offset = mix[l][mix_src(&mut rnd)] as usize % PROGPOW_CACHE_WORDS;
				let data32 = c_dag[offset];
				merge(&mut mix[l][mix_dst()], data32, rnd.next_u32());
			}
			if i < PROGPOW_CNT_MATH {
				// Random math
				let data32 = math(mix[l][mix_src(&mut rnd)], mix[l][mix_src(&mut rnd)], rnd.next_u32());
				merge(&mut mix[l][mix_dst()], data32, rnd.next_u32());
			}
		}

		// Consume the global load data at the very end of the loop.
		// Allows full latency hiding
		merge(&mut mix[l][0], data64 as u32, rnd.next_u32());
		merge(&mut mix[l][mix_dst()], (data64 >> 32) as u32, rnd.next_u32());
	}
}

pub fn progpow<F>(
	header_hash: H256,
	nonce: u64,
	size: u64,
	block_number: u64,
	c_dag: &[u32; PROGPOW_CACHE_WORDS],
	lookup: F,
) -> (H256, H256)
	where F: Fn(usize) -> Node
{
	let mut mix = [[0u32; PROGPOW_REGS]; PROGPOW_LANES];
	let mut lane_results = [0u32; PROGPOW_LANES];
	let mut result = [0u32; 8];

	// Initialize mix for all lanes
	let seed = keccak_f800_short(header_hash, nonce, result);
	for l in 0..mix.len() {
		mix[l] = fill_mix(seed, l as u32);
	}

	// Execute the randomly generated inner loop
	let period = block_number / PROGPOW_PERIOD_LENGTH as u64;
	for i in 0..PROGPOW_CNT_MEM {
		progpow_loop(
			period,
			i,
			&mut mix,
			c_dag,
			&lookup,
			size as usize / PROGPOW_MIX_BYTES,
		);
	}

	// Reduce mix data to a single per-lane result
	for l in 0..lane_results.len() {
		lane_results[l] = FNV_HASH;
		for i in 0..PROGPOW_REGS {
			fnv1a_hash(&mut lane_results[l], mix[l][i]);
		}
	}

	// Reduce all lanes to a single 128-bit result
	result = [FNV_HASH; 8];
	for l in 0..PROGPOW_LANES {
		fnv1a_hash(&mut result[l % 8], lane_results[l]);
	}

	let digest = keccak_f800_long(header_hash, seed, result);
	// transmute to little endian bytes
	let result = unsafe { ::std::mem::transmute(result) };

	(digest, result)
}

pub fn progpow_light(
	header_hash: H256,
	nonce: u64,
	size: u64,
	block_number: u64,
	cache: &[Node],
) -> (H256, H256) {
	let c_dag = generate_cdag(cache);
	let lookup = |index: usize| {
		calculate_dag_item((index / 16) as u32, cache)
	};

	progpow(
		header_hash,
		nonce,
		size,
		block_number,
		&c_dag,
		lookup,
	)
}

pub fn generate_cdag(cache: &[Node]) -> [u32; PROGPOW_CACHE_WORDS] {
	let mut c_dag = [0u32; PROGPOW_CACHE_WORDS];

	for i in 0..PROGPOW_CACHE_WORDS / 16 {
		let node = ::compute::calculate_dag_item(i as u32, cache);
		for j in 0..16 {
			c_dag[i * 16 + j] = node.as_words()[j];
		}
	}

	c_dag
}

#[cfg(test)]
mod test {
	use tempdir::TempDir;

	use cache::{NodeCacheBuilder, OptimizeFor};
	use keccak::H256;
	use rustc_hex::FromHex;
	use serde_json::{self, Value};
	use shared::get_data_size;
	use std::collections::VecDeque;
	use super::*;

	fn h256(hex: &str) -> H256 {
		let bytes = FromHex::from_hex(hex).unwrap();
		let mut res = [0; 32];
		res.copy_from_slice(&bytes);
		res
	}

	#[test]
	fn test_cdag() {
		let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
		let tempdir = TempDir::new("").unwrap();
		let cache = builder.new_cache(tempdir.into_path(), 0);

		let c_dag = generate_cdag(cache.as_ref());

		let expected = vec![
			690150178u32, 1181503948, 2248155602, 2118233073, 2193871115,
			1791778428, 1067701239, 724807309, 530799275, 3480325829, 3899029234,
			1998124059, 2541974622, 1100859971, 1297211151, 3268320000, 2217813733,
			2690422980, 3172863319, 2651064309
		];

		assert_eq!(
			c_dag.iter().take(20).cloned().collect::<Vec<_>>(),
			expected,
		);
	}

	#[test]
	fn test_random_merge() {
		let tests = [
			(1000000u32, 101u32, 33000101u32),
			(2000000, 102, 66003366),
			(3000000, 103, 2999975),
			(4000000, 104, 4000104),
			(1000000, 0, 33000000),
			(2000000, 0, 66000000),
			(3000000, 0, 3000000),
			(4000000, 0, 4000000),
		];

		for (i, &(mut a, b, expected)) in tests.iter().enumerate() {
			merge(&mut a, b, i as u32);
			assert_eq!(a, expected);
		}
	}

	#[test]
	fn test_random_math() {
		let tests = [
			(20u32, 22u32, 42u32),
			(70000, 80000, 1305032704),
			(70000, 80000, 1),
			(1, 2, 1),
			(3, 10000, 196608),
			(3, 0, 3),
			(3, 6, 2),
			(3, 6, 7),
			(3, 6, 5),
			(0, 0xffffffff, 32),
			(3 << 13, 1 << 5, 3),
			(22, 20, 42),
			(80000, 70000, 1305032704),
			(80000, 70000, 1),
			(2, 1, 1),
			(10000, 3, 80000),
			(0, 3, 0),
			(6, 3, 2),
			(6, 3, 7),
			(6, 3, 5),
			(0, 0xffffffff, 32),
			(3 << 13, 1 << 5, 3),
		];

		for (i, &(a, b, expected)) in tests.iter().enumerate() {
			assert_eq!(
				math(a, b, i as u32),
				expected,
			);
		}
	}

	#[test]
	fn test_keccak_256() {
		let expected = "5dd431e5fbc604f499bfa0232f45f8f142d0ff5178f539e5a7800bf0643697af";
		assert_eq!(
			keccak_f800_long([0; 32], 0, [0; 8]),
			h256(expected),
		);
	}

	#[test]
	fn test_progpow_hash() {
		let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
		let tempdir = TempDir::new("").unwrap();
		let cache = builder.new_cache(tempdir.into_path(), 0);
		let data_size = get_data_size(0) as u64;
		let c_dag = generate_cdag(cache.as_ref());

		let lookup = |index: usize| {
			::compute::calculate_dag_item((index / 16) as u32, cache.as_ref())
		};

		let header_hash = [0; 32];

		let (digest, result) = progpow(
			header_hash,
			0,
			data_size,
			0,
			&c_dag,
			lookup,
		);

		let expected_digest = FromHex::from_hex("7d5b1d047bfb2ebeff3f60d6cc935fc1eb882ece1732eb4708425d2f11965535").unwrap();
		let expected_result = FromHex::from_hex("8c091b4eebc51620ca41e2b90a167d378dbfe01c0a255f70ee7004d85a646e17").unwrap();

		assert_eq!(
			digest.to_vec(),
			expected_digest,
		);

		assert_eq!(
			result.to_vec(),
			expected_result,
		);
	}

	#[test]
	fn test_progpow_testvectors() {
		struct ProgpowTest {
			block_number: u64,
			header_hash: H256,
			nonce: u64,
			mix_hash: H256,
			final_hash: H256,
		}

		let tests: Vec<VecDeque<Value>> =
			serde_json::from_slice(include_bytes!("../res/progpow_testvectors.json")).unwrap();

		let tests: Vec<ProgpowTest> = tests.into_iter().map(|mut test: VecDeque<Value>| {
			assert!(test.len() == 5);

			let block_number: u64 = serde_json::from_value(test.pop_front().unwrap()).unwrap();
			let header_hash: String = serde_json::from_value(test.pop_front().unwrap()).unwrap();
			let nonce: String = serde_json::from_value(test.pop_front().unwrap()).unwrap();
			let mix_hash: String = serde_json::from_value(test.pop_front().unwrap()).unwrap();
			let final_hash: String = serde_json::from_value(test.pop_front().unwrap()).unwrap();

			ProgpowTest {
				block_number,
				header_hash: h256(&header_hash),
				nonce: u64::from_str_radix(&nonce, 16).unwrap(),
				mix_hash: h256(&mix_hash),
				final_hash: h256(&final_hash),
			}
		}).collect();

		for test in tests {
			let builder = NodeCacheBuilder::new(OptimizeFor::Memory);
			let tempdir = TempDir::new("").unwrap();
			let cache = builder.new_cache(tempdir.path().to_owned(), test.block_number);
			let data_size = get_data_size(test.block_number) as u64;
			let c_dag = generate_cdag(cache.as_ref());

			let lookup = |index: usize| {
				::compute::calculate_dag_item((index / 16) as u32, cache.as_ref())
			};

			let (digest, result) = progpow(
				test.header_hash,
				test.nonce,
				data_size,
				test.block_number,
				&c_dag,
				lookup,
			);

			assert_eq!(digest, test.final_hash);
			assert_eq!(result, test.mix_hash);
		}
	}
}
