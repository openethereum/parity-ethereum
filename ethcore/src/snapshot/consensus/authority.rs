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

//! Secondary chunk creation and restoration, implementation for proof-of-authority
//! based engines.
//!
//! The chunks here contain state proofs of transitions, along with validator proofs.

use super::{SnapshotComponents, Rebuilder, ChunkSink};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use engines::{EthEngine, EpochVerifier, EpochTransition};
use machine::EthereumMachine;
use snapshot::{Error, ManifestData, Progress};

use blockchain::{BlockChain, BlockChainDB, BlockProvider};
use bytes::Bytes;
use ethereum_types::{H256, U256};
use itertools::{Position, Itertools};
use kvdb::KeyValueDB;
use rlp::{RlpStream, Rlp};
use types::encoded;
use types::header::Header;
use types::ids::BlockId;
use types::receipt::Receipt;

/// Snapshot creation and restoration for PoA chains.
/// Chunk format:
///
/// [FLAG, [header, epoch data], ...]
///   - Header data at which transition occurred,
///   - epoch data (usually list of validators and proof of change)
///
/// FLAG is a bool: true for last chunk, false otherwise.
///
/// The last item of the last chunk will be a list containing data for the warp target block:
/// [header, transactions, uncles, receipts, parent_td].
pub struct PoaSnapshot;

impl SnapshotComponents for PoaSnapshot {
	fn chunk_all(
		&mut self,
		chain: &BlockChain,
		block_at: H256,
		sink: &mut ChunkSink,
		_progress: &Progress,
		preferred_size: usize,
	) -> Result<(), Error> {
		let number = chain.block_number(&block_at)
			.ok_or_else(|| Error::InvalidStartingBlock(BlockId::Hash(block_at)))?;

		let mut pending_size = 0;
		let mut rlps = Vec::new();

		for (_, transition) in chain.epoch_transitions()
			.take_while(|&(_, ref t)| t.block_number <= number)
		{
			// this can happen when our starting block is non-canonical.
			if transition.block_number == number && transition.block_hash != block_at {
				break
			}

			let header = chain.block_header_data(&transition.block_hash)
				.ok_or(Error::BlockNotFound(transition.block_hash))?;

			let entry = {
				let mut entry_stream = RlpStream::new_list(2);
				entry_stream
					.append_raw(&header.into_inner(), 1)
					.append(&transition.proof);

				entry_stream.out()
			};

			// cut of the chunk if too large.
			let new_loaded_size = pending_size + entry.len();
			pending_size = if new_loaded_size > preferred_size && !rlps.is_empty() {
				write_chunk(false, &mut rlps, sink)?;
				entry.len()
			} else {
				new_loaded_size
			};

			rlps.push(entry);
		}

		let (block, receipts) = chain.block(&block_at)
			.and_then(|b| chain.block_receipts(&block_at).map(|r| (b, r)))
			.ok_or(Error::BlockNotFound(block_at))?;
		let block = block.decode()?;

		let parent_td = chain.block_details(block.header.parent_hash())
			.map(|d| d.total_difficulty)
			.ok_or(Error::BlockNotFound(block_at))?;

		rlps.push({
			let mut stream = RlpStream::new_list(5);
			stream
				.append(&block.header)
				.append_list(&block.transactions)
				.append_list(&block.uncles)
				.append(&receipts)
				.append(&parent_td);
			stream.out()
		});

		write_chunk(true, &mut rlps, sink)?;

		Ok(())
	}

	fn rebuilder(
		&self,
		chain: BlockChain,
		db: Arc<BlockChainDB>,
		manifest: &ManifestData,
	) -> Result<Box<Rebuilder>, ::error::Error> {
		Ok(Box::new(ChunkRebuilder {
			manifest: manifest.clone(),
			warp_target: None,
			chain: chain,
			db: db.key_value().clone(),
			had_genesis: false,
			unverified_firsts: Vec::new(),
			last_epochs: Vec::new(),
		}))
	}

	fn min_supported_version(&self) -> u64 { 3 }
	fn current_version(&self) -> u64 { 3 }
}

