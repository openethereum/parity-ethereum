// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Light client header chain.
//!
//! Unlike a full node's `BlockChain` this doesn't store much in the database.
//! It stores candidates for the last 2048-4096 blocks as well as CHT roots for
//! historical blocks all the way to the genesis. If the engine makes use
//! of epoch transitions, those are stored as well.
//!
//! This is separate from the `BlockChain` for two reasons:
//!   - It stores only headers (and a pruned subset of them)
//!   - To allow for flexibility in the database layout..

use std::collections::BTreeMap;
use std::sync::Arc;

use cache::Cache;
use cht;
use common_types::{
	block_status::BlockStatus,
	encoded,
	engines::epoch::{
		Transition as EpochTransition,
		PendingTransition as PendingEpochTransition,
	},
	errors::{EthcoreError as Error, BlockError, EthcoreResult},
	header::Header,
	ids::BlockId,
};
use spec::{Spec, SpecHardcodedSync};
use ethereum_types::{H256, H264, U256};
use parity_util_mem::{MallocSizeOf, MallocSizeOfOps};
use kvdb::{DBTransaction, KeyValueDB};
use parking_lot::{Mutex, RwLock};
use fastmap::H256FastMap;
use rlp::{Encodable, Decodable, DecoderError, RlpStream, Rlp};
use smallvec::SmallVec;

/// Store at least this many candidate headers at all times.
/// Also functions as the delay for computing CHTs as they aren't
/// relevant to any blocks we've got in memory.
const HISTORY: u64 = 2048;

/// The best block key. Maps to an RLP list: [best_era, last_era]
const CURRENT_KEY: &[u8] = &*b"best_and_latest";

/// Key storing the last canonical epoch transition.
const LAST_CANONICAL_TRANSITION: &[u8] = &*b"canonical_transition";

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

// best block data
#[derive(RlpEncodable, RlpDecodable)]
struct BestAndLatest {
	best_num: u64,
	latest_num: u64
}

