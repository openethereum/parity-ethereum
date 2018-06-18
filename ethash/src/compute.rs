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

//! Ethash implementation
//! See https://github.com/ethereum/wiki/wiki/Ethash

// TODO: fix endianess for big endian

use keccak::{keccak_512, keccak_256, H256};
use cache::{NodeCache, NodeCacheBuilder};
use seed_compute::SeedHashCompute;
use shared::*;
use std::io;

use std::{mem, ptr};
use std::path::Path;

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

pub struct Light {
	block_number: u64,
	cache: NodeCache,
}

/// Light cache structure
impl Light {
	pub fn new_with_builder(
		builder: &NodeCacheBuilder,
		cache_dir: &Path,
		block_number: u64,
	) -> Self {
		let cache = builder.new_cache(cache_dir.to_path_buf(), block_number);

		Light {
			block_number: block_number,
			cache: cache,
		}
	}

	/// Calculate the light boundary data
	/// `header_hash` - The header hash to pack into the mix
	/// `nonce` - The nonce to pack into the mix
	pub fn compute(&self, header_hash: &H256, nonce: u64) -> ProofOfWork {
		light_compute(self, header_hash, nonce)
	}

	pub fn from_file_with_builder(
		builder: &NodeCacheBuilder,
		cache_dir: &Path,
		block_number: u64,
	) -> io::Result<Self> {
		let cache = builder.from_file(cache_dir.to_path_buf(), block_number)?;
		Ok(Light {
			block_number: block_number,
			cache: cache,
		})
	}

	pub fn to_file(&mut self) -> io::Result<&Path> {
		self.cache.flush()?;
		Ok(self.cache.cache_path())
	}
}

pub fn slow_hash_block_number(block_number: u64) -> H256 {
	SeedHashCompute::resume_compute_seedhash([0u8; 32], 0, block_number / ETHASH_EPOCH_LENGTH)
}

fn fnv_hash(x: u32, y: u32) -> u32 {
	return x.wrapping_mul(FNV_PRIME) ^ y;
}

