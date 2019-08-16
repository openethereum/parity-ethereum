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

use compute::Light;
use either::Either;
use keccak::{H256, keccak_512};
use memmap::MmapMut;
use parking_lot::Mutex;
use seed_compute::SeedHashCompute;

use shared::{ETHASH_CACHE_ROUNDS, NODE_BYTES, NODE_DWORDS, Node, epoch, get_cache_size, to_hex};

use std::borrow::Cow;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::slice;
use std::sync::Arc;

use common_types::engines::OptimizeFor;

type Cache = Either<Vec<Node>, MmapMut>;

fn byte_size(cache: &Cache) -> usize {
	use self::Either::{Left, Right};

	match *cache {
		Left(ref vec) => vec.len() * NODE_BYTES,
		Right(ref mmap) => mmap.len(),
	}
}

fn new_buffer(path: &Path, num_nodes: usize, ident: &H256, optimize_for: OptimizeFor) -> Cache {
	let memmap = match optimize_for {
		OptimizeFor::Cpu => None,
		OptimizeFor::Memory => make_memmapped_cache(path, num_nodes, ident).ok(),
	};

	memmap.map(Either::Right).unwrap_or_else(|| {
		Either::Left(make_memory_cache(num_nodes, ident))
	})
}

#[derive(Clone)]
pub struct NodeCacheBuilder {
	// TODO: Remove this locking and just use an `Rc`?
	seedhash: Arc<Mutex<SeedHashCompute>>,
	optimize_for: OptimizeFor,
	progpow_transition: u64,
}

// TODO: Abstract the "optimize for" logic
pub struct NodeCache {
	builder: NodeCacheBuilder,
	cache_dir: Cow<'static, Path>,
	cache_path: PathBuf,
	epoch: u64,
	cache: Cache,
}

impl NodeCacheBuilder {
	pub fn light(&self, cache_dir: &Path, block_number: u64) -> Light {
		Light::new_with_builder(self, cache_dir, block_number, self.progpow_transition)
	}

	pub fn light_from_file(&self, cache_dir: &Path, block_number: u64) -> io::Result<Light> {
		Light::from_file_with_builder(self, cache_dir, block_number, self.progpow_transition)
	}

	pub fn new<T: Into<Option<OptimizeFor>>>(optimize_for: T, progpow_transition: u64) -> Self {
		NodeCacheBuilder {
			seedhash: Arc::new(Mutex::new(SeedHashCompute::default())),
			optimize_for: optimize_for.into().unwrap_or_default(),
			progpow_transition
		}
	}

	fn block_number_to_ident(&self, block_number: u64) -> H256 {
		self.seedhash.lock().hash_block_number(block_number)
	}

	fn epoch_to_ident(&self, epoch: u64) -> H256 {
		self.seedhash.lock().hash_epoch(epoch)
	}

	pub fn from_file<P: Into<Cow<'static, Path>>>(
		&self,
		cache_dir: P,
		block_number: u64,
	) -> io::Result<NodeCache> {
		let cache_dir = cache_dir.into();
		let ident = self.block_number_to_ident(block_number);

		let path = cache_path(cache_dir.as_ref(), &ident);

		let cache = cache_from_path(&path, self.optimize_for)?;
		let expected_cache_size = get_cache_size(block_number);

		if byte_size(&cache) == expected_cache_size {
			Ok(NodeCache {
				builder: self.clone(),
				epoch: epoch(block_number),
				cache_dir: cache_dir,
				cache_path: path,
				cache: cache,
			})
		} else {
			Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Node cache is of incorrect size",
			))
		}
	}

	pub fn new_cache<P: Into<Cow<'static, Path>>>(
		&self,
		cache_dir: P,
		block_number: u64,
	) -> NodeCache {
		let cache_dir = cache_dir.into();
		let ident = self.block_number_to_ident(block_number);

		let cache_size = get_cache_size(block_number);

		// We use `debug_assert` since it is impossible for `get_cache_size` to return an unaligned
		// value with the current implementation. If the implementation changes, CI will catch it.
		debug_assert!(cache_size % NODE_BYTES == 0, "Unaligned cache size");
		let num_nodes = cache_size / NODE_BYTES;

		let path = cache_path(cache_dir.as_ref(), &ident);
		let nodes = new_buffer(&path, num_nodes, &ident, self.optimize_for);

		NodeCache {
			builder: self.clone(),
			epoch: epoch(block_number),
			cache_dir: cache_dir.into(),
			cache_path: path,
			cache: nodes,
		}
	}
}

impl NodeCache {
	pub fn cache_path(&self) -> &Path {
		&self.cache_path
	}

	pub fn flush(&mut self) -> io::Result<()> {
		if let Some(last) = self.epoch.checked_sub(2).map(|ep| {
			cache_path(self.cache_dir.as_ref(), &self.builder.epoch_to_ident(ep))
		})
		{
			fs::remove_file(last).unwrap_or_else(|error| match error.kind() {
				io::ErrorKind::NotFound => (),
				_ => warn!("Error removing stale DAG cache: {:?}", error),
			});
		}

		consume_cache(&mut self.cache, &self.cache_path)
	}
}

fn make_memmapped_cache(path: &Path, num_nodes: usize, ident: &H256) -> io::Result<MmapMut> {
	use std::fs::OpenOptions;

	let file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(&path)?;
	file.set_len((num_nodes * NODE_BYTES) as _)?;

	let mut memmap = unsafe { MmapMut::map_mut(&file)? };

	unsafe { initialize_memory(memmap.as_mut_ptr() as *mut Node, num_nodes, ident) };

	Ok(memmap)
}