impl BestAndLatest {
	fn new(best_num: u64, latest_num: u64) -> Self {
		BestAndLatest {
			best_num,
			latest_num,
		}
	}
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

impl MallocSizeOf for Entry {
	fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
		if self.candidates.spilled() {
			self.candidates.capacity() * ::std::mem::size_of::<Candidate>()
		} else {
			0
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
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
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
			candidates,
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

fn pending_transition_key(block_hash: H256) -> H264 {
	const LEADING: u8 = 1;

	let mut key = H264::default();

	{
		let bytes = key.as_bytes_mut();
		bytes[0] = LEADING;
		bytes[1..].copy_from_slice(block_hash.as_bytes());
	}

	key
}

fn transition_key(block_hash: H256) -> H264 {
	const LEADING: u8 = 2;

	let mut key = H264::default();

	{
		let bytes = key.as_bytes_mut();
		bytes[0] = LEADING;
		bytes[1..].copy_from_slice(block_hash.as_bytes());
	}

	key
}

// encode last canonical transition entry: header and proof.
fn encode_canonical_transition(header: &Header, proof: &[u8]) -> Vec<u8> {
	let mut stream = RlpStream::new_list(2);
	stream.append(header).append(&proof);
	stream.out()
}

// decode last canonical transition entry.
fn decode_canonical_transition(t: &[u8]) -> Result<(Header, &[u8]), DecoderError> {
	let rlp = Rlp::new(t);

	Ok((rlp.val_at(0)?, rlp.at(1)?.data()?))
}

/// Pending changes from `insert` to be applied after the database write has finished.
pub struct PendingChanges {
	best_block: Option<BlockDescriptor>, // new best block.
}

/// Whether or not the hardcoded sync feature is allowed.
pub enum HardcodedSync {
	Allow,
	Deny,
}

#[derive(MallocSizeOf)]
/// Header chain. See module docs for more details.
pub struct HeaderChain {
	#[ignore_malloc_size_of = "ignored for performance reason"]
	genesis_header: encoded::Header, // special-case the genesis.
	candidates: RwLock<BTreeMap<u64, Entry>>,
	#[ignore_malloc_size_of = "ignored for performance reason"]
	best_block: RwLock<BlockDescriptor>,
	#[ignore_malloc_size_of = "ignored for performance reason"]
	live_epoch_proofs: RwLock<H256FastMap<EpochTransition>>,
	#[ignore_malloc_size_of = "ignored for performance reason"]
	db: Arc<dyn KeyValueDB>,
	#[ignore_malloc_size_of = "ignored for performance reason"]
	col: u32,
	#[ignore_malloc_size_of = "ignored for performance reason"]
	cache: Arc<Mutex<Cache>>,
}

impl HeaderChain {
	/// Create a new header chain given this genesis block and database to read from.
	pub fn new(
		db: Arc<dyn KeyValueDB>,
		col: u32,
		spec: &Spec,
		cache: Arc<Mutex<Cache>>,
		allow_hs: HardcodedSync,
	) -> Result<Self, Error> {
		let mut live_epoch_proofs = ::std::collections::HashMap::default();

		let genesis = ::rlp::encode(&spec.genesis_header());
		let decoded_header = spec.genesis_header();

		let chain = if let Some(current) = db.get(col, CURRENT_KEY)? {
			let curr : BestAndLatest = ::rlp::decode(&current).expect("decoding db value failed");

			let mut cur_number = curr.latest_num;
			let mut candidates = BTreeMap::new();

			// load all era entries, referenced headers within them,
			// and live epoch proofs.
			while let Some(entry) = db.get(col, era_key(cur_number).as_bytes())? {
				let entry: Entry = ::rlp::decode(&entry).expect("decoding db value failed");
				trace!(target: "chain", "loaded header chain entry for era {} with {} candidates",
					cur_number, entry.candidates.len());

				for c in &entry.candidates {
					let key = transition_key(c.hash);

					if let Some(proof) = db.get(col, key.as_bytes())? {
						live_epoch_proofs.insert(c.hash, EpochTransition {
							block_hash: c.hash,
							block_number: cur_number,
							proof,
						});
					}
				}
				candidates.insert(cur_number, entry);

				cur_number -= 1;
			}

			// fill best block block descriptor.
			let best_block = {
				let era = match candidates.get(&curr.best_num) {
					Some(era) => era,
					None => return Err("Database corrupt: highest block referenced but no data.".into()),
				};

				let best = &era.candidates[0];
				BlockDescriptor {
					hash: best.hash,
					number: curr.best_num,
					total_difficulty: best.total_difficulty,
				}
			};

			HeaderChain {
				genesis_header: encoded::Header::new(genesis),
				best_block: RwLock::new(best_block),
				candidates: RwLock::new(candidates),
				live_epoch_proofs: RwLock::new(live_epoch_proofs),
				db,
				col,
				cache,
			}

		} else {
			let chain = HeaderChain {
				genesis_header: encoded::Header::new(genesis),
				best_block: RwLock::new(BlockDescriptor {
					hash: decoded_header.hash(),
					number: 0,
					total_difficulty: *decoded_header.difficulty(),
				}),
				candidates: RwLock::new(BTreeMap::new()),
				live_epoch_proofs: RwLock::new(live_epoch_proofs),
				db: db.clone(),
				col,
				cache,
			};

			// insert the hardcoded sync into the database.
			if let (&Some(ref hardcoded_sync), HardcodedSync::Allow) = (&spec.hardcoded_sync, allow_hs) {
				let mut batch = db.transaction();

				// insert the hardcoded CHT roots into the database.
				for (cht_num, cht_root) in hardcoded_sync.chts.iter().enumerate() {
					batch.put(col, cht_key(cht_num as u64).as_bytes(), &::rlp::encode(cht_root));
				}

				let decoded_header = hardcoded_sync.header.decode()?;
				let decoded_header_num = decoded_header.number();

				// write the block in the DB.
				info!(target: "chain", "Inserting hardcoded block #{} in chain", decoded_header_num);
				let pending = chain.insert_with_td(&mut batch, &decoded_header,
												hardcoded_sync.total_difficulty, None)?;

				// check that we have enough hardcoded CHT roots. avoids panicking later.
				let cht_num = cht::block_to_cht_number(decoded_header_num - 1)
					.expect("specs provided a hardcoded block with height 0");
				if cht_num >= hardcoded_sync.chts.len() as u64 {
					warn!(target: "chain", "specs didn't provide enough CHT roots for its \
											hardcoded block ; falling back to non-hardcoded sync \
											mode");
				} else {
					db.write_buffered(batch);
					chain.apply_pending(pending);
				}
			}

			chain
		};

		// instantiate genesis epoch data if it doesn't exist.
		if chain.db.get(col, LAST_CANONICAL_TRANSITION)?.is_none() {
			let genesis_data = spec.genesis_epoch_data()?;

			{
				let mut batch = chain.db.transaction();
				let data = encode_canonical_transition(&decoded_header, &genesis_data);
				batch.put_vec(col, LAST_CANONICAL_TRANSITION, data);
				chain.db.write(batch)?;
			}
		}

		Ok(chain)
	}

	/// Insert a pre-verified header.
	///
	/// This blindly trusts that the data given to it is sensible.
	/// Returns a set of pending changes to be applied with `apply_pending`
	/// before the next call to insert and after the transaction has been written.
	///
	/// If the block is an epoch transition, provide the transition along with
	/// the header.
	pub fn insert(
		&self,
		transaction: &mut DBTransaction,
		header: &Header,
		transition_proof: Option<Vec<u8>>,
	) -> EthcoreResult<PendingChanges> {
		self.insert_inner(transaction, header, None, transition_proof)
	}

	/// Insert a pre-verified header, with a known total difficulty. Similary to `insert`.
	///
	/// This blindly trusts that the data given to it is sensible.
	pub fn insert_with_td(
		&self,
		transaction: &mut DBTransaction,
		header: &Header,
		total_difficulty: U256,
		transition_proof: Option<Vec<u8>>,
	) -> EthcoreResult<PendingChanges> {
		self.insert_inner(transaction, header, Some(total_difficulty), transition_proof)
	}

	fn insert_inner(
		&self,
		transaction: &mut DBTransaction,
		header: &Header,
		total_difficulty: Option<U256>,
		transition_proof: Option<Vec<u8>>,
	) -> EthcoreResult<PendingChanges> {
		let hash = header.hash();
		let number = header.number();
		let parent_hash = *header.parent_hash();
		let transition = transition_proof.map(|proof| EpochTransition {
			block_hash: hash,
			block_number: number,
			proof,
		});

		let mut pending = PendingChanges {
			best_block: None,
		};

		// hold candidates the whole time to guard import order.
		let mut candidates = self.candidates.write();

		// find total difficulty.
		let total_difficulty = match total_difficulty {
			Some(td) => td,
			None => {
				let parent_td =
					if number == 1 {
						self.genesis_header.difficulty()
					} else {
						candidates.get(&(number - 1))
							.and_then(|entry| entry.candidates.iter().find(|c| c.hash == parent_hash))
							.map(|c| c.total_difficulty)
							.ok_or_else(|| BlockError::UnknownParent(parent_hash))
							.map_err(Error::Block)?
					};

				parent_td + *header.difficulty()
			},
		};

		// insert headers and candidates entries and write era to disk.
		{
			let cur_era = candidates.entry(number)
				.or_insert_with(|| Entry { candidates: SmallVec::new(), canonical_hash: hash });
			cur_era.candidates.push(Candidate {
				hash,
				parent_hash,
				total_difficulty,
			});

			// fix ordering of era before writing.
			if total_difficulty > cur_era.candidates[0].total_difficulty {
				let cur_pos = cur_era.candidates.len() - 1;
				cur_era.candidates.swap(cur_pos, 0);
				cur_era.canonical_hash = hash;
			}

			transaction.put(self.col, era_key(number).as_bytes(), &::rlp::encode(&*cur_era))
		}

		if let Some(transition) = transition {
			transaction.put(self.col, transition_key(hash).as_bytes(), &transition.proof);
			self.live_epoch_proofs.write().insert(hash, transition);
		}

		let raw = header.encoded().into_inner();
		transaction.put_vec(self.col, &hash[..], raw);

		// TODO: For engines when required, use cryptoeconomic guarantees.
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
				hash,
				number,
				total_difficulty,
			});

			// produce next CHT root if it's time.
			let earliest_era = *candidates.keys().next().expect("at least one era just created; qed");
			if earliest_era + HISTORY + cht::SIZE <= number {
				let cht_num = cht::block_to_cht_number(earliest_era)
					.expect("fails only for number == 0; genesis never imported; qed");

				let mut last_canonical_transition = None;
				let cht_root = {
					let mut i = earliest_era;
					let mut live_epoch_proofs = self.live_epoch_proofs.write();

					// iterable function which removes the candidates as it goes
					// along. this will only be called until the CHT is complete.
					let iter = || {
						let era_entry = candidates.remove(&i)
							.expect("all eras are sequential with no gaps; qed");
						transaction.delete(self.col, era_key(i).as_bytes());

						i += 1;

						// prune old blocks and epoch proofs.
						for ancient in &era_entry.candidates {
							let maybe_transition = live_epoch_proofs.remove(&ancient.hash);
							if let Some(epoch_transition) = maybe_transition {
								transaction.delete(self.col, transition_key(ancient.hash).as_bytes());

								if ancient.hash == era_entry.canonical_hash {
									last_canonical_transition = match self.db.get(self.col, ancient.hash.as_bytes()) {
										Err(e) => {
											warn!(target: "chain", "Error reading from DB: {}\n
												", e);
											None
										}
										Ok(None) => panic!("stored candidates always have corresponding headers; qed"),
										Ok(Some(header)) => Some((
											epoch_transition,
											::rlp::decode(&header).expect("decoding value from db failed")
										)),
									};
								}
							}

							transaction.delete(self.col, ancient.hash.as_bytes());
						}

						let canon = &era_entry.candidates[0];
						(canon.hash, canon.total_difficulty)
					};
					cht::compute_root(cht_num, std::iter::repeat_with(iter))
						.expect("fails only when too few items; this is checked; qed")
				};

				// write the CHT root to the database.
				debug!(target: "chain", "Produced CHT {} root: {:?}", cht_num, cht_root);
				transaction.put(self.col, cht_key(cht_num).as_bytes(), &::rlp::encode(&cht_root));

				// update the last canonical transition proof
				if let Some((epoch_transition, header)) = last_canonical_transition {
					let x = encode_canonical_transition(&header, &epoch_transition.proof);
					transaction.put_vec(self.col, LAST_CANONICAL_TRANSITION, x);
				}
			}
		}

		// write the best and latest eras to the database.
		{
			let latest_num = *candidates.iter().rev().next().expect("at least one era just inserted; qed").0;
			let curr = BestAndLatest::new(best_num, latest_num);
			transaction.put(self.col, CURRENT_KEY, &::rlp::encode(&curr))
		}
		Ok(pending)
	}