/// Difficulty quick check for POW preverification
///
/// `header_hash`      The hash of the header
/// `nonce`            The block's nonce
/// `mix_hash`         The mix digest hash
/// Boundary recovered from mix hash
pub fn quick_get_difficulty(header_hash: &H256, nonce: u64, mix_hash: &H256) -> H256 {
	unsafe {
		// This is safe - the `keccak_512` call below reads the first 40 bytes (which we explicitly set
		// with two `copy_nonoverlapping` calls) but writes the first 64, and then we explicitly write
		// the next 32 bytes before we read the whole thing with `keccak_256`.
		//
		// This cannot be elided by the compiler as it doesn't know the implementation of
		// `keccak_512`.
		let mut buf: [u8; 64 + 32] = mem::uninitialized();

		ptr::copy_nonoverlapping(header_hash.as_ptr(), buf.as_mut_ptr(), 32);
		ptr::copy_nonoverlapping(&nonce as *const u64 as *const u8, buf[32..].as_mut_ptr(), 8);

		keccak_512::unchecked(buf.as_mut_ptr(), 64, buf.as_ptr(), 40);
		ptr::copy_nonoverlapping(mix_hash.as_ptr(), buf[64..].as_mut_ptr(), 32);

		// This is initialized in `keccak_256`
		let mut hash: [u8; 32] = mem::uninitialized();
		keccak_256::unchecked(hash.as_mut_ptr(), hash.len(), buf.as_ptr(), buf.len());

		hash
	}
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
	macro_rules! make_const_array {
		($n:expr, $value:expr) => {{
			// We use explicit lifetimes to ensure that val's borrow is invalidated until the
			// transmuted val dies.
			unsafe fn make_const_array<T, U>(val: &mut [T]) -> &mut [U; $n] {
				use ::std::mem;

				debug_assert_eq!(val.len() * mem::size_of::<T>(), $n * mem::size_of::<U>());
 				&mut *(val.as_mut_ptr() as *mut [U; $n])
			}

			make_const_array($value)
		}}
	}

	#[repr(C)]
	struct MixBuf {
		half_mix: Node,
		compress_bytes: [u8; MIX_WORDS],
	};

	if full_size % MIX_WORDS != 0 {
		panic!("Unaligned full size");
	}

	// You may be asking yourself: what in the name of Crypto Jesus is going on here? So: we need
	// `half_mix` and `compress_bytes` in a single array later down in the code (we hash them
	// together to create `value`) so that we can hash the full array. However, we do a bunch of
	// reading and writing to these variables first. We originally allocated two arrays and then
	// stuck them together with `ptr::copy_nonoverlapping` at the end, but this method is
	// _significantly_ faster - by my benchmarks, a consistent 3-5%. This is the most ridiculous
	// optimization I have ever done and I am so sorry. I can only chalk it up to cache locality
	// improvements, since I can't imagine that 3-5% of our runtime is taken up by catting two
	// arrays together.
	let mut buf: MixBuf = MixBuf {
		half_mix: unsafe {
			// Pack `header_hash` and `nonce` together
			// We explicitly write the first 40 bytes, leaving the last 24 as uninitialized. Then
			// `keccak_512` reads the first 40 bytes (4th parameter) and overwrites the entire array,
			// leaving it fully initialized.
			let mut out: [u8; NODE_BYTES] = mem::uninitialized();

			ptr::copy_nonoverlapping(header_hash.as_ptr(), out.as_mut_ptr(), header_hash.len());
			ptr::copy_nonoverlapping(
				&nonce as *const u64 as *const u8,
				out[header_hash.len()..].as_mut_ptr(),
				mem::size_of::<u64>(),
			);

			// compute keccak-512 hash and replicate across mix
			keccak_512::unchecked(
				out.as_mut_ptr(),
				NODE_BYTES,
				out.as_ptr(),
				header_hash.len() + mem::size_of::<u64>(),
			);

			Node { bytes: out }
		},
		// This is fully initialized before being read, see `let mut compress = ...` below
		compress_bytes: unsafe { mem::uninitialized() },
	};

	let mut mix: [_; MIX_NODES] = [buf.half_mix.clone(), buf.half_mix.clone()];

	let page_size = 4 * MIX_WORDS;
	let num_full_pages = (full_size / page_size) as u32;
	// deref once for better performance
	let cache: &[Node] = light.cache.as_ref();
	let first_val = buf.half_mix.as_words()[0];

	debug_assert_eq!(MIX_NODES, 2);
	debug_assert_eq!(NODE_WORDS, 16);

	for i in 0..ETHASH_ACCESSES as u32 {
		let index = {
			// This is trivially safe, but does not work on big-endian. The safety of this is
			// asserted in debug builds (see the definition of `make_const_array!`).
			let mix_words: &mut [u32; MIX_WORDS] =
				unsafe { make_const_array!(MIX_WORDS, &mut mix) };

			fnv_hash(first_val ^ i, mix_words[i as usize % MIX_WORDS]) % num_full_pages
		};

		unroll! {
			// MIX_NODES
			for n in 0..2 {
				let tmp_node = calculate_dag_item(
					index * MIX_NODES as u32 + n as u32,
					cache,
				);

				unroll! {
					// NODE_WORDS
					for w in 0..16 {
						mix[n].as_words_mut()[w] =
							fnv_hash(
								mix[n].as_words()[w],
								tmp_node.as_words()[w],
							);
					}
				}
			}
		}
	}

	let mix_words: [u32; MIX_WORDS] = unsafe { mem::transmute(mix) };

	{
		// This is an uninitialized buffer to begin with, but we iterate precisely `compress.len()`
		// times and set each index, leaving the array fully initialized. THIS ONLY WORKS ON LITTLE-
		// ENDIAN MACHINES. See a future PR to make this and the rest of the code work correctly on
		// big-endian arches like mips.
		let compress: &mut [u32; MIX_WORDS / 4] =
			unsafe { make_const_array!(MIX_WORDS / 4, &mut buf.compress_bytes) };

		// Compress mix
		debug_assert_eq!(MIX_WORDS / 4, 8);
		unroll! {
			for i in 0..8 {
				let w = i * 4;

				let mut reduction = mix_words[w + 0];
				reduction = reduction.wrapping_mul(FNV_PRIME) ^ mix_words[w + 1];
				reduction = reduction.wrapping_mul(FNV_PRIME) ^ mix_words[w + 2];
				reduction = reduction.wrapping_mul(FNV_PRIME) ^ mix_words[w + 3];
				compress[i] = reduction;
			}
		}
	}

	let mix_hash = buf.compress_bytes;

	let value: H256 = {
		// We can interpret the buffer as an array of `u8`s, since it's `repr(C)`.
		let read_ptr: *const u8 = &buf as *const MixBuf as *const u8;
		// We overwrite the second half since `keccak_256` has an internal buffer and so allows
		// overlapping arrays as input.
		let write_ptr: *mut u8 = &mut buf.compress_bytes as *mut [u8; 32] as *mut u8;
		unsafe { 
			keccak_256::unchecked(
				write_ptr,
				buf.compress_bytes.len(),
				read_ptr,
				buf.half_mix.bytes.len() + buf.compress_bytes.len(),
			);
		}
		buf.compress_bytes
	};

	ProofOfWork { mix_hash: mix_hash, value: value }
}

