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

//! Ethash implementation
//! See https://github.com/ethereum/wiki/wiki/Ethash

// TODO: fix endianess for big endian

use primal::is_prime;
use std::cell::Cell;
use std::sync::Mutex;
use std::mem;
use std::ptr;
use sha3;
use std::slice;
use std::path::PathBuf;
use std::io::{self, Read, Write};
use std::fs::{self, File};

pub const ETHASH_EPOCH_LENGTH: u64 = 30000;
pub const ETHASH_CACHE_ROUNDS: usize = 3;
pub const ETHASH_MIX_BYTES: usize = 128;
pub const ETHASH_ACCESSES: usize = 64;
pub const ETHASH_DATASET_PARENTS: u32 = 256;

const DATASET_BYTES_INIT: u64 = 1 << 30;
const DATASET_BYTES_GROWTH: u64 = 1 << 23;
const CACHE_BYTES_INIT: u64 = 1 << 24;
const CACHE_BYTES_GROWTH: u64 = 1 << 17;
const NODE_WORDS: usize = 64 / 4;
const NODE_BYTES: usize = 64;
const MIX_WORDS: usize = ETHASH_MIX_BYTES / 4;
const MIX_NODES: usize = MIX_WORDS / NODE_WORDS;
const FNV_PRIME: u32 = 0x01000193;

/// Computation result
pub struct ProofOfWork {
	/// Difficulty boundary
	pub value: H256,
	/// Mix
	pub mix_hash: H256,
}

struct Node {
	bytes: [u8; NODE_BYTES],
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
	#[inline]
	fn as_words(&self) -> &[u32; NODE_WORDS] {
		unsafe { mem::transmute(&self.bytes) }
	}

	#[inline]
	fn as_words_mut(&mut self) -> &mut [u32; NODE_WORDS] {
		unsafe { mem::transmute(&mut self.bytes) }
	}
}

pub type H256 = [u8; 32];

pub struct Light {
	block_number: u64,
	cache: Vec<Node>,
	seed_compute: Mutex<SeedHashCompute>,
}

/// Light cache structur
impl Light {
	/// Create a new light cache for a given block number
	pub fn new(block_number: u64) -> Light {
		light_new(block_number)
	}

	/// Calculate the light boundary data
	/// `header_hash` - The header hash to pack into the mix
	/// `nonce` - The nonce to pack into the mix
	pub fn compute(&self, header_hash: &H256, nonce: u64) -> ProofOfWork {
		light_compute(self, header_hash, nonce)
	}

	pub fn file_path(seed_hash: H256) -> PathBuf {
		let mut home = ::std::env::home_dir().unwrap();
		home.push(".ethash");
		home.push("light");
		home.push(to_hex(&seed_hash));
		home
	}

	pub fn from_file(block_number: u64) -> io::Result<Light> {
		let seed_compute = SeedHashCompute::new();
		let path = Light::file_path(seed_compute.get_seedhash(block_number));
		let mut file = try!(File::open(path));

		let cache_size = get_cache_size(block_number);
		if try!(file.metadata()).len() != cache_size as u64 {
			return Err(io::Error::new(io::ErrorKind::Other, "Cache file size mismatch"));
		}
		let num_nodes = cache_size / NODE_BYTES;
		let mut nodes: Vec<Node> = Vec::new();
		nodes.resize(num_nodes, unsafe { mem::uninitialized() });
		let buf = unsafe { slice::from_raw_parts_mut(nodes.as_mut_ptr() as *mut u8, cache_size) };
		try!(file.read_exact(buf));
		Ok(Light {
			cache: nodes,
			block_number: block_number,
			seed_compute: Mutex::new(seed_compute),
		})
	}

	pub fn to_file(&self) -> io::Result<()> {
		let seed_compute = self.seed_compute.lock().unwrap();
		let path = Light::file_path(seed_compute.get_seedhash(self.block_number));
		try!(fs::create_dir_all(path.parent().unwrap()));
		let mut file = try!(File::create(path));

		let cache_size = self.cache.len() * NODE_BYTES;
		let buf = unsafe { slice::from_raw_parts(self.cache.as_ptr() as *const u8, cache_size) };
		try!(file.write(buf));
		Ok(())
	}
}

