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

use std::collections::BTreeMap;
use std::sync::Arc;

use cht;

use ethcore::block_status::BlockStatus;
use ethcore::error::BlockError;
use ethcore::encoded;
use ethcore::header::Header;
use ethcore::ids::BlockId;

use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp, UntrustedRlp};
use heapsize::HeapSizeOf;
use bigint::prelude::U256;
use bigint::hash::H256;
use util::kvdb::{DBTransaction, KeyValueDB};

use cache::Cache;
use parking_lot::{Mutex, RwLock};

use smallvec::SmallVec;

/// Store at least this many candidate headers at all times.
/// Also functions as the delay for computing CHTs as they aren't
/// relevant to any blocks we've got in memory.
const HISTORY: u64 = 2048;

/// The best block key. Maps to an RLP list: [best_era, last_era]
const CURRENT_KEY: &'static [u8] = &*b"best_and_latest";

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

impl Encodable for Entry {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(self.candidates.len());

		for candidate in &self.candidates {
			s.begin_list(3)
				.append(&candidate.hash)
				.append(&candidate.parent_hash)
				.append(&candidate.total_difficulty);
		}
	}
}

impl Decodable for Entry {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {

		let mut candidates = SmallVec::<[Candidate; 3]>::new();

		for item in rlp.iter() {
			candidates.push(Candidate {
				hash: item.val_at(0)?,
				parent_hash: item.val_at(1)?,
				total_difficulty: item.val_at(2)?,
			})
		}

		if candidates.is_empty() { return Err(DecoderError::Custom("Empty candidates vector submitted.")) }

		// rely on the invariant that the canonical entry is always first.
		let canon_hash = candidates[0].hash;
		Ok(Entry {
			candidates: candidates,
			canonical_hash: canon_hash,
		})
	}
}

fn cht_key(number: u64) -> String {
	format!("{:08x}_canonical", number)
}

fn era_key(number: u64) -> String {
	format!("candidates_{}", number)
}

/// Pending changes from `insert` to be applied after the database write has finished.
pub struct PendingChanges {
	best_block: Option<BlockDescriptor>, // new best block.
}

/// Header chain. See module docs for more details.
pub struct HeaderChain {
	genesis_header: encoded::Header, // special-case the genesis.
	candidates: RwLock<BTreeMap<u64, Entry>>,
	best_block: RwLock<BlockDescriptor>,
	db: Arc<KeyValueDB>,
	col: Option<u32>,
	cache: Arc<Mutex<Cache>>,
}

impl HeaderChain {
	/// Create a new header chain given this genesis block and database to read from.
	pub fn new(db: Arc<KeyValueDB>, col: Option<u32>, genesis: &[u8], cache: Arc<Mutex<Cache>>) -> Result<Self, String> {
		use ethcore::views::HeaderView;

		let chain = if let Some(current) = db.get(col, CURRENT_KEY)? {
			let (best_number, highest_number) = {
				let rlp = Rlp::new(&current);
				(rlp.val_at(0), rlp.val_at(1))
			};

			let mut cur_number = highest_number;
			let mut candidates = BTreeMap::new();

			// load all era entries and referenced headers within them.
			while let Some(entry) = db.get(col, era_key(cur_number).as_bytes())? {
				let entry: Entry = ::rlp::decode(&entry);
				trace!(target: "chain", "loaded header chain entry for era {} with {} candidates",
					cur_number, entry.candidates.len());

				candidates.insert(cur_number, entry);

				cur_number -= 1;
			}

			// fill best block block descriptor.
			let best_block = {
				let era = match candidates.get(&best_number) {
					Some(era) => era,
					None => return Err(format!("Database corrupt: highest block referenced but no data.")),
				};

				let best = &era.candidates[0];
				BlockDescriptor {
					hash: best.hash,
					number: best_number,
					total_difficulty: best.total_difficulty,
				}
			};

			HeaderChain {
				genesis_header: encoded::Header::new(genesis.to_owned()),
				best_block: RwLock::new(best_block),
				candidates: RwLock::new(candidates),
				db: db,
				col: col,
				cache: cache,
			}
		} else {
			let g_view = HeaderView::new(genesis);
			HeaderChain {
				genesis_header: encoded::Header::new(genesis.to_owned()),
				best_block: RwLock::new(BlockDescriptor {
					hash: g_view.hash(),
					number: 0,
					total_difficulty: g_view.difficulty(),
				}),
				candidates: RwLock::new(BTreeMap::new()),
				db: db,
				col: col,
				cache: cache,
			}
		};

		Ok(chain)
	}