fn make_memory_cache(num_nodes: usize, ident: &H256) -> Vec<Node> {
	let mut nodes: Vec<Node> = Vec::with_capacity(num_nodes);
	// Use uninit instead of unnecessarily writing `size_of::<Node>() * num_nodes` 0s
	unsafe {
		initialize_memory(nodes.as_mut_ptr(), num_nodes, ident);
		nodes.set_len(num_nodes);
	}

	nodes
}

fn cache_path<'a, P: Into<Cow<'a, Path>>>(path: P, ident: &H256) -> PathBuf {
	let mut buf = path.into().into_owned();
	buf.push(to_hex(ident));
	buf
}

fn consume_cache(cache: &mut Cache, path: &Path) -> io::Result<()> {
	use std::fs::OpenOptions;

	match *cache {
		Either::Left(ref mut vec) => {
			let mut file = OpenOptions::new()
				.read(true)
				.write(true)
				.create(true)
				.open(&path)?;

			let buf = unsafe {
				slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, vec.len() * NODE_BYTES)
			};

			file.write_all(buf).map(|_| ())
		}
		Either::Right(ref mmap) => {
			mmap.flush()
		}
	}
}

fn cache_from_path(path: &Path, optimize_for: OptimizeFor) -> io::Result<Cache> {
	let memmap = match optimize_for {
		OptimizeFor::Cpu => None,
		OptimizeFor::Memory => {
			let file = fs::OpenOptions::new().read(true).write(true).create(true).open(path)?;
			unsafe { MmapMut::map_mut(&file).ok() }
		},
	};

	memmap.map(Either::Right).ok_or(()).or_else(|_| {
		read_from_path(path).map(Either::Left)
	})
}

fn read_from_path(path: &Path) -> io::Result<Vec<Node>> {
	use std::fs::File;
	use std::mem;

	let mut file = File::open(path)?;

	let mut nodes: Vec<u8> = Vec::with_capacity(file.metadata().map(|m| m.len() as _).unwrap_or(
		NODE_BYTES * 1_000_000,
	));
	file.read_to_end(&mut nodes)?;

	nodes.shrink_to_fit();

	if nodes.len() % NODE_BYTES != 0 || nodes.capacity() % NODE_BYTES != 0 {
		return Err(io::Error::new(
			io::ErrorKind::Other,
			"Node cache is not a multiple of node size",
		));
	}

	let out: Vec<Node> = unsafe {
		Vec::from_raw_parts(
			nodes.as_mut_ptr() as *mut _,
			nodes.len() / NODE_BYTES,
			nodes.capacity() / NODE_BYTES,
		)
	};

	mem::forget(nodes);

	Ok(out)
}

impl AsRef<[Node]> for NodeCache {
	fn as_ref(&self) -> &[Node] {
		match self.cache {
			Either::Left(ref vec) => vec,
			Either::Right(ref mmap) => unsafe {
				let bytes = mmap.as_ptr();
				// This isn't a safety issue, so we can keep this a debug lint. We don't care about
				// people manually messing with the files unless it can cause unsafety, but if we're
				// generating incorrect files then we want to catch that in CI.
				debug_assert_eq!(mmap.len() % NODE_BYTES, 0);
				slice::from_raw_parts(bytes as _, mmap.len() / NODE_BYTES)
			},
		}
	}
}

// This takes a raw pointer and a counter because `memory` may be uninitialized. `memory` _must_ be
// a pointer to the beginning of an allocated but possibly-uninitialized block of
// `num_nodes * NODE_BYTES` bytes
//
// We have to use raw pointers to read/write uninit, using "normal" indexing causes LLVM to freak
// out. It counts as a read and causes all writes afterwards to be elided. Yes, really. I know, I
// want to refactor this to use less `unsafe` as much as the next rustacean.
unsafe fn initialize_memory(memory: *mut Node, num_nodes: usize, ident: &H256) {
	let dst = memory as *mut u8;

	debug_assert_eq!(ident.len(), 32);
	keccak_512::unchecked(dst, NODE_BYTES, ident.as_ptr(), ident.len());

	for i in 1..num_nodes {
		// We use raw pointers here, see above
		let dst = memory.offset(i as _) as *mut u8;
		let src = memory.offset(i as isize - 1) as *mut u8;

		keccak_512::unchecked(dst, NODE_BYTES, src, NODE_BYTES);
	}

	// Now this is initialized, we can treat it as a slice.
	let nodes: &mut [Node] = slice::from_raw_parts_mut(memory, num_nodes);

	// For `unroll!`, see below. If the literal in `unroll!` is not the same as the RHS here then
	// these have got out of sync! Don't let this happen!
	debug_assert_eq!(NODE_DWORDS, 8);

	// This _should_ get unrolled by the compiler, since it's not using the loop variable.
	for _ in 0..ETHASH_CACHE_ROUNDS {
		for i in 0..num_nodes {
			let data_idx = (num_nodes - 1 + i) % num_nodes;
			let idx = nodes.get_unchecked_mut(i).as_words()[0] as usize % num_nodes;

			let data = {
				let mut data: Node = nodes.get_unchecked(data_idx).clone();
				let rhs: &Node = nodes.get_unchecked(idx);

				unroll! {
					for w in 0..8 {
						*data.as_dwords_mut().get_unchecked_mut(w) ^=
							*rhs.as_dwords().get_unchecked(w);
					}
				}

				data
			};

			keccak_512::write(&data.bytes, &mut nodes.get_unchecked_mut(i).bytes);
		}
	}
}
