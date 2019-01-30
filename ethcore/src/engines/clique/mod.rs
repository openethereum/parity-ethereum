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

//! Implemenation of Clique (POA) Engine.
//!
//! mod.rs -> CliqueEngine, the engine api implementation, with additional block state tracking.
//!
//! CliqueEngine -> {
//!   block_state_by_hash  -> block state indexed by header hash.
//!   ...rest
//! }
//!
//! snapshot.rs -> CliqueBlockState , record clique state for given block.
//! param.rs -> Clique Params.
//!

/// How syncing code path works:
/// 1. Client calls `engine.verify_block_basic()` and `engine.verify_block_unordered()`.
/// 2. Client calls `engine.verify_block_family(header, parent)`.
/// 3. Engine first find parent state: `last_state = self.state(parent.hash())`
///    if not found, trigger an back-fill from last checkpoint.
/// 4. Engine calls `last_state.apply(header)`
///
/// executive_author()
/// Clique use author field for voting, the real author is hidden in the extra_data field. So
/// When executing transactions, Client will calls engine.executive_author().

/// How sealing works:
/// 1. implement `engine.set_signer()` . on startup, if miner account was setup on config/cli,
///    miner.set_author() which will eventually pass to here.
/// 2. make `engine.seals_internally()` return Some(true) if signer is present.
/// 3. on Clique::new setup IOService that impalement an timer that just calls `engine.step()`,
///    which just calls `engine.client.update_sealing()` which triggers generating an new block.
/// 4. `engine.generate_seal()` will be called by miner, which should return Seal::None or Seal:Regular
///   a. if period == 0 and block has transactions -> Seal::Regular, else Seal::None
///   b. if block.timestamp() > parent().timestamp() + period -> Seal::Regular
///   c. Seal:: None
/// 5. Miner will create new block, in process it will call several engine method, which they need to do following:
///   a. `engine_open_header_timestamp()` can use default impl.
///   b. `engine.populate_from_parent()` must set difficulty to correct value.
/// 5. Implement `engine.on_seal_block()`, which is the new hook that allow modifying header after block is locked.
///    This is also where we should implement an delay timer??
/// 6. engine.verify_local_seal() will be called, then normal syncing code path will also be called.

use core::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::sync::{Arc, Weak};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use ethereum_types::{Address, H160, H256, Public, U256};
use hash::KECCAK_EMPTY_LIST_RLP;
use lru_cache::LruCache;
use parking_lot::RwLock;
use parking_lot::RwLockUpgradableReadGuard;
use rand::thread_rng;
use rlp::encode;

use account_provider::AccountProvider;
use block::*;
use client::{BlockId, EngineClient};
use engines::{ConstructedVerifier, Engine, Headers, PendingTransitionStore, Seal};
use engines::clique::util::{extract_signers, recover_creator};
use error::Error;
use ethkey::{Password, public_to_address, recover as ec_recover, Signature};
use io::IoService;
use machine::{AuxiliaryData, Call, EthereumMachine};
use types::BlockNumber;
use types::header::{ExtendedHeader, Header};
use itertools::Itertools;
use super::signer::EngineSigner;

use self::params::CliqueParams;
use self::block_state::CliqueBlockState;
use self::step_service::StepService;
use std::time;

mod params;
mod block_state;
mod step_service;
mod util;

// protocol constants
/// Fixed number of extra-data prefix bytes reserved for signer vanity
pub const SIGNER_VANITY_LENGTH: usize = 32;
/// Fixed number of extra-data suffix bytes reserved for signer signature
pub const SIGNER_SIG_LENGTH: usize = 65;
pub const NONCE_DROP_VOTE: [u8; 8] = [0x00; 8];
pub const NONCE_AUTH_VOTE: [u8; 8] = [0xff; 8];
pub const DIFF_INTURN: U256 = U256([2, 0, 0, 0]);
pub const DIFF_NOT_INTURN: U256 = U256([1, 0, 0, 0]);
pub const NULL_AUTHOR: Address = H160([0x00; 20]);
pub const NULL_NONCE: [u8; 8] = NONCE_DROP_VOTE;
pub const NULL_MIXHASH: [u8; 32] = [0x00; 32];
pub const NULL_UNCLES_HASH: H256 = KECCAK_EMPTY_LIST_RLP;