pub struct SeedHashCompute {
	prev_epoch: Cell<u64>,
	prev_seedhash: Cell<H256>,
}

impl SeedHashCompute {
	#[inline]
	pub fn new() -> SeedHashCompute {
		SeedHashCompute {
			prev_epoch: Cell::new(0),
			prev_seedhash: Cell::new([0u8; 32]),
		}
	}

	#[inline]
	fn reset_cache(&self) {
		self.prev_epoch.set(0);
		self.prev_seedhash.set([0u8; 32]);
	}

	#[inline]
	pub fn get_seedhash(&self, block_number: u64) -> H256 {
		let epoch = block_number / ETHASH_EPOCH_LENGTH;
		if epoch < self.prev_epoch.get() {
			// can't build on previous hash if requesting an older block
			self.reset_cache();
		}
		if epoch > self.prev_epoch.get() {
			let seed_hash = SeedHashCompute::resume_compute_seedhash(self.prev_seedhash.get(), self.prev_epoch.get(), epoch);
			self.prev_seedhash.set(seed_hash);
			self.prev_epoch.set(epoch);
		}
		self.prev_seedhash.get()
	}

	#[inline]
	pub fn resume_compute_seedhash(mut hash: H256, start_epoch: u64, end_epoch: u64) -> H256 {
		for _ in start_epoch..end_epoch {
			unsafe { sha3::sha3_256(hash[..].as_mut_ptr(), 32, hash[..].as_ptr(), 32) };
		}
		hash
	}
}


#[inline]
fn fnv_hash(x: u32, y: u32) -> u32 {
	return x.wrapping_mul(FNV_PRIME) ^ y;
}

#[inline]
fn sha3_512(input: &[u8], output: &mut [u8]) {
	unsafe { sha3::sha3_512(output.as_mut_ptr(), output.len(), input.as_ptr(), input.len()) };
}

#[inline]
fn get_cache_size(block_number: u64) -> usize {
	let mut sz: u64 = CACHE_BYTES_INIT + CACHE_BYTES_GROWTH * (block_number / ETHASH_EPOCH_LENGTH);
	sz = sz - NODE_BYTES as u64;
	while !is_prime(sz / NODE_BYTES as u64) {
		sz = sz - 2 * NODE_BYTES as u64;
	}
	sz as usize
}

#[inline]
fn get_data_size(block_number: u64) -> usize {
	let mut sz: u64 = DATASET_BYTES_INIT + DATASET_BYTES_GROWTH * (block_number / ETHASH_EPOCH_LENGTH);
	sz = sz - ETHASH_MIX_BYTES as u64;
	while !is_prime(sz / ETHASH_MIX_BYTES as u64) {
		sz = sz - 2 * ETHASH_MIX_BYTES as u64;
	}
	sz as usize
}


/// Difficulty quick check for POW preverification
///
/// `header_hash`      The hash of the header
/// `nonce`            The block's nonce
/// `mix_hash`         The mix digest hash
/// Boundary recovered from mix hash
pub fn quick_get_difficulty(header_hash: &H256, nonce: u64, mix_hash: &H256) -> H256 {
	let mut buf = [0u8; 64 + 32];
	unsafe { ptr::copy_nonoverlapping(header_hash.as_ptr(), buf.as_mut_ptr(), 32) };
	unsafe { ptr::copy_nonoverlapping(mem::transmute(&nonce), buf[32..].as_mut_ptr(), 8) };

	unsafe { sha3::sha3_512(buf.as_mut_ptr(), 64, buf.as_ptr(), 40) };
	unsafe { ptr::copy_nonoverlapping(mix_hash.as_ptr(), buf[64..].as_mut_ptr(), 32) };

	let mut hash = [0u8; 32];
	unsafe { sha3::sha3_256(hash.as_mut_ptr(), hash.len(), buf.as_ptr(), buf.len()) };
	hash.as_mut_ptr();
	hash
}

/// Calculate the light client data
/// `light` - The light client handler
/// `header_hash` - The header hash to pack into the mix
/// `nonce` - The nonce to pack into the mix
pub fn light_compute(light: &Light, header_hash: &H256, nonce: u64) -> ProofOfWork {
	let full_size = get_data_size(light.block_number);
	hash_compute(light, full_size, header_hash, nonce)
}

