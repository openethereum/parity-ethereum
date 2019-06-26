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

//! Canonical hash trie definitions and helper functions.
//!
//! Each CHT is a trie mapping block numbers to canonical hashes and total difficulty.
//! One is generated for every `SIZE` blocks, allowing us to discard those blocks in
//! favor of the trie root. When the "ancient" blocks need to be accessed, we simply
//! request an inclusion proof of a specific block number against the trie with the
//! root has. A correct proof implies that the claimed block is identical to the one
//! we discarded.

use common_types::ids::BlockId;
use ethereum_types::{H256, U256};
use hash_db::HashDB;
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;
use memory_db::MemoryDB;
use journaldb::new_memory_db;
use bytes::Bytes;
use trie::{TrieMut, Trie, Recorder};
use ethtrie::{self, TrieDB, TrieDBMut};
use rlp::{RlpStream, Rlp};

// encode a key.
macro_rules! key {
	($num: expr) => { ::rlp::encode(&$num) }
}

macro_rules! val {
	($hash: expr, $td: expr) => {{
		let mut stream = RlpStream::new_list(2);
		stream.append(&$hash).append(&$td);
		stream.drain()
	}}
}

/// The size of each CHT.
pub const SIZE: u64 = 2048;

/// A canonical hash trie. This is generic over any database it can query.
/// See module docs for more details.
#[derive(Debug, Clone)]
pub struct CHT<DB: HashDB<KeccakHasher, DBValue>> {
	db: DB,
	root: H256, // the root of this CHT.
	number: u64,
}

impl<DB: HashDB<KeccakHasher, DBValue>> CHT<DB> {
	/// Query the root of the CHT.
	pub fn root(&self) -> H256 { self.root }

	/// Query the number of the CHT.
	pub fn number(&self) -> u64 { self.number }

	/// Generate an inclusion proof for the entry at a specific block.
	/// Nodes before level `from_level` will be omitted.
	/// Returns an error on an incomplete trie, and `Ok(None)` on an unprovable request.
	pub fn prove(&self, num: u64, from_level: u32) -> ethtrie::Result<Option<Vec<Bytes>>> {
		if block_to_cht_number(num) != Some(self.number) { return Ok(None) }

		let mut recorder = Recorder::with_depth(from_level);
		let db: &HashDB<_,_> = &self.db;
		let t = TrieDB::new(&db, &self.root)?;
		t.get_with(&key!(num), &mut recorder)?;

		Ok(Some(recorder.drain().into_iter().map(|x| x.data).collect()))
	}
}

/// Block information necessary to build a CHT.
pub struct BlockInfo {
	/// The block's hash.
	pub hash: H256,
	/// The block's parent's hash.
	pub parent_hash: H256,
	/// The block's total difficulty.
	pub total_difficulty: U256,
}

/// Build an in-memory CHT from a closure which provides necessary information
/// about blocks. If the fetcher ever fails to provide the info, the CHT
/// will not be generated.
pub fn build<F>(cht_num: u64, mut fetcher: F)
	-> Option<CHT<MemoryDB<KeccakHasher, memory_db::HashKey<KeccakHasher>, DBValue>>>
	where F: FnMut(BlockId) -> Option<BlockInfo>
{
	let mut db = new_memory_db();

	// start from the last block by number and work backwards.
	let last_num = start_number(cht_num + 1) - 1;
	let mut id = BlockId::Number(last_num);

	let mut root = H256::zero();

	{
		let mut t = TrieDBMut::new(&mut db, &mut root);
		for blk_num in (0..SIZE).map(|n| last_num - n) {
			let info = match fetcher(id) {
				Some(info) => info,
				None => return None,
			};

			id = BlockId::Hash(info.parent_hash);
			t.insert(&key!(blk_num), &val!(info.hash, info.total_difficulty))
				.expect("fresh in-memory database is infallible; qed");
		}
	}

	Some(CHT {
		db,
		root,
		number: cht_num,
	})
}

/// Compute a CHT root from an iterator of (hash, td) pairs. Fails if shorter than
/// SIZE items. The items are assumed to proceed sequentially from `start_number(cht_num)`.
/// Discards the trie's nodes.
pub fn compute_root<I>(cht_num: u64, iterable: I) -> Option<H256>
	where I: IntoIterator<Item=(H256, U256)>
{
	let mut v = Vec::with_capacity(SIZE as usize);
	let start_num = start_number(cht_num) as usize;

	for (i, (h, td)) in iterable.into_iter().take(SIZE as usize).enumerate() {
		v.push((key!(i + start_num), val!(h, td)))
	}

	if v.len() == SIZE as usize {
		Some(::triehash::trie_root(v))
	} else {
		None
	}
}

/// Check a proof for a CHT.
/// Given a set of a trie nodes, a number to query, and a trie root,
/// verify the given trie branch and extract the canonical hash and total difficulty.
// TODO: better support for partially-checked queries.
pub fn check_proof(proof: &[Bytes], num: u64, root: H256) -> Option<(H256, U256)> {
	let mut db = new_memory_db();

	for node in proof { db.insert(hash_db::EMPTY_PREFIX, &node[..]); }
	let res = match TrieDB::new(&db, &root) {
		Err(_) => return None,
		Ok(trie) => trie.get_with(&key!(num), |val: &[u8]| {
			let rlp = Rlp::new(val);
			rlp.val_at::<H256>(0)
				.and_then(|h| rlp.val_at::<U256>(1).map(|td| (h, td)))
				.ok()
		})
	};

	match res {
		Ok(Some(Some((hash, td)))) => Some((hash, td)),
		_ => None,
	}
}

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
	fn size_is_lt_usize() {
		// to ensure safe casting on the target platform.
		assert!(::cht::SIZE < usize::max_value() as u64)
	}

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