// Caches
/// How many CliqueBlockState to cache in the memory.
pub const STATE_CACHE_NUM: usize = 128;

/// Clique Engine implementation
pub struct Clique {
	epoch_length: u64,
	period: u64,
	machine: EthereumMachine,
	client: RwLock<Option<Weak<EngineClient>>>,
	block_state_by_hash: RwLock<LruCache<H256, CliqueBlockState>>,
	active_prop_delay: Option<(H256, SystemTime, Duration)>,
	signer: RwLock<EngineSigner>,
	step_service: IoService<Duration>,
}

impl Clique {
	/// initialize Clique engine from empty state.
	pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(
			Clique {
				epoch_length: our_params.epoch,
				period: our_params.period,
				client: RwLock::new(Option::default()),
				block_state_by_hash: RwLock::new(LruCache::new(STATE_CACHE_NUM)),
				signer: RwLock::new(Default::default()),
				active_prop_delay: None,
				machine,
				step_service: IoService::<Duration>::start()?,
			});

		if engine.period > 0 {
			let handler = StepService::new(Arc::downgrade(&engine) as Weak<Engine<_>>, Duration::from_secs(our_params.period / 2));
			engine.step_service.register_handler(Arc::new(handler))?;
		}

		return Ok(engine);
	}

	fn sign_header(&self, header: &Header) -> Result<(Signature, H256), Error> {
		let digest = header.hash();

		match (*self.signer.read()).sign(digest) {
			Ok(sig) => { Ok((sig, digest)) }
			Err(e) => { Err(From::from("failed to sign header")) }
		}
	}

	/// Construct an new state from given checkpoint header.
	#[inline]
	fn new_checkpoint_state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		assert_eq!(header.number() % self.epoch_length, 0);

		let state = CliqueBlockState::new(
			match header.number() {
				0 => NULL_AUTHOR,
				_ => recover_creator(header)?,
			},
			extract_signers(header)?);

		Ok(state)
	}

	fn state_no_backfill(&self, hash: &H256) -> Option<CliqueBlockState> {
		return self.block_state_by_hash.write().get_mut(hash).cloned()
	}

	/// get CliqueBlockState for given header, backfill from last checkpoint if needed.
	fn state(&self, header: &Header) -> Result<CliqueBlockState, Error> {
		let mut block_state_by_hash = self.block_state_by_hash.write();
		if let Some(state) = block_state_by_hash.get_mut(&header.hash()) {
			return Ok(state.clone());
		}
		if header.number() % self.epoch_length == 0 {
			let state = self.new_checkpoint_state(header)?;
			block_state_by_hash.insert(header.hash().clone(), state.clone());
			return Ok(state);
		}
		// parent BlockState is not found in memory, which means we need to reconstruct state from last
		// checkpoint.
		match self.client.read().as_ref().and_then(|w| { w.upgrade() }) {
			None => {
				return Err(From::from("failed to upgrade client reference"));
			}
			Some(c) => {
				let last_checkpoint_number = (header.number() / self.epoch_length as u64) * self.epoch_length;
				assert_ne!(last_checkpoint_number, header.number());

				let mut chain: &mut VecDeque<Header> = &mut VecDeque::with_capacity((header.number() - last_checkpoint_number + 1) as usize);

				// Put ourselves in.
				chain.push_front(header.clone());

				// populate chain to last checkpoint
				let mut last = chain.front().unwrap().clone();

				while last.number() != last_checkpoint_number + 1 {
					match c.block_header(BlockId::Hash(*last.parent_hash())) {
						None => {
							return Err(From::from(format!("parent block {} of {} could not be recovered.",
							                              &last.parent_hash(),
							                              &last.hash())));
						}
						Some(next) => {
							chain.push_front(next.decode().unwrap().clone());
							last = chain.front().unwrap().clone();
						}
					}
				}

				// Catching up state, note that we don't really store block state for intermediary blocks,
				// for speed.
				let backfill_start = time::Instant::now();
				info!(target: "engine",
				       "Back-filling block state. last_checkpoint_number: {}, target: {}({}).",
				       last_checkpoint_number, header.number(), header.hash());

				// Get the state for last checkpoint.
				let last_checkpoint_hash = *(chain.front().unwrap().parent_hash());
				let last_checkpoint_header = match c.block_header(BlockId::Hash(last_checkpoint_hash)) {
					None => return Err(From::from("Unable to find last checkpoint block")),
					Some(header) => header.decode().unwrap(),
				};

				let last_checkpoint_state: CliqueBlockState;

				// We probably don't have it cached, but try anyway.
				if let Some(st) = block_state_by_hash.get_mut(&last_checkpoint_hash) {
					last_checkpoint_state = (*st).clone();
				} else {
					last_checkpoint_state = self.new_checkpoint_state(&last_checkpoint_header)?;
				}
				block_state_by_hash.insert(last_checkpoint_header.hash().clone(),last_checkpoint_state.clone());

				// Backfill!
				let mut new_state = last_checkpoint_state.clone();
				for item in chain {
					new_state.apply(item, false)?;
				}
				block_state_by_hash.insert(header.hash(), new_state.clone());

				let elapsed = backfill_start.elapsed();
				info!(target: "engine",
				      "Back-filling succeed, took {} ms.",
				      elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64,
				);

				Ok(new_state)
			}
		}
	}
}

