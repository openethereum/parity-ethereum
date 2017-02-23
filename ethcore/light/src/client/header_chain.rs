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

//! Light client header chain.
//!
//! Unlike a full node's `BlockChain` this doesn't store much in the database.
//! It stores candidates for the last 2048-4096 blocks as well as CHT roots for
//! historical blocks all the way to the genesis.
//!
//! This is separate from the `BlockChain` for two reasons:
//!   - It stores only headers (and a pruned subset of them)
//!   - To allow for flexibility in the database layout once that's incorporated.
// TODO: use DB instead of memory. DB Layout: just the contents of `candidates`/`headers`
//

use std::collections::{BTreeMap, HashMap};

use cht;

use ethcore::block_status::BlockStatus;
use ethcore::error::BlockError;
use ethcore::encoded;
use ethcore::header::Header;
use ethcore::ids::BlockId;
use util::{H256, U256, HeapSizeOf, Mutex, RwLock};

use smallvec::SmallVec;

/// Store at least this many candidate headers at all times.
/// Also functions as the delay for computing CHTs as they aren't
/// relevant to any blocks we've got in memory.
const HISTORY: u64 = 2048;

/// Information about a block.
#[derive(Debug, Clone)]
pub struct BlockDescriptor {
	/// The block's hash
	pub hash: H256,
	/// The block's number
	pub number: u64,
	/// The block's total difficulty.
	pub total_difficulty: U256,
}

// candidate block description.
struct Candidate {
	hash: H256,
	parent_hash: H256,
	total_difficulty: U256,
}

struct Entry {
	candidates: SmallVec<[Candidate; 3]>, // 3 arbitrarily chosen
	canonical_hash: H256,
}

impl HeapSizeOf for Entry {
	fn heap_size_of_children(&self) -> usize {
		match self.candidates.spilled() {
			false => 0,
			true => self.candidates.capacity() * ::std::mem::size_of::<Candidate>(),
		}
	}
}

/// Header chain. See module docs for more details.
pub struct HeaderChain {
	genesis_header: encoded::Header, // special-case the genesis.
	candidates: RwLock<BTreeMap<u64, Entry>>,
	headers: RwLock<HashMap<H256, encoded::Header>>,
	best_block: RwLock<BlockDescriptor>,
	cht_roots: Mutex<Vec<H256>>,
}

impl HeaderChain {
	/// Create a new header chain given this genesis block.
	pub fn new(genesis: &[u8]) -> Self {
		use ethcore::views::HeaderView;

		let g_view = HeaderView::new(genesis);

		HeaderChain {
			genesis_header: encoded::Header::new(genesis.to_owned()),
			best_block: RwLock::new(BlockDescriptor {
				hash: g_view.hash(),
				number: 0,
				total_difficulty: g_view.difficulty(),
			}),
			candidates: RwLock::new(BTreeMap::new()),
			headers: RwLock::new(HashMap::new()),
			cht_roots: Mutex::new(Vec::new()),
		}
	}