// writes a chunk composed of the inner RLPs here.
// flag indicates whether the chunk is the last chunk.
fn write_chunk(last: bool, chunk_data: &mut Vec<Bytes>, sink: &mut ChunkSink) -> Result<(), Error> {
	let mut stream = RlpStream::new_list(1 + chunk_data.len());

	stream.append(&last);
	for item in chunk_data.drain(..) {
		stream.append_raw(&item, 1);
	}

	(sink)(stream.out().as_slice()).map_err(Into::into)
}

// rebuilder checks state proofs for all transitions, and checks that each
// transition header is verifiable from the epoch data of the one prior.
struct ChunkRebuilder {
	manifest: ManifestData,
	warp_target: Option<Header>,
	chain: BlockChain,
	db: Arc<KeyValueDB>,
	had_genesis: bool,

	// sorted vectors of unverified first blocks in a chunk
	// and epoch data from last blocks in chunks.
	// verification for these will be done at the end.
	unverified_firsts: Vec<(Header, Bytes, H256)>,
	last_epochs: Vec<(Header, Box<EpochVerifier<EthereumMachine>>)>,
}

// verified data.
struct Verified {
	epoch_transition: EpochTransition,
	header: Header,
}

impl ChunkRebuilder {
	fn verify_transition(
		&mut self,
		last_verifier: &mut Option<Box<EpochVerifier<EthereumMachine>>>,
		transition_rlp: Rlp,
		engine: &EthEngine,
	) -> Result<Verified, ::error::Error> {
		use engines::ConstructedVerifier;

		// decode.
		let header: Header = transition_rlp.val_at(0)?;
		let epoch_data: Bytes = transition_rlp.val_at(1)?;

		trace!(target: "snapshot", "verifying transition to epoch at block {}", header.number());

		// check current transition against validators of last epoch.
		let new_verifier = match engine.epoch_verifier(&header, &epoch_data) {
			ConstructedVerifier::Trusted(v) => v,
			ConstructedVerifier::Unconfirmed(v, finality_proof, hash) => {
				match *last_verifier {
					Some(ref last) =>
						if last.check_finality_proof(finality_proof).map_or(true, |hashes| !hashes.contains(&hash))
					{
						return Err(Error::BadEpochProof(header.number()).into());
					},
					None if header.number() != 0 => {
						// genesis never requires additional validation.

						let idx = self.unverified_firsts
							.binary_search_by_key(&header.number(), |&(ref h, _, _)| h.number())
							.unwrap_or_else(|x| x);

						let entry = (header.clone(), finality_proof.to_owned(), hash);
						self.unverified_firsts.insert(idx, entry);
					}
					None => {}
				}

				v
			}
			ConstructedVerifier::Err(e) => return Err(e),
		};

		// create new epoch verifier.
		*last_verifier = Some(new_verifier);

		Ok(Verified {
			epoch_transition: EpochTransition {
				block_hash: header.hash(),
				block_number: header.number(),
				proof: epoch_data,
			},
			header: header,
		})
	}
}

