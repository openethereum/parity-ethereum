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

//! Secondary chunk creation and restoration, implementation for proof-of-authority
//! based engines.
//!
//! The chunks here contain state proofs of transitions, along with validator proofs.

use super::{SnapshotComponents, Rebuilder, ChunkSink};

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use blockchain::{BlockChain, BlockProvider, EpochTransition};
use engines::{Engine, EpochVerifier};
use ids::BlockId;
use snapshot::{Error, ManifestData};
use snapshot::block::AbridgedBlock;
use receipt::Receipt;

use rlp::{RlpStream, UntrustedRlp};
use util::{Bytes, H256, KeyValueDB, DBValue};

/// Snapshot creation and restoration for PoA chains.
/// Chunk format:
///
/// [FLAG, [header, epoch_number, epoch data, state proof, last hashes], ...]
///   - Header data at which transition occurred,
///   - epoch data (usually list of validators)
///   - state items required to check epoch data
///   - last 256 hashes before the transition; required for checking state changes.
///
/// FLAG is a bool: true for last chunk, false otherwise.
///
/// The last item of the last chunk will be a list containing data for the warp target block:
/// [abridged_block, receipts, last_hashes].
/// If this block is not a transition block, the epoch data should be the same as that
/// for the last transition.
pub struct PoaSnapshot;