	/// Insert a pre-verified header.
	///
	/// This blindly trusts that the data given to it is sensible.
	/// Returns a set of pending changes to be applied with `apply_pending`
	/// before the next call to insert and after the transaction has been written.
	pub fn insert(&self, transaction: &mut DBTransaction, header: Header) -> Result<PendingChanges, BlockError> {
		let hash = header.hash();
		let number = header.number();
		let parent_hash = *header.parent_hash();
		let mut pending = PendingChanges {
			best_block: None,
		};

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

		// insert headers and candidates entries and write era to disk.
		{
			let cur_era = candidates.entry(number)
				.or_insert_with(|| Entry { candidates: SmallVec::new(), canonical_hash: hash });
			cur_era.candidates.push(Candidate {
				hash: hash,
				parent_hash: parent_hash,
				total_difficulty: total_difficulty,
			});

			// fix ordering of era before writing.
			if total_difficulty > cur_era.candidates[0].total_difficulty {
				let cur_pos = cur_era.candidates.len() - 1;
				cur_era.candidates.swap(cur_pos, 0);
				cur_era.canonical_hash = hash;
			}

			transaction.put(self.col, era_key(number).as_bytes(), &::rlp::encode(&*cur_era))
		}

		let raw = ::rlp::encode(&header);
		transaction.put(self.col, &hash[..], &*raw);

		let (best_num, is_new_best) = {
			let cur_best = self.best_block.read();
			if cur_best.total_difficulty < total_difficulty {
				(number, true)
			} else {
				(cur_best.number, false)
			}
		};

		// reorganize ancestors so canonical entries are first in their
		// respective candidates vectors.
		if is_new_best {
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

				// write altered era to disk
				if height != number {
					let rlp_era = ::rlp::encode(&*entry);
					transaction.put(self.col, era_key(height).as_bytes(), &rlp_era);
				}
			}

			trace!(target: "chain", "New best block: ({}, {}), TD {}", number, hash, total_difficulty);
			pending.best_block = Some(BlockDescriptor {
				hash: hash,
				number: number,
				total_difficulty: total_difficulty,
			});

			// produce next CHT root if it's time.
			let earliest_era = *candidates.keys().next().expect("at least one era just created; qed");
			if earliest_era + HISTORY + cht::SIZE <= number {
				let cht_num = cht::block_to_cht_number(earliest_era)
					.expect("fails only for number == 0; genesis never imported; qed");

				let cht_root = {
					let mut i = earliest_era;

					// iterable function which removes the candidates as it goes
					// along. this will only be called until the CHT is complete.
					let iter = || {
						let era_entry = candidates.remove(&i)
							.expect("all eras are sequential with no gaps; qed");
						transaction.delete(self.col, era_key(i).as_bytes());

						i += 1;

						for ancient in &era_entry.candidates {
							transaction.delete(self.col, &ancient.hash);
						}

						let canon = &era_entry.candidates[0];
						(canon.hash, canon.total_difficulty)
					};
					cht::compute_root(cht_num, ::itertools::repeat_call(iter))
						.expect("fails only when too few items; this is checked; qed")
				};

				// write the CHT root to the database.
				debug!(target: "chain", "Produced CHT {} root: {:?}", cht_num, cht_root);
				transaction.put(self.col, cht_key(cht_num).as_bytes(), &::rlp::encode(&cht_root));
			}
		}

		// write the best and latest eras to the database.
		{
			let latest_num = *candidates.iter().rev().next().expect("at least one era just inserted; qed").0;
			let mut stream = RlpStream::new_list(2);
			stream.append(&best_num).append(&latest_num);
			transaction.put(self.col, CURRENT_KEY, &stream.out())
		}
		Ok(pending)
	}

	/// Apply pending changes from a previous `insert` operation.
	/// Must be done before the next `insert` call.
	pub fn apply_pending(&self, pending: PendingChanges) {
		if let Some(best_block) = pending.best_block {
			*self.best_block.write() = best_block;
		}
	}

	/// Get a block's hash by ID. In the case of query by number, only canonical results
	/// will be returned.
	pub fn block_hash(&self, id: BlockId) -> Option<H256> {
		match id {
			BlockId::Earliest => Some(self.genesis_hash()),
			BlockId::Hash(hash) => Some(hash),
			BlockId::Number(num) => {
				if self.best_block.read().number < num { return None }
				self.candidates.read().get(&num).map(|entry| entry.canonical_hash)
			}
			BlockId::Latest | BlockId::Pending => {
				Some(self.best_block.read().hash)
			}
		}
	}

	/// Get a block header. In the case of query by number, only canonical blocks
	/// will be returned.
	pub fn block_header(&self, id: BlockId) -> Option<encoded::Header> {
		let load_from_db = |hash: H256| {
			let mut cache = self.cache.lock();

			match cache.block_header(&hash) {
				Some(header) => Some(header),
				None => {
					match self.db.get(self.col, &hash) {
						Ok(db_value) => {
							db_value.map(|x| x.into_vec()).map(encoded::Header::new)
								.and_then(|header| {
									cache.insert_block_header(hash.clone(), header.clone());
									Some(header)
								 })
						},
						Err(e) => {
							warn!(target: "chain", "Failed to read from database: {}", e);
							None
						}
					}
				}
			}
		};

		match id {
			BlockId::Earliest | BlockId::Number(0) => Some(self.genesis_header.clone()),
			BlockId::Hash(hash) if hash == self.genesis_hash() => { Some(self.genesis_header.clone()) }
			BlockId::Hash(hash) => load_from_db(hash),
			BlockId::Number(num) => {
				if self.best_block.read().number < num { return None }

				self.candidates.read().get(&num).map(|entry| entry.canonical_hash)
					.and_then(load_from_db)
			}
			BlockId::Latest | BlockId::Pending => {
				// hold candidates hear to prevent deletion of the header
				// as we read it.
				let _candidates = self.candidates.read();
				let hash = {
					let best = self.best_block.read();
					if best.number == 0 {
						return Some(self.genesis_header.clone())
					}

					best.hash
				};

				load_from_db(hash)
			}
		}
	}

	/// Get a block's chain score.
	/// Returns nothing for non-canonical blocks.
	pub fn score(&self, id: BlockId) -> Option<U256> {
		let genesis_hash = self.genesis_hash();
		match id {
			BlockId::Earliest | BlockId::Number(0) => Some(self.genesis_header.difficulty()),
			BlockId::Hash(hash) if hash == genesis_hash => Some(self.genesis_header.difficulty()),
			BlockId::Hash(hash) => match self.block_header(BlockId::Hash(hash)) {
				Some(header) => self.candidates.read().get(&header.number())
					.and_then(|era| era.candidates.iter().find(|e| e.hash == hash))
					.map(|c| c.total_difficulty),
				None => None,
			},
			BlockId::Number(num) => {
				let candidates = self.candidates.read();
				if self.best_block.read().number < num { return None }
				candidates.get(&num).map(|era| era.candidates[0].total_difficulty)
			}
			BlockId::Latest | BlockId::Pending => Some(self.best_block.read().total_difficulty)
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
		match self.db.get(self.col, cht_key(n as u64).as_bytes()) {
			Ok(val) => val.map(|x| ::rlp::decode(&x)),
			Err(e) => {
				warn!(target: "chain", "Error reading from database: {}", e);
				None
			}
		}
	}

	/// Get the genesis hash.
	pub fn genesis_hash(&self) -> H256 {
		self.genesis_header.hash()
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
		match self.db.get(self.col, &*hash).ok().map_or(false, |x| x.is_some()) {
			true => BlockStatus::InChain,
			false => BlockStatus::Unknown,
		}
	}
}