	/// Insert a pre-verified header.
	///
	/// This blindly trusts that the data given to it is sensible.
	pub fn insert(&self, header: Header) -> Result<(), BlockError> {
		let hash = header.hash();
		let number = header.number();
		let parent_hash = *header.parent_hash();

		// hold candidates the whole time to guard import order.
		let mut candidates = self.candidates.write();

		// find parent details.
		let parent_td =
			if number == 1 {
				self.genesis_header.difficulty()
			} else {
				candidates.get(&(number - 1))
					.and_then(|entry| entry.candidates.iter().find(|c| c.hash == parent_hash))
					.map(|c| c.total_difficulty)
					.ok_or_else(|| BlockError::UnknownParent(parent_hash))?
			};

		let total_difficulty = parent_td + *header.difficulty();

		// insert headers and candidates entries.
		candidates.entry(number).or_insert_with(|| Entry { candidates: SmallVec::new(), canonical_hash: hash })
			.candidates.push(Candidate {
				hash: hash,
				parent_hash: parent_hash,
				total_difficulty: total_difficulty,
		});

		let raw = ::rlp::encode(&header).to_vec();
		self.headers.write().insert(hash, encoded::Header::new(raw));

		// reorganize ancestors so canonical entries are first in their
		// respective candidates vectors.
		if self.best_block.read().total_difficulty < total_difficulty {
			let mut canon_hash = hash;
			for (&height, entry) in candidates.iter_mut().rev().skip_while(|&(height, _)| *height > number) {
				if height != number && entry.canonical_hash == canon_hash { break; }

				trace!(target: "chain", "Setting new canonical block {} for block height {}",
					canon_hash, height);

				let canon_pos = entry.candidates.iter().position(|x| x.hash == canon_hash)
					.expect("blocks are only inserted if parent is present; or this is the block we just added; qed");

				// move the new canonical entry to the front and set the
				// era's canonical hash.
				entry.candidates.swap(0, canon_pos);
				entry.canonical_hash = canon_hash;

				// what about reorgs > cht::SIZE + HISTORY?
				// resetting to the last block of a given CHT should be possible.
				canon_hash = entry.candidates[0].parent_hash;
			}

			trace!(target: "chain", "New best block: ({}, {}), TD {}", number, hash, total_difficulty);
			*self.best_block.write() = BlockDescriptor {
				hash: hash,
				number: number,
				total_difficulty: total_difficulty,
			};

			// produce next CHT root if it's time.
			let earliest_era = *candidates.keys().next().expect("at least one era just created; qed");
			if earliest_era + HISTORY + cht::SIZE <= number {
				let cht_num = cht::block_to_cht_number(earliest_era)
					.expect("fails only for number == 0; genesis never imported; qed");
				debug_assert_eq!(cht_num as usize, self.cht_roots.lock().len());

				let mut headers = self.headers.write();

				let cht_root = {
					let mut i = earliest_era;

					// iterable function which removes the candidates as it goes
					// along. this will only be called until the CHT is complete.
					let iter = || {
						let era_entry = candidates.remove(&i)
							.expect("all eras are sequential with no gaps; qed");
						i += 1;

						for ancient in &era_entry.candidates {
							headers.remove(&ancient.hash);
						}

						let canon = &era_entry.candidates[0];
						(canon.hash, canon.total_difficulty)
					};
					cht::compute_root(cht_num, ::itertools::repeat_call(iter))
						.expect("fails only when too few items; this is checked; qed")
				};

				debug!(target: "chain", "Produced CHT {} root: {:?}", cht_num, cht_root);

				self.cht_roots.lock().push(cht_root);
			}
		}

		Ok(())
	}

	/// Get a block header. In the case of query by number, only canonical blocks
	/// will be returned.
	pub fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		match id {
			BlockId::Earliest | BlockId::Number(0) => Some(self.genesis_header.clone()),
			BlockId::Hash(hash) => self.headers.read().get(&hash).cloned(),
			BlockId::Number(num) => {
				if self.best_block.read().number < num { return None }

				self.candidates.read().get(&num).map(|entry| entry.canonical_hash)
					.and_then(|hash| self.headers.read().get(&hash).cloned())
			}
			BlockId::Latest | BlockId::Pending => {
				let hash = {
					let best = self.best_block.read();
					if best.number == 0 {
						return Some(self.genesis_header.clone())
					}

					best.hash
				};

				self.headers.read().get(&hash).cloned()
			}
		}
	}

	/// Get the best block's header.
	pub fn best_header(&self) -> encoded::Header {
		self.block_header(BlockId::Latest).expect("Header for best block always stored; qed")
	}

	/// Get an iterator over a block and its ancestry.
	pub fn ancestry_iter(&self, start: BlockId) -> AncestryIter {
		AncestryIter {
			next: self.block_header(start),
			chain: self,
		}
	}

	/// Get the nth CHT root, if it's been computed.
	///
	/// CHT root 0 is from block `1..2048`.
	/// CHT root 1 is from block `2049..4096`
	/// and so on.
	///
	/// This is because it's assumed that the genesis hash is known,
	/// so including it within a CHT would be redundant.
	pub fn cht_root(&self, n: usize) -> Option<H256> {
		self.cht_roots.lock().get(n).map(|h| h.clone())
	}

	/// Get the genesis hash.
	pub fn genesis_hash(&self) -> H256 {
		::util::Hashable::sha3(&self.genesis_header)
	}

	/// Get the best block's data.
	pub fn best_block(&self) -> BlockDescriptor {
		self.best_block.read().clone()
	}

	/// If there is a gap between the genesis and the rest
	/// of the stored blocks, return the first post-gap block.
	pub fn first_block(&self) -> Option<BlockDescriptor> {
		let candidates = self.candidates.read();
		match candidates.iter().next() {
			None | Some((&1, _)) => None,
			Some((&height, entry)) => Some(BlockDescriptor {
				number: height,
				hash: entry.canonical_hash,
				total_difficulty: entry.candidates.iter().find(|x| x.hash == entry.canonical_hash)
					.expect("entry always stores canonical candidate; qed").total_difficulty,
			})
		}
	}

	/// Get block status.
	pub fn status(&self, hash: &H256) -> BlockStatus {
		match self.headers.read().contains_key(hash) {
			true => BlockStatus::InChain,
			false => BlockStatus::Unknown,
		}
	}
}