impl Rebuilder for ChunkRebuilder {
	fn feed(
		&mut self,
		chunk: &[u8],
		engine: &EthEngine,
		abort_flag: &AtomicBool,
	) -> Result<(), ::error::Error> {
		let rlp = Rlp::new(chunk);
		let is_last_chunk: bool = rlp.val_at(0)?;
		let num_items = rlp.item_count()?;

		// number of transitions in the chunk.
		let num_transitions = if is_last_chunk {
			num_items - 2
		} else {
			num_items - 1
		};

		if num_transitions == 0 && !is_last_chunk {
			return Err(Error::WrongChunkFormat("Found non-last chunk without any data.".into()).into());
		}

		let mut last_verifier = None;
		let mut last_number = None;
		for transition_rlp in rlp.iter().skip(1).take(num_transitions).with_position() {
			if !abort_flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

			let (is_first, is_last) = match transition_rlp {
				Position::First(_) => (true, false),
				Position::Middle(_) => (false, false),
				Position::Last(_) => (false, true),
				Position::Only(_) => (true, true),
			};

			let transition_rlp = transition_rlp.into_inner();
			let verified = self.verify_transition(
				&mut last_verifier,
				transition_rlp,
				engine,
			)?;

			if last_number.map_or(false, |num| verified.header.number() <= num) {
				return Err(Error::WrongChunkFormat("Later epoch transition in earlier or same block.".into()).into());
			}

			last_number = Some(verified.header.number());

			// book-keep borders for verification later.
			if is_first {
				// make sure the genesis transition was included,
				// but it doesn't need verification later.
				if verified.header.number() == 0 {
					if verified.header.hash() != self.chain.genesis_hash() {
						return Err(Error::WrongBlockHash(0, verified.header.hash(), self.chain.genesis_hash()).into());
					}

					self.had_genesis = true;
				}
			}
			if is_last {
				let idx = self.last_epochs
					.binary_search_by_key(&verified.header.number(), |&(ref h, _)| h.number())
					.unwrap_or_else(|x| x);

				let entry = (
					verified.header.clone(),
					last_verifier.take().expect("last_verifier always set after verify_transition; qed"),
				);
				self.last_epochs.insert(idx, entry);
			}

			// write epoch transition into database.
			let mut batch = self.db.transaction();
			self.chain.insert_epoch_transition(&mut batch, verified.header.number(),
				verified.epoch_transition);
			self.db.write_buffered(batch);

			trace!(target: "snapshot", "Verified epoch transition for epoch at block {}", verified.header.number());
		}

		if is_last_chunk {
			use types::block::Block;

			let last_rlp = rlp.at(num_items - 1)?;
			let block = Block {
				header: last_rlp.val_at(0)?,
				transactions: last_rlp.list_at(1)?,
				uncles: last_rlp.list_at(2)?,
			};
			let block_data = block.rlp_bytes();
			let receipts: Vec<Receipt> = last_rlp.list_at(3)?;

			{
				let hash = block.header.hash();
				let best_hash = self.manifest.block_hash;
				if hash != best_hash {
					return Err(Error::WrongBlockHash(block.header.number(), best_hash, hash).into())
				}
			}

			let parent_td: U256 = last_rlp.val_at(4)?;

			let mut batch = self.db.transaction();
			self.chain.insert_unordered_block(&mut batch, encoded::Block::new(block_data), receipts, Some(parent_td), true, false);
			self.db.write_buffered(batch);

			self.warp_target = Some(block.header);
		}

		Ok(())
	}

	fn finalize(&mut self, _engine: &EthEngine) -> Result<(), ::error::Error> {
		if !self.had_genesis {
			return Err(Error::WrongChunkFormat("No genesis transition included.".into()).into());
		}

		let target_header = match self.warp_target.take() {
			Some(x) => x,
			None => return Err(Error::WrongChunkFormat("Warp target block not included.".into()).into()),
		};

		// verify the first entries of chunks we couldn't before.
		// we store all last verifiers, but not all firsts.
		// match each unverified first epoch with a last epoch verifier.
		let mut lasts_reversed = self.last_epochs.iter().rev();
		for &(ref header, ref finality_proof, hash) in self.unverified_firsts.iter().rev() {
			let mut found = false;
			while let Some(&(ref last_header, ref last_verifier)) = lasts_reversed.next() {
				if last_header.number() < header.number() {
					if last_verifier.check_finality_proof(&finality_proof).map_or(true, |hashes| !hashes.contains(&hash)) {
						return Err(Error::BadEpochProof(header.number()).into());
					}
					found = true;
					break;
				}
			}

			if !found {
				return Err(Error::WrongChunkFormat("Inconsistent chunk ordering.".into()).into());
			}
		}

		// verify that the warp target verifies correctly the
		// most recent epoch. if the warp target was a transition itself,
		// it's already verified and doesn't need any more verification.
		let &(ref header, ref last_epoch) = self.last_epochs.last()
			.expect("last_epochs known to have at least one element by the check above; qed");

		if header != &target_header {
			last_epoch.verify_heavy(&target_header)?;
		}

		Ok(())
	}
}