fn hash_compute(light: &Light, full_size: usize, header_hash: &H256, nonce: u64) -> ProofOfWork {
	if full_size % MIX_WORDS != 0 {
		panic!("Unaligned full size");
	}
	// pack hash and nonce together into first 40 bytes of s_mix
	let mut s_mix: [Node; MIX_NODES + 1] = [Node::default(), Node::default(), Node::default()];
	unsafe { ptr::copy_nonoverlapping(header_hash.as_ptr(), s_mix.get_unchecked_mut(0).bytes.as_mut_ptr(), 32) };
	unsafe { ptr::copy_nonoverlapping(mem::transmute(&nonce), s_mix.get_unchecked_mut(0).bytes[32..].as_mut_ptr(), 8) };

	// compute sha3-512 hash and replicate across mix
	unsafe {
		sha3::sha3_512(s_mix.get_unchecked_mut(0).bytes.as_mut_ptr(), NODE_BYTES, s_mix.get_unchecked(0).bytes.as_ptr(), 40);
		let (f_mix, mut mix) = s_mix.split_at_mut(1);
		for w in 0..MIX_WORDS {
			*mix.get_unchecked_mut(0).as_words_mut().get_unchecked_mut(w) = *f_mix.get_unchecked(0).as_words().get_unchecked(w % NODE_WORDS);
		}

		let page_size = 4 * MIX_WORDS;
		let num_full_pages = (full_size / page_size) as u32;

		for i in 0..(ETHASH_ACCESSES as u32) {
			let index = fnv_hash(f_mix.get_unchecked(0).as_words().get_unchecked(0) ^ i, *mix.get_unchecked(0).as_words().get_unchecked((i as usize) % MIX_WORDS)) % num_full_pages;
			for n in 0..MIX_NODES {
				let tmp_node = calculate_dag_item(index * MIX_NODES as u32 + n as u32, light);
				for w in 0..NODE_WORDS {
					*mix.get_unchecked_mut(n).as_words_mut().get_unchecked_mut(w) = fnv_hash(*mix.get_unchecked(n).as_words().get_unchecked(w), *tmp_node.as_words().get_unchecked(w));
				}
			}
		}

		// compress mix
		for i in 0..(MIX_WORDS / 4) {
			let w = i * 4;
			let mut reduction = *mix.get_unchecked(0).as_words().get_unchecked(w + 0);
			reduction = reduction.wrapping_mul(FNV_PRIME) ^ *mix.get_unchecked(0).as_words().get_unchecked(w + 1);
			reduction = reduction.wrapping_mul(FNV_PRIME) ^ *mix.get_unchecked(0).as_words().get_unchecked(w + 2);
			reduction = reduction.wrapping_mul(FNV_PRIME) ^ *mix.get_unchecked(0).as_words().get_unchecked(w + 3);
			*mix.get_unchecked_mut(0).as_words_mut().get_unchecked_mut(i) = reduction;
		}

		let mut mix_hash = [0u8; 32];
		let mut buf = [0u8; 32 + 64];
		ptr::copy_nonoverlapping(f_mix.get_unchecked_mut(0).bytes.as_ptr(), buf.as_mut_ptr(), 64);
		ptr::copy_nonoverlapping(mix.get_unchecked_mut(0).bytes.as_ptr(), buf[64..].as_mut_ptr(), 32);
		ptr::copy_nonoverlapping(mix.get_unchecked_mut(0).bytes.as_ptr(), mix_hash.as_mut_ptr(), 32);
		let mut value: H256 = [0u8; 32];
		sha3::sha3_256(value.as_mut_ptr(), value.len(), buf.as_ptr(), buf.len());
		ProofOfWork {
			mix_hash: mix_hash,
			value: value,
		}
	}
}

