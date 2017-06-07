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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use blockchain::{BlockChain, BlockProvider, EpochTransition};
use engines::{Engine, EpochVerifier};
use env_info::EnvInfo;
use ids::BlockId;
use header::Header;
use receipt::Receipt;
use snapshot::{Error, ManifestData};
use state_db::StateDB;

use itertools::{Position, Itertools};
use rlp::{RlpStream, UntrustedRlp};
use util::{Address, Bytes, H256, KeyValueDB, DBValue};

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
/// [header, transactions, uncles, receipts, last_hashes, parent_td].
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
		let block = block.decode();

		let parent_td = chain.block_details(block.header.parent_hash())
			.map(|d| d.total_difficulty)
			.ok_or(Error::BlockNotFound(block_at))?;

		let last_hashes = make_last_hashes(*block.header.parent_hash());

		rlps.push({
			let mut stream = RlpStream::new_list(6);
			stream
				.append(&block.header)
				.append_list(&block.transactions)
				.append_list(&block.uncles)
				.append(&receipts)
				.append_list(&last_hashes)
				.append(&parent_td);
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
		Ok(Box::new(ChunkRebuilder {
			manifest: manifest.clone(),
			warp_target: None,
			chain: chain,
			db: db,
			had_genesis: false,
			unverified_firsts: Vec::new(),
			last_proofs: Vec::new(),
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
	warp_target: Option<(Header, Vec<H256>)>,
	chain: BlockChain,
	db: Arc<KeyValueDB>,
	had_genesis: bool,

	// sorted vectors of unverified first blocks in a chunk
	// and epoch data from last blocks in chunks.
	// verification for these will be done at the end.
	unverified_firsts: Vec<(u64, Header)>,
	last_proofs: Vec<(u64, Header, Bytes)>,
}

// verified data.
struct Verified {
	epoch_number: u64,
	epoch_transition: EpochTransition,
	header: Header,
}

// make a transaction and env info.
// TODO: hardcoded 50M to match constants in client.
// would be nice to extract magic numbers, or better yet
// off-chain transaction execution, into its own module.
fn make_tx_and_env(
	engine: &Engine,
	addr: Address,
	data: Bytes,
	header: &Header,
	last_hashes: Arc<Vec<H256>>,
) -> (::transaction::SignedTransaction, EnvInfo) {
	use transaction::{Action, Transaction};

	let transaction = Transaction {
		nonce: engine.account_start_nonce(),
		action: Action::Call(addr),
		gas: 50_000_000.into(),
		gas_price: 0.into(),
		value: 0.into(),
		data: data,
	}.fake_sign(Default::default());

	let env = EnvInfo {
		number: header.number(),
		author: *header.author(),
		timestamp: header.timestamp(),
		difficulty: *header.difficulty(),
		gas_limit: 50_000_000.into(),
		last_hashes: last_hashes,
		gas_used: 0.into(),
	};

	(transaction, env)
}

impl ChunkRebuilder {
	fn verify_transition(
		&mut self,
		last_verifier: &mut Option<Box<EpochVerifier>>,
		transition_rlp: UntrustedRlp,
		engine: &Engine,
	) -> Result<Verified, ::error::Error> {
		// decode.
		let header: Header = transition_rlp.val_at(0)?;
		let epoch_number: u64 = transition_rlp.val_at(1)?;
		let epoch_data: Bytes = transition_rlp.val_at(2)?;
		let state_proof: Vec<DBValue> = transition_rlp.at(3)?
			.iter()
			.map(|x| Ok(DBValue::from_slice(x.data()?)))
			.collect::<Result<_, ::rlp::DecoderError>>()?;
		let last_hashes: Vec<H256> = transition_rlp.list_at(4)?;
		let last_hashes = Arc::new(last_hashes);

		trace!(target: "snapshot", "verifying transition to epoch {}", epoch_number);

		// check current transition against validators of last epoch.
		if let Some(verifier) = last_verifier.as_ref() {
			verifier.verify_heavy(&header)?;
		}

		{
			// check the provided state proof actually leads to the
			// given epoch data.
			let caller = |addr, data| {
				use state::{check_proof, ProvedExecution};

				let (transaction, env_info) = make_tx_and_env(
					engine,
					addr,
					data,
					&header,
					last_hashes.clone(),
				);

				let result = check_proof(
					&state_proof,
					header.state_root().clone(),
					&transaction,
					engine,
					&env_info,
				);

				match result {
					ProvedExecution::Complete(executed) => Ok(executed.output),
					_ => Err("Bad state proof".into()),
				}
			};

			let extracted_proof = engine.epoch_proof(&header, &caller)
				.map_err(|_| Error::BadEpochProof(epoch_number))?;

			if extracted_proof != epoch_data {
				return Err(Error::BadEpochProof(epoch_number).into());
			}
		}

		// create new epoch verifier.
		*last_verifier = Some(engine.epoch_verifier(&header, &epoch_data)?);

		Ok(Verified {
			epoch_number: epoch_number,
			epoch_transition: EpochTransition {
				block_hash: header.hash(),
				block_number: header.number(),
				state_proof: state_proof,
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
		engine: &Engine,
		abort_flag: &AtomicBool,
	) -> Result<(), ::error::Error> {
		let rlp = UntrustedRlp::new(chunk);
		let is_last_chunk: bool = rlp.val_at(0)?;
		let num_items = rlp.item_count()?;

		// number of transitions in the chunk.
		let num_transitions = if is_last_chunk {
			num_items - 2
		} else {
			num_items - 1
		};

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
				if verified.epoch_number == 0 && verified.header.number() == 0 {
					if verified.header.hash() != self.chain.genesis_hash() {
						return Err(Error::WrongBlockHash(0, verified.header.hash(), self.chain.genesis_hash()).into());
					}

					self.had_genesis = true;
				} else {
					let idx = self.unverified_firsts
						.binary_search_by_key(&verified.epoch_number, |&(a, _)| a)
						.unwrap_or_else(|x| x);

					let entry = (verified.epoch_number, verified.header.clone());
					self.unverified_firsts.insert(idx, entry);
				}
			}
			if is_last {
				let idx = self.last_proofs
					.binary_search_by_key(&verified.epoch_number, |&(a, _, _)| a)
					.unwrap_or_else(|x| x);

				let entry = (
					verified.epoch_number,
					verified.header.clone(),
					verified.epoch_transition.proof.clone()
				);
				self.last_proofs.insert(idx, entry);
			}

			// write epoch transition into database.
			let mut batch = self.db.transaction();
			self.chain.insert_epoch_transition(&mut batch, verified.epoch_number,
				verified.epoch_transition);
			self.db.write_buffered(batch);

			trace!(target: "snapshot", "Verified epoch transition for epoch {}", verified.epoch_number);
		}

		if is_last_chunk {
			use block::Block;

			let last_rlp = rlp.at(num_items - 1)?;
			let block = Block {
				header: last_rlp.val_at(0)?,
				transactions: last_rlp.list_at(1)?,
				uncles: last_rlp.list_at(2)?,
			};
			let block_data = block.rlp_bytes(::basic_types::Seal::With);
			let receipts: Vec<Receipt> = last_rlp.list_at(3)?;

			{
				let hash = block.header.hash();
				let best_hash = self.manifest.block_hash;
				if hash != best_hash {
					return Err(Error::WrongBlockHash(block.header.number(), best_hash, hash).into())
				}
			}

			let last_hashes: Vec<H256> = last_rlp.list_at(4)?;
			let parent_td: ::util::U256 = last_rlp.val_at(5)?;

			let mut batch = self.db.transaction();
			self.chain.insert_unordered_block(&mut batch, &block_data, receipts, Some(parent_td), true, false);
			self.db.write_buffered(batch);

			self.warp_target = Some((block.header, last_hashes));
		}

		Ok(())
	}

	fn finalize(&mut self, db: StateDB, engine: &Engine) -> Result<(), ::error::Error> {
		use state::State;

		if !self.had_genesis {
			return Err(Error::WrongChunkFormat("No genesis transition included.".into()).into());
		}

		let (target_header, target_last_hashes) = match self.warp_target.take() {
			Some(x) => x,
			None => return Err(Error::WrongChunkFormat("Warp target block not included.".into()).into()),
		};

		// we store the last data even for the last chunk for easier verification
		// of warp target, but we don't store genesis transition data.
		// other than that, there should be a one-to-one correspondence of
		// chunk ends to chunk beginnings.
		if self.last_proofs.len() != self.unverified_firsts.len() + 1 {
			return Err(Error::WrongChunkFormat("More than one 'last' chunk".into()).into());
		}

		// verify the first entries of chunks we couldn't before.
		let lasts_iter = self.last_proofs.iter().map(|&(_, ref hdr, ref proof)| (hdr, &proof[..]));
		let firsts_iter = self.unverified_firsts.iter().map(|&(_, ref hdr)| hdr);

		for ((last_hdr, last_proof), first_hdr) in lasts_iter.zip(firsts_iter) {
			let verifier = engine.epoch_verifier(&last_hdr, &last_proof)?;
			verifier.verify_heavy(&first_hdr)?;
		}

		// verify that the validator set of the warp target is the same as that of the
		// most recent transition. if the warp target was a transition itself,
		// `last_data` will still be correct
		let &(_, _, ref last_data) = self.last_proofs.last()
			.expect("last_proofs known to have at least one element by the check above; qed");

		let target_last_hashes = Arc::new(target_last_hashes);
		let caller = |addr, data| {
			use executive::{Executive, TransactOptions};

			let factories = ::factory::Factories::default();
			let mut state = State::from_existing(
				db.boxed_clone(),
				self.manifest.state_root.clone(),
				engine.account_start_nonce(),
				factories,
			).map_err(|e| format!("State root mismatch: {}", e))?;

			let (tx, env_info) = make_tx_and_env(
				engine,
				addr,
				data,
				&target_header,
				target_last_hashes.clone(),
			);

			let options = TransactOptions { tracing: false, vm_tracing: false, check_nonce: false };
			Executive::new(&mut state, &env_info, engine)
				.transact_virtual(&tx, options)
				.map(|e| e.output)
				.map_err(|e| format!("Error executing: {}", e))
		};

		let data = engine.epoch_proof(&target_header, &caller)?;
		if &data[..] != &last_data[..] {
			return Err(Error::WrongChunkFormat("Warp target has different epoch data than epoch transition.".into()).into())
		}

		Ok(())
	}
}