impl SnapshotComponents for PoaSnapshot {
	fn chunk_all(
		&mut self,
		chain: &BlockChain,
		block_at: H256,
		sink: &mut ChunkSink,
		preferred_size: usize,
	) -> Result<(), Error> {
		use basic_types::Seal;

		let number = chain.block_number(&block_at)
			.ok_or_else(|| Error::InvalidStartingBlock(BlockId::Hash(block_at)))?;

		let mut pending_size = 0;
		let mut rlps = Vec::new();

		// TODO: this will become irrelevant after recent block hashes are moved into
		// the state. can we optimize it out in that case?
		let make_last_hashes = |parent_hash| chain.ancestry_iter(parent_hash)
			.into_iter()
			.flat_map(|inner| inner)
			.take(255)
			.collect::<Vec<_>>();

		for (epoch_number, transition) in chain.epoch_transitions()
			.take_while(|&(_, ref t)| t.block_number <= number)
		{
			let header = chain.block_header_data(&transition.block_hash)
				.ok_or(Error::BlockNotFound(transition.block_hash))?;

			let last_hashes: Vec<_> = make_last_hashes(header.parent_hash());

			let entry = {
				let mut entry_stream = RlpStream::new_list(5);
				entry_stream
					.append_raw(&header.into_inner(), 1)
					.append(&epoch_number)
					.append(&transition.proof);

				entry_stream.begin_list(transition.state_proof.len());
				for item in transition.state_proof {
					entry_stream.append(&&*item);
				}

				entry_stream.append_list(&last_hashes);
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
		let abridged_rlp = AbridgedBlock::from_block_view(&block.view()).into_inner();
		let last_hashes = make_last_hashes(block.parent_hash());

		rlps.push({
			let mut stream = RlpStream::new_list(3);
			stream.append_raw(&abridged_rlp, 1).append(&receipts).append_list(&last_hashes);
			stream.out()
		});

		write_chunk(true, &mut rlps, sink)?;

		Ok(())
	}

	fn rebuilder(
		&self,
		chain: BlockChain,
		db: Arc<KeyValueDB>,
		manifest: &ManifestData,
	) -> Result<Box<Rebuilder>, ::error::Error> {
		Ok(Box::new(ChunkRebuilder))
	}
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
	warp_target: Option<(Bytes, Vec<Receipt>, Vec<H256>)>,
	chain: BlockChain,
	db: Arc<KeyValueDB>,

	// sorted vectors of unverified first blocks in a chunk
	// and epoch data from last blocks in chunks.
	// verification for these will be done at the end.
	unverified_firsts: Vec<(u64, Header)>,
	last_proofs: Vec<(u64, Bytes)>,
}

// verified data.
struct Verified {
	epoch_number: u64,
	epoch_transition: EpochTransition,
	header: Header,
}

impl Rebuilder {
	fn verify_transition(
		&mut self,
		last_verifier: &mut Option<EpochVerifier>,
		transition_rlp: UntrustedRlp,
		engine: &Engine,
	) -> Result<Verified, ::error::Error> {
		// decode.
		let header: Header = transition_rlp.val_at(0)?;
		let epoch_number: u64 = transition_rlp.val_at(1)?;
		let epoch_data: Bytes = transition_rlp.val_at(2)?;
		let state_proof = transition_rlp.at(3)?
			.iter()
			.map(|x| Ok(DBValue::from_slice(x.data()?)))
			.collect::<Result<Vec<_>, _>>()?;
		let last_hashes: Vec<H256> = transition_rlp.list_at(4)?;
		let last_hashes = Arc::new(last_hashes);

		// check current transition against validators of last epoch.
		if let Some(verifier) = last_verifier.as_ref() {
			verifier.verify_heavy(&header)?;
		}

		{
			// check the provided state proof actually leads to the
			// given epoch data.
			//
			// TODO: hardcoded 50M to match constants in client.
			// would be nice to extract magic numbers, or better yet
			// off-chain transaction execution, into its own module.
			let caller = |addr, data| {
				use env_info::EnvInfo;
				use state::{check_proof, ProvedExecution};
				use transaction::{Action, Transaction};

				let transaction = Transaction {
					nonce: engine.account_start_nonce(0,
					action: Action::Call(addr),
					gas: 50_000_000.into(),
					gas_price: 0.into(),
					value: 0.into(),
					data: data,
				}.fake_sign(Default::default());

				let result = check_proof(
					&state_proof,
					header.state_root(),
					transaction,
					engine,
					&EnvInfo {
						number: header.number(),
						author: header.author(),
						timestamp: header.timestamp(),
						difficulty: header.difficulty(),
						gas_limit: 50_000_000.into(),
						last_hashes: last_hashes.clone(),
						gas_used: 0.into(),
					}
				);

				match result {
					ProvedExecution::Complete(executed) => Ok(executed.output),
					_ => Err("Bad state proof".into()),
				}
			};

			let extracted_proof = engine.epoch_proof(&header, caller)
				.map_err(|_| Error::BadEpochProof(epoch_number))?;

			if extracted_proof != epoch_data {
				return Err(Error::BadEpochProof(epoch_number).into());
			}
		}

		// create new epoch verifier.
		*last_verifier = Some(engine.epoch_verifier(&header, &epoch_data)?);

		Ok(Verified {
			epoch_number: epoch_num,
			epoch_transition: EpochTransition {
				block_hash: header.hash(),
				block_number: header.number(),
				state_proof: state_proof,
				proof: epoch_data,
			},
			header: Header,
		})
	}
}

impl Rebuilder for ChunkRebuilder {
	fn feed(
		&mut self,
		chunk: &[u8],
		engine: &Engine,
		abort_flag: &AtomicBool,
	) -> Result<(), ::error::Error> {
		use itertools::{Position, Itertools};

		let rlp = UntrustedRlp::new(chunk);
		let last_chunk: bool = rlp.val_at(0)?;

		// number of transitions in the chunk.
		let num_transitions = if last_chunk {
			rlp.item_count() - 2
		} else {
			rlp.item_count() - 1
		};

		let mut last_verifier = None;
		for transition_rlp in rlp.iter().skip(1).take(num_transitions) {
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

			// book-keep borders for verification later.
			if is_first {
				let idx = self.unverified_firsts
					.binary_search_by_key(&verified.epoch_number, |&(a, _)| a)
					.unwrap_or_else(|x| x);

				self.unverified_firsts.insert(idx, verified.header.clone());
			}
			if is_last {
				let idx = self.last_proofs
					.binary_search_by_key(&verified.epoch_number, |&(a, _)| a)
					.unwrap_or_else(|x| x);

				self.last_proofs.insert(idx, verified.epoch_transition.proof.clone());
			}

			// write epoch transition into database.
			let mut batch = self.db.transaction();
			self.chain.insert_epoch_transition(&mut batch, verified.epoch_number,
				verified.epoch_transition);
			self.db.write_buffered(batch);
		}

		if last_chunk {
			let last_rlp = transition_rlp.at(transition_rlp.item_count() - 1)?;
			let abridged_rlp = last_rlp.at(0)?.as_raw().to_owned();
			let abridged_block = AbridgedBlock::from_raw(abridged_rlp);
			let receipts: Vec<Receipt> = last_rlp.list_at(1)?;

			let receipts_root = ::util::triehash::ordered_trie_root(
				last_rlp.at(1)?.iter().map(|r| r.as_raw().to_owned())
			);

			// TODO: validate best block hash.

			let last_hashes: Vec<H256> = last_rlp.last_at(2)?;
			self.warp_target = Some((Vec::new(), receipts, last_hashes));
		}
	}

	fn finalize(&mut self) -> Result<(), Error> {
		// TODO: use current state to get epoch data for latest header.
		Ok(())
	}
}