// TODO: Use the `simd` crate
fn calculate_dag_item(node_index: u32, cache: &[Node]) -> Node {
	let num_parent_nodes = cache.len();
	let mut ret = cache[node_index as usize % num_parent_nodes].clone();
	ret.as_words_mut()[0] ^= node_index;

	keccak_512::inplace(ret.as_bytes_mut());

	debug_assert_eq!(NODE_WORDS, 16);
	for i in 0..ETHASH_DATASET_PARENTS as u32 {
		let parent_index = fnv_hash(node_index ^ i, ret.as_words()[i as usize % NODE_WORDS]) %
			num_parent_nodes as u32;
		let parent = &cache[parent_index as usize];

		unroll! {
			for w in 0..16 {
				ret.as_words_mut()[w] = fnv_hash(ret.as_words()[w], parent.as_words()[w]);
			}
		}
	}

	keccak_512::inplace(ret.as_bytes_mut());

	ret
}

#[cfg(test)]
mod test {
	use super::*;
	use std::fs;
	use tempdir::TempDir;

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
		let hash = [
			0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe, 0xe4, 0x0a, 0xb3,
			0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f, 0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94,
			0x05, 0x52, 0x7d, 0x72,
		];
		let mix_hash = [
			0x1f, 0xff, 0x04, 0xce, 0xc9, 0x41, 0x73, 0xfd, 0x59, 0x1e, 0x3d, 0x89, 0x60, 0xce,
			0x6b, 0xdf, 0x8b, 0x19, 0x71, 0x04, 0x8c, 0x71, 0xff, 0x93, 0x7b, 0xb2, 0xd3, 0x2a,
			0x64, 0x31, 0xab, 0x6d,
		];
		let nonce = 0xd7b3ac70a301a249;
		let boundary_good = [
			0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3e, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2,
			0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a,
			0xe9, 0x7e, 0x53, 0x84,
		];
		assert_eq!(quick_get_difficulty(&hash, nonce, &mix_hash)[..], boundary_good[..]);
		let boundary_bad = [
			0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3a, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2,
			0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a,
			0xe9, 0x7e, 0x53, 0x84,
		];
		assert!(quick_get_difficulty(&hash, nonce, &mix_hash)[..] != boundary_bad[..]);
	}

	#[test]
	fn test_light_compute() {
		let hash = [
			0xf5, 0x7e, 0x6f, 0x3a, 0xcf, 0xc0, 0xdd, 0x4b, 0x5b, 0xf2, 0xbe, 0xe4, 0x0a, 0xb3,
			0x35, 0x8a, 0xa6, 0x87, 0x73, 0xa8, 0xd0, 0x9f, 0x5e, 0x59, 0x5e, 0xab, 0x55, 0x94,
			0x05, 0x52, 0x7d, 0x72,
		];
		let mix_hash = [
			0x1f, 0xff, 0x04, 0xce, 0xc9, 0x41, 0x73, 0xfd, 0x59, 0x1e, 0x3d, 0x89, 0x60, 0xce,
			0x6b, 0xdf, 0x8b, 0x19, 0x71, 0x04, 0x8c, 0x71, 0xff, 0x93, 0x7b, 0xb2, 0xd3, 0x2a,
			0x64, 0x31, 0xab, 0x6d,
		];
		let boundary = [
			0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3e, 0x9b, 0x6c, 0x69, 0xbc, 0x2c, 0xe2, 0xa2,
			0x4a, 0x8e, 0x95, 0x69, 0xef, 0xc7, 0xd7, 0x1b, 0x33, 0x35, 0xdf, 0x36, 0x8c, 0x9a,
			0xe9, 0x7e, 0x53, 0x84,
		];
		let nonce = 0xd7b3ac70a301a249;

		let tempdir = TempDir::new("").unwrap();
		// difficulty = 0x085657254bd9u64;
		let light = NodeCacheBuilder::new(None).light(tempdir.path(), 486382);
		let result = light_compute(&light, &hash, nonce);
		assert_eq!(result.mix_hash[..], mix_hash[..]);
		assert_eq!(result.value[..], boundary[..]);
	}

	#[test]
	fn test_drop_old_data() {
		let tempdir = TempDir::new("").unwrap();
		let builder = NodeCacheBuilder::new(None);
		let first = builder.light(tempdir.path(), 0).to_file().unwrap().to_owned();

		let second = builder.light(tempdir.path(), ETHASH_EPOCH_LENGTH).to_file().unwrap().to_owned();
		assert!(fs::metadata(&first).is_ok());

		let _ = builder.light(tempdir.path(), ETHASH_EPOCH_LENGTH * 2).to_file();
		assert!(fs::metadata(&first).is_err());
		assert!(fs::metadata(&second).is_ok());

		let _ = builder.light(tempdir.path(), ETHASH_EPOCH_LENGTH * 3).to_file();
		assert!(fs::metadata(&second).is_err());
	}
}