	/// Generates the specifications for hardcoded sync. This is typically only called manually
	/// from time to time by a Parity developer in order to update the chain specifications.
	///
	/// Returns `None` if we are at the genesis block, or if an error happens .
	pub fn read_hardcoded_sync(&self) -> Result<Option<SpecHardcodedSync>, Error> {
		let mut chts = Vec::new();
		let mut cht_num = 0;

		loop {
			let cht = match self.cht_root(cht_num) {
				Some(cht) => cht,
				None if cht_num != 0 => {
					// end of the iteration
					let h_num = 1 + cht_num as u64 * cht::SIZE;
					let header = if let Some(header) = self.block_header(BlockId::Number(h_num)) {
						header
					} else {
						let msg = format!("header of block #{} not found in DB ; database in an \
											inconsistent state", h_num);
						return Err(msg.into());
					};

					let decoded = header.decode().expect("decoding db value failed");

					let entry: Entry = {
						let bytes = self.db.get(self.col, era_key(h_num).as_bytes())?
							.ok_or_else(|| {
								format!("entry for era #{} not found in DB ; database \
										in an inconsistent state", h_num)
							})?;
						::rlp::decode(&bytes).expect("decoding db value failed")
					};

					let total_difficulty = entry.candidates.iter()
						.find(|c| c.hash == decoded.hash())
						.ok_or_else(|| {
							"no candidate matching block found in DB ; database in an \
										inconsistent state"
						})?
						.total_difficulty;

					break Ok(Some(SpecHardcodedSync {
						header,
						total_difficulty,
						chts,
					}));
				},
				None => {
					break Ok(None);
				},
			};

			chts.push(cht);
			cht_num += 1;
		}
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
			BlockId::Earliest | BlockId::Number(0) => Some(self.genesis_hash()),
			BlockId::Hash(hash) => Some(hash),
			BlockId::Number(num) => {
				if self.best_block.read().number < num { return None }
				self.candidates.read().get(&num).map(|entry| entry.canonical_hash)
			}
			BlockId::Latest => {
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
					match self.db.get(self.col, hash.as_bytes()) {
						Ok(db_value) => {
							db_value
								.map(encoded::Header::new)
								.and_then(|header| {
									cache.insert_block_header(hash, header.clone());
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
			BlockId::Latest => {
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
			BlockId::Latest => Some(self.best_block.read().total_difficulty)
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
			Ok(db_fetch) => db_fetch.map(|bytes| ::rlp::decode(&bytes).expect("decoding value from db failed")),
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
		if self.db.get(self.col, hash.as_bytes()).ok().map_or(false, |x| x.is_some()) {
			BlockStatus::InChain
		} else {
			BlockStatus::Unknown
		}
	}

	/// Insert a pending transition.
	pub fn insert_pending_transition(&self, batch: &mut DBTransaction, hash: H256, t: &PendingEpochTransition) {
		let key = pending_transition_key(hash);
		batch.put(self.col, key.as_bytes(), &*::rlp::encode(t));
	}

	/// Get pending transition for a specific block hash.
	pub fn pending_transition(&self, hash: H256) -> Option<PendingEpochTransition> {
		let key = pending_transition_key(hash);
		match self.db.get(self.col, key.as_bytes()) {
			Ok(db_fetch) => db_fetch.map(|bytes| ::rlp::decode(&bytes).expect("decoding value from db failed")),
			Err(e) => {
				warn!(target: "chain", "Error reading from database: {}", e);
				None
			}
		}
	}

	/// Get the transition to the epoch the given parent hash is part of
	/// or transitions to.
	/// This will give the epoch that any children of this parent belong to.
	///
	/// The header corresponding the the parent hash must be stored already.
	pub fn epoch_transition_for(&self, parent_hash: H256) -> Option<(Header, Vec<u8>)> {
		// slow path: loop back block by block
		let live_proofs = self.live_epoch_proofs.read();

		for hdr in self.ancestry_iter(BlockId::Hash(parent_hash)) {
			if let Some(transition) = live_proofs.get(&hdr.hash()).cloned() {
				return hdr.decode().map(|decoded_hdr| {
					(decoded_hdr, transition.proof)
				}).ok();
			}
		}

		// any blocks left must be descendants of the last canonical transition block.
		match self.db.get(self.col, LAST_CANONICAL_TRANSITION) {
			Ok(x) => {
				let x = x.expect("last canonical transition always instantiated; qed");

				let (hdr, proof) = decode_canonical_transition(&x)
					.expect("last canonical transition always encoded correctly; qed");

				Some((hdr, proof.to_vec()))
			}
			Err(e) => {
				warn!("Error reading from DB: {}", e);
				None
			}
		}
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
	use super::{HeaderChain, HardcodedSync};
	use std::sync::Arc;

	use cache::Cache;
	use common_types::header::Header;
	use common_types::ids::BlockId;
	use spec;
	use ethereum_types::U256;
	use kvdb::KeyValueDB;
	use kvdb_memorydb;

	use std::time::Duration;
	use parking_lot::Mutex;

	fn make_db() -> Arc<dyn KeyValueDB> {
		Arc::new(kvdb_memorydb::create(1))
	}

	#[test]
	fn basic_chain() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();

		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache, HardcodedSync::Allow).unwrap();

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..10000 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i as u32);
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, &header, None).unwrap();
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
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache, HardcodedSync::Allow).unwrap();

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..6 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i as u32);
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, &header, None).unwrap();
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
				header.set_difficulty(*genesis_header.difficulty() * i as u32);
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, &header, None).unwrap();
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
				header.set_difficulty(*genesis_header.difficulty() * U256::from(i * i));
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, &header, None).unwrap();
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
		let spec = spec::new_test();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache, HardcodedSync::Allow).unwrap();

		assert!(chain.block_header(BlockId::Earliest).is_some());
		assert!(chain.block_header(BlockId::Latest).is_some());
	}

	#[test]
	fn restore_from_db() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		{
			let chain = HeaderChain::new(db.clone(), 0, &spec, cache.clone(),
										HardcodedSync::Allow).unwrap();
			let mut parent_hash = genesis_header.hash();
			let mut rolling_timestamp = genesis_header.timestamp();
			for i in 1..10000 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i as u32);
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, &header, None).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

				rolling_timestamp += 10;
			}
		}

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache.clone(),
									HardcodedSync::Allow).unwrap();
		assert!(chain.block_header(BlockId::Number(10)).is_none());
		assert!(chain.block_header(BlockId::Number(9000)).is_some());
		assert!(chain.cht_root(2).is_some());
		assert!(chain.cht_root(3).is_none());
		assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 9999);
	}

	#[test]
	fn restore_higher_non_canonical() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		{
			let chain = HeaderChain::new(db.clone(), 0, &spec, cache.clone(),
										HardcodedSync::Allow).unwrap();
			let mut parent_hash = genesis_header.hash();
			let mut rolling_timestamp = genesis_header.timestamp();

			// push 100 low-difficulty blocks.
			for i in 1..101 {
				let mut header = Header::new();
				header.set_parent_hash(parent_hash);
				header.set_number(i);
				header.set_timestamp(rolling_timestamp);
				header.set_difficulty(*genesis_header.difficulty() * i as u32);
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, &header, None).unwrap();
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
				header.set_difficulty(*genesis_header.difficulty() * U256::from(i as u32 * 1000u32));
				parent_hash = header.hash();

				let mut tx = db.transaction();
				let pending = chain.insert(&mut tx, &header, None).unwrap();
				db.write(tx).unwrap();
				chain.apply_pending(pending);

				rolling_timestamp += 10;
			}

			assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 10);
		}

		// after restoration, non-canonical eras should still be loaded.
		let chain = HeaderChain::new(db.clone(), 0, &spec, cache.clone(),
									HardcodedSync::Allow).unwrap();
		assert_eq!(chain.block_header(BlockId::Latest).unwrap().number(), 10);
		assert!(chain.candidates.read().get(&100).is_some())
	}

	#[test]
	fn genesis_header_available() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache.clone(),
									HardcodedSync::Allow).unwrap();

		assert!(chain.block_header(BlockId::Earliest).is_some());
		assert!(chain.block_header(BlockId::Number(0)).is_some());
		assert!(chain.block_header(BlockId::Hash(genesis_header.hash())).is_some());
	}

	#[test]
	fn epoch_transitions_available_after_cht() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache, HardcodedSync::Allow).unwrap();

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		for i in 1..6 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i as u32);
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let epoch_proof = if i == 3 {
				Some(vec![1, 2, 3, 4])
			} else {
				None
			};

			let pending = chain.insert(&mut tx, &header, epoch_proof).unwrap();
			db.write(tx).unwrap();
			chain.apply_pending(pending);

			rolling_timestamp += 10;
		}

		// these 3 should end up falling back to the genesis epoch proof in DB
		for i in 0..3 {
			let hash = chain.block_hash(BlockId::Number(i)).unwrap();
			assert_eq!(chain.epoch_transition_for(hash).unwrap().1, Vec::<u8>::new());
		}

		// these are live.
		for i in 3..6 {
			let hash = chain.block_hash(BlockId::Number(i)).unwrap();
			assert_eq!(chain.epoch_transition_for(hash).unwrap().1, vec![1, 2, 3, 4]);
		}

		for i in 6..10000 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			header.set_difficulty(*genesis_header.difficulty() * i as u32);
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, &header, None).unwrap();
			db.write(tx).unwrap();
			chain.apply_pending(pending);

			rolling_timestamp += 10;
		}

		// no live blocks have associated epoch proofs -- make sure we aren't leaking memory.
		assert!(chain.live_epoch_proofs.read().is_empty());
		assert_eq!(chain.epoch_transition_for(parent_hash).unwrap().1, vec![1, 2, 3, 4]);
	}

	#[test]
	fn hardcoded_sync_gen() {
		let spec = spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = make_db();

		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		let chain = HeaderChain::new(db.clone(), 0, &spec, cache, HardcodedSync::Allow).expect("failed to instantiate a new HeaderChain");

		let mut parent_hash = genesis_header.hash();
		let mut rolling_timestamp = genesis_header.timestamp();
		let mut total_difficulty = *genesis_header.difficulty();
		let h_num = 3 * ::cht::SIZE + 1;
		for i in 1..10000 {
			let mut header = Header::new();
			header.set_parent_hash(parent_hash);
			header.set_number(i);
			header.set_timestamp(rolling_timestamp);
			let diff = *genesis_header.difficulty() * i as u32;
			header.set_difficulty(diff);
			if i <= h_num {
				total_difficulty = total_difficulty + diff;
			}
			parent_hash = header.hash();

			let mut tx = db.transaction();
			let pending = chain.insert(&mut tx, &header, None).expect("failed inserting a transaction");
			db.write(tx).unwrap();
			chain.apply_pending(pending);

			rolling_timestamp += 10;
		}

		let hardcoded_sync = chain.read_hardcoded_sync().expect("failed reading hardcoded sync").expect("failed unwrapping hardcoded sync");
		assert_eq!(hardcoded_sync.chts.len(), 3);
		assert_eq!(hardcoded_sync.total_difficulty, total_difficulty);
		let decoded: Header = hardcoded_sync.header.decode().expect("decoding failed");
		assert_eq!(decoded.number(), h_num);
	}
}