fn calculate_dag_item(node_index: u32, light: &Light) -> Node {
	unsafe {
		let num_parent_nodes = light.cache.len();
		let cache_nodes = &light.cache;
		let init = cache_nodes.get_unchecked(node_index as usize % num_parent_nodes);
		let mut ret = init.clone();
		*ret.as_words_mut().get_unchecked_mut(0) ^= node_index;
		sha3::sha3_512(ret.bytes.as_mut_ptr(), ret.bytes.len(), ret.bytes.as_ptr(), ret.bytes.len());

		for i in 0..ETHASH_DATASET_PARENTS {
			let parent_index = fnv_hash(node_index ^ i, *ret.as_words().get_unchecked(i as usize % NODE_WORDS)) % num_parent_nodes as u32;
			let parent = cache_nodes.get_unchecked(parent_index as usize);
			for w in 0..NODE_WORDS {
				*ret.as_words_mut().get_unchecked_mut(w) = fnv_hash(*ret.as_words().get_unchecked(w), *parent.as_words().get_unchecked(w));
			}
		}
		sha3::sha3_512(ret.bytes.as_mut_ptr(), ret.bytes.len(), ret.bytes.as_ptr(), ret.bytes.len());
		ret
	}
}

fn light_new(block_number: u64) -> Light {

	let seed_compute = SeedHashCompute::new();
	let seedhash = seed_compute.get_seedhash(block_number);
	let cache_size = get_cache_size(block_number);

	if cache_size % NODE_BYTES != 0 {
		panic!("Unaligned cache size");
	}
	let num_nodes = cache_size / NODE_BYTES;

	let mut nodes = Vec::with_capacity(num_nodes);
	nodes.resize(num_nodes, Node::default());
	unsafe {
		sha3_512(&seedhash[0..32], &mut nodes.get_unchecked_mut(0).bytes);
		for i in 1..num_nodes {
			sha3::sha3_512(nodes.get_unchecked_mut(i).bytes.as_mut_ptr(), NODE_BYTES, nodes.get_unchecked(i - 1).bytes.as_ptr(), NODE_BYTES);
		}

		for _ in 0..ETHASH_CACHE_ROUNDS {
			for i in 0..num_nodes {
				let idx = *nodes.get_unchecked_mut(i).as_words().get_unchecked(0) as usize % num_nodes;
				let mut data = nodes.get_unchecked((num_nodes - 1 + i) % num_nodes).clone();
				for w in 0..NODE_WORDS {
					*data.as_words_mut().get_unchecked_mut(w) ^= *nodes.get_unchecked(idx).as_words().get_unchecked(w);
				}
				sha3_512(&data.bytes, &mut nodes.get_unchecked_mut(i).bytes);
			}
		}
	}

	Light {
		cache: nodes,
		block_number: block_number,
		seed_compute: Mutex::new(seed_compute),
	}
}

static CHARS: &'static [u8] = b"0123456789abcdef";
fn to_hex(bytes: &[u8]) -> String {
	let mut v = Vec::with_capacity(bytes.len() * 2);
	for &byte in bytes.iter() {
		v.push(CHARS[(byte >> 4) as usize]);
		v.push(CHARS[(byte & 0xf) as usize]);
	}

	unsafe { String::from_utf8_unchecked(v) }
}

#[test]
fn test_get_cache_size() {
	// https://github.com/ethereum/wiki/wiki/Ethash/ef6b93f9596746a088ea95d01ca2778be43ae68f#data-sizes
	assert_eq!(16776896usize, get_cache_size(0));
	assert_eq!(16776896usize, get_cache_size(1));
	assert_eq!(16776896usize, get_cache_size(ETHASH_EPOCH_LENGTH - 1));
	assert_eq!(16907456usize, get_cache_size(ETHASH_EPOCH_LENGTH));
	assert_eq!(16907456usize, get_cache_size(ETHASH_EPOCH_LENGTH + 1));
	assert_eq!(284950208usize, get_cache_size(2046 * ETHASH_EPOCH_LENGTH));
	assert_eq!(285081536usize, get_cache_size(2047 * ETHASH_EPOCH_LENGTH));
	assert_eq!(285081536usize, get_cache_size(2048 * ETHASH_EPOCH_LENGTH - 1));
}