impl Engine<EthereumMachine> for Clique {
	fn name(&self) -> &str { "Clique" }

	fn machine(&self) -> &EthereumMachine { &self.machine }
	/// Clique use same fields, nonce + mixHash
	fn seal_fields(&self, _header: &Header) -> usize { 2 }
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		_epoch_begin: bool,
		ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
		Ok(())
	}

	// Our task here is to set difficulty
	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		// TODO: this is a horrible hack, it is due to the fact that enact_verified and miner both use
		// OpenBlock::new() which will both call this function. more refactoring is definitely needed.
		match header.extra_data().len() >= SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH {
			true => {
				// we are importing blocks, do nothing.
			}
			false => {
				trace!(target: "engine", "populate_from_parent in sealing");

				// It's unclear how to prevent creating new blocks unless we are authorized, the best way (and geth does this too)
				// it's just to ignore setting an correct difficulty here, we will check authorization in next step in generate_seal anyway.
				if let Some(signer) = self.signer.read().address() {
					match self.state(&parent) {
						Err(e) => {
							trace!(target: "engine", "populate_from_parent: Unable to find parent state: {}, ignored.", e);
						}
						Ok(state) => {
							if state.is_authoirzed(&signer) {
								if state.inturn(header.number(), &signer) {
									header.set_difficulty(DIFF_INTURN);
								} else {
									header.set_difficulty(DIFF_NOT_INTURN);
								}
							}
						}
					}
				}
			}
		}
	}

	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		Ok(())
	}

	fn on_seal_block(&self, block: &ExecutedBlock) -> Result<Option<Header>, Error> {
		trace!(target: "engine", "on_seal_block");
		// TODO: implement wiggle here?

		let mut header = block.header().clone();

		header.set_author(NULL_AUTHOR);

		let state = self.state_no_backfill(header.parent_hash()).ok_or_else(
			|| format!("on_seal_block: Unable to get parent state: {}", header.parent_hash())
		)?;

		// TODO: cast an random Vote if not checkpoint

		// Work on clique seal.

		let mut seal: Vec<u8> = Vec::with_capacity(SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH);

		// At this point, extra_data should only contain miner vanity.
		if header.extra_data().len() > SIGNER_VANITY_LENGTH {
			warn!(target: "engine", "on_seal_block: unexpected extra extra_data: {:?}", header);
		}
		// vanity
		{
			let mut vanity = header.extra_data()[0..SIGNER_VANITY_LENGTH - 1].to_vec();
			vanity.resize(SIGNER_VANITY_LENGTH, 0u8);
			seal.extend_from_slice(&vanity[..]);
		}

		// If we are building an checkpoint block, add all signers now.
		if header.number() % self.epoch_length == 0 {
			seal.reserve(state.signers.len() * 20);
			state.signers.iter().foreach(|addr| {
				seal.extend_from_slice(&addr[..]);
			});
		}

		header.set_extra_data(seal.clone());

		// append signature onto extra_data
		let (sig, msg) = self.sign_header(&header)?;
		seal.extend_from_slice(&sig[..]);
		header.set_extra_data(seal.clone());

		// Record state
		let mut new_state = state.clone();
		new_state.apply(&header, header.number() % self.epoch_length == 0)?;
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		trace!(target: "engine", "on_seal_block: finished, final header: {:?}", header);

		Ok(Some(header))
	}

	// No Uncle in Clique
	fn maximum_uncle_age(&self) -> usize { 0 }

	/// Clique doesn't require external work to seal. once signer is set we will begin sealing.
	fn seals_internally(&self) -> Option<bool> {
		Some(self.signer.read().is_some())
	}

	/// Returns if we are ready to seal, the real sealing (signing extra_data) is actually done in `on_seal_block()`.
	fn generate_seal(&self, block: &ExecutedBlock, parent: &Header) -> Seal {
		// make this pub
		let NULL_SEAL = vec!(encode(&vec![0; 32]), encode(&vec![0; 8]));

		trace!(target: "engine", "tried to generate seal");

		if block.header.number() == 0 {
			trace!(target: "engine", "attempted to seal genesis block");
			return Seal::None;
		}

		// if sealing period is 0, and not an checkpoint block, refuse to seal
		if self.period == 0 {
			if block.transactions.is_empty() && block.header.number() % self.epoch_length != 0 {
				return Seal::None;
			}
			return Seal::Regular(NULL_SEAL);
		}

		// If we are too early
		if block.header.timestamp() <= parent.timestamp() + self.period {
			return Seal::None;
		}

		// Check we actually have authority to seal.
		let author = self.signer.read().address();
		if author.is_none() {
			return Seal::None;
		}

		// ensure the voting state exists
		match self.state(&parent) {
			Err(e) => {
				warn!(target: "engine", "generate_seal: can't get parent state(number: {}, hash: {}): {} ",
				      parent.number(), parent.hash(), e);
				return Seal::None;
			}
			Ok(state) => {
				// Are we authorized to seal?
				if state.is_authoirzed(&author.unwrap()) {
					trace!(target: "engine", "generate_seal: seal ready for block {}, txs: {}.",
					       block.header.number(), block.transactions.len());
					return Seal::Regular(NULL_SEAL);
				}
				trace!(target: "engine", "generate_seal: Not authorized to sign right now.")
			}
		}
		Seal::None
	}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		// Largely same as https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L275

		// Ignore genesis block.
		if header.number() == 0 {
			return Ok(());
		}

		// Don't waste time checking blocks from the future
		{
			let limit = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() + self.period;
			if header.timestamp() > limit {
				return Err(Box::new(
					format!("Block is too far in the future, timestamp: {}, limit: {}", header.timestamp(), limit)
				).into());
			}
		}

		let is_checkpoint = header.number() % self.epoch_length == 0;

		if is_checkpoint && *header.author() != NULL_AUTHOR {
			return Err(Box::new("Checkpoint block must enforce zero beneficiary").into());
		}

		// Nonce must be 0x00..0 or 0xff..f
		let nonce = header.decode_seal::<Vec<&[u8]>>()?[1];
		if *nonce != NONCE_DROP_VOTE && *nonce != NONCE_AUTH_VOTE {
			return Err(Box::new("nonce must be 0x00..0 or 0xff..f").into());
		}
		if is_checkpoint && *nonce != NULL_NONCE[..] {
			return Err(Box::new("Checkpoint block must have zero nonce").into());
		}

		// Ensure that the extra-data contains a signer list on checkpoint, but none otherwise.
		if (!is_checkpoint && header.extra_data().len() != (SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH))
			|| (is_checkpoint && header.extra_data().len() <= (SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH))
			|| (is_checkpoint && (header.extra_data().len() - (SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH)) % 20 != 0) {
			return Err(Box::new(format!("Invalid extra_data length, got {}", header.extra_data().len())).into());
		}

		// Ensure that the mix digest is zero as we don't have fork protection currently
		let mixhash = header.decode_seal::<Vec<&[u8]>>()?[0];
		if mixhash != NULL_MIXHASH {
			return Err(Box::new("mixhash must be 0x00..0 or 0xff..f.").into())
		}

		// Ensure that the block doesn't contain any uncles which are meaningless in PoA
		if *header.uncles_hash() != NULL_UNCLES_HASH {
			return Err(Box::new(format!(
				"Invalid uncle hash, got: {}, expected: {}.",
				header.uncles_hash(),
				NULL_UNCLES_HASH,
			)).into())
		}

		// Ensure that the block's difficulty is meaningful (may not be correct at this point)
		if *header.difficulty() != DIFF_INTURN && *header.difficulty() != DIFF_NOT_INTURN {
			return Err(Box::new(format!(
				"invalid difficulty: expected {} or {}, got: {}.",
				DIFF_INTURN,
				DIFF_NOT_INTURN,
				header.difficulty(),
			)).into())
		}

		// All basic checks passed, continue to next phase
		Ok(())
	}

	fn verify_block_unordered(&self, _header: &Header) -> Result<(), Error> {
		// Nothing to check here.
		Ok(())
	}

	/// Verify block family by looking up parent state (backfill if needed), then try to apply current header.
	/// see https://github.com/ethereum/go-ethereum/blob/master/consensus/clique/clique.go#L338
	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		// Ignore genesis block.
		if header.number() == 0 {
			return Ok(());
		}

		// parent sanity check
		if parent.hash() != *header.parent_hash() || header.number() != parent.number() + 1 {
			return Err(Box::new("invalid parent").into());
		}

		// Ensure that the block's timestamp isn't too close to it's parent
		if parent.timestamp() + self.period > header.timestamp() {
			return Err(Box::new("invalid timestamp").into());
		}

		// Retrieve the parent state
		let parent_state = self.state(&parent)?;

		// Try to apply current state, apply() will further check signer and recent signer.
		let mut new_state = parent_state.clone();
		new_state.apply(header, header.number() % self.epoch_length == 0)?;
		self.block_state_by_hash.write().insert(header.hash(), new_state);

		Ok(())
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		let state = self.new_checkpoint_state(header).expect("Unable to parse genesis data.");
		self.block_state_by_hash.write().insert(header.hash(), state);

		Ok(Vec::new())
	}

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
	                     -> super::EpochChange<EthereumMachine>
	{
		super::EpochChange::No
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		_finalized: &[H256],
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
		ConstructedVerifier::Trusted(Box::new(super::epoch::NoOp))
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
		trace!(target: "engine", "called set_signer");
		self.signer.write().set(ap, address, password);
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}

	fn step(&self) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.update_sealing();
			}
		}
	}

	fn stop(&mut self) {
		self.step_service.borrow_mut().stop();
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp + self.period
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}

	fn executive_author(&self, header: &Header) -> Address {
		// Should have been verified now.
		return recover_creator(header).unwrap();
	}
}