impl HeapSizeOf for HeaderChain {
	fn heap_size_of_children(&self) -> usize {
		self.candidates.read().heap_size_of_children()
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
	use std::sync::Arc;

	use ethcore::ids::BlockId;
	use ethcore::header::Header;
	use ethcore::spec::Spec;
  	use cache::Cache;

	use time::Duration;
	use parking_lot::Mutex;

	fn make_db() -> Arc<::util::KeyValueDB> {
		Arc::new(::util::kvdb::in_memory(0))
	}

	#[test]
	fn basic_chain() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();

		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache).unwrap();

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..10000 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i.into());
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, header).unwrap();
			db.write(tx).unwrap();
			chain.apply_pending(pending);

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
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache).unwrap();

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..6 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i.into());
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, header).unwrap();
			db.write(tx).unwrap();
			chain.apply_pending(pending);

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

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, header).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

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

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, header).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

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
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache).unwrap();


		assert!(chain.block_header(BlockId::Earliest).is_some());
		assert!(chain.block_header(BlockId::Latest).is_some());
		assert!(chain.block_header(BlockId::Pending).is_some());
	}

	#[test]
	fn restore_from_db() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		{
			let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache.clone()).unwrap();
			let mut parent_hash = genesis_header.hash();
			let mut rolling_timestamp = genesis_header.timestamp();
			for i in 1..10000 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i.into());
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, header).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

				rolling_timestamp += 10;
			}
		}

		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache.clone()).unwrap();
		assert!(chain.block_header(BlockId::Number(10)).is_none());
		assert!(chain.block_header(BlockId::Number(9000)).is_some());
		assert!(chain.cht_root(2).is_some());
		assert!(chain.cht_root(3).is_none());
		assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 9999);
	}

	#[test]
	fn restore_higher_non_canonical() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		{
			let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache.clone()).unwrap();
			let mut parent_hash = genesis_header.hash();
			let mut rolling_timestamp = genesis_header.timestamp();

			// push 100 low-difficulty blocks.
			for i in 1..101 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i.into());
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, header).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

				rolling_timestamp += 10;
			}

			// push fewer high-difficulty blocks.
			for i in 1..11 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i.into() * 1000.into());
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, header).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

				rolling_timestamp += 10;
			}

			assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 10);
		}

		// after restoration, non-canonical eras should still be loaded.
		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache.clone()).unwrap();
		assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 10);
		assert!(chain.candidates.read().get(&100).is_some())
	}

	#[test]
	fn genesis_header_available() {
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		let chain = HeaderChain::new(db.clone(), None, &::rlp::encode(&genesis_header), cache.clone()).unwrap();

		assert!(chain.block_header(BlockId::Earliest).is_some());
		assert!(chain.block_header(BlockId::Number(0)).is_some());
		assert!(chain.block_header(BlockId::Hash(genesis_header.hash())).is_some());
	}
}