#[test]
fn test_get_data_size() {
	// https://github.com/ethereum/wiki/wiki/Ethash/ef6b93f9596746a088ea95d01ca2778be43ae68f#data-sizes
	assert_eq!(1073739904usize, get_data_size(0));
	assert_eq!(1073739904usize, get_data_size(1));
	assert_eq!(1073739904usize, get_data_size(ETHASH_EPOCH_LENGTH - 1));
	assert_eq!(1082130304usize, get_data_size(ETHASH_EPOCH_LENGTH));
	assert_eq!(1082130304usize, get_data_size(ETHASH_EPOCH_LENGTH + 1));
	assert_eq!(18236833408usize, get_data_size(2046 * ETHASH_EPOCH_LENGTH));
	assert_eq!(18245220736usize, get_data_size(2047 * ETHASH_EPOCH_LENGTH));
}

#[test]
fn test_difficulty_test() {
	let hash = [0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe, 0xe4, 0x0a, 0xb3, 0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f, 0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94, 0x05, 0x52, 0x7d, 0x72];
	let mix_hash = [0x1f, 0xff, 0x04, 0xce, 0xc9, 0x41, 0x73, 0xfd, 0x59, 0x1e, 0x3d, 0x89, 0x60, 0xce, 0x6b, 0xdf, 0x8b, 0x19, 0x71, 0x04, 0x8c, 0x71, 0xff, 0x93, 0x7b, 0xb2, 0xd3, 0x2a, 0x64, 0x31, 0xab, 0x6d];
	let nonce = 0xd7b3ac70a301a249;
	let boundary_good = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3e, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2, 0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a, 0xe9, 0x7e, 0x53, 0x84];
	assert_eq!(quick_get_difficulty(&hash, nonce, &mix_hash)[..], boundary_good[..]);
	let boundary_bad = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3a, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2, 0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a, 0xe9, 0x7e, 0x53, 0x84];
	assert!(quick_get_difficulty(&hash, nonce, &mix_hash)[..] != boundary_bad[..]);
}

#[test]
fn test_light_compute() {
	let hash = [0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe, 0xe4, 0x0a, 0xb3, 0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f, 0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94, 0x05, 0x52, 0x7d, 0x72];
	let mix_hash = [0x1f, 0xff, 0x04, 0xce, 0xc9, 0x41, 0x73, 0xfd, 0x59, 0x1e, 0x3d, 0x89, 0x60, 0xce, 0x6b, 0xdf, 0x8b, 0x19, 0x71, 0x04, 0x8c, 0x71, 0xff, 0x93, 0x7b, 0xb2, 0xd3, 0x2a, 0x64, 0x31, 0xab, 0x6d];
	let boundary = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3e, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2, 0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a, 0xe9, 0x7e, 0x53, 0x84];
	let nonce = 0xd7b3ac70a301a249;
	// difficulty = 0x085657254bd9u64;
	let light = Light::new(486382);
	let result = light_compute(&light, &hash, nonce);
	assert_eq!(result.mix_hash[..], mix_hash[..]);
	assert_eq!(result.value[..], boundary[..]);
}

#[test]
fn test_seed_compute_once() {
	let seed_compute = SeedHashCompute::new();
	let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
	assert_eq!(seed_compute.get_seedhash(486382), hash);
}

#[test]
fn test_seed_compute_zero() {
	let seed_compute = SeedHashCompute::new();
	assert_eq!(seed_compute.get_seedhash(0), [0u8; 32]);
}

#[test]
fn test_seed_compute_after_older() {
	let seed_compute = SeedHashCompute::new();
	// calculating an older value first shouldn't affect the result
	let _ = seed_compute.get_seedhash(50000);
	let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
	assert_eq!(seed_compute.get_seedhash(486382), hash);
}

#[test]
fn test_seed_compute_after_newer() {
	let seed_compute = SeedHashCompute::new();
	// calculating an newer value first shouldn't affect the result
	let _ = seed_compute.get_seedhash(972764);
	let hash = [241, 175, 44, 134, 39, 121, 245, 239, 228, 236, 43, 160, 195, 152, 46, 7, 199, 5, 253, 147, 241, 206, 98, 43, 3, 104, 17, 40, 192, 79, 106, 162];
	assert_eq!(seed_compute.get_seedhash(486382), hash);
}