//impl CliqueState {
//	pub fn new(epoch_length: u64) -> Self {
//		CliqueState {
//			epoch_length: epoch_length,
//			states: RwLock::new(Default::default()),
//		}
//	}
//	/// Get an valid block state
//	pub fn state(&mut self, hash: &H256) -> Option<CliqueBlockState> {
//		let db = self.states_by_hash.borrow_mut();
//		return db.get_mut(hash).cloned();
//	}
//
//	pub fn turn_delay(&mut self, header: &Header) -> bool {
//		match self.active_prop_delay {
//			Some((parent_hash, start, duration)) => {
//				if *header.parent_hash() != parent_hash {
//					// reorg.  make sure the timer is reset
//					self.active_prop_delay = Some((header.parent_hash().clone(),
//					                               SystemTime::now(),
//					                               Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
//					return false;
//				}
//
//				if start.elapsed().expect("start delay was after current time") >= duration {
//					return true;
//				} else {
//					return false;
//				}
//			}
//			None => {
//				self.active_prop_delay = Some((header.parent_hash().clone(),
//				                               SystemTime::now(),
//				                               Duration::from_millis(thread_rng().gen_range::<u64>(0, self.state(header.parent_hash()).unwrap().signers.len() as u64 * 500))));
//				return false;
//			}
//		}
//	}
//
//	/// Apply an new header
//	pub fn apply(&mut self, header: &Header) -> Result<(), Error> {
//		let db = self.states_by_hash.borrow_mut();
//
//		// make sure current hash is not in the db
//		match db.get_mut(header.parent_hash()).cloned() {
//			Some(ref mut new_state) => {
//				let creator = match process_header(&header, new_state, self.epoch_length) {
//					Err(e) => {
//						return Err(From::from(
//							format!("Error applying header: {}, current state: {:?}", e, new_state)
//						));
//					}
//					Ok(creator) => { creator }
//				};
//
//				db.insert(header.hash(), new_state.clone());
//				Ok(())
//			}
//			None => {
//				Err(From::from(
//					format!("Parent block (hash: {}) for Block {}, hash {} is not found!",
//					        header.parent_hash(),
//					        header.number(), header.hash())))
//			}
//		}
//	}
//
//	pub fn apply_checkpoint(&mut self, header: &Header) -> Result<(), Error> {
//		let db = self.states.write().borrow_mut();
//		let state = &mut CliqueBlockState {
//			votes: HashMap::new(),
//			votes_history: Vec::new(),
//			signers: Vec::new(),
//			recent_signers: VecDeque::new(),
//		};
//		process_genesis_header(header, state)?;
//
//		trace!("inserting {} {:?}", header.hash(), &state);
//		db.insert(header.hash(), state.clone());
//
//		Ok(())
//	}
//
//	pub fn set_signer_address(&self, signer_address: Address) {
//		trace!(target: "engine", "setting signer {}", signer_address);
//		*self.signer.write() = Some(signer_address.clone());
//	}
//
//	pub fn proposer_authorization(&mut self, header: &Header) -> SignerAuthorization {
//		let mut db = self.states_by_hash.borrow_mut();
//
//		let proposer_address = match *self.signer.read() {
//			Some(address) => address.clone(),
//			None => { return SignerAuthorization::Unauthorized; }
//		};
//
//		match db.get_mut(header.parent_hash()).cloned() {
//			Some(ref state) => {
//				return state.get_signer_authorization(header.number(), &proposer_address);
//			}
//			None => {
//				panic!("Parent block (hash: {}) for Block {}, hash {} is not found!",
//				       header.parent_hash(),
//				       header.number(), header.hash())
//			}
//		}
//	}
//}