impl HeapSizeOf for HeaderChain {
	fn heap_size_of_children(&self) -> usize {
		self.candidates.read().heap_size_of_children() +
			self.headers.read().heap_size_of_children() +
			self.cht_roots.lock().heap_size_of_children()
	}
}

/// Iterator over a block's ancestry.
pub struct AncestryIter<'a> {
	next: Option<encoded::Header>,
	chain: &'a HeaderChain,
}

impl<'a> Iterator for AncestryIter<'a> {
	type Item = encoded::Header;

	fn next(&mut self) -> Option<encoded::Header> {
		let next = self.next.take();
		if let Some(p_hash) = next.as_ref().map(|hdr| hdr.parent_hash()) {
			self.next = self.chain.block_header(BlockId::Hash(p_hash));
		}

		next
	}
}

#[cfg(test)]
mod tests {
	use super::HeaderChain;
	use ethcore::ids::BlockId;
	use ethcore::header::Header;
	use ethcore::spec::Spec;

	#[test]
	fn basic_chain() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();

		let chain = HeaderChain::new(&::rlp::encode(&genesis_header));

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..10000 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i.into());
			parent_hash = header.hash();

			chain.insert(header).unwrap();

			rolling_timestamp += 10;
		}

		assert!(chain.block_header(BlockId::Number(10)).is_none());
		assert!(chain.block_header(BlockId::Number(9000)).is_some());
		assert!(chain.cht_root(2).is_some());
		assert!(chain.cht_root(3).is_none());
	}

	#[test]
	fn reorganize() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();

		let chain = HeaderChain::new(&::rlp::encode(&genesis_header));

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..6 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i.into());
			parent_hash = header.hash();

			chain.insert(header).unwrap();

			rolling_timestamp += 10;
		}

		{
			let mut rolling_timestamp = rolling_timestamp;
			let mut parent_hash = parent_hash;
			for i in 6..16 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i.into());
				parent_hash = header.hash();

				chain.insert(header).unwrap();

				rolling_timestamp += 10;
			}
		}

		assert_eq!(chain.best_block().number, 15);

		{
			let mut rolling_timestamp = rolling_timestamp;
			let mut parent_hash = parent_hash;

			// import a shorter chain which has better TD.
			for i in 6..13 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * (i * i).into());
				parent_hash = header.hash();

				chain.insert(header).unwrap();

				rolling_timestamp += 11;
			}
		}

		let (mut num, mut canon_hash) = (chain.best_block().number, chain.best_block().hash);
		assert_eq!(num, 12);

		while num > 0 {
			let header = chain.block_header(BlockId::Number(num)).unwrap();
			assert_eq!(header.hash(), canon_hash);

			canon_hash = header.parent_hash();
			num -= 1;
		}
	}

	#[test]
	fn earliest_is_latest() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();

		let chain = HeaderChain::new(&::rlp::encode(&genesis_header));

		assert!(chain.block_header(BlockId::Earliest).is_some());
		assert!(chain.block_header(BlockId::Latest).is_some());
		assert!(chain.block_header(BlockId::Pending).is_some());
	}
}
